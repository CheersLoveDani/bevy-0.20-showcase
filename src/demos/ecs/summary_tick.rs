//! Bevy 0.20: `#[component(summary_tick)]`, a change tick for a whole table column.
//!
//! With the attribute, every table also remembers when any component in the column last
//! changed. Contiguous (table by table) iteration exposes that as
//! `summary_tick_is_changed()`, so a system can skip an untouched table without looking at
//! its rows. Two otherwise identical components are scanned every frame to compare.

use std::{marker::PhantomData, time::Instant};

use bevy::{ecs::change_detection::ContiguousRef, prelude::*, ui_widgets::Activate};

use super::ui::{Readout, button, spawn_panel};
use crate::{
    demo::Demo,
    shell::{ContentRoot, DemoLog, ShellAppExt},
};

const ENTITIES: u32 = 500_000;

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::SummaryTick)
        .add_systems(OnEnter(Demo::SummaryTick), setup)
        .add_systems(OnExit(Demo::SummaryTick), cleanup)
        .add_systems(
            Update,
            (
                (
                    scan_rows::<Plain>,
                    scan_rows::<Summarized>,
                    scan_tables::<Plain>,
                    scan_tables::<Summarized>,
                ),
                (report, auto_change),
            )
                .chain()
                .run_if(in_state(Demo::SummaryTick)),
        );
}

#[derive(Component)]
struct Plain(u32);

#[derive(Component)]
#[component(summary_tick)]
struct Summarized(u32);

/// What the two scans of component `T` measured.
#[derive(Resource)]
struct Scan<T> {
    /// Smoothed time of a `Changed<T>` query, in microseconds.
    rows_micros: f64,
    /// Smoothed time of the contiguous scan, in microseconds.
    tables_micros: f64,
    /// How many changed components each scan found the last time it found any.
    rows_found: usize,
    tables_found: usize,
    marker: PhantomData<fn() -> T>,
}

impl<T> Default for Scan<T> {
    fn default() -> Self {
        Self {
            rows_micros: 0.0,
            tables_micros: 0.0,
            rows_found: 0,
            tables_found: 0,
            marker: PhantomData,
        }
    }
}

/// Moves a running average a little towards the time since `start`.
fn smooth(average: &mut f64, start: Instant) {
    let sample = start.elapsed().as_secs_f64() * 1e6;
    *average += (sample - *average) * 0.05;
}

/// The ordinary way to find changes. A `Changed<T>` filter looks at the tick of every row
/// and does not use the summary tick.
fn scan_rows<T: Component>(query: Query<(), Changed<T>>, mut scan: ResMut<Scan<T>>) {
    let start = Instant::now();
    let found = query.iter().count();
    smooth(&mut scan.rows_micros, start);
    if found > 0 {
        scan.rows_found = found;
    }
}

/// Table by table: ask the column's summary tick first and only look at the rows if it says
/// something changed. For a component without a summary tick the answer is `None`.
fn scan_tables<T: Component>(query: Query<Ref<T>>, mut scan: ResMut<Scan<T>>) -> Result {
    let start = Instant::now();
    let mut found = 0;
    for column in query.contiguous_iter()? {
        let (_, ticks) = ContiguousRef::split(column);
        if ticks.summary_tick_is_changed() == Some(false) {
            continue;
        }
        found += ticks.is_changed_iter().filter(|changed| *changed).count();
    }
    smooth(&mut scan.tables_micros, start);
    if found > 0 {
        scan.tables_found = found;
    }
    Ok(())
}

/// Changes one entity a couple of seconds after the demo opens, like the button does.
#[derive(Resource)]
struct AutoChange(Timer);

fn setup(mut commands: Commands, root: Single<Entity, With<ContentRoot>>) {
    commands.insert_resource(AutoChange(Timer::from_seconds(2.0, TimerMode::Once)));
    commands.init_resource::<Scan<Plain>>();
    commands.init_resource::<Scan<Summarized>>();
    commands.spawn_batch((0..ENTITIES).map(|i| (Plain(i), Summarized(i))));
    spawn_panel(
        &mut commands,
        *root,
        Demo::SummaryTick,
        "500,000 entities have both a Plain and a Summarized component, and nothing changes \
         them. Every frame each component is scanned for changes in two ways; the times are \
         running averages. One entity is changed after two seconds, and by the button.",
        bsn_list! {
            @button("Change one entity", |_: On<Activate>, mut commands: Commands| {
                commands.run_system_cached(change_one);
            })
        },
    );
}

/// The entities have no `DespawnOnExit`: this is cheaper than tagging half a million of them.
fn cleanup(mut commands: Commands) {
    commands.despawn_all::<With<Plain>>();
    commands.remove_resource::<Scan<Plain>>();
    commands.remove_resource::<Scan<Summarized>>();
    commands.remove_resource::<AutoChange>();
}

fn auto_change(time: Res<Time>, mut auto: ResMut<AutoChange>, mut commands: Commands) {
    if auto.0.tick(time.delta()).just_finished() {
        commands.run_system_cached(change_one);
    }
}

fn change_one(mut components: Query<(&mut Plain, &mut Summarized)>, mut log: ResMut<DemoLog>) {
    if let Some((mut plain, mut summarized)) = components.iter_mut().next() {
        plain.0 += 1;
        summarized.0 += 1;
        log.push(format!(
            "changed one entity: Plain({}), Summarized({})",
            plain.0, summarized.0
        ));
    }
}

fn report(
    plain: Res<Scan<Plain>>,
    summarized: Res<Scan<Summarized>>,
    mut readout: ResMut<Readout>,
) {
    readout.0 = format!(
        "{:<44}{:>12}{:>14}\n\
         {:<44}{:>9.1} us{:>11.1} us\n\
         {:<44}{:>9.1} us{:>11.1} us\n\n\
         {:<44}{:>12}{:>14}\n\
         {:<44}{:>12}{:>14}",
        "",
        "Plain",
        "Summarized",
        "Query<(), Changed<T>>",
        plain.rows_micros,
        summarized.rows_micros,
        "contiguous_iter + summary_tick_is_changed",
        plain.tables_micros,
        summarized.tables_micros,
        "last change seen by Changed<T>",
        plain.rows_found,
        summarized.rows_found,
        "last change seen by contiguous scan",
        plain.tables_found,
        summarized.tables_found,
    );
}
