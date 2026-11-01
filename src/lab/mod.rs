//! The lab scene: loads the jackdaw scene environment without any gameplay.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use jackdaw_runtime::prelude::*;

use crate::{GameState, camera::GameplayCamera};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::LabScene),
        (spawn_lab_scene, set_gameplay_camera_active::<false>),
    )
    .add_systems(
        OnExit(GameState::LabScene),
        set_gameplay_camera_active::<true>,
    )
    .add_systems(
        Update,
        return_to_title
            .run_if(in_state(GameState::LabScene).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_lab_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("lab_scene.bsn")),
        DespawnOnExit(GameState::LabScene),
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
