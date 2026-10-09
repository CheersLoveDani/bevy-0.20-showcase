//! The panel shared by the ECS demos: an explanation, a row of buttons and a monospace
//! readout that the active demo fills with its results.

use bevy::{
    ecs::system::IntoObserverSystem,
    feathers::{constants::fonts, controls::FeathersButton, display::caption},
    prelude::*,
    text::FontSourceTemplate,
    ui_widgets::Activate,
};

use crate::demo::Demo;

/// The results of the active demo. Whatever is written here is shown in the panel.
#[derive(Resource, Default)]
pub struct Readout(pub String);

#[derive(Component, Default, Clone)]
struct ReadoutText;

pub fn plugin(app: &mut App) {
    app.init_resource::<Readout>()
        .add_systems(Update, show_readout);
}

/// Spawns the panel for `demo` into the shell's content area.
pub fn spawn_panel(
    commands: &mut Commands,
    root: Entity,
    demo: Demo,
    intro: &'static str,
    buttons: impl SceneList,
) {
    commands
        .spawn_scene(panel(intro, buttons))
        .insert((ChildOf(root), DespawnOnExit(demo)));
}

/// A Feathers button that runs `action` (any observer system) when pressed.
pub fn button<M: 'static>(
    label: &'static str,
    action: impl IntoObserverSystem<Activate, M> + Clone + Sync,
) -> impl Scene {
    bsn! {
        @FeathersButton
        on(action)
        Children [ @caption(label) ]
    }
}

fn panel(intro: &'static str, buttons: impl SceneList) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(20)),
            row_gap: px(14),
        }
        Children [
            Text(intro)
            TextFont {
                font: FontSourceTemplate::Handle(fonts::REGULAR),
                font_size: FontSize::Rem(0.95),
            }
            TextColor(Color::srgb(0.85, 0.86, 0.9))
            --
            Node { column_gap: px(8) }
            Children [ {buttons} ]
            --
            ReadoutText
            Text("")
            TextFont {
                font: FontSourceTemplate::Handle(fonts::MONO),
                font_size: FontSize::Rem(0.8),
            }
        ]
    }
}

fn show_readout(readout: Res<Readout>, mut text: Single<&mut Text, With<ReadoutText>>) {
    if text.0 != readout.0 {
        text.0.clone_from(&readout.0);
    }
}
