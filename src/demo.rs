//! The list of showcase scenes. Each variant is one Bevy 0.20 feature.

use bevy::prelude::*;

#[derive(States, Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Demo {
    #[default]
    Home,
    // UI
    FeathersWidgets,
    HeadlessTabs,
    ContextualTheming,
    InlineContent,
    EmRem,
    FixedNode,
    BorderRadius,
    // Scenes
    BsnSyntax,
    ReadyEvent,
    // 2D
    SpriteMaterial,
    ExtendedMaterial2d,
    SpriteStress,
    // 3D / rendering
    PanOrbitCamera,
    WeslMaterial,
    TextureCompression,
    MeshShaders,
    Solari,
    // ECS
    ChainWeak,
    ScheduleShuffle,
    CatchPanics,
    DespawnAll,
    ErrorContext,
    SummaryTick,
}

impl Demo {
    pub const ALL: [Demo; 24] = [
        Demo::Home,
        Demo::FeathersWidgets,
        Demo::HeadlessTabs,
        Demo::ContextualTheming,
        Demo::InlineContent,
        Demo::EmRem,
        Demo::FixedNode,
        Demo::BorderRadius,
        Demo::BsnSyntax,
        Demo::ReadyEvent,
        Demo::SpriteMaterial,
        Demo::ExtendedMaterial2d,
        Demo::SpriteStress,
        Demo::PanOrbitCamera,
        Demo::WeslMaterial,
        Demo::TextureCompression,
        Demo::MeshShaders,
        Demo::Solari,
        Demo::ChainWeak,
        Demo::ScheduleShuffle,
        Demo::CatchPanics,
        Demo::DespawnAll,
        Demo::ErrorContext,
        Demo::SummaryTick,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Demo::Home => "Bevy 0.20 Showcase",
            Demo::FeathersWidgets => "More Feathers Widgets",
            Demo::HeadlessTabs => "Headless Tab Widgets",
            Demo::ContextualTheming => "Contextual Theming",
            Demo::InlineContent => "InlineBox and InlineImage",
            Demo::EmRem => "Val::Em and Val::Rem",
            Demo::FixedNode => "FixedNode",
            Demo::BorderRadius => "Elliptical Border Radius",
            Demo::BsnSyntax => "BSN Syntax Improvements",
            Demo::ReadyEvent => "Ready Event",
            Demo::SpriteMaterial => "Sprite Materials",
            Demo::ExtendedMaterial2d => "2D Extended Materials",
            Demo::SpriteStress => "Sprite Backend Unification",
            Demo::PanOrbitCamera => "Pan Orbit Camera",
            Demo::WeslMaterial => "WESL Shaders",
            Demo::TextureCompression => "Better Texture Compression",
            Demo::MeshShaders => "Mesh Shaders",
            Demo::Solari => "Solari and DLSS",
            Demo::ChainWeak => "Weak System Ordering",
            Demo::ScheduleShuffle => "Schedule Randomization",
            Demo::CatchPanics => "Catching Panics",
            Demo::DespawnAll => "Faster Bulk Despawning",
            Demo::ErrorContext => "Error Context Messages",
            Demo::SummaryTick => "Per-Column Change Ticks",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Demo::Home => "Pick a feature on the left. This launcher is itself built with BSN, Feathers, a FixedNode sidebar and rem-sized text.",
            Demo::FeathersWidgets => "Color input with picker popup, list view, dropdown select, lazy menu and a scrubbable number input.",
            Demo::HeadlessTabs => "Unstyled TabList / Tab behavior from bevy_ui_widgets, with keyboard navigation and external selection state.",
            Demo::ContextualTheming => "Theme colors resolve differently depending on the SurfaceLevel of the parent container.",
            Demo::InlineContent => "InlineBox reserves space inside a text layout; InlineImage places an image inline with the glyphs.",
            Demo::EmRem => "UI lengths relative to the current font size (em) and the root font size (rem).",
            Demo::FixedNode => "A node positioned relative to the viewport, ignoring its parent's layout, clipping and transform.",
            Demo::BorderRadius => "BorderRadius corners are CornerRadius values with separate horizontal and vertical radii.",
            Demo::BsnSyntax => "@scene references, `--` entity separators and plain enum variants in bsn! / bsn_list!.",
            Demo::ReadyEvent => "Ready fires bottom-up on each BSN entity once it and all of its descendants have spawned.",
            Demo::SpriteMaterial => "Sprites drawn with a custom shader through SpriteMaterial.",
            Demo::ExtendedMaterial2d => "ExtendedMaterial2d layers a custom fragment shader on top of ColorMaterial.",
            Demo::SpriteStress => "Sprites now render through the same retained infrastructure as 3D meshes. Spawn a lot of them.",
            Demo::PanOrbitCamera => "The CAD-style PanOrbitCamera from bevy_camera_controller. Drag to orbit, right-drag to pan, scroll to zoom.",
            Demo::WeslMaterial => "A custom material written in WESL, using `import` and `@if` conditional compilation.",
            Demo::TextureCompression => "CompressedImageSaver produces BCn / ASTC textures with mipmaps during asset processing.",
            Demo::MeshShaders => "Geometry generated on the GPU by a mesh shader instead of a vertex shader.",
            Demo::Solari => "Realtime path-traced lighting, with optional DLSS Ray Reconstruction.",
            Demo::ChainWeak => "chain_weak() only orders systems whose data access conflicts, so the rest can run in parallel.",
            Demo::ScheduleShuffle => "Shuffle the order of ambiguous systems with a seed to expose hidden order dependencies.",
            Demo::CatchPanics => "Panics in systems, commands and observers become errors routed to the fallback error handler.",
            Demo::DespawnAll => "A batched despawn_all command, compared against despawning entities one by one.",
            Demo::ErrorContext => "BevyError::context / with_context attach extra messages to an error as it propagates.",
            Demo::SummaryTick => "#[component(summary_tick)] lets contiguous queries skip whole unchanged tables.",
        }
    }
}
