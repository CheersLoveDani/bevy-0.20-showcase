//! Contextual theming, new in Feathers for Bevy 0.20: a `ThemeContext` propagated from a
//! container picks the `SurfaceLevel` its descendants are on, and each theme token resolves
//! through a semantic token to a color that can differ per level. Every column below spawns
//! the same widgets; only the propagated `ThemeContext` differs.

use bevy::{
    app::Propagate,
    feathers::{
        containers::flex_spacer,
        controls::*,
        display::{caption, label, label_dim, label_small},
        theme::{
            SurfaceLevel, ThemeBackgroundColor, ThemeBorderColor, ThemeContext, ThemeToken, UiTheme,
        },
        tokens,
    },
    input_focus::tab_navigation::TabGroup,
    prelude::*,
    ui::Checked,
    ui_widgets::{SliderValue, checkbox_self_update, listbox_update_selection, slider_self_update},
};

use crate::{
    demo::Demo,
    shell::{ContentRoot, ShellAppExt},
};

/// Each surface level, with the background token Feathers' own containers use at that level.
const SURFACES: [(SurfaceLevel, &str, ThemeToken); 4] = [
    (
        SurfaceLevel::Base,
        "window, pane headers",
        tokens::WINDOW_BG,
    ),
    (
        SurfaceLevel::Higher,
        "pane and subpane bodies",
        tokens::PANE_BODY_BG,
    ),
    (
        SurfaceLevel::Highest,
        "groups, subpane headers",
        tokens::GROUP_BG,
    ),
    (SurfaceLevel::Floating, "dialogs and menus", tokens::MENU_BG),
];

/// Tokens whose resolved color is printed in every column.
const TOKENS: [(&str, ThemeToken); 4] = [
    ("BUTTON_BG", tokens::BUTTON_BG),
    ("TEXT_INPUT_BG", tokens::TEXT_INPUT_BG),
    ("LISTROW_BG_HOVER", tokens::LISTROW_BG_HOVER),
    ("BUTTON_PRIMARY_BG", tokens::BUTTON_PRIMARY_BG),
];

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::ContextualTheming)
        .add_systems(OnEnter(Demo::ContextualTheming), setup);
}

fn setup(theme: Res<UiTheme>, root: Single<Entity, With<ContentRoot>>, mut commands: Commands) {
    let surfaces: Vec<_> = SURFACES
        .into_iter()
        .map(|(level, used_for, background)| surface(&theme, level, used_for, background))
        .collect();
    commands
        .spawn_scene(bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(20)),
                row_gap: px(14),
            }
            TabGroup
            Children [
                @label_dim("Four copies of one scene. Compare the button, field and slider fills, \
                    and hover the list rows: the same tokens resolve to different colors on each surface.")
                --
                Node {
                    column_gap: px(16),
                    align_items: AlignItems::Start,
                }
                Children [ {surfaces} ]
            ]
        })
        .insert((ChildOf(*root), DespawnOnExit(Demo::ContextualTheming)));
}

fn surface(
    theme: &UiTheme,
    level: SurfaceLevel,
    used_for: &'static str,
    background: ThemeToken,
) -> impl Scene {
    let swatches: Vec<_> = TOKENS
        .into_iter()
        .map(|(name, token)| {
            let hex = theme.context_color(&token, level).to_srgba().to_hex();
            swatch(name, token, hex)
        })
        .collect();
    bsn! {
        Node {
            width: px(262),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(12)),
            row_gap: px(10),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)),
        }
        ThemeBackgroundColor({background})
        ThemeBorderColor(tokens::MENU_BORDER)
        // The one line that differs between the columns.
        Propagate::<ThemeContext>({ThemeContext(level)})
        Children [
            @label(format!("SurfaceLevel::{level:?}"))
            --
            @label_dim(used_for)
            --
            Node { column_gap: px(8) }
            Children [
                @FeathersButton { @caption: bsn! { @caption("Button") } }
                Node { flex_grow: 1.0 }
                --
                @FeathersButton {
                    @caption: bsn! { @caption("Primary") },
                    @variant: ButtonVariant::Primary,
                }
                Node { flex_grow: 1.0 }
            ]
            --
            @FeathersCheckbox { @caption: bsn! { @caption("Checkbox") } }
            on(checkbox_self_update)
            --
            @FeathersToggleSwitch
            Checked
            on(checkbox_self_update)
            --
            @FeathersSlider { @max: 100.0 }
            SliderValue(40.0)
            on(slider_self_update)
            --
            @FeathersNumberInput
            NumberInputValue::F32(1.0)
            NumberInputPrecision(2)
            --
            @FeathersListView {
                @rows: list_rows_from_strings(["Hover these rows", "Selected row", "Another row"], Some(1)),
            }
            on(listbox_update_selection)
            --
            @label_dim("Resolved theme tokens")
            --
            {swatches}
        ]
    }
}

/// A chip colored by `token` (resolved by Feathers using the inherited `ThemeContext`) next to
/// the color the theme reports for that token at this surface level.
fn swatch(name: &'static str, token: ThemeToken, hex: String) -> impl Scene {
    bsn! {
        Node {
            align_items: AlignItems::Center,
            column_gap: px(8),
        }
        Children [
            Node {
                width: px(26),
                height: px(16),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(3)),
            }
            ThemeBackgroundColor({token})
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.25))
            --
            @label_small(name)
            --
            @flex_spacer()
            --
            @label_small(hex)
        ]
    }
}
