//! Player-specific behavior.

use avian2d::prelude::*;
use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    demo::{
        animation::PlayerAnimation,
        movement::{MovementController, ScreenWrap},
    },
};

const SHIP_SPEED: f32 = 320.0;
const ROTATION_SPEED: f32 = 240.0;
const POWERED_ANIMATION_INDICES: AnimationIndices = AnimationIndices { first: 0, last: 7 };

#[derive(Component)]
pub struct ShipSpeed(pub f32);

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        record_player_directional_input
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

/// The player character.
pub fn player(
    max_speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    (
        Name::new("Player"),
        Player,
        Sprite {
            image: player_assets.ducky.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            }),
            ..default()
        },
        Transform::from_scale(Vec2::splat(8.0).extend(1.0)),
        MovementController {
            max_speed,
            ..default()
        },
        ScreenWrap,
        player_animation,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Player;

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.intent = intent;
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                "images/ducky.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ShipAssets {
    #[dependency]
    pub fighter_base: Handle<Image>,
    #[dependency]
    pub fighter_engine_effect_sheet: Handle<Image>,
}

impl FromWorld for ShipAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            fighter_base: assets.load_with_settings(
                "placeholder/ships/nairan/base/Fighter - Base.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            fighter_engine_effect_sheet: assets.load_with_settings(
                "placeholder/ships/nairan/engine_effects/Fighter - Engine.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}

// pub fn nairan_fighter_ship(
//     ship_assets: &Res<ShipAssets>,
//     texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
// ) -> impl Bundle {
//     // A texture atlas is a way to split a single image into a grid of related images.
//     // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
//     let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 8, 1, None, None);
//     let texture_atlas_layout = texture_atlas_layouts.add(layout);

//     (
//         Name::new("Nairan Fighter"),
//         ShipSpeed(SHIP_SPEED),
//         Collider::capsule(8.0, 12.0),
//         Transform::from_xyz(0.0, 0.0, 10.0).with_scale(Vec3::splat(1.6)),
//         children![
//             (
//                 NairanFighterBase,
//                 Sprite::from_image(ship_assets.fighter_base.clone()),
//                 Transform::from_xyz(0.0, 0.0, 2.0),
//             ),
//             (
//                 NairanFighterEngineEffect,
//                 Sprite {
//                     image: ship_assets.fighter_engine_effect_sheet.clone(),
//                     texture_atlas: Some(TextureAtlas {
//                         layout: texture_atlas_layout,
//                         index: 0,
//                     }),
//                     ..default()
//                 },
//                 Transform::from_xyz(0.0, -0.3, 0.0),
//                 Visibility::Hidden, // will show effect later
//                 POWERED_ANIMATION_INDICES,
//                 AnimationTimer::with_fps(12.0),
//             ),
//         ],
//     )
// }
