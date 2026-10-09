//! A runnable tour of the features introduced in Bevy 0.20.

mod demo;
mod demos;
mod shell;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy 0.20 Showcase".into(),
                        resolution: (1400, 860).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // The texture compression demo needs the asset processor, which is only
                    // compiled in together with `demo_render`.
                    mode: if cfg!(feature = "demo_render") {
                        AssetMode::Processed
                    } else {
                        AssetMode::Unprocessed
                    },
                    ..default()
                }),
        )
        .add_plugins((shell::plugin, demos::plugin))
        .run();
}
