//! BSN syntax improvements: `@` on every scene reference (`@scene_fn()`, `@{expr}`,
//! `@SceneComponent { @prop: .. }`), `--` between entities, enum variants written like plain
//! Rust, and `#Name` entity references.

use bevy::{
    feathers::{controls::FeathersButton, display::caption},
    prelude::*,
    ui_widgets::Activate,
};

use super::{note, page, BLUE, DIM, GREEN, ORANGE, PANEL, PINK, TEXT};
use crate::{
    demo::Demo,
    shell::{ContentRoot, ShellAppExt},
};

const LAMP_OFF: Color = Color::srgb(0.25, 0.25, 0.3);
const LAMP_ON: Color = Color::srgb(1.0, 0.85, 0.3);

/// A plain `Default + Clone` enum. 0.19 needed `#[derive(VariantDefaults)]` to name a variant
/// in `bsn!`; now variants are written exactly as in Rust, with every field given.
#[derive(Component, Default, Clone)]
#[require(Node, BackgroundColor)]
enum Meter {
    #[default]
    Empty,
    Filled {
        percent: f32,
        color: Color,
    },
}

/// A scene component whose scene is parameterized by props, set with `@prop: value`.
#[derive(SceneComponent, Default, Clone)]
#[scene(CardProps)]
struct Card;

#[derive(Default)]
struct CardProps {
    title: &'static str,
    accent: Color,
}

impl Card {
    fn scene(props: CardProps) -> impl Scene {
        bsn! {
            Node {
                padding: UiRect::axes(px(14), px(8)),
                border: UiRect::left(px(6)),
            }
            BackgroundColor(Color::srgb(0.2, 0.21, 0.27))
            BorderColor::all(props.accent)
            Children [ @text(props.title, 15.0, TEXT) ]
        }
    }
}

/// Points at another entity of the same scene. `#Name` resolves to the `Entity` at spawn time.
#[derive(Component, FromTemplate)]
struct Switches(Entity);

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::BsnSyntax)
        .add_systems(OnEnter(Demo::BsnSyntax), setup)
        .add_systems(Update, fill_meters.run_if(in_state(Demo::BsnSyntax)));
}

fn setup(root: Single<Entity, With<ContentRoot>>, mut commands: Commands) {
    commands
        .spawn_scene(ui())
        .insert((ChildOf(*root), DespawnOnExit(Demo::BsnSyntax)));
}

fn ui() -> impl Scene {
    let chips = bsn_list! {
        @chip("Ok")
        --
        @chip("Cancel")
        --
        @chip("Help")
    };

    let ring = |enabled: bool| {
        enabled.then(|| bsn! { Outline { width: px(3), offset: px(3), color: ORANGE } })
    };
    let optional = bsn_list! {
        @chip("ring(false)")
        @{ring(false)}
        --
        @chip("ring(true)")
        @{ring(true)}
    };

    let meters = bsn_list! {
        @meter_track()
        Children [ Meter::Empty ]
        --
        @meter_track()
        Children [ Meter::Filled { percent: 70.0, color: GREEN } ]
    };

    let cards = bsn_list! {
        @Card { @title: "Blue card", @accent: BLUE }
        --
        @Card { @title: "Pink card", @accent: PINK }
    };

    let lamp = bsn_list! {
        #Lamp
        Node { width: px(34), height: px(34), border_radius: BorderRadius::MAX }
        BackgroundColor(LAMP_OFF)
        --
        @FeathersButton
        Switches(#Lamp)
        on(toggle_lamp)
        Children [ @caption("Toggle #Lamp") ]
    };

    bsn! {
        @page()
        Node { row_gap: px(10) }
        Children [
            @note("Each row shows a piece of 0.20 BSN and, on the right, the entities it spawns. \
                The rows are themselves separated by `--`.")
            --
            @example(
                "@scene_fn() and the -- separator",
                "@chip(\"Ok\")\n--\n@chip(\"Cancel\")\n--\n@chip(\"Help\")",
                chips,
            )
            --
            @example(
                "@{expr}: any expression that is a Scene",
                "@chip(\"ring(true)\")\n@{enabled.then(|| bsn! { Outline { .. } })}",
                optional,
            )
            --
            @example(
                "Enum variants, no VariantDefaults",
                "Meter::Empty\n--\nMeter::Filled { percent: 70.0, color: GREEN }",
                meters,
            )
            --
            @example(
                "@SceneComponent { @prop: value }",
                "@Card { @title: \"Blue card\", @accent: BLUE }",
                cards,
            )
            --
            @example(
                "#Name entity references",
                "#Lamp Node { .. }\n--\n@FeathersButton Switches(#Lamp)",
                lamp,
            )
        ]
    }
}

/// One row: the syntax and a snippet on the left, the spawned `live` entities on the right.
fn example(title: &'static str, snippet: &'static str, live: impl SceneList) -> impl Scene {
    bsn! {
        Node {
            padding: UiRect::axes(px(16), px(10)),
            column_gap: px(24),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(8)),
        }
        BackgroundColor(PANEL)
        Children [
            Node { width: px(430), flex_direction: FlexDirection::Column, row_gap: px(4) }
            Children [
                @text(title, 16.0, TEXT)
                --
                @text(snippet, 13.0, DIM)
            ]
            --
            Node { column_gap: px(14), align_items: AlignItems::Center }
            Children [ {live} ]
        ]
    }
}

fn text(text: &'static str, size: f32, color: Color) -> impl Scene {
    bsn! {
        Text(text)
        TextFont { font_size: FontSize::Px(size) }
        TextColor(color)
    }
}

fn chip(label: &'static str) -> impl Scene {
    bsn! {
        Node { padding: UiRect::axes(px(14), px(6)), border_radius: BorderRadius::MAX }
        BackgroundColor(BLUE)
        Children [ @text(label, 14.0, TEXT) ]
    }
}

fn meter_track() -> impl Scene {
    bsn! {
        Node { width: px(180), height: px(16) }
        BackgroundColor(Color::srgb(0.07, 0.07, 0.09))
    }
}

fn fill_meters(mut meters: Query<(&Meter, &mut Node, &mut BackgroundColor), Changed<Meter>>) {
    for (meter, mut node, mut background) in &mut meters {
        if let Meter::Filled { percent: filled, color } = *meter {
            node.width = percent(filled);
            background.0 = color;
        }
    }
}

fn toggle_lamp(
    activate: On<Activate>,
    switches: Query<&Switches>,
    mut lamps: Query<&mut BackgroundColor>,
) -> Result {
    let mut lamp = lamps.get_mut(switches.get(activate.entity)?.0)?;
    lamp.0 = if lamp.0 == LAMP_OFF { LAMP_ON } else { LAMP_OFF };
    Ok(())
}
