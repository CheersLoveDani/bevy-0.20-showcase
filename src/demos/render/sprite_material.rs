//! Sprite materials: `SpriteMaterial<M>` draws a regular `Sprite` with a custom fragment
//! shader. `M` is a `MaterialExtension2d`, and its shader can call the sprite helpers
//! `sample_sprite_texture` / `get_final_color` (see `assets/render/sprite_dissolve.wesl`).

use bevy::{
    camera::ClearColorConfig, input::common_conditions::input_just_pressed, prelude::*,
    render::render_resource::AsBindGroup, shader::ShaderRef,
};

use super::{ContentViewport, image_from_fn, smoothstep, spawn_panel};
use crate::{demo::Demo, shell::ContentRoot, shell::ShellAppExt};

const WOBBLE: f32 = 0.03;

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::SpriteMaterial)
        .add_plugins(SpriteMaterialPlugin::<DissolveMaterial>::default())
        .add_systems(OnEnter(Demo::SpriteMaterial), setup)
        .add_systems(
            Update,
            (
                animate_dissolve,
                toggle_wobble.run_if(input_just_pressed(KeyCode::Space)),
            )
                .run_if(in_state(Demo::SpriteMaterial)),
        );
}

#[derive(AsBindGroup, Asset, Reflect, Clone)]
struct DissolveMaterial {
    // Bindings start at 20 so they do not collide with the sprite's own.
    #[uniform(20)]
    edge_color: LinearRgba,
    #[uniform(20)]
    amount: f32,
    #[uniform(20)]
    wobble: f32,
}

impl MaterialExtension2d for DissolveMaterial {
    fn fragment_shader() -> Option<ShaderRef> {
        Some("render/sprite_dissolve.wesl".into())
    }
}

/// Offsets each sprite's dissolve cycle.
#[derive(Component)]
struct Phase(f32);

fn setup(
    root: Single<Entity, With<ContentRoot>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<DissolveMaterial>>,
) {
    let demo = DespawnOnExit(Demo::SpriteMaterial);
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        ContentViewport,
        demo.clone(),
    ));

    let image = images.add(image_from_fn(256, emblem));
    let sprites = [
        (Color::WHITE, LinearRgba::rgb(4.0, 2.0, 0.2)),
        (Color::srgb(0.6, 1.0, 0.7), LinearRgba::rgb(0.3, 4.0, 1.0)),
        (Color::srgb(1.0, 0.6, 0.9), LinearRgba::rgb(3.0, 0.4, 4.0)),
    ];
    for (i, (tint, edge_color)) in sprites.into_iter().enumerate() {
        commands.spawn((
            Sprite {
                image: image.clone(),
                // The shader's `get_final_color` still applies the sprite's tint.
                color: tint,
                custom_size: Some(Vec2::splat(280.0)),
                ..default()
            },
            SpriteMaterial(materials.add(DissolveMaterial {
                edge_color,
                amount: 0.0,
                wobble: WOBBLE,
            })),
            Transform::from_xyz((i as f32 - 1.0) * 330.0, -20.0, 0.0),
            Phase(i as f32 * 1.3),
            demo.clone(),
        ));
    }

    spawn_panel(
        &mut commands,
        *root,
        Demo::SpriteMaterial,
        "Three ordinary Sprites sharing one generated texture, each with its own \
         SpriteMaterial<DissolveMaterial>.\nSpace: toggle the UV wobble uniform",
        (),
    );
}

/// A ring of coloured wedges on a transparent background.
fn emblem(p: Vec2) -> Srgba {
    let radius = p.length();
    let turn = p.to_angle() / std::f32::consts::TAU + 0.5;
    let wedge = (turn * 8.0).floor();
    let ring = (radius * 4.0).floor();
    let lightness = if (wedge + ring) % 2.0 == 0.0 {
        0.62
    } else {
        0.4
    };
    let alpha = 1.0 - smoothstep(0.86, 0.9, radius);
    Srgba::from(Hsla::new(
        wedge * 45.0 + ring * 20.0,
        0.75,
        lightness,
        alpha,
    ))
}

fn animate_dissolve(
    sprites: Query<(&SpriteMaterial<DissolveMaterial>, &Phase)>,
    mut materials: ResMut<Assets<DissolveMaterial>>,
    time: Res<Time>,
) {
    for (material, phase) in &sprites {
        if let Some(mut material) = materials.get_mut(&material.0) {
            // Stay fully visible for a while, then dissolve and come back.
            let wave = (time.elapsed_secs() * 0.9 + phase.0).sin();
            material.amount = smoothstep(0.0, 1.0, wave);
        }
    }
}

fn toggle_wobble(mut materials: ResMut<Assets<DissolveMaterial>>) {
    for (_, material) in materials.iter_mut() {
        material.wobble = if material.wobble == 0.0 { WOBBLE } else { 0.0 };
    }
}
