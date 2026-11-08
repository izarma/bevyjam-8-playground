use bevy::prelude::*;
use jackdaw_runtime::prelude::*;
use pixel_art::PixelSprite;

use crate::{
    GameState,
    gameplay::{
        Facing, PlayerAnimation, apply_player_animation, clock::Clock, tick_player_animation,
    },
    loading::{PLAYER_FRAME, PlayerAssets},
    ui::{font, palette::HEADER_TEXT, scale::texels},
};

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct TitlePlayerSpawn;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_title_player)
        .add_systems(
            OnEnter(GameState::Title),
            (spawn_title_scene, spawn_game_title),
        )
        .add_systems(
            Update,
            (tick_player_animation, apply_player_animation)
                .chain()
                .run_if(in_state(GameState::Title)),
        );
}

const TITLE_HOUR: f32 = 16.75;

fn spawn_title_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("title.bsn")),
        DespawnOnExit(GameState::Title),
    ));
    commands.spawn((
        Name::new("Title Clock"),
        Clock {
            hours: TITLE_HOUR,
            ..default()
        },
        DespawnOnExit(GameState::Title),
    ));
}

fn spawn_game_title(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #GameTitle
        Node {
            position_type: PositionType::Absolute,
            top: texels(12),
            right: texels(12),
        }
        GlobalZIndex(2)
        Pickable::IGNORE
        DespawnOnExit<GameState>(GameState::Title)
        Children [
            Text("Fever")
            @font::text(4)
            TextColor(HEADER_TEXT)
            Pickable::IGNORE
        ]
    });
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
