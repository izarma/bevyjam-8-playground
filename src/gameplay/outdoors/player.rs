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
    GameplayScene, InControl, PausableSystems,
    camera::{CameraSensitivity, SceneCamera, face_camera},
    gameplay::{
        CharacterName, Player, PlayerAnimation,
        animation::PlayerAnimationSystems,
        entrance::{Entrance, through_door},
        time_of_day::Nightlit,
    },
    loading::{PLAYER_FRAME, PlayerAssets},
    loading_screen::{Arrived, Travel},
};

const WALK_SPEED: f32 = 7.0;
// body sized to the sprite: a 32 texel frame is 2 m at 16 texels per meter
const CAPSULE_RADIUS: f32 = 0.5;
const CAPSULE_HEIGHT: f32 = 2.0;
const CAM_RADIUS: f32 = 10.0;
const CAM_LOOK_OFFSET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
// how far the camera keeps from whatever it would clip into
const CAM_CLEARANCE: f32 = 0.3;
const CAM_RETURN_RATE: f32 = 4.0;

const LANTERN_AT: Vec3 = Vec3::new(0.4, 0.0, 0.0);
const LANTERN_COLOR: Color = Color::srgb(1.0, 0.72, 0.4);
const LANTERN_INTENSITY: f32 = 600_000.0;
const LANTERN_RADIUS: f32 = 2.0;
const LANTERN_RANGE: f32 = 15.0;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_player)
        .add_observer(come_outside)
        .add_input_context::<ExamplePlayer>()
        .sync_context_to_state::<ExamplePlayer, InControl>()
        .add_systems(OnEnter(GameplayScene::Outdoors), set_cursor_locked::<true>)
        .add_systems(OnExit(GameplayScene::Outdoors), set_cursor_locked::<false>)
        .add_systems(
            OnExit(InControl),
            set_cursor_locked::<false>.run_if(in_state(GameplayScene::Outdoors)),
        )
        .add_systems(
            OnEnter(InControl),
            set_cursor_locked::<true>.run_if(in_state(GameplayScene::Outdoors)),
        )
        .add_systems(
            RunFixedMainLoop,
            (rotate_camera, apply_look)
                .chain()
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop)
                .run_if(in_state(InControl))
                .run_if(in_state(GameplayScene::Outdoors)),
        )
        .add_systems(
            Update,
            (
                (aim_camera, follow_player).chain().before(face_camera),
                update_animation
                    .in_set(PlayerAnimationSystems::UpdateState)
                    .in_set(PausableSystems),
            )
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
    arrived: Single<&Arrived>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    player_assets: Res<PlayerAssets>,
) {
    if through_door(&arrived) {
        return;
    }
    if let Ok(feet) = transforms.get(spawn.entity) {
        spawn_outdoor_player(&mut commands, feet, &player_assets);
    }
}

fn come_outside(
    entrance: On<Insert<Entrance>>,
    arrived: Single<&Arrived>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    player_assets: Res<PlayerAssets>,
) {
    if arrived.0 != Travel::LeaveHouse {
        return;
    }
    if let Ok(feet) = transforms.get(entrance.entity) {
        spawn_outdoor_player(&mut commands, feet, &player_assets);
    }
}

fn spawn_outdoor_player(
    commands: &mut Commands,
    feet: &GlobalTransform,
    player_assets: &PlayerAssets,
) {
    let spawn = feet.translation() + Vec3::Y * CAPSULE_HEIGHT / 2.0;
    let facing = feet.forward().xz().normalize_or(Vec2::NEG_Y);
    let look = CharacterLook {
        yaw: facing.x.atan2(facing.y),
        pitch: 0.0,
    };

    let mut animation = PlayerAnimation::default();
    animation.face_towards(facing);
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
            ~ActiveInStates::<ExamplePlayer, InControl>::single(InControl)
            look
            DespawnOnExit::<GameplayScene>(GameplayScene::Outdoors)
        })
        .insert(CharacterName("Aruna"))
        .insert((
            ExamplePlayer,
            children![
                (
                    ExamplePlayerSprite,
                    // feet on the ground
                    Transform::from_xyz(
                        0.0,
                        -CAPSULE_HEIGHT / 2.0
                            - CharacterController::default().move_and_slide.skin_width,
                        0.0,
                    ),
                    animation,
                    sprite,
                ),
                (
                    Name::new("Lantern"),
                    Transform::from_translation(LANTERN_AT),
                    PointLight {
                        color: LANTERN_COLOR,
                        intensity: 0.0,
                        range: LANTERN_RANGE,
                        radius: LANTERN_RADIUS,
                        shadow_maps_enabled: false,
                        ..default()
                    },
                    Nightlit {
                        intensity: LANTERN_INTENSITY,
                    },
                ),
            ],
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

fn aim_camera(
    players: Query<&CharacterLook, Added<ExamplePlayer>>,
    mut camera: Single<&mut Transform, SceneCamera>,
) {
    for look in &players {
        let (_, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
        camera.rotation = Quat::from_euler(EulerRot::YXZ, look.yaw, pitch, roll);
    }
}

fn follow_player(
    player: Single<(Entity, &Transform), (With<ExamplePlayer>, Without<Camera3d>)>,
    mut camera: Single<&mut Transform, SceneCamera>,
    spatial: SpatialQuery,
    time: Res<Time>,
    mut distance: Local<f32>,
) {
    let (player, at) = *player;
    let look_at = at.translation + CAM_LOOK_OFFSET;
    let back = camera.rotation * Dir3::Z;
    let clear = spatial
        .cast_shape(
            &Collider::sphere(CAM_CLEARANCE),
            look_at,
            Quat::IDENTITY,
            back,
            &ShapeCastConfig {
                max_distance: CAM_RADIUS,
                ignore_origin_penetration: true,
                ..default()
            },
            &SpatialQueryFilter::from_excluded_entities([player]),
        )
        .map_or(CAM_RADIUS, |hit| hit.distance);
    *distance = if clear < *distance {
        clear
    } else {
        clear.lerp(*distance, (-CAM_RETURN_RATE * time.delta_secs()).exp())
    };
    camera.translation = look_at + back * *distance;
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
