//! WESL shaders: Bevy's shaders (and custom ones) are now written in WESL instead of the
//! naga_oil dialect. The material below uses `assets/render/wesl_material.wesl`, which
//! `import`s from Bevy and from a second local module (`wesl_patterns.wesl`), and picks one
//! of two function declarations with `@if(RAINBOW_STRIPES)` / `@else`.

use bevy::{
    camera::ClearColorConfig,
    input::common_conditions::input_just_pressed,
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
    ui_widgets::Activate,
};

use super::{ContentViewport, PanelText, button, spawn_panel};
use crate::{demo::Demo, shell::ContentRoot, shell::ShellAppExt};

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::WeslMaterial)
        .add_plugins(MaterialPlugin::<WeslMaterial>::default())
        .add_systems(OnEnter(Demo::WeslMaterial), setup)
        .add_systems(
            Update,
            (
                spin,
                toggle_stripes.run_if(input_just_pressed(KeyCode::Space)),
            )
                .run_if(in_state(Demo::WeslMaterial)),
        );
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[bind_group_data(WeslMaterialKey)]
struct WeslMaterial {
    #[uniform(0)]
    color: LinearRgba,
    /// Not sent to the GPU: it selects which variant of the shader gets compiled.
    rainbow_stripes: bool,
}

/// The part of the material that pipelines are specialized on.
#[repr(C)]
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct WeslMaterialKey {
    rainbow_stripes: bool,
}

impl From<&WeslMaterial> for WeslMaterialKey {
    fn from(material: &WeslMaterial) -> Self {
        Self {
            rainbow_stripes: material.rainbow_stripes,
        }
    }
}

impl Material for WeslMaterial {
    fn fragment_shader() -> ShaderRef {
        "render/wesl_material.wesl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.bind_group_data.rainbow_stripes
            && let Some(fragment) = descriptor.fragment.as_mut()
        {
            // A boolean shader def is a WESL conditional compilation flag: `@if(RAINBOW_STRIPES)`.
            fragment.shader_defs.push("RAINBOW_STRIPES".into());
        }
        Ok(())
    }
}

fn setup(
    root: Single<Entity, With<ContentRoot>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WeslMaterial>>,
) {
    let demo = DespawnOnExit(Demo::WeslMaterial);
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Transform::from_xyz(0.0, 1.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ContentViewport,
        demo.clone(),
    ));

    let shapes = [
        (
            meshes.add(Torus::new(0.5, 1.1)),
            Color::srgb(0.9, 0.35, 0.3),
        ),
        (
            meshes.add(Sphere::new(1.1).mesh().uv(64, 32)),
            Color::srgb(0.3, 0.6, 0.9),
        ),
        (
            meshes.add(Cuboid::from_length(1.7)),
            Color::srgb(0.4, 0.8, 0.45),
        ),
    ];
    for (i, (mesh, color)) in shapes.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(WeslMaterial {
                color: color.into(),
                rainbow_stripes: true,
            })),
            Transform::from_xyz((i as f32 - 1.0) * 3.2, -0.3, 0.0),
            demo.clone(),
        ));
    }

    spawn_panel(
        &mut commands,
        *root,
        Demo::WeslMaterial,
        panel_text(true),
        bsn_list! {
            @button("Toggle RAINBOW_STRIPES", |_: On<Activate>, mut commands: Commands| {
                commands.run_system_cached(toggle_stripes);
            })
        },
    );
}

fn panel_text(rainbow_stripes: bool) -> String {
    format!(
        "wesl_material.wesl imports `rainbow` and `stripes` from super::wesl_patterns.\n\
         Shader def RAINBOW_STRIPES: {}, so the `{}` declaration of surface_color() is compiled.\n\
         Space or the button: toggle the shader def",
        if rainbow_stripes { "ON" } else { "OFF" },
        if rainbow_stripes {
            "@if(RAINBOW_STRIPES)"
        } else {
            "@else"
        },
    )
}

/// Flipping the flag changes the material's key, so its pipeline is specialized again.
fn toggle_stripes(
    mut materials: ResMut<Assets<WeslMaterial>>,
    mut text: Single<&mut Text, With<PanelText>>,
) {
    for (_, material) in materials.iter_mut() {
        material.rainbow_stripes = !material.rainbow_stripes;
        text.0 = panel_text(material.rainbow_stripes);
    }
}

fn spin(mut shapes: Query<&mut Transform, With<MeshMaterial3d<WeslMaterial>>>, time: Res<Time>) {
    for mut transform in &mut shapes {
        transform.rotate_y(time.delta_secs() * 0.5);
        transform.rotate_x(time.delta_secs() * 0.2);
    }
}
