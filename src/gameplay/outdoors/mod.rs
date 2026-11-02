use bevy::{prelude::*, ui::IsDefaultUiCamera};
use jackdaw_runtime::prelude::*;

use crate::{GameplayScene, camera::GameplayCamera};

mod camera_effects;
mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        player::plugin,
        pixel_sprite::PixelSpritePlugin,
        camera_effects::plugin,
    ))
    .add_systems(
        OnEnter(GameplayScene::Outdoors),
        (spawn_example_scene, set_gameplay_camera_active::<false>),
    )
    .add_systems(
        OnExit(GameplayScene::Outdoors),
        set_gameplay_camera_active::<true>,
    )
    // UI should be rendered on the scene camera.
    .add_systems(
        Update,
        mark_scene_ui_camera.run_if(in_state(GameplayScene::Outdoors)),
    );
}

/// Bevy fallback picks the *oldest* camera, so when this camera is despawned
/// gameplay camera becomes the default UI camera again.
pub(crate) fn mark_scene_ui_camera(
    mut commands: Commands,
    scene_cameras: Query<
        Entity,
        (
            With<Camera3d>,
            Without<GameplayCamera>,
            Without<IsDefaultUiCamera>,
        ),
    >,
) {
    for entity in &scene_cameras {
        commands.entity(entity).insert(IsDefaultUiCamera);
    }
}

fn spawn_example_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("example_scene.bsn")),
        DespawnOnExit(GameplayScene::Outdoors),
    ));
}

pub(crate) fn set_gameplay_camera_active<const ACTIVE: bool>(
    mut cameras: Query<&mut Camera, With<GameplayCamera>>,
) {
    for mut camera in &mut cameras {
        camera.is_active = ACTIVE;
    }
}
