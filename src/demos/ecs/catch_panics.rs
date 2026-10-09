//! Bevy 0.20: panics in systems, commands and observers are caught and turned into errors.
//!
//! A caught panic arrives at the `FallbackErrorHandler` as a `BevyError` carrying the panic
//! payload. The default handler resumes the panic; while this demo is open a handler is
//! installed that reports the panic and lets the app carry on.

use std::sync::Mutex;

use bevy::{
    ecs::error::{ErrorContext, FallbackErrorHandler, match_severity},
    prelude::*,
    ui_widgets::Activate,
};

use super::ui::{Readout, button, spawn_panel};
use crate::{
    demo::Demo,
    shell::{ContentRoot, DemoLog, ShellAppExt},
};

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::CatchPanics)
        .add_observer(faulty_observer)
        .add_systems(OnEnter(Demo::CatchPanics), (setup, install_handler))
        .add_systems(OnExit(Demo::CatchPanics), restore_handler)
        .add_systems(
            Update,
            (faulty_system, report).run_if(in_state(Demo::CatchPanics)),
        );
}

/// An error handler is a plain `fn`, so it cannot reach into the `World`. Caught panics are
/// parked here until [`report`] picks them up.
static CAUGHT: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// The fallback error handler used while this demo is open.
fn report_panics(mut error: BevyError, context: ErrorContext) {
    // Only an error that came from a panic carries a payload. Everything else is an ordinary
    // error and keeps Bevy's default behavior.
    let Some(payload) = error.take_payload() else {
        return match_severity(error, context);
    };
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("(payload is not a string)");
    error!("Caught a panic. {context}: {message}");
    // The full name of a command is long; the short one is enough on screen.
    let (kind, name) = (context.kind(), context.name());
    let caught = format!("in {kind} `{}`: {message}", name.shortname());
    CAUGHT.lock().unwrap().push(caught);
}

/// The handler that was active before the demo was opened.
#[derive(Resource)]
struct PreviousHandler(FallbackErrorHandler);

fn install_handler(world: &mut World) {
    let previous = FallbackErrorHandler(world.fallback_error_handler());
    world.insert_resource(PreviousHandler(previous));
    world.insert_resource(FallbackErrorHandler(report_panics));
}

fn restore_handler(world: &mut World) {
    if let Some(PreviousHandler(previous)) = world.remove_resource() {
        world.insert_resource(previous);
    }
}

/// The places a panic can be caught in.
#[derive(Clone, Copy)]
enum Fault {
    System,
    Command,
    Observer,
}

#[derive(Resource)]
struct Faults {
    /// Makes [`faulty_system`] panic the next time it runs.
    system_armed: bool,
    /// Faults that are triggered without a button press after the demo opens, last one first.
    script: Vec<Fault>,
    script_timer: Timer,
    frames: u32,
    caught: Vec<String>,
}

#[derive(Event)]
struct Explode;

fn faulty_system(mut faults: ResMut<Faults>) {
    if std::mem::take(&mut faults.system_armed) {
        panic!("faulty_system hit a bug");
    }
}

fn faulty_observer(_: On<Explode>) {
    panic!("faulty_observer hit a bug");
}

fn trigger(fault: Fault, faults: &mut Faults, commands: &mut Commands) {
    match fault {
        Fault::System => faults.system_armed = true,
        Fault::Command => commands.queue(|_: &mut World| {
            panic!("a queued command hit a bug");
        }),
        Fault::Observer => commands.trigger(Explode),
    }
}

fn setup(mut commands: Commands, root: Single<Entity, With<ContentRoot>>) {
    commands.insert_resource(Faults {
        system_armed: false,
        script: vec![Fault::Observer, Fault::Command, Fault::System],
        script_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
        frames: 0,
        caught: Vec::new(),
    });
    spawn_panel(
        &mut commands,
        *root,
        Demo::CatchPanics,
        "Each button really calls panic!(), and one panic of each kind is triggered \
         automatically after opening this demo. The frame counter keeps going because the \
         panics end up in the fallback error handler instead of unwinding out of the app. An \
         observer has no catch of its own: its panic is caught by the command that triggered it.",
        bsn_list! {
            @fault_button("Panic in system", Fault::System)
            --
            @fault_button("Panic in command", Fault::Command)
            --
            @fault_button("Panic in observer", Fault::Observer)
        },
    );
}

fn fault_button(label: &'static str, fault: Fault) -> impl Scene {
    button(
        label,
        move |_: On<Activate>, mut faults: ResMut<Faults>, mut commands: Commands| {
            trigger(fault, &mut faults, &mut commands);
        },
    )
}

fn report(
    time: Res<Time>,
    mut faults: ResMut<Faults>,
    mut log: ResMut<DemoLog>,
    mut readout: ResMut<Readout>,
    mut commands: Commands,
) {
    faults.frames += 1;
    if faults.script_timer.tick(time.delta()).just_finished()
        && let Some(fault) = faults.script.pop()
    {
        trigger(fault, &mut faults, &mut commands);
    }
    for caught in CAUGHT.lock().unwrap().drain(..) {
        log.push(format!(
            "caught panic #{} {caught}",
            faults.caught.len() + 1
        ));
        faults.caught.push(caught);
    }

    let recent = faults.caught.iter().rev().take(5).rev();
    readout.0 = format!(
        "frames since opening   {}\npanics caught          {}\n\n{}",
        faults.frames,
        faults.caught.len(),
        recent.map(String::as_str).collect::<Vec<_>>().join("\n"),
    );
}
