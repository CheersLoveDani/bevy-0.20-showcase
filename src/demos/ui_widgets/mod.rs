//! UI widget demos: new Feathers widgets, headless tabs, contextual theming and inline content.

use bevy::{
    feathers::{
        containers::{subpane, subpane_body, subpane_header},
        display::{caption, label_dim},
    },
    prelude::*,
};

mod contextual_theming;
mod feathers_widgets;
mod headless_tabs;
mod inline_content;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        feathers_widgets::plugin,
        headless_tabs::plugin,
        contextual_theming::plugin,
        inline_content::plugin,
    ));
}

/// A titled Feathers sub-pane, `width` pixels wide, with a dimmed hint line above its `body`.
fn card(title: &'static str, hint: &'static str, width: f32, body: impl SceneList) -> impl Scene {
    bsn! {
        @subpane()
        Node { width: px(width) }
        Children [
            @subpane_header() Children [ @caption(title) ]
            --
            @subpane_body()
            Node {
                row_gap: px(10),
                padding: UiRect::all(px(12)),
            }
            Children [
                @label_dim(hint)
                --
                {body}
            ]
        ]
    }
}
