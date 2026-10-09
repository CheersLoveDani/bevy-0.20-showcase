//! Headless tabs from `bevy_ui_widgets`, new in Bevy 0.20: `TabList` + `Tab` provide focus,
//! keyboard navigation and accessibility, but no visuals and no tab panels. Selection lives in
//! `SelectedTab` and is either applied by the stock `tablist_self_update` observer, or owned by
//! the app (here: the [`ActivePage`] resource).

use bevy::{
    ecs::template::{EntityTemplate, OptionTemplate},
    feathers::{
        controls::FeathersButton,
        display::{caption, label, label_dim},
    },
    input_focus::{InputFocus, InputFocusVisible, tab_navigation::TabGroup},
    picking::hover::Hovered,
    prelude::*,
    ui::{InteractionDisabled, Selected},
    ui_widgets::{
        Activate, ControlOrientation, SelectedTab, Tab, TabActivation, TabList, ValueChange,
        tablist_self_update,
    },
};

use crate::{
    demo::Demo,
    shell::{ContentRoot, DemoLog, ShellAppExt},
};

const PAGES: [&str; 3] = ["Scene", "Assets", "Inspector"];

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::HeadlessTabs)
        .init_resource::<ActivePage>()
        .add_systems(OnEnter(Demo::HeadlessTabs), setup)
        .add_systems(
            Update,
            (
                apply_active_page,
                show_active_page,
                show_selected_panels,
                style_tabs,
            )
                .run_if(in_state(Demo::HeadlessTabs)),
        );
}

/// App-side state for the externally controlled tab list: an index into [`PAGES`].
#[derive(Resource, Default)]
struct ActivePage(usize);

/// The tab list whose selection follows [`ActivePage`].
#[derive(Component, Default, Clone)]
struct ControlledTabs;

/// The text showing the current value of [`ActivePage`].
#[derive(Component, Default, Clone)]
struct ActivePageReadout;

/// A tab panel, shown while the tab it points at is selected. Panels are not part of the
/// headless widget, so this link is ours.
#[derive(Component, Clone, FromTemplate)]
struct PanelFor(Entity);

fn setup(root: Single<Entity, With<ContentRoot>>, mut commands: Commands) {
    commands
        .spawn_scene(scene())
        .insert((ChildOf(*root), DespawnOnExit(Demo::HeadlessTabs)));
}

fn scene() -> impl Scene {
    bsn! {
        Node {
            padding: UiRect::all(px(20)),
            column_gap: px(32),
            align_items: AlignItems::Start,
        }
        TabGroup
        Children [
            @self_updating_tabs()
            --
            @controlled_tabs()
        ]
    }
}

fn self_updating_tabs() -> impl Scene {
    bsn! {
        @column(480.0)
        Children [
            @label("Self-updating")
            --
            @label_dim("on(tablist_self_update) writes every request straight into SelectedTab. \
                TabActivation::Automatic: focus a tab, then Left / Right / Home / End select as \
                focus moves. Disabled tabs are skipped.")
            --
            @tab_strip(FlexDirection::Row)
            TabList { activation: TabActivation::Automatic }
            @selected_tab(#general)
            on(tablist_self_update)
            Children [
                #general @tab("General")
                --
                #rendering @tab("Rendering")
                --
                #audio @tab("Audio")
                --
                @tab("Disabled") InteractionDisabled
            ]
            --
            @panel(#general, "General", "Window title, vsync and language.")
            --
            @panel(#rendering, "Rendering", "Anti-aliasing, shadows and render scale.")
            --
            @panel(#audio, "Audio", "Master volume and output device.")
        ]
    }
}

fn controlled_tabs() -> impl Scene {
    bsn! {
        @column(520.0)
        Children [
            @label("Externally controlled")
            --
            @label_dim("No self-update observer here. A click, or Enter / Space on a focused tab \
                (TabActivation::Manual), only emits ValueChange<Option<Entity>>. The app stores \
                the choice in a resource and writes SelectedTab itself, so code can drive it too.")
            --
            Node {
                align_items: AlignItems::Center,
                column_gap: px(8),
            }
            Children [
                @FeathersButton { @caption: bsn! { @caption("Previous") } }
                on(|_: On<Activate>, mut page: ResMut<ActivePage>| {
                    page.0 = (page.0 + PAGES.len() - 1) % PAGES.len();
                })
                --
                @FeathersButton { @caption: bsn! { @caption("Next") } }
                on(|_: On<Activate>, mut page: ResMut<ActivePage>| {
                    page.0 = (page.0 + 1) % PAGES.len();
                })
                --
                ActivePageReadout
                @label("")
            ]
            --
            Node { column_gap: px(12) }
            Children [
                @tab_strip(FlexDirection::Column)
                TabList { orientation: ControlOrientation::Vertical }
                ControlledTabs
                on(request_page)
                Children [
                    #scene @tab(PAGES[0])
                    --
                    #assets @tab(PAGES[1])
                    --
                    #inspector @tab(PAGES[2])
                ]
                --
                @panel(#scene, PAGES[0], "The entity hierarchy of the open scene.")
                --
                @panel(#assets, PAGES[1], "Meshes, textures and scenes on disk.")
                --
                @panel(#inspector, PAGES[2], "Components of the selected entity.")
            ]
        ]
    }
}

/// The tab list asks for a tab to be selected; the app decides what that means.
fn request_page(
    change: On<ValueChange<Option<Entity>>>,
    tab_lists: Query<&Children>,
    mut page: ResMut<ActivePage>,
    mut log: ResMut<DemoLog>,
) {
    let index = tab_lists
        .get(change.source)
        .ok()
        .zip(change.value)
        .and_then(|(tabs, requested)| tabs.iter().position(|tab| tab == requested));
    if let Some(index) = index {
        page.0 = index;
        log.push(format!("Tab list requested \"{}\"", PAGES[index]));
    }
}

/// Mirrors [`ActivePage`] into the controlled tab list's `SelectedTab`.
fn apply_active_page(
    page: Res<ActivePage>,
    tab_list: Single<(Entity, &Children, &SelectedTab), With<ControlledTabs>>,
    mut commands: Commands,
) {
    let (entity, tabs, selected) = *tab_list;
    let wanted = tabs.get(page.0).copied();
    if selected.0 != wanted {
        commands.entity(entity).insert(SelectedTab(wanted));
    }
}

fn show_active_page(
    page: Res<ActivePage>,
    mut readout: Single<&mut Text, With<ActivePageReadout>>,
) {
    let text = format!("ActivePage({}) = {}", page.0, PAGES[page.0]);
    if readout.0 != text {
        readout.0 = text;
    }
}

/// `Tab` entities get `Selected` derived from their list's `SelectedTab`.
fn show_selected_panels(
    tabs: Query<Has<Selected>, With<Tab>>,
    mut panels: Query<(&PanelFor, &mut Node)>,
) {
    for (panel, mut node) in &mut panels {
        let display = if tabs.get(panel.0).unwrap_or(false) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
}

/// All visuals are up to the app: color tabs by their state and outline the focused one.
fn style_tabs(
    focus: Res<InputFocus>,
    focus_visible: Res<InputFocusVisible>,
    mut tabs: Query<
        (
            Entity,
            &Hovered,
            Has<Selected>,
            Has<InteractionDisabled>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Tab>,
    >,
) {
    for (entity, hovered, selected, disabled, mut background, mut border) in &mut tabs {
        background.0 = match (disabled, selected, hovered.get()) {
            (true, _, _) => Color::srgb(0.10, 0.10, 0.11),
            (false, true, _) => Color::srgb(0.18, 0.34, 0.52),
            (false, false, true) => Color::srgb(0.22, 0.23, 0.29),
            (false, false, false) => Color::srgb(0.14, 0.15, 0.19),
        };
        border.set_all(if focus_visible.0 && focus.get() == Some(entity) {
            Color::srgb(0.45, 0.78, 1.0)
        } else {
            Color::srgb(0.26, 0.27, 0.32)
        });
    }
}

fn column(width: f32) -> impl Scene {
    bsn! {
        Node {
            width: px(width),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
        }
    }
}

fn tab_strip(flex_direction: FlexDirection) -> impl Scene {
    bsn! {
        Node {
            flex_direction,
            align_self: AlignSelf::Start,
        }
    }
}

fn tab(name: &'static str) -> impl Scene {
    bsn! {
        Tab
        Hovered
        Node {
            min_width: px(110),
            padding: UiRect::axes(px(14), px(8)),
            border: UiRect::all(px(1)),
            justify_content: JustifyContent::Center,
        }
        BackgroundColor(Color::NONE)
        BorderColor::all(Color::NONE)
        Children [
            Text(name)
            TextFont { font_size: FontSize::Rem(0.9) }
        ]
    }
}

fn selected_tab(tab: EntityTemplate) -> impl Scene {
    let tab = OptionTemplate::Some(tab);
    bsn! { SelectedTab({tab}) }
}

fn panel(tab: EntityTemplate, title: &'static str, body: &'static str) -> impl Scene {
    bsn! {
        PanelFor({tab})
        Node {
            display: Display::None,
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            min_height: px(110),
            padding: UiRect::all(px(16)),
            row_gap: px(6),
            border: UiRect::all(px(1)),
        }
        BackgroundColor(Color::srgb(0.11, 0.12, 0.15))
        BorderColor::all(Color::srgb(0.18, 0.34, 0.52))
        Children [
            Text(title)
            TextFont { font_size: FontSize::Rem(1.2) }
            --
            Text(body)
            TextFont { font_size: FontSize::Rem(0.9) }
            TextColor(Color::srgb(0.7, 0.72, 0.78))
        ]
    }
}
