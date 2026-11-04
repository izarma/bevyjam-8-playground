use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_ahoy::{input::AccumulatedInput, prelude::*};
use bevy_enhanced_input::prelude::*;
use bevy_yarnspinner::prelude::DialogueRunner;
use pixel_art::PixelSprite;

use crate::{
    GameplayScene, InControl, PausableSystems,
    camera::SceneCamera,
    gameplay::{
        Player, PlayerAnimation,
        animation::PlayerAnimationSystems,
        base::{BODY_HEIGHT, indoor_body},
    },
    loading::{PLAYER_FRAME, PlayerAssets},
};

const WALK_SPEED: f32 = 5.0;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct IndoorPlayerSpawn;

#[derive(Component, Clone, Default)]
struct IndoorPlayer;

/// Marker for the indoor player's sprite child
#[derive(Component)]
struct IndoorPlayerSprite;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(super) struct Touch;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_player_base)
        .add_input_context::<IndoorPlayer>()
        .sync_context_to_state::<IndoorPlayer, InControl>()
        .add_systems(
            Update,
            (
                sync_camera_x,
                exit_to_outdoors,
                update_player_animation.in_set(PlayerAnimationSystems::UpdateState),
            )
                .in_set(PausableSystems)
                .run_if(in_state(GameplayScene::Base)),
        )
        .add_systems(
            FixedPostUpdate,
            stop_player_during_dialogue
                .before(AhoySystems::MoveCharacters)
                .in_set(PausableSystems)
                .run_if(in_state(GameplayScene::Base)),
        );
}

const CAM_X_MIN: f32 = -15.0;
const CAM_X_MAX: f32 = 15.0;

/// Past this X coordinate the player leaves the base through the right exit.
const EXIT_X: f32 = 20.0;

fn sync_camera_x(
    player: Query<&Transform, (With<IndoorPlayer>, Without<Camera3d>)>,
    mut cameras: Query<&mut Transform, SceneCamera>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let x = player.translation.x.clamp(CAM_X_MIN, CAM_X_MAX);
    for mut camera in &mut cameras {
        camera.translation.x = x;
    }
}

fn exit_to_outdoors(
    player: Query<&Transform, With<IndoorPlayer>>,
    mut next_scene: ResMut<NextState<GameplayScene>>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    if player.translation.x >= EXIT_X {
        next_scene.set(GameplayScene::Outdoors);
    }
}

fn spawn_player_base(
    spawn: On<Insert<IndoorPlayerSpawn>>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    player_assets: Res<PlayerAssets>,
) {
    let Ok(feet) = transforms.get(spawn.entity) else {
        return;
    };
    let spawn = feet.translation() + Vec3::Y * BODY_HEIGHT / 2.0;

    let animation = PlayerAnimation::default();
    let sprite = PixelSprite {
        image: player_assets.player_sheet.clone(),
        size: Some(PLAYER_FRAME),
        cell: animation.frame_index() as u32,
        ..default()
    };
    let (collider, controller) = indoor_body(WALK_SPEED);

    commands
        .spawn_scene(bsn! {
            IndoorPlayer
            Player
            Transform::from_translation(spawn)
            collider
            controller
            ~ActiveInStates::<IndoorPlayer, InControl>::single(InControl)
            DespawnOnExit::<GameplayScene>(GameplayScene::Base)
        })
        .insert(children![(
            IndoorPlayerSprite,
            Transform::from_xyz(0.0, -BODY_HEIGHT / 2.0, 0.0),
            animation,
            sprite,
        )])
        .queue_spawn_related_scenes::<Actions<IndoorPlayer>>(bsn_list! {
            Action::<Movement>::new()
            DeadZone
            Bindings [
                {preset::cardinal::wasd_keys()}
                --
                {preset::axial::left_stick()}
            ]
            --
            Action::<Touch>::new()
            Bindings [
                Binding(KeyCode::KeyF)
                --
                Binding(GamepadButton::West)
            ]
        });
}

fn update_player_animation(
    camera: Single<&Transform, SceneCamera>,
    player: Single<(&LinearVelocity, &CharacterControllerState), With<IndoorPlayer>>,
    mut animation: Single<&mut PlayerAnimation, With<IndoorPlayerSprite>>,
    runners: Query<&DialogueRunner>,
) {
    let (velocity, state) = *player;

    let velocity = if runners.iter().any(|runner| runner.is_running()) {
        Vec3::ZERO
    } else {
        **velocity
    };
    animation.update(velocity, camera.rotation, state.grounded.is_some());
}

/// Cancels the player's horizontal movement while a dialogue is running so
/// they can't walk away mid-conversation
fn stop_player_during_dialogue(
    mut player: Query<(&mut AccumulatedInput, &mut LinearVelocity), With<IndoorPlayer>>,
    runners: Query<&DialogueRunner>,
) {
    let dialogue_active = runners.iter().any(|runner| runner.is_running());
    if !dialogue_active {
        return;
    }
    for (mut input, mut velocity) in &mut player {
        input.last_movement = None;
        velocity.x = 0.0;
        velocity.z = 0.0;
    }
}
