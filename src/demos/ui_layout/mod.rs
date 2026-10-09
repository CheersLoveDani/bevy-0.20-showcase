//! UI layout and scene demos: em / rem units, `FixedNode`, elliptical corners, the BSN syntax
//! changes and the `Ready` event.

use bevy::{
    feathers::controls::FeathersSlider,
    prelude::*,
    ui_widgets::{slider_self_update, SliderPrecision, SliderStep, SliderValue},
};

mod border_radius;
mod bsn_syntax;
mod em_rem;
mod fixed_node;
mod ready_event;

pub fn plugin(app: &mut App) {
    em_rem::plugin(app);
    fixed_node::plugin(app);
    border_radius::plugin(app);
    bsn_syntax::plugin(app);
    ready_event::plugin(app);
}

const TEXT: Color = Color::srgb(0.9, 0.91, 0.94);
const DIM: Color = Color::srgb(0.66, 0.68, 0.75);
const PANEL: Color = Color::srgb(0.13, 0.14, 0.18);
const BLUE: Color = Color::srgb(0.25, 0.47, 0.85);
const GREEN: Color = Color::srgb(0.2, 0.6, 0.42);
const ORANGE: Color = Color::srgb(0.85, 0.5, 0.2);
const PINK: Color = Color::srgb(0.78, 0.3, 0.55);

/// The column every demo in this group puts its content in.
fn page() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(24)),
            row_gap: px(18),
        }
    }
}

/// Explanatory text. Sized in pixels so it stays put while demos change the root font size.
fn note(text: impl Into<String>) -> impl Scene {
    bsn! {
        Text(text)
        TextFont { font_size: FontSize::Px(15.0) }
        TextColor(DIM)
    }
}

/// A fixed-width Feathers slider with whole-number steps that keeps its own value. Add an
/// `on(|change: On<ValueChange<f32>>| ..)` observer next to it to react.
fn slider(min: f32, max: f32, value: f32) -> impl Scene {
    bsn! {
        @FeathersSlider { @min: {min}, @max: {max} }
        Node { width: px(220), flex_grow: 0.0 }
        SliderValue(value)
        SliderStep(1.0)
        SliderPrecision(0)
        on(slider_self_update)
    }
}

/// A `label` followed by a control, laid out in a row.
fn labeled(label: &'static str, control: impl Scene) -> impl Scene {
    bsn! {
        Node { align_items: AlignItems::Center, column_gap: px(12) }
        Children [
            Text(label)
            TextFont { font_size: FontSize::Px(15.0) }
            TextColor(TEXT)
            --
            @{control}
        ]
    }
}
