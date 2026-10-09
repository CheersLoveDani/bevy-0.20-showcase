//! 2D extended materials: `ExtendedMaterial2d<ColorMaterial, Ripple>` keeps `ColorMaterial`'s
//! data and bindings and layers a `MaterialExtension2d` with its own uniform and WESL
//! fragment shader on top (see `assets/render/ripple_extension.wesl`).

use bevy::{
    camera::ClearColorConfig, input::common_conditions::input_just_pressed, prelude::*,
    render::render_resource::AsBindGroup, shader::ShaderRef, sprite_render::Material2dPlugin,
};

use super::{ContentViewport, spawn_panel};
use crate::{demo::Demo, shell::ContentRoot, shell::ShellAppExt};

type RippleMaterial = ExtendedMaterial2d<ColorMaterial, Ripple>;

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::ExtendedMaterial2d)
        .add_plugins(Material2dPlugin::<RippleMaterial>::default())
        .add_systems(OnEnter(Demo::ExtendedMaterial2d), setup)
        .add_systems(
            Update,
            (
                spin,
                shift_base_colors.run_if(input_just_pressed(KeyCode::Space)),
            )
                .run_if(in_state(Demo::ExtendedMaterial2d)),
        );
}

#[derive(AsBindGroup, Reflect, Clone)]
struct Ripple {
    // Bindings start at 20 so they do not collide with the base material's.
    #[uniform(20)]
    ring_color: LinearRgba,
    #[uniform(20)]
    rings: f32,
    #[uniform(20)]
    speed: f32,
}

impl MaterialExtension2d for Ripple {
    fn fragment_shader() -> Option<ShaderRef> {
        Some("render/ripple_extension.wesl".into())
    }
}

fn setup(
    root: Single<Entity, With<ContentRoot>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<RippleMaterial>>,
) {
    let demo = DespawnOnExit(Demo::ExtendedMaterial2d);
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        ContentViewport,
        demo.clone(),
    ));

    let shapes = [
        meshes.add(Circle::new(120.0)),
        meshes.add(RegularPolygon::new(130.0, 6)),
        meshes.add(Rectangle::new(210.0, 210.0)),
        meshes.add(Annulus::new(60.0, 125.0)),
    ];
    for (i, mesh) in shapes.into_iter().enumerate() {
        let hue = i as f32 * 80.0 + 200.0;
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ExtendedMaterial2d {
                base: ColorMaterial::from_color(Hsla::hsl(hue, 0.7, 0.35)),
                extension: Ripple {
                    ring_color: LinearRgba::new(1.0, 1.0, 1.0, 0.75),
                    rings: 2.0 + i as f32,
                    speed: 0.4 + 0.2 * i as f32,
                },
            })),
            Transform::from_xyz((i as f32 - 1.5) * 275.0, -20.0, 0.0),
            demo.clone(),
        ));
    }

    spawn_panel(
        &mut commands,
        *root,
        Demo::ExtendedMaterial2d,
        "Mesh2d shapes drawn with ExtendedMaterial2d<ColorMaterial, Ripple>: the fill is \
         ColorMaterial::color, the rings come from the extension.\n\
         Space: change the base ColorMaterial colors",
        (),
    );
}

fn spin(mut shapes: Query<&mut Transform, With<MeshMaterial2d<RippleMaterial>>>, time: Res<Time>) {
    for mut transform in &mut shapes {
        transform.rotate_z(time.delta_secs() * 0.3);
    }
}

/// Only touches the base material: the extension keeps drawing its rings on top.
fn shift_base_colors(mut materials: ResMut<Assets<RippleMaterial>>) {
    for (_, material) in materials.iter_mut() {
        material.base.color = material.base.color.rotate_hue(50.0);
    }
}
