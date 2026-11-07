use bevy::prelude::*;
use jackdaw_runtime::prelude::*;
use pixel_art::PixelSprite;

use crate::{
    GameState,
    gameplay::{Facing, PlayerAnimation, apply_player_animation, tick_player_animation},
    loading::{PLAYER_FRAME, PlayerAssets},
};

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct TitlePlayerSpawn;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_title_player)
        .add_systems(OnEnter(GameState::Title), spawn_title_scene)
        .add_systems(
            Update,
            (tick_player_animation, apply_player_animation)
                .chain()
                .run_if(in_state(GameState::Title)),
        );
}

fn spawn_title_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("title.bsn")),
        DespawnOnExit(GameState::Title),
    ));
}

fn spawn_title_player(
    spawn: On<Insert<TitlePlayerSpawn>>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    player_assets: Res<PlayerAssets>,
) {
    let Ok(feet) = transforms.get(spawn.entity) else {
        return;
    };

    let animation = PlayerAnimation {
        facing: Facing::Side,
        ..default()
    };
    let sprite = PixelSprite {
        image: player_assets.player_sheet.clone(),
        size: Some(PLAYER_FRAME),
        cell: animation.frame_index() as u32,
        ..default()
    };

    commands.spawn((
        Name::new("Title Player"),
        Transform::from_translation(feet.translation()),
        animation,
        sprite,
        DespawnOnExit(GameState::Title),
    ));
}
