use bevy::prelude::*;
use jackdaw_runtime::prelude::*;

use crate::GameplayScene;

mod anim;
mod npc;
mod player_indoor;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((anim::plugin, npc::plugin, player_indoor::plugin));
    app.add_systems(OnEnter(GameplayScene::Base), spawn_lab_scene);
}

fn spawn_lab_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("lab_scene.bsn")),
        DespawnOnExit(GameplayScene::Base),
    ));
}
