use bevy::prelude::*;
use jackdaw_runtime::prelude::*;
use pixel_art::PixelSprite;

use crate::{
    GameState, Survivors,
    gameplay::{
        Facing, PlayerAnimation, apply_player_animation, clock::Clock, npc_idle_sprite,
        tick_player_animation,
    },
    loading::{NPCAssets, PLAYER_FRAME, PlayerAssets},
    ui::{font, palette::HEADER_TEXT, scale::texels},
};

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct TitlePlayerSpawn;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Campfire;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct EndingSpot(pub usize);

#[derive(Component)]
struct Gathered;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_title_player)
        .add_systems(
            OnEnter(GameState::Title),
            (spawn_title_scene, spawn_game_title),
        )
        .add_systems(OnEnter(GameState::GameOver), spawn_ending_scene)
        .add_systems(
            Update,
            (tick_player_animation, apply_player_animation)
                .chain()
                .run_if(in_state(GameState::Title).or_else(in_state(GameState::GameOver))),
        )
        .add_systems(
            Update,
            (light_campfire, gather_survivors).run_if(in_state(GameState::GameOver)),
        );
}

const TITLE_HOUR: f32 = 16.75;
const ENDING_HOUR: f32 = 21.0;

fn spawn_title_scene(commands: Commands, asset_server: Res<AssetServer>) {
    spawn_scene_at(commands, asset_server, GameState::Title, TITLE_HOUR);
}

fn spawn_ending_scene(commands: Commands, asset_server: Res<AssetServer>) {
    spawn_scene_at(commands, asset_server, GameState::GameOver, ENDING_HOUR);
}

fn spawn_scene_at(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: GameState,
    hours: f32,
) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("title.bsn")),
        DespawnOnExit(state.clone()),
    ));
    commands.spawn((
        Name::new("Title Clock"),
        Clock { hours, ..default() },
        DespawnOnExit(state.clone()),
    ));
}

fn light_campfire(mut campfires: Query<&mut Visibility, With<Campfire>>) {
    for mut visibility in &mut campfires {
        visibility.set_if_neq(Visibility::Inherited);
    }
}

fn gather_survivors(
    mut commands: Commands,
    spots: Query<(Entity, &EndingSpot), Without<Gathered>>,
    survivors: Option<Res<Survivors>>,
    assets: Res<NPCAssets>,
) {
    for (spot, &EndingSpot(index)) in &spots {
        commands.entity(spot).insert(Gathered);
        let Some(&name) = survivors
            .as_ref()
            .and_then(|survivors| survivors.0.get(index))
        else {
            continue;
        };
        let (image, frame) = assets.sprite(name);
        commands.spawn((
            Name::new(name),
            npc_idle_sprite(image, frame),
            Transform::default(),
            ChildOf(spot),
        ));
    }
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
    state: Res<State<GameState>>,
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
        DespawnOnExit(state.get().clone()),
    ));
}
