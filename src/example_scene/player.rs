use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::*;
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_ahoy::{CharacterLook, prelude::*};
use bevy_enhanced_input::prelude::*;
use bevy_sprite3d::Sprite3d;
use jackdaw_runtime::TerrainHeights;

use crate::{
    GameState,
    camera::{CameraSensitivity, GameplayCamera},
    gameplay::{PlayerAnimation, apply_player_animation, tick_player_animation},
    loading::PlayerAssets,
};

const WALK_SPEED: f32 = 4.0;
const RUN_SPEED: f32 = 7.0;
const CAPSULE_RADIUS: f32 = 0.25;
const CAPSULE_HEIGHT: f32 = 1.0;
const CAM_RADIUS: f32 = 5.0;
const CAM_LOOK_OFFSET: Vec3 = Vec3::new(0.0, 0.5, 0.0);

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AhoyPlugins::default())
        .add_input_context::<ExamplePlayer>()
        .add_systems(OnEnter(GameState::ExampleScene), set_cursor_locked::<true>)
        .add_systems(OnExit(GameState::ExampleScene), set_cursor_locked::<false>)
        .add_systems(
            RunFixedMainLoop,
            (rotate_camera, apply_look, apply_run)
                .chain()
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop)
                .run_if(in_state(GameState::ExampleScene)),
        )
        .add_systems(
            Update,
            (
                spawn_player,
                follow_player,
                face_camera,
                tick_player_animation,
                update_animation,
                apply_player_animation,
            )
                .chain()
                .run_if(in_state(GameState::ExampleScene)),
        );
}

#[derive(Component)]
struct ExamplePlayer;

#[derive(Component)]
struct ExamplePlayerSprite;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct Run;

fn spawn_player(
    mut commands: Commands,
    terrains: Query<(&TerrainHeights, &GlobalTransform), Added<TerrainHeights>>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let Some((heights, terrain_transform)) = terrains.iter().next() else {
        return;
    };
    let centre = heights.0.origin + heights.0.size / 2.0;
    let Some(ground) = heights.0.height_at_local(centre) else {
        return;
    };
    let spawn = terrain_transform.transform_point(Vec3::new(centre.x, ground, centre.y))
        + Vec3::Y * CAPSULE_HEIGHT;

    let animation = PlayerAnimation::default();
    let sprite = Sprite {
        image: player_assets.player_sheet.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(32),
                35,
                7,
                None,
                None,
            )),
            index: animation.frame_index(),
        }),
        ..default()
    };

    commands
        .spawn((
            ExamplePlayer,
            Transform::from_translation(spawn),
            Collider::capsule(CAPSULE_RADIUS, CAPSULE_HEIGHT - 2.0 * CAPSULE_RADIUS),
            CharacterController {
                speed: WALK_SPEED,
                standing_view_height: 0.9,
                crouch_height: 0.7,
                crouch_view_height: 0.6,
                step_size: 0.35,
                jump_height: 1.0,
                mantle_height: 0.55,
                crane_height: 0.8,
                min_ledge_grab_space: Cuboid::new(0.1, 0.05, 0.1),
                ..default()
            },
            CharacterLook::default(),
            DespawnOnExit(GameState::ExampleScene),
            children![(
                ExamplePlayerSprite,
                animation,
                sprite,
                Sprite3d {
                    pixels_per_metre: 32.0,
                    double_sided: true,
                    ..default()
                },
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
            Action::<Run>::new()
            Bindings [
                Binding(KeyCode::ShiftLeft)
                --
                Binding(KeyCode::ShiftRight)
                --
                Binding(GamepadButton::LeftThumb)
            ]
        });
}

type SceneCamera = (With<Camera3d>, Without<GameplayCamera>);

fn rotate_camera(
    mouse_motion: Res<AccumulatedMouseMotion>,
    sensitivity: Single<&CameraSensitivity>,
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

fn apply_run(
    run: Single<&Action<Run>>,
    mut controller: Single<&mut CharacterController, With<ExamplePlayer>>,
) {
    controller.speed = if ***run { RUN_SPEED } else { WALK_SPEED };
}

fn follow_player(
    player: Single<&Transform, (With<ExamplePlayer>, Without<Camera3d>)>,
    mut camera: Single<&mut Transform, SceneCamera>,
) {
    let look_at = player.translation + CAM_LOOK_OFFSET;
    camera.translation = look_at + camera.rotation * Vec3::new(0.0, 0.0, CAM_RADIUS);
}

fn face_camera(
    camera: Single<&Transform, SceneCamera>,
    mut sprite: Single<&mut Transform, (With<ExamplePlayerSprite>, Without<Camera3d>)>,
) {
    let (yaw, _, _) = camera.rotation.to_euler(EulerRot::YXZ);
    sprite.rotation = Quat::from_rotation_y(yaw);
}

fn update_animation(
    camera: Single<&Transform, SceneCamera>,
    run: Single<&Action<Run>>,
    player: Single<(&LinearVelocity, &CharacterControllerState), With<ExamplePlayer>>,
    mut animation: Single<&mut PlayerAnimation, With<ExamplePlayerSprite>>,
) {
    let (velocity, state) = *player;
    animation.update(
        **velocity,
        camera.rotation,
        ***run,
        state.grounded.is_some(),
    );
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
