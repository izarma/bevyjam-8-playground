use crate::{GameState, PausableSystems};
use bevy::{
    anti_alias::taa::TemporalAntiAliasing, camera::Hdr, core_pipeline::tonemapping::Tonemapping,
    pbr::ScreenSpaceAmbientOcclusion, post_process::bloom::Bloom, prelude::*,
    ui::IsDefaultUiCamera,
};

/// Camera outside gameplay scenes.
#[derive(Component, Clone, Default)]
pub(crate) struct TitleCamera;

pub(crate) type SceneCamera = (With<Camera3d>, Without<TitleCamera>);

// `Clone` + `Default` are required for these tokens in BSN spawn scenes.
#[derive(Component, Clone, Default)]
pub(crate) struct FaceCamera; // tag entity to make it always face the camera

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
#[derive(Debug, Resource, Clone, Deref, DerefMut)]
pub(crate) struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(DEFAULT_CAMERA_SENSITIVITY)
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CameraSensitivity>()
        .add_systems(Startup, spawn_title_camera)
        .add_systems(OnExit(GameState::Gameplay), spawn_title_camera)
        .add_systems(
            Update,
            (
                face_camera
                    .in_set(PausableSystems)
                    .run_if(in_state(GameState::Gameplay)),
                add_default_ui_camera,
            ),
        );
}

fn spawn_title_camera(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #TitleCamera
        Camera3d
        Camera {
            // Use the global ClearColor resource so splash/title/gameplay can control the background.
            clear_color: ClearColorConfig::Default,
        }
        IsDefaultUiCamera
        TitleCamera
        ~DespawnOnEnter::<GameState>(GameState::Gameplay)
        Hdr
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
    });
}

/// Tag scene cameras when they spawn. May want a better way to do this.
fn add_default_ui_camera(
    mut commands: Commands,
    cameras: Query<Entity, (Added<Camera3d>, Without<TitleCamera>)>,
) {
    for entity in &cameras {
        commands.entity(entity).insert(IsDefaultUiCamera);
    }
}

/// Rotate billboarded entities so they face the active scene camera.
pub(crate) fn face_camera(
    cam_transform: Single<&Transform, SceneCamera>,
    mut query: Query<&mut Transform, (With<FaceCamera>, Without<Camera3d>)>,
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
