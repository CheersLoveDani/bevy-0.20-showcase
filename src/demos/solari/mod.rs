//! Solari (Bevy 0.20): realtime path-traced direct and indirect lighting from `bevy_solari`.
//!
//! A Cornell box lit only by an emissive ceiling panel. Every mesh carries a `RaytracingMesh3d`
//! next to its `Mesh3d`, and the camera carries `SolariLighting`. Solari also runs on Metal now.
//!
//! Solari needs hardware ray tracing: the wgpu features in `SolariPlugins::required_wgpu_features`.
//! Bevy's default `WgpuSettings` already request every feature the adapter offers, and the Solari
//! plugins switch themselves off when one is missing, so the demo only has to check the device.
//!
//! # Getting a clean image
//!
//! Solari's raw output is noisy by design: it expects a denoiser behind it. Three vendor-neutral
//! ways to clean it up can be switched at runtime (see [`Modes`]):
//!
//! - `R`: ReSTIR, new as an opt-in in 0.20 through `SolariLighting::restir`. Reuses light samples
//!   across neighbouring pixels and previous frames.
//! - `T`: `TemporalAntiAliasing`, which blends each frame into a history buffer and so doubles as
//!   a simple temporal denoiser.
//! - `P`: `Pathtracer`, Solari's progressive reference path tracer. It replaces the realtime
//!   lighting and averages frames for as long as nothing moves.
//!
//! The denoiser Solari is designed around is DLSS Ray Reconstruction. That needs an NVIDIA RTX
//! GPU, the Vulkan backend and the proprietary DLSS SDK at build time (see
//! <https://github.com/bevyengine/dlss_wgpu>), so it is not compiled into this showcase. Bevy 0.20
//! has no FSR integration, and FSR is an upscaler rather than a denoiser. To add DLSS-RR:
//!
//! 1. enable Bevy's `dlss` cargo feature,
//! 2. insert a `DlssProjectId` resource before `DefaultPlugins`,
//! 3. if `Option<Res<DlssRayReconstructionSupported>>` is `Some`, add
//!    `Dlss::<DlssRayReconstructionFeature>::default()` to the `SolariLighting` camera.

use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    camera::{CameraMainTextureUsages, RenderTarget},
    material::OpaqueRendererMethod,
    pbr::DefaultOpaqueRendererMethod,
    prelude::*,
    render::{
        camera::{MipBias, TemporalJitter},
        render_resource::{TextureFormat, TextureUsages},
        renderer::RenderDevice,
    },
    solari::{
        pathtracer::{Pathtracer, PathtracingPlugin},
        prelude::{RaytracingMesh3d, SolariLighting, SolariPlugins},
    },
    ui::widget::ViewportNode,
};

use crate::{
    demo::Demo,
    shell::{ContentRoot, ShellAppExt},
};

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::Solari)
        .add_plugins((SolariPlugins, PathtracingPlugin))
        // Solari shades a deferred G-buffer, so its plugin makes deferred the default for every
        // opaque material in the app. Keep the other demos on forward rendering; the materials
        // in this scene ask for deferred explicitly.
        .insert_resource(DefaultOpaqueRendererMethod::forward())
        .add_systems(OnEnter(Demo::Solari), setup)
        .add_systems(
            Update,
            (
                // First, so a freshly added path tracer is extracted once with `reset` set.
                finish_pathtracer_reset,
                toggle_modes,
                apply_modes,
                update_status,
                spin,
            )
                .chain()
                .run_if(in_state(Demo::Solari)),
        );
}

/// Which of the noise-reduction options are switched on. Lives on the camera.
#[derive(Component, Clone, Copy)]
struct Modes {
    restir: bool,
    taa: bool,
    pathtracer: bool,
}

impl Default for Modes {
    fn default() -> Self {
        Self {
            restir: false, // As in `SolariLighting::default()`.
            taa: true,
            pathtracer: false,
        }
    }
}

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct Spin;

fn setup(
    root: Single<Entity, With<ContentRoot>>,
    device: Res<RenderDevice>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let missing = SolariPlugins::required_wgpu_features().difference(device.features());
    if !missing.is_empty() {
        commands.spawn(status_text(
            *root,
            format!(
                "Solari is unsupported on this GPU.\n\nIt needs hardware ray tracing, and the \
                 render device is missing these wgpu features:\n{missing:?}"
            ),
        ));
        return;
    }

    // The camera renders into a texture that a `ViewportNode` shows in the content area and
    // keeps at the node's size.
    let target = images.add(Image::new_target_texture(
        0,
        0,
        TextureFormat::Bgra8UnormSrgb,
        None,
    ));
    let camera = commands
        .spawn((
            DespawnOnExit(Demo::Solari),
            Camera3d::default(),
            Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            RenderTarget::Image(target.into()),
            Transform::from_xyz(0.0, 2.65, 8.6).looking_at(Vec3::new(0.0, 2.65, 0.0), Vec3::Y),
            // `apply_modes` adds `SolariLighting` or `Pathtracer` to match.
            Modes::default(),
            // Solari writes its result into the main texture from a compute shader, and cannot
            // resolve multisampled targets.
            CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING),
            Msaa::Off,
        ))
        .id();
    commands.spawn((
        ChildOf(*root),
        DespawnOnExit(Demo::Solari),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        ViewportNode::new(camera),
    ));
    // Filled in by `update_status`.
    commands.spawn(status_text(*root, String::new()));

    // Raytracing meshes need exactly these attributes: position, normal, UV and tangent.
    let mut raytraceable = |mesh: Mesh| meshes.add(mesh.with_generated_tangents().unwrap());
    let cube = raytraceable(Cuboid::default().mesh().build());
    let sphere = raytraceable(Sphere::new(0.75).mesh().build());

    let mut material = |material: StandardMaterial| {
        materials.add(StandardMaterial {
            opaque_render_method: OpaqueRendererMethod::Deferred,
            ..material
        })
    };
    let white = material(Color::srgb(0.8, 0.8, 0.8).into());
    let red = material(Color::srgb(0.75, 0.08, 0.06).into());
    let green = material(Color::srgb(0.1, 0.55, 0.12).into());
    let mirror = material(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.95),
        metallic: 1.0,
        perceptual_roughness: 0.1,
        ..default()
    });
    // The only light in the scene: emissive triangles are sampled directly as area lights.
    let light = material(StandardMaterial {
        emissive: LinearRgba::rgb(1.0, 0.8, 0.55) * 25_000.0,
        ..default()
    });

    let slab = |x, y, z, width, height, depth| {
        Transform::from_xyz(x, y, z).with_scale(Vec3::new(width, height, depth))
    };
    let boxes = [
        (white.clone(), slab(0.0, -0.05, 0.0, 4.2, 0.1, 4.2)), // floor
        (white.clone(), slab(0.0, 4.05, 0.0, 4.2, 0.1, 4.2)),  // ceiling
        (white.clone(), slab(0.0, 2.0, -2.05, 4.2, 4.2, 0.1)), // back wall
        (red, slab(-2.05, 2.0, 0.0, 0.1, 4.2, 4.2)),
        (green, slab(2.05, 2.0, 0.0, 0.1, 4.2, 4.2)),
        (light, slab(0.0, 3.97, 0.0, 1.4, 0.05, 1.4)),
    ];
    for (material, transform) in boxes {
        commands.spawn((
            DespawnOnExit(Demo::Solari),
            Mesh3d(cube.clone()),
            RaytracingMesh3d(cube.clone()),
            MeshMaterial3d(material),
            transform,
        ));
    }
    // A slowly turning block: shadows and bounce light follow it with no baking.
    commands.spawn((
        DespawnOnExit(Demo::Solari),
        Spin,
        Mesh3d(cube.clone()),
        RaytracingMesh3d(cube),
        MeshMaterial3d(white),
        slab(-0.75, 1.25, -0.6, 1.2, 2.5, 1.2),
    ));
    commands.spawn((
        DespawnOnExit(Demo::Solari),
        Mesh3d(sphere.clone()),
        RaytracingMesh3d(sphere),
        MeshMaterial3d(mirror),
        Transform::from_xyz(0.9, 0.75, 0.6),
    ));
}

fn status_text(root: Entity, text: String) -> impl Bundle {
    (
        ChildOf(root),
        DespawnOnExit(Demo::Solari),
        Node {
            padding: UiRect::all(px(16)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        children![(
            StatusText,
            Text(text),
            TextFont {
                font_size: FontSize::Rem(0.9),
                ..default()
            },
        )],
    )
}

fn toggle_modes(keys: Res<ButtonInput<KeyCode>>, mut modes: Single<&mut Modes>) {
    if keys.just_pressed(KeyCode::KeyR) {
        modes.restir = !modes.restir;
    }
    if keys.just_pressed(KeyCode::KeyT) {
        modes.taa = !modes.taa;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        modes.pathtracer = !modes.pathtracer;
    }
}

/// Turns [`Modes`] into camera components. All of them can be swapped while the app runs.
fn apply_modes(
    camera: Single<(Entity, &Modes, Option<&mut SolariLighting>), Changed<Modes>>,
    mut commands: Commands,
) {
    let (entity, modes, lighting) = camera.into_inner();
    let mut camera = commands.entity(entity);

    if modes.pathtracer {
        // The path tracer overwrites the whole image, so the realtime lighting only costs time.
        // Clear its accumulated image in case it is left over from an earlier run.
        camera
            .remove::<SolariLighting>()
            .insert(Pathtracer { reset: true });
    } else if let Some(mut lighting) = lighting {
        lighting.restir = modes.restir;
    } else {
        camera.remove::<Pathtracer>().insert(SolariLighting {
            restir: modes.restir,
            ..default()
        });
    }

    if modes.taa && !modes.pathtracer {
        camera.insert(TemporalAntiAliasing::default());
    } else {
        // TAA brings the sub-pixel camera jitter along as required components.
        camera.remove::<(TemporalAntiAliasing, TemporalJitter, MipBias)>();
    }
}

/// `Pathtracer::reset` is not cleared for us, and accumulation only starts once it is false.
fn finish_pathtracer_reset(mut pathtracer: Single<&mut Pathtracer>) {
    if pathtracer.reset {
        pathtracer.reset = false;
    }
}

fn update_status(
    modes: Single<&Modes, Changed<Modes>>,
    mut text: Single<&mut Text, With<StatusText>>,
) {
    let on_off = |on| if on { "on" } else { "off" };
    let realtime = if modes.pathtracer {
        " (not used by the path tracer)"
    } else {
        ""
    };
    text.0 = format!(
        "Path-traced lighting from one emissive panel. No shadow maps, light probes or baking.\n\
         [R] ReSTIR {}: reuses samples across pixels and frames{realtime}\n\
         [T] TAA {}: blends frames over time, a simple temporal denoiser{realtime}\n\
         [P] Reference path tracer {}: {}\n\
         DLSS Ray Reconstruction is NVIDIA-only and not compiled in, and Bevy 0.20 has no FSR \
         integration. The three modes above are the vendor-neutral ways to a clean image.",
        on_off(modes.restir),
        on_off(modes.taa),
        on_off(modes.pathtracer),
        if modes.pathtracer {
            "accumulating, the block is paused so the image can converge"
        } else {
            "replaces the realtime lighting with a progressive render"
        },
    );
}

/// Turns the block, except under the path tracer, which needs a static scene to accumulate.
fn spin(
    time: Res<Time>,
    modes: Single<&Modes>,
    mut blocks: Query<&mut Transform, With<Spin>>,
) {
    if modes.pathtracer {
        return;
    }
    for mut transform in &mut blocks {
        transform.rotate_y(0.4 * time.delta_secs());
    }
}
