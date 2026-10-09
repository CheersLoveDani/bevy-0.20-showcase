//! One module per demo group, each behind its own cargo feature.

use bevy::prelude::*;

#[cfg(feature = "demo_ecs")]
mod ecs;
#[cfg(feature = "demo_mesh_shaders")]
mod mesh_shaders;
#[cfg(feature = "demo_render")]
mod render;
#[cfg(feature = "demo_solari")]
mod solari;
#[cfg(feature = "demo_ui_layout")]
mod ui_layout;
#[cfg(feature = "demo_ui_widgets")]
mod ui_widgets;

pub fn plugin(_app: &mut App) {
    #[cfg(feature = "demo_ui_widgets")]
    _app.add_plugins(ui_widgets::plugin);
    #[cfg(feature = "demo_ui_layout")]
    _app.add_plugins(ui_layout::plugin);
    #[cfg(feature = "demo_ecs")]
    _app.add_plugins(ecs::plugin);
    #[cfg(feature = "demo_render")]
    _app.add_plugins(render::plugin);
    #[cfg(feature = "demo_mesh_shaders")]
    _app.add_plugins(mesh_shaders::plugin);
    #[cfg(feature = "demo_solari")]
    _app.add_plugins(solari::plugin);
}
