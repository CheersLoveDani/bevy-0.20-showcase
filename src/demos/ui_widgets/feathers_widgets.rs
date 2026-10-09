//! The Feathers widgets added in Bevy 0.20: `FeathersColorInput` (a swatch that opens a color
//! picker popup), `FeathersListView`, `FeathersSelect`, `FeathersLazyMenu` (its popup only
//! exists while it is open) and the scrubbable `FeathersNumberInput`.

use std::sync::Arc;

use bevy::{
    color::palettes::tailwind,
    feathers::{
        controls::*,
        display::{caption, label, label_small},
        tokens,
    },
    input_focus::tab_navigation::TabGroup,
    prelude::*,
    ui_widgets::{Activate, ValueChange, listbox_update_selection},
};

use super::card;
use crate::{
    demo::Demo,
    shell::{ContentRoot, DemoLog, ShellAppExt},
};

const START_TINT: Srgba = tailwind::AMBER_500;
const PLANETS: [&str; 8] = [
    "Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune",
];
const BLEND_MODES: [&str; 4] = ["Opaque", "Alpha blend", "Additive", "Multiply"];

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::FeathersWidgets)
        .add_systems(OnEnter(Demo::FeathersWidgets), setup)
        .add_systems(
            Update,
            update_popup_status.run_if(in_state(Demo::FeathersWidgets)),
        );
}

/// The node tinted by the color input.
#[derive(Component, Default, Clone)]
struct TintPreview;

/// Tags a menu and the label reporting how many popup entities that menu currently has.
#[derive(Component, Default, Clone, Copy, PartialEq)]
enum MenuKind {
    #[default]
    Eager,
    Lazy,
}

fn setup(root: Single<Entity, With<ContentRoot>>, mut commands: Commands) {
    commands
        .spawn_scene(scene())
        .insert((ChildOf(*root), DespawnOnExit(Demo::FeathersWidgets)));
}

fn scene() -> impl Scene {
    bsn! {
        Node {
            padding: UiRect::all(px(20)),
            column_gap: px(16),
            row_gap: px(16),
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Start,
            align_content: AlignContent::Start,
        }
        TabGroup
        Children [
            @list_card()
            --
            @color_card()
            --
            @select_card()
            --
            @menu_card()
            --
            @number_card()
        ]
    }
}

fn color_card() -> impl Scene {
    card(
        "FeathersColorInput",
        "Click the swatch to open the picker popup: wheel / RGB / HSL modes, hex entry and recent colors.",
        250.0,
        bsn_list! {
            @row()
            Children [
                @label("Tint")
                --
                @FeathersColorInput
                ColorInputValue(START_TINT)
                on(color_input_self_update)
                on(apply_tint)
            ]
            --
            TintPreview
            Node {
                height: px(56),
                border_radius: BorderRadius::all(px(6)),
            }
            BackgroundColor(START_TINT)
        },
    )
}

fn apply_tint(
    change: On<ValueChange<Color>>,
    mut preview: Single<&mut BackgroundColor, With<TintPreview>>,
    mut log: ResMut<DemoLog>,
) {
    preview.0 = change.value;
    if change.is_final {
        log.push(format!("Color input: {}", change.value.to_srgba().to_hex()));
    }
}

fn list_card() -> impl Scene {
    card(
        "FeathersListView",
        "A scrolling, single-selection list. Click a row, or focus the list and use the arrow keys.",
        250.0,
        bsn_list! {
            @FeathersListView {
                @rows: list_rows_from_strings(PLANETS, Some(2)),
            }
            Node { height: px(130) }
            on(listbox_update_selection)
            on(log_choice("List view", &PLANETS))
        },
    )
}

fn select_card() -> impl Scene {
    card(
        "FeathersSelect",
        "A dropdown built from a menu popup and a list view. It scrolls past @max_visible rows.",
        250.0,
        bsn_list! {
            @FeathersSelect {
                @options: list_rows_from_strings(BLEND_MODES, Some(0)),
            }
            on(log_choice("Select", &BLEND_MODES))
            --
            @FeathersSelect {
                @options: list_rows_from_strings(PLANETS, Some(2)),
                @max_visible: 4,
            }
            on(log_choice("Select", &PLANETS))
        },
    )
}

/// Both the list view and the select report the chosen row entity; `list_rows_from_strings`
/// tagged each row with its [`OptionIndex`].
fn log_choice(
    widget: &'static str,
    names: &'static [&'static str],
) -> impl Fn(On<ValueChange<Entity>>, Query<&OptionIndex>, ResMut<DemoLog>) + Clone {
    move |change, rows, mut log| {
        if let Ok(index) = rows.get(change.value) {
            log.push(format!("{widget}: {}", names[index.0]));
        }
    }
}

fn menu_card() -> impl Scene {
    // `FeathersLazyMenu` calls this every time it opens, and despawns the result on close.
    let popup: Arc<dyn Fn() -> Box<dyn Scene> + Sync + Send> =
        Arc::new(|| Box::new(bsn! { @FeathersMenuPopup Children [ {menu_items("Lazy")} ] }));

    card(
        "FeathersLazyMenu",
        "FeathersMenu keeps a hidden popup around. FeathersLazyMenu spawns it on open and despawns it on close.",
        290.0,
        bsn_list! {
            @row()
            Children [
                @FeathersMenu
                MenuKind::Eager
                Children [
                    @FeathersMenuButton { @caption: bsn! { @caption("Menu") } }
                    --
                    @FeathersMenuPopup Children [ {menu_items("Eager")} ]
                ]
                --
                MenuKind::Eager
                @label_small("")
            ]
            --
            @row()
            Children [
                @FeathersLazyMenu { popup }
                MenuKind::Lazy
                Children [
                    @FeathersMenuButton { @caption: bsn! { @caption("Lazy menu") } }
                ]
                --
                MenuKind::Lazy
                @label_small("")
            ]
        },
    )
}

fn menu_items(menu: &'static str) -> impl SceneList {
    ["Cut", "Copy", "Paste"]
        .into_iter()
        .map(|item| {
            bsn! {
                @FeathersMenuItem { @caption: bsn! { @caption(item) } }
                on(move |_: On<Activate>, mut log: ResMut<DemoLog>| {
                    log.push(format!("{menu} menu: {item}"));
                })
            }
        })
        .collect::<Vec<_>>()
}

/// Shows, next to each menu, whether its popup entity currently exists.
fn update_popup_status(
    menus: Query<(&MenuKind, &Children), Without<Text>>,
    popups: Query<(), With<FeathersMenuPopup>>,
    mut labels: Query<(&MenuKind, &mut Text)>,
) {
    for (kind, mut text) in &mut labels {
        let alive = menus
            .iter()
            .filter(|(menu_kind, _)| *menu_kind == kind)
            .flat_map(|(_, children)| children.iter())
            .filter(|child| popups.contains(*child))
            .count();
        let status = format!("popup entities: {alive}");
        if text.0 != status {
            text.0 = status;
        }
    }
}

fn number_card() -> impl Scene {
    card(
        "FeathersNumberInput",
        "Drag sideways to scrub, click to type, or use the end chevrons.",
        290.0,
        bsn_list! {
            @label_small("Scrubber (no limits)")
            --
            @FeathersNumberInput
            NumberInputValue::F32(1.5)
            NumberInputPrecision(2)
            on(number_input_self_update)
            --
            @label_small("SoftLimit(0..=100) makes it a slider")
            --
            @FeathersNumberInput
            NumberInputValue::F32(40.0)
            NumberInputPrecision(1)
            SoftLimit::f32(0.0..=100.0)
            on(number_input_self_update)
            --
            @label_small("Axis sigil, HardLimit(-180..=180), wrapping")
            --
            @FeathersNumberInput {
                @sigil_color: tokens::TEXT_INPUT_X_AXIS,
                @label_text: "X",
            }
            NumberInputValue::F32(90.0)
            NumberInputPrecision(0)
            HardLimit::f32(-180.0..=180.0)
            NumberInputWrap::Wrap
            on(number_input_self_update)
        },
    )
}

/// Feathers widgets are controlled: the app writes the requested value back to the widget.
fn number_input_self_update(
    change: On<ValueChange<f32>>,
    mut commands: Commands,
    mut log: ResMut<DemoLog>,
) {
    commands
        .entity(change.source)
        .insert(NumberInputValue::F32(change.value));
    if change.is_final {
        log.push(format!("Number input: {:.2}", change.value));
    }
}

fn row() -> impl Scene {
    bsn! {
        Node {
            align_items: AlignItems::Center,
            column_gap: px(10),
        }
    }
}
