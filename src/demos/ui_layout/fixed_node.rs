//! `FixedNode`: a UI node that is laid out against the viewport like a root node, even though
//! it is declared as a child. It ignores its parent's layout, clipping, scrolling and transform.

use bevy::{prelude::*, ui_widgets::ValueChange};

use super::{labeled, note, page, slider, BLUE, DIM, ORANGE, PANEL, PINK, TEXT};
use crate::{
    demo::Demo,
    shell::{ContentRoot, ShellAppExt, HEADER_HEIGHT},
};

const INITIAL_ROTATION: f32 = 6.0;
const INITIAL_SCROLL: f32 = 0.0;

/// The clipped, scrolling, rotated parent.
#[derive(Component, Default, Clone)]
struct Panel;

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::FixedNode)
        .add_systems(OnEnter(Demo::FixedNode), setup);
}

fn setup(root: Single<Entity, With<ContentRoot>>, mut commands: Commands) {
    commands
        .spawn_scene(ui())
        .insert((ChildOf(*root), DespawnOnExit(Demo::FixedNode)));
}

fn ui() -> impl Scene {
    let rotate = bsn! {
        @slider(-30.0, 30.0, INITIAL_ROTATION)
        on(|change: On<ValueChange<f32>>, mut panel: Single<&mut UiTransform, With<Panel>>| {
            panel.rotation = Rot2::degrees(change.value);
        })
    };
    let scroll = bsn! {
        @slider(0.0, 400.0, INITIAL_SCROLL)
        on(|change: On<ValueChange<f32>>, mut panel: Single<&mut ScrollPosition, With<Panel>>| {
            panel.y = change.value;
        })
    };
    bsn! {
        @page()
        Children [
            @note("Both badges are children of a row inside the panel below. The pink one is a \
                normal absolute child. The orange one\nhas FixedNode, so its top / right offsets \
                are measured from the window. Rotate and scroll the panel to compare.")
            --
            @labeled("Rotate parent", rotate)
            --
            @labeled("Scroll parent", scroll)
            --
            @panel()
        ]
    }
}

fn panel() -> impl Scene {
    let rows: Vec<_> = (1..=16).map(row).collect();
    bsn! {
        Panel
        Node {
            width: px(460),
            height: px(320),
            margin: UiRect::new(px(60), px(0), px(40), px(0)),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(10)),
            row_gap: px(8),
            border: UiRect::all(px(2)),
            overflow: Overflow::scroll_y(),
        }
        UiTransform { rotation: Rot2::degrees(INITIAL_ROTATION) }
        ScrollPosition(Vec2::new(0.0, INITIAL_SCROLL))
        BackgroundColor(PANEL)
        BorderColor::all(BLUE)
        Children [
            // The badges live two levels down: panel > row > badge.
            @row(0)
            Node { min_height: px(72) }
            Children [
                @badge("Absolute child\nmoves with its parent", PINK)
                Node { position_type: PositionType::Absolute, right: px(8), top: px(8) }
                --
                @badge("FixedNode\ntop / right relative to the viewport", ORANGE)
                FixedNode
                Node {
                    position_type: PositionType::Absolute,
                    top: px(HEADER_HEIGHT + 150.0),
                    right: px(40),
                }
            ]
            --
            {rows}
        ]
    }
}

fn row(index: usize) -> impl Scene {
    bsn! {
        Node {
            min_height: px(36),
            padding: UiRect::horizontal(px(10)),
            align_items: AlignItems::Center,
        }
        BackgroundColor(Color::srgb(0.19, 0.2, 0.26))
        Children [
            Text({format!("Scrolling row {index}")})
            TextFont { font_size: FontSize::Px(15.0) }
            TextColor(DIM)
        ]
    }
}

fn badge(text: &'static str, color: Color) -> impl Scene {
    bsn! {
        Node {
            width: px(250),
            padding: UiRect::all(px(10)),
            border_radius: BorderRadius::all(px(8)),
        }
        BackgroundColor(color)
        Children [
            Text(text)
            TextFont { font_size: FontSize::Px(14.0) }
            TextColor(TEXT)
        ]
    }
}
