//! The launcher: sidebar navigation, header, log panel and the cameras the UI renders to.
//!
//! Demos spawn their UI as children of the [`ContentRoot`] entity and tag everything they
//! spawn with `DespawnOnExit(Demo::X)`.

use std::collections::{HashSet, VecDeque};

use bevy::{
    camera::{visibility::RenderLayers, ClearColorConfig},
    feathers::{
        constants::fonts,
        controls::{FeathersListRow, FeathersListView},
        dark_theme::create_dark_theme,
        display::{caption, label_dim},
        theme::{ThemeBackgroundColor, UiTheme},
        tokens, FeathersPlugins,
    },
    input_focus::tab_navigation::TabGroup,
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
    text::FontSourceTemplate,
    ui::Selected,
    ui_widgets::{listbox_update_selection, ValueChange},
};

use crate::demo::Demo;

/// Width of the navigation sidebar, in logical pixels.
pub const SIDEBAR_WIDTH: f32 = 260.0;
/// Height of the title / description header, in logical pixels.
pub const HEADER_HEIGHT: f32 = 84.0;
/// Height of the log panel when it is visible, in logical pixels.
pub const LOG_HEIGHT: f32 = 170.0;

/// The UI node demos parent their own UI to. It fills the window area right of the sidebar
/// and below the header, and ignores picking so 2D / 3D scenes underneath stay interactive.
#[derive(Component)]
pub struct ContentRoot;

/// An on-screen log shown at the bottom of the window. Cleared whenever the demo changes.
#[derive(Resource, Default)]
pub struct DemoLog {
    lines: VecDeque<String>,
}

impl DemoLog {
    const MAX_LINES: usize = 200;
    const VISIBLE_LINES: usize = 8;

    pub fn push(&mut self, line: impl Into<String>) {
        if self.lines.len() == Self::MAX_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(line.into());
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

/// Demos that are compiled into this build. Anything else shows a placeholder.
#[derive(Resource, Default)]
struct Implemented(HashSet<Demo>);

pub trait ShellAppExt {
    /// Marks `demo` as available, so the shell does not show its "not built" placeholder.
    fn register_demo(&mut self, demo: Demo) -> &mut Self;
}

impl ShellAppExt for App {
    fn register_demo(&mut self, demo: Demo) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<Implemented>()
            .0
            .insert(demo);
        self
    }
}

#[derive(Component, Default, Clone)]
struct NavItem(Demo);

#[derive(Component, Default, Clone)]
struct HeaderTitle;

#[derive(Component, Default, Clone)]
struct HeaderDescription;

#[derive(Component, Default, Clone)]
struct LogPanel;

#[derive(Component, Default, Clone)]
struct LogText;

pub fn plugin(app: &mut App) {
    app.add_plugins(FeathersPlugins)
        .insert_resource(UiTheme(create_dark_theme()))
        .init_resource::<DemoLog>()
        .init_resource::<Implemented>()
        .insert_state(initial_demo())
        .add_systems(Startup, (spawn_cameras, shell.spawn()))
        .add_systems(
            Update,
            (
                (update_header, sync_selection, spawn_placeholder).run_if(state_changed::<Demo>),
                update_log_panel.run_if(resource_changed::<DemoLog>),
            ),
        );

    if let Ok(path) = std::env::var("SHOWCASE_SCREENSHOT") {
        app.insert_resource(AutoScreenshot { path, taken: false })
            .add_systems(Update, auto_screenshot);
    }

    if let Some(seconds) = std::env::var("SHOWCASE_TOUR")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
    {
        app.insert_resource(Tour(Timer::from_seconds(seconds, TimerMode::Repeating)))
            .add_systems(Update, tour);
    }

    for demo in Demo::ALL {
        app.add_systems(OnExit(demo), |mut log: ResMut<DemoLog>| log.clear());
    }

    // Spawned here rather than in `Startup` so it already exists when the first `OnEnter` runs.
    app.world_mut().spawn((
        ContentRoot,
        Node {
            position_type: PositionType::Absolute,
            left: px(SIDEBAR_WIDTH),
            top: px(HEADER_HEIGHT),
            right: px(0),
            bottom: px(0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Pickable::IGNORE,
    ));
}

/// The demo shown at startup. `SHOWCASE_DEMO=<Variant>` (e.g. `SpriteMaterial`) overrides it.
fn initial_demo() -> Demo {
    let Ok(name) = std::env::var("SHOWCASE_DEMO") else {
        return Demo::default();
    };
    Demo::ALL
        .into_iter()
        .find(|demo| format!("{demo:?}").eq_ignore_ascii_case(&name))
        .unwrap_or_else(|| {
            warn!("SHOWCASE_DEMO={name} is not a known demo");
            Demo::default()
        })
}

/// Set by `SHOWCASE_SCREENSHOT=<path>`: saves one screenshot a few seconds in, then exits.
/// `SHOWCASE_SHOT_AT=<seconds>` changes when it is taken.
#[derive(Resource)]
struct AutoScreenshot {
    path: String,
    taken: bool,
}

fn auto_screenshot(
    time: Res<Time>,
    mut shot: ResMut<AutoScreenshot>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    let at = std::env::var("SHOWCASE_SHOT_AT")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(4.0);
    if !shot.taken && time.elapsed_secs() > at {
        shot.taken = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(shot.path.clone()));
    }
    if time.elapsed_secs() > at + 1.5 {
        exit.write(AppExit::Success);
    }
}

/// Set by `SHOWCASE_TOUR=<seconds>`: steps through every demo in order, then exits.
#[derive(Resource)]
struct Tour(Timer);

fn tour(
    time: Res<Time>,
    mut tour: ResMut<Tour>,
    demo: Res<State<Demo>>,
    mut next: ResMut<NextState<Demo>>,
    mut exit: MessageWriter<AppExit>,
) {
    if !tour.0.tick(time.delta()).just_finished() {
        return;
    }
    let index = Demo::ALL.iter().position(|d| d == demo.get()).unwrap_or(0);
    match Demo::ALL.get(index + 1) {
        Some(demo) => {
            info!("Tour: {demo:?}");
            next.set(*demo);
        }
        None => {
            exit.write(AppExit::Success);
        }
    }
}

/// The shell owns two cameras so demos are free to spawn (and despawn) their own at order 0:
/// one that clears the window first, and one that draws the UI last.
fn spawn_cameras(mut commands: Commands) {
    // Neither camera should draw the demos' sprites or meshes.
    let shell_only = RenderLayers::layer(31);
    commands.spawn((
        Camera2d,
        Camera {
            order: -100,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.07, 0.07, 0.09)),
            ..default()
        },
        shell_only.clone(),
    ));
    commands.spawn((
        Camera2d,
        Camera {
            order: 100,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
        shell_only,
    ));
}

fn shell() -> impl SceneList {
    let rows: Box<dyn SceneList> =
        Box::new(Demo::ALL.into_iter().map(nav_row).collect::<Vec<_>>());
    bsn_list! {
        // Sidebar
        FixedNode
        Node {
            left: px(0),
            top: px(0),
            width: px(SIDEBAR_WIDTH),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(8)),
            row_gap: px(8),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        TabGroup
        Children [
            Node { padding: UiRect::all(px(4)) }
            Children [ @label_dim("FEATURES") ]
            --
            @FeathersListView { @rows: {rows} }
            Node { flex_grow: 1.0, min_height: px(0) }
            on(listbox_update_selection)
            on(navigate)
        ]
        --
        // Header
        Node {
            position_type: PositionType::Absolute,
            left: px(SIDEBAR_WIDTH),
            top: px(0),
            right: px(0),
            height: px(HEADER_HEIGHT),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            padding: UiRect::horizontal(px(20)),
            row_gap: px(4),
            // The Em / Rem demo resizes the root font; don't let the text spill over the demo.
            overflow: Overflow::clip(),
        }
        BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.9))
        Pickable::IGNORE
        Children [
            HeaderTitle
            Text("")
            TextFont {
                font: FontSourceTemplate::Handle(fonts::BOLD),
                font_size: FontSize::Rem(1.5),
            }
            --
            HeaderDescription
            Text("")
            TextFont {
                font: FontSourceTemplate::Handle(fonts::REGULAR),
                font_size: FontSize::Rem(0.85),
            }
            TextColor(Color::srgb(0.7, 0.72, 0.78))
        ]
        --
        // Log panel
        LogPanel
        Node {
            position_type: PositionType::Absolute,
            left: px(SIDEBAR_WIDTH),
            right: px(0),
            bottom: px(0),
            height: px(LOG_HEIGHT),
            display: Display::None,
            padding: UiRect::all(px(12)),
            overflow: Overflow::clip(),
        }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8))
        GlobalZIndex(10)
        Pickable::IGNORE
        Children [
            LogText
            Text("")
            TextFont { font_size: FontSize::Rem(0.8) }
            TextColor(Color::srgb(0.6, 0.9, 0.65))
        ]
    }
}

fn nav_row(demo: Demo) -> impl Scene {
    let selected = (demo == initial_demo()).then(|| bsn! { Selected });
    bsn! {
        @FeathersListRow
        NavItem({demo})
        @{selected}
        Children [ @caption(demo.title()) ]
    }
}

fn navigate(
    change: On<ValueChange<Entity>>,
    items: Query<&NavItem>,
    mut next: ResMut<NextState<Demo>>,
) {
    if let Ok(item) = items.get(change.value) {
        next.set(item.0);
    }
}

fn update_header(
    demo: Res<State<Demo>>,
    mut title: Single<&mut Text, (With<HeaderTitle>, Without<HeaderDescription>)>,
    mut description: Single<&mut Text, (With<HeaderDescription>, Without<HeaderTitle>)>,
) {
    title.0 = demo.title().into();
    description.0 = demo.description().into();
}

/// Keeps the sidebar highlight right when the demo changes by other means than a click.
fn sync_selection(
    demo: Res<State<Demo>>,
    items: Query<(Entity, &NavItem, Has<Selected>)>,
    mut commands: Commands,
) {
    for (entity, item, selected) in &items {
        match (item.0 == *demo.get(), selected) {
            (true, false) => {
                commands.entity(entity).insert(Selected);
            }
            (false, true) => {
                commands.entity(entity).remove::<Selected>();
            }
            _ => {}
        }
    }
}

/// Fills the content area for [`Demo::Home`] and for demos that are not part of this build.
fn spawn_placeholder(
    demo: Res<State<Demo>>,
    implemented: Res<Implemented>,
    root: Single<Entity, With<ContentRoot>>,
    mut commands: Commands,
) {
    let demo = *demo.get();
    if implemented.0.contains(&demo) {
        return;
    }
    let message = match demo {
        Demo::Home => "Bevy 0.20 ships 23 headline features. Each entry in the sidebar is a small, \
            live scene exercising one of them.\n\nNon-visual ECS features print to a log panel at \
            the bottom of the window."
            .to_string(),
        Demo::Solari => "Not part of this build.\n\nSolari needs a GPU with hardware ray tracing. \
            Run with:\n    cargo run --features demo_solari"
            .to_string(),
        Demo::MeshShaders => "Not part of this build.\n\nMesh shaders need GPU and driver \
            support. Run with:\n    cargo run --features demo_mesh_shaders"
            .to_string(),
        _ => "This demo's cargo feature is disabled in this build.".to_string(),
    };
    commands
        .spawn_scene(bsn! {
            Node {
                padding: UiRect::all(px(24)),
                max_width: px(640),
            }
            Children [
                Text(message)
                TextFont {
                    font: FontSourceTemplate::Handle(fonts::REGULAR),
                    font_size: FontSize::Rem(1.0),
                }
                TextColor(Color::srgb(0.85, 0.86, 0.9))
            ]
        })
        .insert((ChildOf(*root), DespawnOnExit(demo)));
}

fn update_log_panel(
    log: Res<DemoLog>,
    mut panel: Single<&mut Node, With<LogPanel>>,
    mut text: Single<&mut Text, With<LogText>>,
) {
    panel.display = if log.lines.is_empty() {
        Display::None
    } else {
        Display::Flex
    };
    let skip = log.lines.len().saturating_sub(DemoLog::VISIBLE_LINES);
    text.0 = log
        .lines
        .iter()
        .skip(skip)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n");
}
