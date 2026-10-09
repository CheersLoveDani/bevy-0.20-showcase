//! Bevy 0.20: `ContextExt::context` / `with_context` on `Result` and `Option`.
//!
//! Each level of a fallible call chain attaches a message describing what it was doing. The
//! resulting `BevyError` prints the whole chain, most recent message first.

use bevy::{prelude::*, ui_widgets::Activate};

use super::ui::{Readout, button, spawn_panel};
use crate::{
    demo::Demo,
    shell::{ContentRoot, ShellAppExt},
};

const SETTINGS: [&str; 3] = ["volume", "brightness", "fov"];

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::ErrorContext).add_systems(
        OnEnter(Demo::ErrorContext),
        (setup, |mut readout: ResMut<Readout>| {
            show("fov=wide", &mut readout)
        }),
    );
}

/// Level 1: `Result` with a context message built lazily by a closure.
fn parse_value(raw: &str) -> Result<u32> {
    raw.trim()
        .parse()
        .with_context(|| format!("`{raw}` is not a whole number"))
}

/// Level 2: `context` also turns an `Option` into a `Result`.
fn parse_setting(line: &str) -> Result<(&'static str, u32)> {
    let (key, raw) = line.split_once('=').context("Expected `name=value`")?;
    let name = SETTINGS
        .into_iter()
        .find(|name| *name == key.trim())
        .with_context(|| format!("There is no setting called `{key}`"))?;
    let value = parse_value(raw).with_context(|| format!("Invalid value for `{name}`"))?;
    Ok((name, value))
}

/// Level 3: the caller adds what it was trying to do overall.
fn apply_setting(line: &str) -> Result<String> {
    let (name, value) = parse_setting(line).with_context(|| format!("Failed to apply {line:?}"))?;
    Ok(format!("{name} is now {value}"))
}

fn setup(mut commands: Commands, root: Single<Entity, With<ContentRoot>>) {
    spawn_panel(
        &mut commands,
        *root,
        Demo::ErrorContext,
        "apply_setting calls parse_setting, which calls parse_value. Each one adds context \
         with `?`. Pick an input; an error is printed exactly as BevyError's Display writes it.",
        bsn_list! {
            @input_button("volume=7")
            --
            @input_button("fov=wide")
            --
            @input_button("gamma=2")
            --
            @input_button("brightness")
        },
    );
}

fn input_button(line: &'static str) -> impl Scene {
    button(
        line,
        move |_: On<Activate>, mut readout: ResMut<Readout>| {
            show(line, &mut readout);
        },
    )
}

fn show(line: &str, readout: &mut Readout) {
    readout.0 = match apply_setting(line) {
        Ok(applied) => format!("apply_setting({line:?})\n\nOk: {applied}"),
        // Bevy indents the causes with tabs, which are shown here as spaces.
        Err(error) => format!("apply_setting({line:?})\n\nErr:\n{error}").replace('\t', "    "),
    };
}
