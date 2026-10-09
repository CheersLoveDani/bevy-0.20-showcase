//! Sprite backend unification: `Sprite` is now drawn by the retained `Mesh2d` / `Material2d`
//! renderer, the same infrastructure as 3D meshes. There is nothing new to call, so this is a
//! stress scene: thousands of moving sprites, plus the one new field, `Sprite::alpha_mode`.

use bevy::{
    camera::ClearColorConfig,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::common_conditions::input_just_pressed,
    prelude::*,
    sprite::SpriteAlphaMode,
    ui_widgets::Activate,
};
use rand::Rng;

use super::{ContentViewport, PanelText, button, image_from_fn, smoothstep, spawn_panel};
use crate::{demo::Demo, shell::ContentRoot, shell::ShellAppExt};

const BALL_SIZE: f32 = 28.0;
const GRAVITY: f32 = 900.0;

pub fn plugin(app: &mut App) {
    if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    }
    app.register_demo(Demo::SpriteStress)
        .add_systems(OnEnter(Demo::SpriteStress), setup)
        .add_systems(
            Update,
            (
                add_on_space.run_if(input_just_pressed(KeyCode::Space)),
                cycle_alpha_mode.run_if(input_just_pressed(KeyCode::KeyM)),
                bounce,
                update_readout,
            )
                .run_if(in_state(Demo::SpriteStress)),
        );
}

#[derive(Resource)]
struct BallTexture(Handle<Image>);

#[derive(Component)]
struct Velocity(Vec2);

fn setup(
    root: Single<Entity, With<ContentRoot>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        ContentViewport,
        DespawnOnExit(Demo::SpriteStress),
    ));

    let texture = BallTexture(images.add(image_from_fn(32, ball)));
    spawn_balls(&mut commands, &texture, 5_000);
    commands.insert_resource(texture);

    spawn_panel(
        &mut commands,
        *root,
        Demo::SpriteStress,
        "",
        bsn_list! {
            @button("+1,000", |_: On<Activate>, mut commands: Commands, texture: Res<BallTexture>| {
                spawn_balls(&mut commands, &texture, 1_000);
            })
            --
            @button("+10,000", |_: On<Activate>, mut commands: Commands, texture: Res<BallTexture>| {
                spawn_balls(&mut commands, &texture, 10_000);
            })
            --
            @button("Clear", |_: On<Activate>, mut commands: Commands, balls: Query<Entity, With<Velocity>>| {
                for ball in &balls {
                    commands.entity(ball).despawn();
                }
            })
        },
    );
}

/// A shaded disc with a soft transparent edge.
fn ball(p: Vec2) -> Srgba {
    let shade = 1.0 - 0.5 * p.distance(Vec2::new(-0.3, -0.3));
    let alpha = 1.0 - smoothstep(0.8, 0.95, p.length());
    Srgba::new(shade, shade, shade, alpha)
}

fn spawn_balls(commands: &mut Commands, texture: &BallTexture, count: usize) {
    // Sprites with the same image and color share one material, so stick to a small palette.
    const HUES: [f32; 6] = [0.0, 40.0, 130.0, 190.0, 260.0, 320.0];
    let mut rng = rand::rng();
    let balls: Vec<_> = (0..count)
        .map(|i| {
            let direction = Vec2::from_angle(rng.random_range(0.0..std::f32::consts::TAU));
            (
                Sprite {
                    image: texture.0.clone(),
                    color: Hsla::hsl(HUES[i % HUES.len()], 0.8, 0.6).into(),
                    custom_size: Some(Vec2::splat(BALL_SIZE)),
                    ..default()
                },
                // Overlapping sprites need distinct depths to have a stable draw order.
                Transform::from_xyz(
                    rng.random_range(-400.0..400.0),
                    rng.random_range(-250.0..250.0),
                    rng.random_range(0.0..1.0),
                ),
                Velocity(direction * rng.random_range(100.0..500.0)),
                DespawnOnExit(Demo::SpriteStress),
            )
        })
        .collect();
    commands.spawn_batch(balls);
}

fn add_on_space(mut commands: Commands, texture: Res<BallTexture>) {
    spawn_balls(&mut commands, &texture, 1_000);
}

/// `Sprite::alpha_mode` is new in 0.20: sprites no longer have to be alpha blended.
fn cycle_alpha_mode(mut sprites: Query<&mut Sprite, With<Velocity>>) {
    for mut sprite in &mut sprites {
        sprite.alpha_mode = match sprite.alpha_mode {
            SpriteAlphaMode::Blend => SpriteAlphaMode::Mask(0.5),
            SpriteAlphaMode::Mask(_) => SpriteAlphaMode::Opaque,
            SpriteAlphaMode::Opaque => SpriteAlphaMode::Blend,
        };
    }
}

fn bounce(
    camera: Single<&Camera, With<ContentViewport>>,
    mut balls: Query<(&mut Transform, &mut Velocity)>,
    time: Res<Time>,
) {
    let Some(view_size) = camera.logical_viewport_size() else {
        return;
    };
    let limit = (view_size - BALL_SIZE) / 2.0;
    let dt = time.delta_secs();
    for (mut transform, mut velocity) in &mut balls {
        velocity.0.y -= GRAVITY * dt;
        let position = transform.translation.xy() + velocity.0 * dt;
        // Reflect the velocity off whichever walls the ball has passed.
        if position.x.abs() > limit.x {
            velocity.0.x = -velocity.0.x.abs() * position.x.signum();
        }
        if position.y.abs() > limit.y {
            velocity.0.y = -velocity.0.y.abs() * position.y.signum();
        }
        transform.translation = position
            .clamp(-limit, limit)
            .extend(transform.translation.z);
    }
}

fn update_readout(
    mut text: Single<&mut Text, With<PanelText>>,
    sprites: Query<&Sprite, With<Velocity>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or_default();
    let alpha_mode = sprites.iter().next().map(|sprite| sprite.alpha_mode);
    text.0 = format!(
        "{} sprites   {fps:.0} fps   alpha_mode: {:?}\n\
         Space: add 1,000   M: cycle Sprite::alpha_mode (Blend / Mask / Opaque)",
        sprites.iter().len(),
        alpha_mode.unwrap_or_default(),
    );
}
