use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_ahoy::{input::AccumulatedInput, prelude::*};
use bevy_enhanced_input::prelude::*;
use bevy_yarnspinner::prelude::{DialogueRunner, YarnValue};
use pixel_art::PixelSprite;

use crate::{
    GameplayScene, InControl, PausableSystems,
    camera::SceneCamera,
    gameplay::{
        CharacterName, Player, PlayerAnimation, SpeakerSprite,
        animation::PlayerAnimationSystems,
        base::{BODY_HEIGHT, ground_below, indoor_body},
        entrance::{Entrance, through_door},
    },
    loading::{PLAYER_FRAME, PlayerAssets},
    loading_screen::{Arrived, Travel},
};

use super::npc::InDialogue;

const WALK_SPEED: f32 = 5.0;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct IndoorPlayerSpawn;

#[derive(Component, Clone, Default)]
struct IndoorPlayer;

/// Marker for the indoor player's sprite child.
#[derive(Component)]
#[require(SpeakerSprite)]
struct IndoorPlayerSprite;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(super) struct Touch;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_player_base)
        .add_observer(enter_base)
        .add_input_context::<IndoorPlayer>()
        .sync_context_to_state::<IndoorPlayer, InControl>()
        .add_systems(
            Update,
            (
                sync_camera,
                (
                    exit_to_outdoors,
                    update_player_animation.in_set(PlayerAnimationSystems::UpdateState),
                )
                    .in_set(PausableSystems),
            )
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

/// Camera y/z while the player is exploring the base (matches `lab_scene.bsn`).
const CAM_REST_Y: f32 = 2.1;
const CAM_REST_Z: f32 = 6.2;

/// Camera y/z while the player is talking to an NPC: lower and closer.
const CAM_DIALOGUE_Y: f32 = 1.4;
const CAM_DIALOGUE_Z: f32 = 3.5;

/// How quickly the camera eases between the rest and dialogue framing.
const CAM_ZOOM_SPEED: f32 = 6.0;

/// Past this X coordinate the player leaves the base through the right exit.
const EXIT_X: f32 = 20.0;
/// How far back from the exit the player is turned around when they can't leave yet.
const TURNED_BACK: f32 = 1.0;

fn sync_camera(
    player: Query<&Transform, (With<IndoorPlayer>, Without<Camera3d>)>,
    npc: Option<Single<&Transform, (With<InDialogue>, Without<Camera3d>)>>,
    mut cameras: Query<&mut Transform, SceneCamera>,
    time: Res<Time>,
) {
    let Ok(player) = player.single() else {
        return;
    };

    let talking = npc.is_some();
    // While zoomed into a conversation, frame the NPC instead of the player.
    let focus_x = if let Some(npc) = npc {
        npc.translation.x
    } else {
        player.translation.x
    };
    let x = focus_x.clamp(CAM_X_MIN, CAM_X_MAX);

    let (target_y, target_z) = if talking {
        (CAM_DIALOGUE_Y, CAM_DIALOGUE_Z)
    } else {
        (CAM_REST_Y, CAM_REST_Z)
    };
    let t = 1.0 - (-CAM_ZOOM_SPEED * time.delta_secs()).exp();

    for mut camera in &mut cameras {
        camera.translation.x = x;
        camera.translation.y = camera.translation.y.lerp(target_y, t);
        camera.translation.z = camera.translation.z.lerp(target_z, t);
    }
}

// make sure player treated guardian before leaving the first time
fn exit_to_outdoors(
    mut player: Query<&mut Transform, With<IndoorPlayer>>,
    mut runner: Single<&mut DialogueRunner>,
    mut commands: Commands,
) {
    let Ok(mut player) = player.single_mut() else {
        return;
    };
    if runner.is_running() || player.translation.x < EXIT_X {
        return;
    }
    let treated = matches!(
        runner.variable_storage().get("$treated_guardian"),
        Ok(YarnValue::Boolean(true))
    );
    if treated {
        commands.trigger(Travel::LeaveHouse);
        return;
    }
    player.translation.x = EXIT_X - TURNED_BACK;
    if let Err(error) = runner.try_start_node("LeaveBeforeTending") {
        error!("Failed to start the leave-before-tending dialogue: {error}");
        runner.stop();
    }
}

fn spawn_player_base(
    spawn: On<Insert<IndoorPlayerSpawn>>,
    arrived: Single<&Arrived>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    player_assets: Res<PlayerAssets>,
) {
    if through_door(&arrived) {
        return;
    }
    if let Ok(feet) = transforms.get(spawn.entity) {
        spawn_indoor_player(&mut commands, feet, &player_assets);
    }
}

fn enter_base(
    entrance: On<Insert<Entrance>>,
    arrived: Single<&Arrived>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    player_assets: Res<PlayerAssets>,
) {
    if arrived.0 != Travel::EnterHouse {
        return;
    }
    if let Ok(feet) = transforms.get(entrance.entity) {
        spawn_indoor_player(&mut commands, feet, &player_assets);
    }
}

fn spawn_indoor_player(
    commands: &mut Commands,
    feet: &GlobalTransform,
    player_assets: &PlayerAssets,
) {
    let spawn = feet.translation() + Vec3::Y * BODY_HEIGHT / 2.0;

    let animation = PlayerAnimation::default();
    let sprite = PixelSprite {
        image: player_assets.player_sheet.clone(),
        size: Some(PLAYER_FRAME),
        cell: animation.frame_index() as u32,
        ..default()
    };
    let (collider, controller) = indoor_body(WALK_SPEED);
    let sprite_at = ground_below(&controller);

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
        .insert(CharacterName("Aruna"))
        .insert(children![(
            IndoorPlayerSprite,
            Transform::from_xyz(0.0, sprite_at, 0.0),
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
                Binding(KeyCode::KeyZ)
                --
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
