//! Bevy 0.20: schedule randomization with `ScheduleBuildSettings::shuffle_seed`.
//!
//! Systems that conflict but have no ordering between them run in an arbitrary order. A
//! shuffle seed (available with the `debug` cargo feature) randomizes that order while still
//! honoring every explicit constraint, which flushes out accidental order dependencies.

use std::fmt::Write;

use bevy::{
    ecs::schedule::{ScheduleBuildSettings, SingleThreadedExecutor},
    prelude::*,
    ui_widgets::Activate,
};

use super::ui::{Readout, button, spawn_panel};
use crate::{
    demo::Demo,
    shell::{ContentRoot, DemoLog, ShellAppExt},
};

/// How many runs stay on screen.
const HISTORY: usize = 12;

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::ScheduleShuffle)
        .add_systems(OnEnter(Demo::ScheduleShuffle), (setup, first_runs).chain());
}

/// The names of the systems, in the order they actually ran.
#[derive(Resource, Default)]
struct Order(String);

/// Every instance writes to `Order`, so all of them conflict with each other.
fn step<const NAME: char>(mut order: ResMut<Order>) {
    order.0.push(NAME);
}

/// Builds the schedule with the given seed, runs it once and returns the order it ran in.
fn run_schedule(world: &mut World, shuffle_seed: Option<u64>) -> String {
    let mut schedule = Schedule::default();
    // Ambiguous: nothing says which of these runs first.
    schedule.add_systems((
        step::<'A'>,
        step::<'B'>,
        step::<'C'>,
        step::<'D'>,
        step::<'E'>,
    ));
    // Explicitly ordered: shuffling must keep 1 before 2 before 3.
    schedule.add_systems((step::<'1'>, step::<'2'>, step::<'3'>).chain());
    schedule.set_build_settings(ScheduleBuildSettings {
        shuffle_seed,
        ..default()
    });
    // The multi-threaded executor may start systems out of order, so use the simple one to
    // see exactly the order the schedule was built with.
    schedule.set_executor(SingleThreadedExecutor::new());

    world.init_resource::<Order>();
    schedule.run(world);
    world.remove_resource::<Order>().unwrap().0
}

/// Every run so far, as `(seed, order)`.
#[derive(Resource, Default)]
struct Runs(Vec<(Option<u64>, String)>);

fn setup(mut commands: Commands, root: Single<Entity, With<ContentRoot>>) {
    commands.insert_resource(Runs::default());
    spawn_panel(
        &mut commands,
        *root,
        Demo::ScheduleShuffle,
        "Eight systems append their name to one resource. A to E have no ordering between \
         them; 1, 2 and 3 are chained. Each row is a freshly built schedule.",
        bsn_list! {
            @button("Reseed", |_: On<Activate>, mut commands: Commands| {
                let seed = rand::random();
                commands.queue(move |world: &mut World| run(world, Some(seed)));
            })
            --
            @button("Same seed again", |_: On<Activate>, mut commands: Commands| {
                commands.queue(|world: &mut World| {
                    let last_seed = world.resource::<Runs>().0.last().and_then(|run| run.0);
                    run(world, last_seed);
                });
            })
            --
            @button("No seed", |_: On<Activate>, mut commands: Commands| {
                commands.queue(|world: &mut World| run(world, None));
            })
        },
    );
}

/// One run without shuffling as a baseline, then a few random seeds.
fn first_runs(world: &mut World) {
    run(world, None);
    for _ in 0..4 {
        run(world, Some(rand::random()));
    }
}

fn run(world: &mut World, seed: Option<u64>) {
    let order = run_schedule(world, seed);
    world
        .resource_mut::<DemoLog>()
        .push(format!("shuffle_seed: {seed:?} -> {order}"));
    world.resource_mut::<Runs>().0.push((seed, order));

    let runs = &world.resource::<Runs>().0;
    let mut report = format!(
        "{:<22}{:<12}{}\n",
        "shuffle_seed", "ran as", "1, 2, 3 in order"
    );
    for (seed, order) in &runs[runs.len().saturating_sub(HISTORY)..] {
        let seed = seed.map_or("None".to_string(), |seed| seed.to_string());
        let chain_kept = order.chars().filter(char::is_ascii_digit).eq("123".chars());
        let _ = writeln!(report, "{seed:<22}{order:<12}{chain_kept}");
    }
    let distinct = runs
        .iter()
        .map(|(_, order)| order)
        .collect::<std::collections::HashSet<_>>()
        .len();
    let _ = write!(report, "\n{} runs, {distinct} distinct orders", runs.len());
    world.resource_mut::<Readout>().0 = report;
}
