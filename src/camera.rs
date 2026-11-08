use crate::{GameState, PausableSystems};
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    camera::Hdr,
    core_pipeline::tonemapping::Tonemapping,
    light::{CascadeShadowConfigBuilder, PointLightShadowMap, ShadowFilteringMethod},
    pbr::ScreenSpaceAmbientOcclusion,
    post_process::bloom::Bloom,
    prelude::*,
    ui::IsDefaultUiCamera,
};

#[derive(Component, Clone, Default)]
pub(crate) struct NoSceneCamera;

pub(crate) type SceneCamera = (With<Camera3d>, Without<NoSceneCamera>);

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
        .insert_resource(PointLightShadowMap { size: 256 })
        .add_systems(
            OnEnter(GameState::Loading),
            spawn_no_scene_camera(GameState::Loading),
        )
        .add_systems(
            OnEnter(GameState::GameOver),
            spawn_no_scene_camera(GameState::GameOver),
        )
        .add_systems(
            Update,
            (
                face_camera
                    .in_set(PausableSystems)
                    .run_if(in_state(GameState::Gameplay)),
                set_up_scene_cameras,
                fit_sun_cascades,
            ),
        );
}

fn spawn_no_scene_camera(state: GameState) -> impl Fn(Commands) {
    move |mut commands: Commands| {
        commands.spawn_scene(no_scene_camera(state.clone()));
    }
}

fn no_scene_camera(state: GameState) -> impl Scene {
    bsn! {
        #NoSceneCamera
        Camera3d
        Camera {
            // Use the global ClearColor resource so splash/title/gameplay can control the background.
            clear_color: ClearColorConfig::Default,
        }
        IsDefaultUiCamera
        NoSceneCamera
        DespawnOnExit::<GameState>(state)
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
    }
}

fn set_up_scene_cameras(
    mut commands: Commands,
    cameras: Query<Entity, (Added<Camera3d>, Without<NoSceneCamera>)>,
) {
    for entity in &cameras {
        commands
            .entity(entity)
            .insert((IsDefaultUiCamera, ShadowFilteringMethod::Hardware2x2));
    }
}

fn fit_sun_cascades(mut commands: Commands, suns: Query<Entity, Added<DirectionalLight>>) {
    for sun in &suns {
        commands.entity(sun).insert(
            CascadeShadowConfigBuilder {
                num_cascades: 2,
                first_cascade_far_bound: 15.0,
                maximum_distance: 50.0,
                ..default()
            }
            .build(),
        );
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
