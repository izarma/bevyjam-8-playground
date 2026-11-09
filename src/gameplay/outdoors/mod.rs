use bevy::prelude::*;
use jackdaw_runtime::prelude::*;

use crate::GameplayScene;

mod biomes;
mod bounds;
mod camera_effects;
mod door;
mod harvest;
mod player;

pub(crate) use harvest::HarvestScratch;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        player::plugin,
        camera_effects::plugin,
        harvest::plugin,
        door::plugin,
        biomes::plugin,
        bounds::plugin,
    ))
    .add_systems(OnEnter(GameplayScene::Outdoors), spawn_example_scene);
}

fn spawn_example_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("example_scene.bsn")),
        DespawnOnExit(GameplayScene::Outdoors),
    ));
}
