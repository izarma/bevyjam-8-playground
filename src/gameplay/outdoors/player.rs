use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::*;
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_ahoy::{CharacterLook, prelude::*};
use bevy_enhanced_input::prelude::*;
use pixel_art::PixelSprite;

use crate::{
    GameplayScene, PausableSystems, Pause,
    camera::{CameraSensitivity, SceneCamera, face_camera},
    gameplay::{Player, PlayerAnimation, animation::PlayerAnimationSystems},
    loading::{PLAYER_FRAME, PlayerAssets},
    ui::menus::Menu,
};

const WALK_SPEED: f32 = 7.0;
// body sized to the sprite: a 32 texel frame is 2 m at 16 texels per meter
const CAPSULE_RADIUS: f32 = 0.5;
const CAPSULE_HEIGHT: f32 = 2.0;
const CAM_RADIUS: f32 = 10.0;
const CAM_LOOK_OFFSET: Vec3 = Vec3::new(0.0, 1.0, 0.0);

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_player)
        .add_input_context::<ExamplePlayer>()
        .sync_context_to_state::<ExamplePlayer, Pause>()
        .add_systems(OnEnter(GameplayScene::Outdoors), set_cursor_locked::<true>)
        .add_systems(OnExit(GameplayScene::Outdoors), set_cursor_locked::<false>)
        .add_systems(
            OnExit(Menu::None),
            set_cursor_locked::<false>.run_if(in_state(GameplayScene::Outdoors)),
        )
        .add_systems(
            OnEnter(Menu::None),
            set_cursor_locked::<true>.run_if(in_state(GameplayScene::Outdoors)),
        )
        .add_systems(
            RunFixedMainLoop,
            (rotate_camera, apply_look)
                .chain()
                .in_set(PausableSystems)
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop)
                .run_if(in_state(GameplayScene::Outdoors)),
        )
        .add_systems(
            Update,
            (
                follow_player.before(face_camera),
                update_animation.in_set(PlayerAnimationSystems::UpdateState),
            )
                .in_set(PausableSystems)
                .run_if(in_state(GameplayScene::Outdoors)),
        );
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct PlayerSpawn;

#[derive(Component)]
struct ExamplePlayer;

#[derive(Component)]
struct ExamplePlayerSprite;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(super) struct Harvest;

fn spawn_player(
    spawn: On<Insert<PlayerSpawn>>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    player_assets: Res<PlayerAssets>,
) {
    let Ok(feet) = transforms.get(spawn.entity) else {
        return;
    };
    let spawn = feet.translation() + Vec3::Y * CAPSULE_HEIGHT / 2.0;

    let animation = PlayerAnimation::default();
    let sprite = PixelSprite {
        image: player_assets.player_sheet.clone(),
        size: Some(PLAYER_FRAME),
        cell: animation.frame_index() as u32,
        ..default()
    };

    commands
        .spawn_scene(bsn! {
            Player
            Transform::from_translation(spawn)
            Collider::capsule(CAPSULE_RADIUS, CAPSULE_HEIGHT - 2.0 * CAPSULE_RADIUS)
            CharacterController {
                speed: WALK_SPEED,
                standing_view_height: 1.8,
                crouch_height: 1.4,
                crouch_view_height: 1.2,
                step_size: 0.7,
                jump_height: 2.0,
                mantle_height: 1.1,
                crane_height: 1.6,
                min_ledge_grab_space: Cuboid::new(0.2, 0.1, 0.2),
                ..CharacterController::default()
            }
            ~ActiveInStates::<ExamplePlayer, Pause>::single(Pause(false))
            CharacterLook::default()
            DespawnOnExit::<GameplayScene>(GameplayScene::Outdoors)
        })
        .insert((
            ExamplePlayer,
            children![(
                ExamplePlayerSprite,
                // feet at the bottom of the capsule
                Transform::from_xyz(0.0, -CAPSULE_HEIGHT / 2.0, 0.0),
                animation,
                sprite,
            )],
        ))
        .queue_spawn_related_scenes::<Actions<ExamplePlayer>>(bsn_list! {
            Action::<Movement>::new()
            DeadZone
            Bindings [
                {preset::cardinal::wasd_keys()}
                --
                {preset::axial::left_stick()}
            ]
            --
            Action::<Jump>::new()
            Bindings [
                Binding(KeyCode::Space)
                --
                Binding(GamepadButton::South)
            ]
            --
            Action::<Harvest>::new()
            Bindings [
                Binding(KeyCode::KeyF)
                --
                Binding(GamepadButton::West)
            ]
        });
}

fn rotate_camera(
    mouse_motion: Res<AccumulatedMouseMotion>,
    sensitivity: Res<CameraSensitivity>,
    mut camera: Single<&mut Transform, SceneCamera>,
) {
    let delta = mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }
    const PITCH_MIN: f32 = -0.4 * FRAC_PI_2;
    const PITCH_MAX: f32 = 0.3 * FRAC_PI_2;
    let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
    let yaw = yaw - delta.x * sensitivity.x;
    let pitch = (pitch - delta.y * sensitivity.y).clamp(PITCH_MIN, PITCH_MAX);
    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
}

fn apply_look(
    camera: Single<&Transform, SceneCamera>,
    mut look: Single<&mut CharacterLook, With<ExamplePlayer>>,
) {
    let (yaw, _, _) = camera.rotation.to_euler(EulerRot::YXZ);
    look.yaw = yaw;
}

fn follow_player(
    player: Single<&Transform, (With<ExamplePlayer>, Without<Camera3d>)>,
    mut camera: Single<&mut Transform, SceneCamera>,
) {
    let look_at = player.translation + CAM_LOOK_OFFSET;
    camera.translation = look_at + camera.rotation * Vec3::new(0.0, 0.0, CAM_RADIUS);
}

fn update_animation(
    camera: Single<&Transform, SceneCamera>,
    player: Single<(&LinearVelocity, &CharacterControllerState), With<ExamplePlayer>>,
    mut animation: Single<&mut PlayerAnimation, With<ExamplePlayerSprite>>,
) {
    let (velocity, state) = *player;
    animation.update(**velocity, camera.rotation, state.grounded.is_some());
}

fn set_cursor_locked<const LOCKED: bool>(
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    cursor.grab_mode = if LOCKED {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    cursor.visible = !LOCKED;
}
