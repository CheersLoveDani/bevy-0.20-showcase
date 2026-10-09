//! Inline content in text, new in Bevy 0.20. `InlineImage` (bevy_ui) draws an image in the
//! flow of a `Text`'s glyphs. `InlineBox` (bevy_text) only reserves space in the layout: the
//! resulting rectangle is reported in `TextLayoutInfo::inline_boxes`, and here a real Feathers
//! button is positioned over it.

use bevy::{
    asset::RenderAssetUsages,
    color::palettes::tailwind,
    feathers::{
        constants::icons,
        controls::{FeathersButton, FeathersCheckbox, FeathersSlider},
        display::{caption, label},
    },
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    text::{InlineBox, InlineBoxKind, TextLayoutInfo},
    ui::Checked,
    ui_widgets::{
        Activate, SliderPrecision, SliderValue, ValueChange, checkbox_self_update,
        slider_self_update,
    },
};

use crate::{
    demo::Demo,
    shell::{ContentRoot, DemoLog, ShellAppExt},
};

const START_WIDTH: f32 = 760.0;
const FONT_SIZE: f32 = 22.0;
/// Size reserved in the text for the button, in logical pixels.
const SLOT_SIZE: Vec2 = Vec2::new(104.0, 24.0);

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::InlineContent)
        .add_systems(OnEnter(Demo::InlineContent), setup)
        .add_systems(
            Update,
            place_slot_content.run_if(in_state(Demo::InlineContent)),
        );
}

/// The node whose width the slider changes. It holds the text and the node drawn in its slot.
#[derive(Component)]
struct Paragraph;

/// The text that contains the inline images and the [`ButtonSlot`].
#[derive(Component)]
struct FlowingText;

/// The `InlineBox` that reserves room for a button.
#[derive(Component)]
struct ButtonSlot;

/// The node positioned over the [`ButtonSlot`]'s rectangle.
#[derive(Component, Default, Clone)]
struct SlotContent;

fn setup(
    root: Single<Entity, With<ContentRoot>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let orb = images.add(orb_image());
    let chevron: Handle<Image> = asset_server.load(icons::CHEVRON_RIGHT);

    let page = commands
        .spawn((
            ChildOf(*root),
            DespawnOnExit(Demo::InlineContent),
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(20)),
                row_gap: px(16),
                ..default()
            },
        ))
        .id();
    commands.spawn_scene(controls()).insert(ChildOf(page));

    let frame = commands
        .spawn((
            ChildOf(page),
            Node {
                align_self: AlignSelf::Start,
                padding: UiRect::all(px(16)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.11, 0.12, 0.15)),
        ))
        .id();
    // Inline box rectangles are relative to the text node, so the text sits at the origin of
    // an unpadded parent that the slot content can be absolutely positioned in.
    let paragraph = commands
        .spawn((
            ChildOf(frame),
            Paragraph,
            Node {
                width: px(START_WIDTH),
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(paragraph),
        FlowingText,
        Text::new("Text layouts can now carry more than glyphs. "),
        body_font(),
        children![
            InlineImage {
                image: orb.clone(),
                height: Some(FONT_SIZE),
                ..default()
            },
            span(" An InlineImage flows with the words and can be tinted "),
            InlineImage {
                image: orb.clone(),
                color: tailwind::ROSE_400.into(),
                height: Some(FONT_SIZE),
                ..default()
            },
            span(" or sized by either axis, keeping its aspect ratio "),
            InlineImage {
                image: orb,
                color: tailwind::SKY_400.into(),
                width: Some(FONT_SIZE * 1.4),
                ..default()
            },
            span(". Any image asset works, such as a Feathers icon "),
            InlineImage {
                image: chevron,
                color: tailwind::AMBER_300.into(),
                height: Some(FONT_SIZE * 0.7),
                ..default()
            },
            span(". An InlineBox only reserves space: this "),
            (
                ButtonSlot,
                InlineBox {
                    kind: InlineBoxKind::InFlow,
                    size: SLOT_SIZE,
                },
            ),
            span(
                " is an ordinary button, placed using TextLayoutInfo::inline_boxes. Narrow the \
                paragraph and everything rewraps together."
            ),
        ],
    ));

    commands
        .spawn_scene(bsn! {
            SlotContent
            Node {
                position_type: PositionType::Absolute,
                width: px(SLOT_SIZE.x),
                height: px(SLOT_SIZE.y),
            }
            Children [
                @FeathersButton { @caption: bsn! { @caption("Click me") } }
                Node { flex_grow: 1.0 }
                on(|_: On<Activate>, mut log: ResMut<DemoLog>| {
                    log.push("The button inside the paragraph was clicked");
                })
            ]
        })
        .insert(ChildOf(paragraph));
}

fn controls() -> impl Scene {
    bsn! {
        Node {
            align_items: AlignItems::Center,
            column_gap: px(12),
        }
        Children [
            @label("Paragraph width")
            --
            @FeathersSlider { @min: 280.0, @max: 1040.0 }
            SliderValue(START_WIDTH)
            SliderPrecision(0)
            Node { width: px(260), flex_grow: 0.0 }
            on(slider_self_update)
            on(|change: On<ValueChange<f32>>, mut paragraph: Single<&mut Node, With<Paragraph>>| {
                paragraph.width = px(change.value);
            })
            --
            @FeathersCheckbox { @caption: bsn! { @caption("InlineBoxKind::InFlow (off: OutOfFlow, takes no space)") } }
            Checked
            on(checkbox_self_update)
            on(|change: On<ValueChange<bool>>, mut slot: Single<&mut InlineBox, With<ButtonSlot>>| {
                slot.kind = if change.value {
                    InlineBoxKind::InFlow
                } else {
                    InlineBoxKind::OutOfFlow
                };
            })
        ]
    }
}

/// Moves the [`SlotContent`] node onto the rectangle the text layout assigned to the slot.
fn place_slot_content(
    slot: Single<Entity, With<ButtonSlot>>,
    text: Single<(&TextLayoutInfo, &ComputedNode), With<FlowingText>>,
    mut content: Single<&mut Node, With<SlotContent>>,
) {
    let (layout, computed) = *text;
    let Some((_, _, rect)) = layout
        .inline_boxes
        .iter()
        .find(|(entity, ..)| *entity == *slot)
    else {
        return;
    };
    // Layout rectangles are in physical pixels.
    let origin = rect.min * computed.inverse_scale_factor();
    let (left, top) = (px(origin.x), px(origin.y));
    if content.left != left || content.top != top {
        content.left = left;
        content.top = top;
    }
}

fn span(text: &str) -> impl Bundle {
    (TextSpan::new(text), body_font())
}

fn body_font() -> TextFont {
    TextFont {
        font_size: FontSize::Px(FONT_SIZE),
        ..default()
    }
}

/// A shaded white sphere on a transparent background; `InlineImage::color` tints it.
fn orb_image() -> Image {
    const SIZE: u32 = 64;
    let pixels = (0..SIZE * SIZE).flat_map(|i| {
        // Position in -1..1 with the sphere filling the image.
        let p = (UVec2::new(i % SIZE, i / SIZE).as_vec2() + 0.5) / SIZE as f32 * 2.0 - 1.0;
        let distance = p.length();
        let highlight = 1.0 - p.distance(Vec2::splat(-0.35)) * 0.55;
        let value = (highlight.clamp(0.25, 1.0) * 255.0) as u8;
        // One pixel of anti-aliasing at the rim.
        let alpha = ((1.0 - distance) * SIZE as f32 / 2.0).clamp(0.0, 1.0);
        [value, value, value, (alpha * 255.0) as u8]
    });
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels.collect(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
