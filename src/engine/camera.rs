use bevy::{
    anti_alias::taa::TemporalAntiAliasing, camera::Hdr, core_pipeline::tonemapping::Tonemapping,
    pbr::ScreenSpaceAmbientOcclusion, post_process::bloom::Bloom, prelude::*,
};

use crate::{
    GameState,
    engine::{AppSystems, PausableSystems},
};

// Third-person offset: behind and above the player.
const CAM_OFFSET: Vec3 = Vec3::new(0.0, 3.0, -5.0);
const CAM_LOOK_OFFSET: Vec3 = Vec3::new(0.0, 1.0, 0.0);

pub(super) fn plugin(app: &mut App) {
    // Spawn the camera once at startup and keep it alive across all screen states.
    // One camera renders both the 3D world and the UI.
    app.add_systems(Startup, setup_camera.spawn())
        .add_systems(
            Update,
            follow_target
                .in_set(AppSystems::Update)
                .in_set(PausableSystems)
                // .after(crate::engine::char_controller::apply_movement)
                .run_if(in_state(GameState::Gameplay)),
        )
        .add_systems(
            Update,
            face_camera
                .in_set(AppSystems::Update)
                .in_set(PausableSystems)
                .after(follow_target)
                .run_if(in_state(GameState::Gameplay)),
        );
}

#[derive(Component)]
pub(crate) struct FaceCamera; // tag entity to make it always face the camera

/// Tag an entity the gameplay camera should follow.
#[derive(Component)]
pub struct CameraTarget;

fn setup_camera() -> impl Scene {
    bsn! {
        #Camera
        Camera3d
        Camera {
            // Use the global ClearColor resource so splash/title/gameplay can control the background.
            clear_color: ClearColorConfig::Default,
        }
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

fn follow_target(
    target: Single<&Transform, (With<CameraTarget>, Without<Camera>)>,
    mut transform: Single<&mut Transform, (With<Camera>, Without<CameraTarget>)>,
) {
    let look_at = target.translation + CAM_LOOK_OFFSET;
    transform.translation = look_at + CAM_OFFSET;
    transform.look_at(look_at, Vec3::Y);
}

fn face_camera(
    cam_transform: Single<&Transform, With<Camera>>,
    mut query: Query<&mut Transform, (With<FaceCamera>, Without<Camera>)>,
) {
    for mut transform in query.iter_mut() {
        let mut delta = cam_transform.translation - transform.translation;
        delta.y = 0.0;
        delta += transform.translation;
        transform.look_at(delta, Vec3::Y);
    }
}
