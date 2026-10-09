//! `Val::Em` and `Val::Rem`: UI lengths relative to a node's own font size (`EmSize`, derived
//! from its `TextFont`) and to the root font size (the `RemSize` resource).

use bevy::{prelude::*, text::RemSize, ui_widgets::ValueChange};

use super::{labeled, note, page, slider, BLUE, GREEN, ORANGE, PINK, TEXT};
use crate::{
    demo::Demo,
    shell::{ContentRoot, ShellAppExt},
};

/// Boxes whose `TextFont` is driven by the "local font size" slider.
#[derive(Component, Default, Clone)]
struct LocalFont;

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::EmRem)
        .add_systems(OnEnter(Demo::EmRem), setup)
        // `RemSize` is global: the shell's own rem-sized text scales with it too.
        .add_systems(OnExit(Demo::EmRem), |mut rem: ResMut<RemSize>| {
            *rem = RemSize::default();
        });
}

fn setup(root: Single<Entity, With<ContentRoot>>, mut commands: Commands) {
    commands
        .spawn_scene(ui())
        .insert((ChildOf(*root), DespawnOnExit(Demo::EmRem)));
}

fn ui() -> impl Scene {
    let root_size = bsn! {
        @slider(10.0, 36.0, 20.0)
        on(|change: On<ValueChange<f32>>, mut rem: ResMut<RemSize>| rem.0 = change.value)
    };
    let local_size = bsn! {
        @slider(10.0, 36.0, 20.0)
        on(|change: On<ValueChange<f32>>, mut fonts: Query<&mut TextFont, With<LocalFont>>| {
            for mut font in &mut fonts {
                font.font_size = FontSize::Px(change.value);
            }
        })
    };
    bsn! {
        @page()
        Children [
            @note("Drag the sliders. Each box is one entity: a Text node whose width and height use \
                the unit in its label.\nrem follows the RemSize resource, em follows the TextFont on \
                the same entity, px never moves. The header above is rem-sized too.")
            --
            @labeled("Root font size (RemSize)", root_size)
            --
            @labeled("Local font size (TextFont)", local_size)
            --
            @sized_box("w: px(360) | font: Px(20)", px(360), px(52), FontSize::Px(20.0), BLUE)
            --
            @sized_box("w: rem(18) | font: Rem(1)", rem(18), rem(2.6), FontSize::Rem(1.0), GREEN)
            --
            @sized_box("w: em(18) | font: Px(local)", em(18), em(2.6), FontSize::Px(20.0), ORANGE)
            LocalFont
            --
            @sized_box("w: em(18) | font: Rem(0.8)", em(18), em(2.6), FontSize::Rem(0.8), PINK)
        ]
    }
}

/// A text node sized by `width` / `height`, whose `TextFont` also sets its `EmSize`.
fn sized_box(
    label: &'static str,
    width: Val,
    height: Val,
    font_size: FontSize,
    color: Color,
) -> impl Scene {
    bsn! {
        Text(label)
        TextFont { font_size: font_size }
        TextColor(TEXT)
        Node {
            width: width,
            height: height,
            padding: UiRect::all(px(4)),
            border_radius: BorderRadius::all(px(6)),
        }
        BackgroundColor(color)
    }
}
