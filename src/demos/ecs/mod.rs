//! Demos for the ECS features of Bevy 0.20. None of them are visual, so each one is a small
//! panel that runs the real API and prints what it measured.

use bevy::prelude::*;

mod catch_panics;
mod chain_weak;
mod despawn_all;
mod error_context;
mod schedule_shuffle;
mod summary_tick;
mod ui;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        ui::plugin,
        chain_weak::plugin,
        schedule_shuffle::plugin,
        catch_panics::plugin,
        despawn_all::plugin,
        error_context::plugin,
        summary_tick::plugin,
    ));
}
