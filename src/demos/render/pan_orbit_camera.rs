//! `PanOrbitCamera`, the CAD-style camera controller upstreamed from `bevy_editor_cam`.
//! Motions are anchored to the point under the cursor (found with mesh picking), have
//! momentum, and can dolly-zoom between projections or snap to an axis.
//!
//! The controller does not read input devices itself: `steer_camera` below is the whole
//! input mapping, written against `start_orbit` / `start_pan` / `start_zoom` / `end_move`.

use bevy::{
    camera::ClearColorConfig,
    camera_controller::pan_orbit_camera::{
        extensions::{dolly_zoom::DollyZoomTrigger, look_to::LookToTrigger},
        prelude::*,
    },
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::DVec3,
    picking::{
        PickingSystems,
        mesh_picking::{MeshPickingCamera, MeshPickingPlugin, MeshPickingSettings},
        pointer::{PointerId, PointerInteraction, PointerLocation},
    },
    prelude::*,
};

use super::{ContentViewport, spawn_panel};
use crate::{demo::Demo, shell::ContentRoot, shell::ShellAppExt};

pub fn plugin(app: &mut App) {
    if !app.is_plugin_added::<MeshPickingPlugin>() {
        // Only ray cast cameras and meshes that opt in, so other demos are left alone.
        app.add_plugins(MeshPickingPlugin)
            .insert_resource(MeshPickingSettings {
                require_markers: true,
                ..default()
            });
    }
    app.register_demo(Demo::PanOrbitCamera)
        .add_plugins(DefaultPanOrbitCameraPlugins)
        .add_systems(OnEnter(Demo::PanOrbitCamera), setup)
        .add_systems(
            PreUpdate,
            steer_camera
                .after(PickingSystems::Last)
                .before(PanOrbitCamera::update_camera_positions)
                .run_if(in_state(Demo::PanOrbitCamera)),
        )
        .add_systems(
            Update,
            (toggle_projection, snap_to_axis).run_if(in_state(Demo::PanOrbitCamera)),
        );
}

fn setup(
    root: Single<Entity, With<ContentRoot>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let demo = DespawnOnExit(Demo::PanOrbitCamera);
    let eye = Vec3::new(5.0, 4.0, 7.0);
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Transform::from_translation(eye).looking_at(Vec3::Y * 0.5, Vec3::Y),
        // Where to orbit around until the cursor has hit something.
        PanOrbitCamera::default().with_initial_anchor_depth(eye.length() as f64),
        MeshPickingCamera,
        ContentViewport,
        demo.clone(),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 6_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        demo.clone(),
    ));

    let shapes = [
        (meshes.add(Cuboid::from_length(1.4)), 0.7),
        (meshes.add(Sphere::new(0.8).mesh().uv(48, 24)), 0.8),
        (meshes.add(Torus::new(0.35, 0.8)), 0.45),
        (meshes.add(Cylinder::new(0.6, 1.6)), 0.8),
        (meshes.add(Cone::new(0.7, 1.6)), 0.8),
        (meshes.add(Capsule3d::new(0.45, 0.9)), 0.9),
    ];
    for (i, (mesh, height)) in shapes.into_iter().enumerate() {
        let angle = i as f32 / 6.0 * std::f32::consts::TAU;
        let position = Quat::from_rotation_y(angle) * Vec3::X * 2.6 + Vec3::Y * height;
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(Color::hsl(i as f32 * 60.0, 0.65, 0.55))),
            Transform::from_translation(position),
            Pickable::default(),
            demo.clone(),
        ));
    }
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(5.0, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.25, 0.27, 0.32))),
        Transform::from_xyz(0.0, -0.1, 0.0),
        Pickable::default(),
        demo.clone(),
    ));

    spawn_panel(
        &mut commands,
        *root,
        Demo::PanOrbitCamera,
        "Left drag: orbit around the point under the cursor\n\
         Right drag: pan (the grabbed point stays under the cursor)\n\
         Scroll: zoom towards the cursor\n\
         P: dolly-zoom between perspective and orthographic\n\
         1 / 2 / 3: look along X / Y / Z",
        (),
    );
}

/// Maps the mouse onto the controller: start a motion, feed it screen-space deltas while it
/// lasts, and end it. Momentum, smoothing and anchoring are the controller's job.
fn steer_camera(
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    pointers: Query<(&PointerId, &PointerLocation, &PointerInteraction)>,
    pickable_meshes: Query<(), With<Mesh3d>>,
    camera: Single<(&Camera, &GlobalTransform, &mut PanOrbitCamera)>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    let (camera, transform, mut controller) = camera.into_inner();
    let scroll: f32 = wheel
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 150.0,
            MouseScrollUnit::Pixel => wheel.y,
        })
        .sum();
    let Some((cursor, interaction)) = pointers
        .iter()
        .find(|(id, ..)| **id == PointerId::Mouse)
        .and_then(|(_, location, interaction)| Some((location.location()?.position, interaction)))
    else {
        return;
    };
    let cursor_delta = last_cursor
        .replace(cursor)
        .map_or(Vec2::ZERO, |last| cursor - last);

    // A drag ends with its button; a zoom ends once the smoothed scroll input has died down.
    let zoom_settled = controller.current_motion.is_zooming_only()
        && controller.current_motion.inputs().is_some_and(|inputs| {
            inputs.zoom_velocity_abs(controller.smoothing.zoom.mul_f32(2.0)) <= 0.0
        });
    if mouse.any_just_released([MouseButton::Left, MouseButton::Right]) || zoom_settled {
        controller.end_move();
    }

    let over_viewport = camera
        .logical_viewport_rect()
        .is_some_and(|viewport| viewport.contains(cursor));
    if over_viewport && !controller.is_actively_controlled() {
        // The anchor is the picked point in view space. With `None`, the controller reuses
        // the depth of the last anchor.
        let anchor = interaction
            .iter()
            .find(|(entity, _)| pickable_meshes.contains(*entity))
            .and_then(|(_, hit)| hit.position)
            .map(|hit| {
                transform
                    .affine()
                    .inverse()
                    .transform_point3(hit)
                    .as_dvec3()
            });
        if mouse.just_pressed(MouseButton::Left) {
            controller.start_orbit(anchor);
        } else if mouse.just_pressed(MouseButton::Right) {
            controller.start_pan(anchor);
        } else if scroll != 0.0 {
            controller.start_zoom(anchor);
        }
    }

    controller.send_screenspace_input(cursor_delta);
    controller.send_zoom_input(scroll);
}

fn toggle_projection(
    keys: Res<ButtonInput<KeyCode>>,
    camera: Single<(Entity, &Projection), With<PanOrbitCamera>>,
    mut dolly_zoom: MessageWriter<DollyZoomTrigger>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        let (camera, projection) = *camera;
        let target_projection = match projection {
            Projection::Perspective(_) => OrthographicProjection::default_3d().into(),
            _ => PerspectiveProjection::default().into(),
        };
        dolly_zoom.write(DollyZoomTrigger {
            target_projection,
            camera,
        });
    }
}

fn snap_to_axis(
    keys: Res<ButtonInput<KeyCode>>,
    camera: Single<(Entity, &Transform, &PanOrbitCamera)>,
    mut look_to: MessageWriter<LookToTrigger>,
) {
    let (entity, transform, controller) = *camera;
    for (key, facing) in [
        (KeyCode::Digit1, DVec3::NEG_X),
        (KeyCode::Digit2, DVec3::NEG_Y),
        (KeyCode::Digit3, DVec3::NEG_Z),
    ] {
        if keys.just_pressed(key) {
            look_to.write(LookToTrigger::auto_snap_up_direction(
                facing,
                entity,
                &transform.rotation.as_dquat(),
                controller,
            ));
        }
    }
}
