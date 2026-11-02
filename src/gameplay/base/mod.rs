use bevy::prelude::*;
use jackdaw_runtime::prelude::*;

use crate::{
    GameplayScene,
    gameplay::outdoors::{mark_scene_ui_camera, set_gameplay_camera_active},
};

mod anim;
mod npc;
mod player_indoor;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((anim::plugin, npc::plugin, player_indoor::plugin));
    app.add_systems(
        OnEnter(GameplayScene::Base),
        (spawn_lab_scene, set_gameplay_camera_active::<false>),
    )
    .add_systems(
        OnExit(GameplayScene::Base),
        set_gameplay_camera_active::<true>,
    )
    // UI (including the pause menu) should be rendered on the scene camera.
    .add_systems(
        Update,
        mark_scene_ui_camera.run_if(in_state(GameplayScene::Base)),
    );
}

fn spawn_lab_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("lab_scene.bsn")),
        DespawnOnExit(GameplayScene::Base),
    ));
}
