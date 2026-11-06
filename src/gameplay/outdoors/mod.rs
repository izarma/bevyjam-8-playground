use bevy::prelude::*;
use jackdaw_runtime::prelude::*;

use crate::GameplayScene;

mod camera_effects;
mod harvest;
mod player;
mod time_of_day;

pub(crate) use harvest::HarvestScratch;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        player::plugin,
        camera_effects::plugin,
        harvest::plugin,
        time_of_day::plugin,
        plants::PlantsPlugin {
            pick: plants::Pick::Random,
        },
    ))
    .add_systems(OnEnter(GameplayScene::Outdoors), spawn_example_scene);
}

fn spawn_example_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("example_scene.bsn")),
        DespawnOnExit(GameplayScene::Outdoors),
    ));
}
