use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use jackdaw_runtime::prelude::*;

use crate::{GameState, camera::GameplayCamera};

mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(player::plugin)
        .add_systems(
            OnEnter(GameState::ExampleScene),
            (spawn_example_scene, set_gameplay_camera_active::<false>),
        )
        .add_systems(
            OnExit(GameState::ExampleScene),
            set_gameplay_camera_active::<true>,
        )
        .add_systems(
            Update,
            return_to_title.run_if(
                in_state(GameState::ExampleScene).and_then(input_just_pressed(KeyCode::Escape)),
            ),
        );
}

fn spawn_example_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("example_scene.bsn")),
        DespawnOnExit(GameState::ExampleScene),
    ));
}

fn set_gameplay_camera_active<const ACTIVE: bool>(
    mut cameras: Query<&mut Camera, With<GameplayCamera>>,
) {
    for mut camera in &mut cameras {
        camera.is_active = ACTIVE;
    }
}

fn return_to_title(mut next_screen: ResMut<NextState<GameState>>) {
    next_screen.set(GameState::Title);
}
