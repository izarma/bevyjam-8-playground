use bevy::prelude::*;
use jackdaw_runtime::prelude::*;

use crate::GameState;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Title), spawn_title_scene);
}

fn spawn_title_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("title.bsn")),
        DespawnOnExit(GameState::Title),
    ));
}
