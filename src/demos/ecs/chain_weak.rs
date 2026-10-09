//! Bevy 0.20: `chain_weak()`, `before_weak()` and `after_weak()`.
//!
//! A weak ordering is only kept between systems whose data access conflicts. The same four
//! systems are scheduled three ways and actually run; each system sleeps for a while and
//! records when and on which thread it ran.

use std::{
    fmt::Write,
    sync::Mutex,
    thread::{self, ThreadId},
    time::{Duration, Instant},
};

use bevy::{prelude::*, ui_widgets::Activate};

use super::ui::{Readout, button, spawn_panel};
use crate::{
    demo::Demo,
    shell::{ContentRoot, ShellAppExt},
};

/// Time covered by one character of the timeline.
const COLUMN: Duration = Duration::from_millis(5);

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::ChainWeak)
        .init_resource::<Positions>()
        .init_resource::<Mixer>()
        .init_resource::<Plans>()
        .init_resource::<DrawList>()
        .add_systems(OnEnter(Demo::ChainWeak), (setup, measure));
}

#[derive(Resource, Default)]
struct Positions;
#[derive(Resource, Default)]
struct Mixer;
#[derive(Resource, Default)]
struct Plans;
#[derive(Resource, Default)]
struct DrawList;

// `physics`, `audio` and `ai` write to unrelated resources. Only `draw` conflicts with
// another system: it reads the `Positions` that `physics` writes. `ai` is the slow one.

fn physics(_: ResMut<Positions>, trace: Res<Trace>) {
    trace.work("physics", 40);
}

fn audio(_: ResMut<Mixer>, trace: Res<Trace>) {
    trace.work("audio", 40);
}

fn ai(_: ResMut<Plans>, trace: Res<Trace>) {
    trace.work("ai", 100);
}

fn draw(_: Res<Positions>, _: ResMut<DrawList>, trace: Res<Trace>) {
    trace.work("draw", 40);
}

/// The three ways of ordering the systems that are compared.
fn schedules() -> [(&'static str, Schedule); 3] {
    let mut strict = Schedule::default();
    strict.add_systems((physics, audio, ai, draw).chain());

    let mut weak = Schedule::default();
    weak.add_systems((physics, audio, ai, draw).chain_weak());

    let mut before_weak = Schedule::default();
    before_weak.add_systems(((physics, audio, ai).before_weak(draw), draw));

    [
        ("(physics, audio, ai, draw).chain()", strict),
        ("(physics, audio, ai, draw).chain_weak()", weak),
        ("(physics, audio, ai).before_weak(draw)", before_weak),
    ]
}

/// One run of one system.
struct Span {
    name: &'static str,
    thread: ThreadId,
    start: Duration,
    end: Duration,
}

/// Records what the systems did. It is only ever read through `Res`, with a mutex inside, so
/// tracing does not add a conflict between the systems.
#[derive(Resource)]
struct Trace {
    epoch: Instant,
    spans: Mutex<Vec<Span>>,
}

impl Trace {
    /// Pretends to work for `millis` and records when that happened.
    fn work(&self, name: &'static str, millis: u64) {
        let start = self.epoch.elapsed();
        thread::sleep(Duration::from_millis(millis));
        let end = self.epoch.elapsed();
        self.spans.lock().unwrap().push(Span {
            name,
            thread: thread::current().id(),
            start,
            end,
        });
    }
}

fn setup(mut commands: Commands, root: Single<Entity, With<ContentRoot>>) {
    spawn_panel(
        &mut commands,
        *root,
        Demo::ChainWeak,
        "Four systems that sleep: ai for 100 ms, the others for 40 ms. physics, audio and ai \
         write to unrelated resources; draw reads what physics writes. Each schedule below was \
         just run on the multi-threaded executor.",
        bsn_list! {
            @button("Run again", |_: On<Activate>, mut commands: Commands| commands.queue(measure))
        },
    );
}

/// Runs every schedule once and writes the recorded timelines to the readout.
fn measure(world: &mut World) {
    let mut report = String::new();
    for (label, mut schedule) in schedules() {
        world.insert_resource(Trace {
            epoch: Instant::now(),
            spans: Mutex::default(),
        });
        schedule.run(world);
        let trace = world.remove_resource::<Trace>().unwrap();
        describe(&mut report, label, trace.spans.into_inner().unwrap());
    }
    world.resource_mut::<Readout>().0 = report;
}

fn describe(report: &mut String, label: &str, mut spans: Vec<Span>) {
    spans.sort_by_key(|span| span.start);
    // Measure from the moment the first system started, which leaves out building the schedule.
    let origin = spans[0].start;
    let finished = spans.iter().map(|span| span.end).max().unwrap();
    let most_at_once = spans
        .iter()
        .map(|span| {
            spans
                .iter()
                .filter(|other| other.start <= span.start && span.start < other.end)
                .count()
        })
        .max()
        .unwrap();
    let span_of = |name| spans.iter().find(|span| span.name == name).unwrap();
    let draw_start = span_of("draw").start;
    let waited_for_physics = span_of("physics").end <= draw_start;
    let waited_for_ai = span_of("ai").end <= draw_start;

    let _ = writeln!(
        report,
        "{label}\n  took {} ms, at most {most_at_once} system(s) at once, \
         draw waited for physics: {waited_for_physics}, for ai: {waited_for_ai}",
        (finished - origin).as_millis(),
    );
    for span in &spans {
        let (start, end) = (span.start - origin, span.end - origin);
        let column = |time: Duration| (time.as_micros() / COLUMN.as_micros()) as usize;
        let bar = " ".repeat(column(start)) + &"#".repeat(column(end - start).max(1));
        let _ = writeln!(
            report,
            "  {:<8}{bar:<48}{:>3}..{:<3} ms  {:?}",
            span.name,
            start.as_millis(),
            end.as_millis(),
            span.thread,
        );
    }
    report.push('\n');
}
