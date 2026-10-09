//! Bevy 0.20: `Commands::despawn_all::<Filter>()`.
//!
//! One command that despawns everything matching a query filter, compared against the usual
//! loop that queues one despawn command per entity. Both are timed on freshly spawned
//! entities, from queueing the commands to the moment they have been applied.

use std::{
    fmt::Write,
    time::{Duration, Instant},
};

use bevy::{prelude::*, ui_widgets::Activate};

use super::ui::{Readout, button, spawn_panel};
use crate::{
    demo::Demo,
    shell::{ContentRoot, DemoLog, ShellAppExt},
};

/// Each way of despawning is timed this many times, and the fastest time is kept.
const ROUNDS: usize = 3;

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::DespawnAll).add_systems(
        OnEnter(Demo::DespawnAll),
        (setup, |world: &mut World| measure(world, 100_000)).chain(),
    );
}

#[derive(Component)]
struct Doomed;

/// Some data, so the entities are not just an id.
#[derive(Component, Default)]
struct Payload {
    _data: [f32; 8],
}

struct Measurement {
    count: usize,
    one_by_one: Duration,
    batched: Duration,
    left_over: usize,
}

#[derive(Resource, Default)]
struct Measurements(Vec<Measurement>);

fn setup(mut commands: Commands, root: Single<Entity, With<ContentRoot>>) {
    commands.insert_resource(Measurements::default());
    spawn_panel(
        &mut commands,
        *root,
        Demo::DespawnAll,
        "Each run spawns N entities and despawns them with one command per entity, then \
         spawns N more and despawns them with a single despawn_all command. Times are the \
         best of 3 rounds.",
        bsn_list! {
            @run_button("10,000 entities", 10_000)
            --
            @run_button("100,000 entities", 100_000)
            --
            @run_button("1,000,000 entities", 1_000_000)
        },
    );
}

fn run_button(label: &'static str, count: usize) -> impl Scene {
    button(label, move |_: On<Activate>, mut commands: Commands| {
        commands.queue(move |world: &mut World| measure(world, count));
    })
}

fn measure(world: &mut World, count: usize) {
    let one_by_one = time_despawn(world, count, |world| {
        let doomed: Vec<Entity> = world
            .query_filtered::<Entity, With<Doomed>>()
            .iter(world)
            .collect();
        let mut commands = world.commands();
        for entity in doomed {
            commands.entity(entity).despawn();
        }
    });
    let batched = time_despawn(world, count, |world| {
        world.commands().despawn_all::<With<Doomed>>();
    });
    let left_over = world
        .query_filtered::<(), With<Doomed>>()
        .iter(world)
        .count();

    world.resource_mut::<DemoLog>().push(format!(
        "{count} entities: one by one {one_by_one:.2?}, despawn_all {batched:.2?}"
    ));
    let mut measurements = world.resource_mut::<Measurements>();
    measurements.0.push(Measurement {
        count,
        one_by_one,
        batched,
        left_over,
    });

    let mut report = format!(
        "{:>10}  {:>14}  {:>14}  {:>8}  {}\n",
        "entities", "one by one", "despawn_all", "speedup", "left over"
    );
    for run in measurements.0.iter().rev().take(12).rev() {
        let _ = writeln!(
            report,
            "{:>10}  {:>14.2?}  {:>14.2?}  {:>7.1}x  {}",
            run.count,
            run.one_by_one,
            run.batched,
            run.one_by_one.as_secs_f64() / run.batched.as_secs_f64(),
            run.left_over,
        );
    }
    world.resource_mut::<Readout>().0 = report;
}

/// Spawns `count` entities, then times queueing the despawn commands and applying them.
fn time_despawn(world: &mut World, count: usize, queue: impl Fn(&mut World)) -> Duration {
    let round = |_| {
        world.spawn_batch((0..count).map(|_| (Doomed, Payload::default())));
        let start = Instant::now();
        queue(world);
        world.flush();
        start.elapsed()
    };
    (0..ROUNDS).map(round).min().unwrap()
}
