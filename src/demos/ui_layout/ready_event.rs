//! The `Ready` event: triggered on every entity of a BSN scene once it and all of its
//! descendants have been spawned, so it runs bottom-up (children before parents).

use bevy::{
    feathers::{controls::FeathersButton, display::caption},
    prelude::*,
    scene::Ready,
    ui_widgets::Activate,
};

use super::{note, page, BLUE, GREEN, ORANGE, PINK, TEXT};
use crate::{
    demo::Demo,
    shell::{ContentRoot, DemoLog, ShellAppExt},
};

/// How many `Ready` events the current tree has fired so far.
#[derive(Resource, Default)]
struct ReadyCount(u32);

/// The node the tree is (re)spawned under.
#[derive(Component, Default, Clone)]
struct TreeSlot;

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::ReadyEvent)
        .init_resource::<ReadyCount>()
        .add_systems(OnEnter(Demo::ReadyEvent), setup);
}

fn setup(
    root: Single<Entity, With<ContentRoot>>,
    mut count: ResMut<ReadyCount>,
    mut commands: Commands,
) {
    count.0 = 0;
    commands
        .spawn_scene(ui())
        .insert((ChildOf(*root), DespawnOnExit(Demo::ReadyEvent)));
}

fn ui() -> impl Scene {
    bsn! {
        @page()
        Children [
            @note("Every box is one entity with the same on(|ready: On<Ready>| ..) observer. The \
                number is the order its Ready fired in:\nchildren always come before their parent, \
                so the observer can already see (and here, count) its whole subtree.")
            --
            @FeathersButton
            Node { align_self: AlignSelf::FlexStart }
            on(respawn)
            Children [ @caption("Spawn again") ]
            --
            TreeSlot
            Node
            Children [ @tree() ]
        ]
    }
}

fn tree() -> impl Scene {
    bsn! {
        @entity("Root", BLUE)
        Children [
            @entity("Branch A", GREEN)
            Children [
                @entity("Leaf A1", ORANGE)
                --
                @entity("Leaf A2", ORANGE)
                Children [ @entity("Twig", PINK) ]
            ]
            --
            @entity("Branch B", GREEN)
            Children [ @entity("Leaf B1", ORANGE) ]
        ]
    }
}

/// A named, outlined box. Its first child is its label; callers append child boxes after it.
fn entity(name: &'static str, color: Color) -> impl Scene {
    bsn! {
        Name(name)
        Node {
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(px(12)),
            column_gap: px(12),
            row_gap: px(10),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(8)),
        }
        BorderColor::all(color)
        BackgroundColor({color.with_alpha(0.12)})
        on(on_ready)
        Children [
            Text({format!("?. {name}")})
            TextFont { font_size: FontSize::Px(15.0) }
            TextColor(TEXT)
            // Full width, so the child boxes wrap onto the next line.
            Node { width: percent(100) }
        ]
    }
}

fn on_ready(
    ready: On<Ready>,
    boxes: Query<(&Name, &Children)>,
    mut labels: Query<&mut Text>,
    mut count: ResMut<ReadyCount>,
    mut log: ResMut<DemoLog>,
) -> Result {
    let (name, children) = boxes.get(ready.entity)?;
    count.0 += 1;
    // The label and the child boxes were declared in the scene, so they all exist by now.
    labels.get_mut(children[0])?.0 = format!("{}. {name}", count.0);
    log.push(format!(
        "Ready #{} -> {name} ({} child boxes already spawned)",
        count.0,
        children.len() - 1
    ));
    Ok(())
}

fn respawn(
    _: On<Activate>,
    slot: Single<Entity, With<TreeSlot>>,
    mut count: ResMut<ReadyCount>,
    mut log: ResMut<DemoLog>,
    mut commands: Commands,
) {
    count.0 = 0;
    log.clear();
    log.push("Spawned again:");
    commands.entity(*slot).despawn_related::<Children>();
    commands.spawn_scene(tree()).insert(ChildOf(*slot));
}
