//! Better texture compression: with the `compressed_image_saver` and `asset_processor` cargo
//! features and `AssetMode::Processed`, every PNG / JPEG is run through `CompressedImageSaver`
//! (now backed by `ctt`), which writes a BCn or ASTC KTX2 file with a full mip chain.
//!
//! Both floors below show the same picture: `tiles.png` takes that default path, while
//! `tiles_unprocessed.png` has a `.meta` file that opts out and loads the PNG as it is.
//!
//! Processing is switched on for the whole app, where `AssetPlugin` is configured
//! (`mode: AssetMode::Processed` in `main.rs`). With `AssetMode::Unprocessed` this demo still
//! runs, but both textures are then plain PNGs and it says so on screen.

use bevy::{
    asset::AssetServerMode,
    camera::ClearColorConfig,
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    math::Affine2,
    prelude::*,
};

use super::{ContentViewport, PanelText, spawn_panel};
use crate::{demo::Demo, shell::ContentRoot, shell::ShellAppExt};

pub fn plugin(app: &mut App) {
    app.register_demo(Demo::TextureCompression)
        .add_systems(OnEnter(Demo::TextureCompression), setup)
        .add_systems(
            Update,
            (scroll_floors, update_readout).run_if(in_state(Demo::TextureCompression)),
        );
}

/// One of the two textured floors.
#[derive(Component)]
struct Floor {
    label: &'static str,
    image: Handle<Image>,
}

fn setup(
    root: Single<Entity, With<ContentRoot>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let demo = DespawnOnExit(Demo::TextureCompression);
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Transform::from_xyz(0.0, 1.4, 4.0).looking_at(Vec3::new(0.0, 0.0, -6.0), Vec3::Y),
        ContentViewport,
        demo.clone(),
    ));

    // Long floors running away from the camera, so the texture gets heavily minified.
    let mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::new(2.4, 40.0)));
    let floors = [
        (
            -2.5,
            "Left,  tiles_unprocessed.png",
            "render/tiles_unprocessed.png",
        ),
        (2.5, "Right, tiles.png", "render/tiles.png"),
    ];
    for (x, label, path) in floors {
        let image: Handle<Image> = asset_server
            .load_builder()
            .with_settings(|settings: &mut ImageLoaderSettings| {
                // Tile the texture and blend between mip levels, when there are any.
                settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    mipmap_filter: ImageFilterMode::Linear,
                    ..ImageSamplerDescriptor::linear()
                });
            })
            .load(path);
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(image.clone()),
                unlit: true,
                uv_transform: Affine2::from_scale(Vec2::new(2.0, 32.0)),
                ..default()
            })),
            Transform::from_xyz(x, 0.0, -38.0),
            Floor { label, image },
            demo.clone(),
        ));
    }

    spawn_panel(&mut commands, *root, Demo::TextureCompression, "", ());
}

/// Moving the texture makes the aliasing of the floor without mipmaps easy to see.
fn scroll_floors(
    floors: Query<&MeshMaterial3d<StandardMaterial>, With<Floor>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for material in &floors {
        if let Some(mut material) = materials.get_mut(&material.0) {
            let offset = &mut material.uv_transform.translation.y;
            *offset = (*offset + time.delta_secs() * 0.4).fract();
        }
    }
}

/// Prints what actually got loaded: the format and mip count of each `Image`.
fn update_readout(
    floors: Query<&Floor>,
    images: Res<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut text: Single<&mut Text, With<PanelText>>,
) {
    let mut readout = match asset_server.mode() {
        AssetServerMode::Processed => "Asset processing is on: tiles.png was compressed by \
            CompressedImageSaver and is loaded from imported_assets/."
            .to_string(),
        AssetServerMode::Unprocessed => "Asset processing is OFF (AssetPlugin::mode is \
            AssetMode::Unprocessed), so tiles.png is loaded as a plain PNG too and both floors \
            alias. Set AssetMode::Processed in main.rs to see the compressed texture."
            .to_string(),
    };
    for floor in floors.iter().sort_by_key::<&Floor, _>(|floor| floor.label) {
        let details = match images.get(&floor.image) {
            Some(image) => format!(
                "{:?}, {} mip level(s), {} KiB of pixel data",
                image.texture_descriptor.format,
                image.texture_descriptor.mip_level_count,
                image.data.as_ref().map_or(0, Vec::len) / 1024,
            ),
            None => "loading...".to_string(),
        };
        readout += &format!("\n{}: {details}", floor.label);
    }
    if text.0 != readout {
        text.0 = readout;
    }
}
