//! 2D and 3D rendering demos, plus the few helpers they share.

mod extended_material_2d;
mod pan_orbit_camera;
mod sprite_material;
mod sprite_stress;
mod texture_compression;
mod wesl_material;

use bevy::{
    asset::RenderAssetUsages,
    camera::Viewport,
    ecs::system::IntoObserverSystem,
    feathers::{controls::FeathersButton, display::caption},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    ui_widgets::Activate,
    window::PrimaryWindow,
};

use crate::{
    demo::Demo,
    shell::{HEADER_HEIGHT, SIDEBAR_WIDTH},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, fit_viewport_to_content);
    sprite_material::plugin(app);
    extended_material_2d::plugin(app);
    sprite_stress::plugin(app);
    pan_orbit_camera::plugin(app);
    wesl_material::plugin(app);
    texture_compression::plugin(app);
}

/// Put this on a demo camera to restrict its viewport to the area that is not covered by
/// the shell's sidebar and header, so the world origin sits in the middle of the visible scene.
#[derive(Component, Default, Clone)]
struct ContentViewport;

fn fit_viewport_to_content(
    window: Single<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<ContentViewport>>,
) {
    let offset = (Vec2::new(SIDEBAR_WIDTH, HEADER_HEIGHT) * window.scale_factor()).as_uvec2();
    let size = window.physical_size().saturating_sub(offset);
    if size.min_element() == 0 {
        return;
    }
    for mut camera in &mut cameras {
        let up_to_date = camera
            .viewport
            .as_ref()
            .is_some_and(|v| v.physical_position == offset && v.physical_size == size);
        if !up_to_date {
            camera.viewport = Some(Viewport {
                physical_position: offset,
                physical_size: size,
                ..default()
            });
        }
    }
}

/// The text component of a demo's panel, for demos that update it while running.
#[derive(Component, Default, Clone)]
struct PanelText;

/// Spawns a small panel in the top-left corner of the content area: a line or two of
/// text saying what to try, followed by a row of buttons.
fn spawn_panel(
    commands: &mut Commands,
    root: Entity,
    demo: Demo,
    text: impl Into<String>,
    buttons: impl SceneList,
) {
    let text = text.into();
    commands
        .spawn_scene(bsn! {
            Node {
                align_self: AlignSelf::Start,
                max_width: px(780),
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(px(12)),
                padding: UiRect::all(px(12)),
                row_gap: px(10),
                border_radius: BorderRadius::all(px(6)),
            }
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6))
            // Clicks on the panel fall through to the scene; its buttons still work.
            Pickable::IGNORE
            Children [
                PanelText
                Text({text.clone()})
                TextFont { font_size: FontSize::Rem(0.85) }
                TextColor(Color::srgb(0.85, 0.86, 0.9))
                Pickable::IGNORE
                --
                Node { column_gap: px(8) }
                Pickable::IGNORE
                Children [ {buttons} ]
            ]
        })
        .insert((ChildOf(root), DespawnOnExit(demo)));
}

/// A Feathers button that runs `action` (any observer system) when pressed.
fn button<M: 'static>(
    label: &'static str,
    action: impl IntoObserverSystem<Activate, M> + Clone + Sync,
) -> impl Scene {
    bsn! {
        @FeathersButton
        on(action.clone())
        Children [ @caption(label) ]
    }
}

/// Builds a square sRGB texture by evaluating `pixel` for every texel.
/// The position passed to `pixel` runs from -1 to 1 on both axes.
fn image_from_fn(size: u32, pixel: impl Fn(Vec2) -> Srgba) -> Image {
    let data = (0..size * size)
        .flat_map(|i| {
            let texel = UVec2::new(i % size, i / size).as_vec2() + 0.5;
            pixel(texel / size as f32 * 2.0 - 1.0).to_u8_array()
        })
        .collect();
    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Hermite interpolation from 0 to 1 as `x` goes from `start` to `end`, as in WGSL.
fn smoothstep(start: f32, end: f32, x: f32) -> f32 {
    let t = ((x - start) / (end - start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
