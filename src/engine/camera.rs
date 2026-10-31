use std::f32::consts::FRAC_PI_2;

use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    camera::Hdr,
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::AccumulatedMouseMotion,
    pbr::ScreenSpaceAmbientOcclusion,
    post_process::bloom::Bloom,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    GameState,
    engine::{AppSystems, PausableSystems},
    ui::menus::Menu,
};

// Third-person follow: distance behind the player, and the point above the player the camera looks at.
const CAM_RADIUS: f32 = 5.0;
const CAM_LOOK_OFFSET: Vec3 = Vec3::new(0.0, 1.0, 0.0);

pub(super) fn plugin(app: &mut App) {
    // Spawn the camera once at startup and keep it alive across all screen states.
    // One camera renders both the 3D world and the UI.
    app.add_systems(Startup, setup_camera.spawn())
        .add_systems(
            Update,
            (
                follow_target
                    .in_set(AppSystems::Update)
                    .in_set(PausableSystems)
                    .run_if(in_state(GameState::Gameplay)),
                toggle_cursor.run_if(in_state(GameState::Gameplay).and_then(in_state(Menu::None))),
            ),
        )
        .add_systems(
            Update,
            face_camera
                .in_set(AppSystems::Update)
                .in_set(PausableSystems)
                .after(follow_target)
                .run_if(in_state(GameState::Gameplay)),
        )
        // The camera needs to be rotated before the fixed timestep loop, so that the physics
        // simulation (see `char_controller`) can use the current rotation.
        // Note that if we ran it in `Update`, it would be too late, as the physics simulation would already have been advanced.
        // If we ran this in `FixedUpdate`, it would sometimes not register player input, as that schedule may run zero times per frame.
        .add_systems(
            RunFixedMainLoop,
            rotate_camera
                .in_set(PausableSystems)
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop)
                .run_if(in_state(GameState::Gameplay)),
        )
        // Give the cursor back when leaving gameplay or pausing so menus stay usable.
        .add_systems(OnExit(GameState::Gameplay), release_cursor)
        .add_systems(OnEnter(Menu::Pause), release_cursor)
        // Grab the cursor again once back in gameplay with no menu open.
        .add_systems(
            OnEnter(Menu::None),
            lock_cursor.run_if(in_state(GameState::Gameplay)),
        )
        .add_systems(
            OnEnter(GameState::Gameplay),
            lock_cursor.run_if(in_state(Menu::None)),
        );
}

// `Clone` + `Default` are required for these tokens in BSN spawn scenes.
#[derive(Component, Clone, Default)]
pub(crate) struct FaceCamera; // tag entity to make it always face the camera

/// Tag an entity the gameplay camera should follow.
#[derive(Component, Clone, Default)]
pub struct CameraTarget;

/// Third-person gameplay camera settings.
#[derive(Component, Clone)]
pub(crate) struct GameplayCamera {
    /// The cursor lock toggle key binding.
    pub cursor_lock_key: KeyCode,
    /// Whether the cursor is in a locked state or not.
    /// While unlocked, the mouse moves freely without rotating the camera.
    /// Default is `false`.
    pub cursor_lock_active: bool,
}

/// Default mouse sensitivity (horizontal, vertical) of the gameplay camera, in
/// radians per pixel of mouse motion. It's often nicer to have a faster
/// horizontal sensitivity than vertical, which is why the two axes differ.
pub(crate) const DEFAULT_CAMERA_SENSITIVITY: Vec2 = Vec2::new(0.003, 0.002);

/// Sensible lower and upper bounds for the sensitivity. These let the player
/// adjust each axis from 10% to 200% of its default.
pub(crate) const MIN_CAMERA_SENSITIVITY: Vec2 = Vec2::new(0.0003, 0.0002);
pub(crate) const MAX_CAMERA_SENSITIVITY: Vec2 = Vec2::new(0.006, 0.004);

/// How much a single settings-menu step changes the sensitivity (10% of the
/// default, per axis).
pub(crate) const CAMERA_SENSITIVITY_STEP: Vec2 = Vec2::new(0.0003, 0.0002);

/// Mouse sensitivity (horizontal, vertical) of the gameplay camera.
///
/// This is a component so that it can be made user-configurable at runtime via
/// the settings menu, and inspected in an editor once reflected.
#[derive(Debug, Component, Clone, Deref, DerefMut)]
pub(crate) struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(DEFAULT_CAMERA_SENSITIVITY)
    }
}

impl Default for GameplayCamera {
    fn default() -> Self {
        Self {
            cursor_lock_key: KeyCode::AltLeft,
            cursor_lock_active: false,
        }
    }
}

fn setup_camera() -> impl Scene {
    bsn! {
        #Camera
        Camera3d
        Camera {
            // Use the global ClearColor resource so splash/title/gameplay can control the background.
            clear_color: ClearColorConfig::Default,
        }
        Hdr
        GameplayCamera::default()
        CameraSensitivity::default()
        Msaa::Off
        Bloom {
            intensity: 0.3
        }
        bevy::prelude::Projection::Perspective(PerspectiveProjection {
            fov: { std::f32::consts::PI / 6.0 },
        })
        ScreenSpaceAmbientOcclusion::default()
        TemporalAntiAliasing::default()
        Tonemapping::AcesFitted
    }
}

/// Keep the camera at a fixed distance behind the player, based on the camera's own
/// rotation (which is driven by mouse input in `rotate_camera`).
fn follow_target(
    target: Single<&Transform, (With<CameraTarget>, Without<Camera>)>,
    mut transform: Single<&mut Transform, (With<Camera>, Without<CameraTarget>)>,
) {
    let look_at = target.translation + CAM_LOOK_OFFSET;
    let offset = Mat3::from_quat(transform.rotation) * Vec3::new(0.0, 0.0, CAM_RADIUS);
    transform.translation = look_at + offset;
}

/// Toggle the cursor lock when the toggle key is pressed,
fn toggle_cursor(
    keys: Res<ButtonInput<KeyCode>>,
    mut cam_q: Query<&mut GameplayCamera>,
    mut window_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cam) = cam_q.single_mut() else {
        return;
    };
    if keys.just_pressed(cam.cursor_lock_key) {
        cam.cursor_lock_active = !cam.cursor_lock_active;
    }
    apply_cursor_lock(&cam, &mut window_q);
}

/// Unlock the cursor when leaving gameplay (e.g. for pause/game-over menus),
/// and reset it so gameplay starts locked again next time.
fn release_cursor(
    mut cam_q: Query<&mut GameplayCamera>,
    mut window_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cam) = cam_q.single_mut() else {
        return;
    };
    cam.cursor_lock_active = false;
    apply_cursor_lock(&cam, &mut window_q);
}

/// Lock the cursor when (re)entering active gameplay.
fn lock_cursor(
    mut cam_q: Query<&mut GameplayCamera>,
    mut window_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cam) = cam_q.single_mut() else {
        return;
    };
    cam.cursor_lock_active = true;
    apply_cursor_lock(&cam, &mut window_q);
}

fn apply_cursor_lock(
    cam: &GameplayCamera,
    window_q: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Ok(mut cursor_options) = window_q.single_mut() {
        if cam.cursor_lock_active {
            cursor_options.grab_mode = CursorGrabMode::Locked;
            cursor_options.visible = false;
        } else {
            cursor_options.grab_mode = CursorGrabMode::None;
            cursor_options.visible = true;
        }
    }
}

/// Rotate the camera from mouse motion. Runs before the fixed timestep loop, see `plugin`.
pub(crate) fn rotate_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    player: Single<(&mut Transform, &CameraSensitivity, &GameplayCamera), With<Camera>>,
) {
    let (mut transform, camera_sensitivity, camera) = player.into_inner();

    // While the cursor is unlocked (e.g. browsing UI), the mouse moves freely
    // and the camera's transform doesn't change.
    if !camera.cursor_lock_active {
        return;
    }

    let delta = accumulated_mouse_motion.delta;

    if delta != Vec2::ZERO {
        // Note that we are not multiplying by delta time here.
        // The reason is that for mouse movement, we already get the full movement that happened since the last frame.
        // This means that if we multiply by delta time, we will get a smaller rotation than intended by the user.
        let delta_yaw = -delta.x * camera_sensitivity.x;
        let delta_pitch = -delta.y * camera_sensitivity.y;

        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;

        // If the pitch was ±¹⁄₂ π, the camera would look straight up or down.
        // When the user wants to move the camera back to the horizon, which way should the camera face?
        // The camera has no way of knowing what direction was "forward" before landing in that extreme position,
        // so the direction picked will for all intents and purposes be arbitrary.
        // Another issue is that for mathematical reasons, the yaw will effectively be flipped when the pitch is at the extremes.
        // To not run into these issues, we clamp the pitch to a safe range.
        const PITCH_MIN: f32 = -0.4 * FRAC_PI_2;
        const PITCH_MAX: f32 = 0.3 * FRAC_PI_2;
        let pitch = (pitch + delta_pitch).clamp(PITCH_MIN, PITCH_MAX);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

pub(crate) fn face_camera(
    cam_transform: Single<&Transform, With<Camera>>,
    mut query: Query<&mut Transform, (With<FaceCamera>, Without<Camera>)>,
) {
    for mut transform in query.iter_mut() {
        let mut delta = cam_transform.translation - transform.translation;
        delta.y = 0.0;
        delta += transform.translation;
        transform.look_at(delta, Vec3::Y);
        // `look_at` points the entity's forward (-Z) at the camera, but `Sprite3d`
        // quads face +Z
        transform.rotate_y(std::f32::consts::PI);
    }
}
