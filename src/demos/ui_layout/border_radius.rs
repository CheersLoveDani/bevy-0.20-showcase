//! Elliptical border radius: each corner of a `BorderRadius` is now a `CornerRadius` with
//! separate horizontal (`x`) and vertical (`y`) radii.

use bevy::{prelude::*, ui_widgets::ValueChange};

use super::{labeled, note, page, slider, BLUE, DIM, GREEN, ORANGE, PINK, TEXT};
use crate::{
    demo::Demo,
    shell::{ContentRoot, ShellAppExt},
};

const INITIAL_X: f32 = 120.0;
const INITIAL_Y: f32 = 40.0;

/// The box whose corners are driven by the two sliders.
#[derive(Component, Default, Clone)]
struct Preview;

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::BorderRadius)
        .add_systems(OnEnter(Demo::BorderRadius), setup);
}

fn setup(root: Single<Entity, With<ContentRoot>>, mut commands: Commands) {
    commands
        .spawn_scene(ui())
        .insert((ChildOf(*root), DespawnOnExit(Demo::BorderRadius)));
}

fn ui() -> impl Scene {
    let radius_x = bsn! {
        @slider(0.0, 180.0, INITIAL_X)
        on(|change: On<ValueChange<f32>>, mut preview: Single<&mut Node, With<Preview>>| {
            for corner in corners(&mut preview.border_radius) {
                corner.x = px(change.value);
            }
        })
    };
    let radius_y = bsn! {
        @slider(0.0, 110.0, INITIAL_Y)
        on(|change: On<ValueChange<f32>>, mut preview: Single<&mut Node, With<Preview>>| {
            for corner in corners(&mut preview.border_radius) {
                corner.y = px(change.value);
            }
        })
    };
    bsn! {
        @page()
        Children [
            @note("A Val still converts into a circular corner. A pair of Vals, or \
                CornerRadius::new(x, y), gives an elliptical one.")
            --
            Node { column_gap: px(16), align_items: AlignItems::FlexStart }
            Children [
                @shape("all(px(30))", 140.0, 100.0, BLUE, BorderRadius::all(px(30)))
                --
                @shape("all([px(65), px(25)])", 140.0, 100.0, GREEN,
                    BorderRadius::all([px(65), px(25)]))
                --
                @shape("MAX", 140.0, 100.0, BLUE, BorderRadius::MAX)
                --
                @shape("MAX_ELLIPTICAL", 140.0, 100.0, GREEN, BorderRadius::MAX_ELLIPTICAL)
                --
                @shape("new(px(8), [px(90), px(30)],\n  px(45), [px(25), px(70)])", 140.0, 100.0, ORANGE,
                    BorderRadius::new(px(8), [px(90), px(30)], px(45), [px(25), px(70)]))
                --
                @shape("top([percent(50), percent(65)])\n.with_bottom([.., percent(35)])",
                    80.0, 100.0, PINK,
                    BorderRadius::top([percent(50), percent(65)])
                        .with_bottom([percent(50), percent(35)]))
            ]
            --
            @note("Drag to set every corner of the box below to CornerRadius::new(px(x), px(y)).")
            --
            Node { column_gap: px(32) }
            Children [
                @labeled("x radius", radius_x)
                --
                @labeled("y radius", radius_y)
            ]
            --
            Preview
            Node {
                width: px(380),
                height: px(230),
                border: UiRect::all(px(8)),
                border_radius: BorderRadius::all(CornerRadius::new(px(INITIAL_X), px(INITIAL_Y))),
            }
            BackgroundColor(BLUE)
            BorderColor::all(TEXT)
        ]
    }
}

fn corners(radius: &mut BorderRadius) -> [&mut CornerRadius; 4] {
    [
        &mut radius.top_left,
        &mut radius.top_right,
        &mut radius.bottom_right,
        &mut radius.bottom_left,
    ]
}

/// A bordered box with the given corners, and the `BorderRadius` constructor used below it.
fn shape(
    label: &'static str,
    width: f32,
    height: f32,
    color: Color,
    radius: BorderRadius,
) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            row_gap: px(8),
            align_items: AlignItems::Center,
        }
        Children [
            Node {
                width: px(width),
                height: px(height),
                border: UiRect::all(px(5)),
                border_radius: radius,
            }
            BackgroundColor(color)
            BorderColor::all(TEXT)
            --
            Text(label)
            TextFont { font_size: FontSize::Px(12.0) }
            TextColor(DIM)
        ]
    }
}
