use bevy::prelude::*;

use crate::GameState;

#[derive(Component, Clone, Default)]
pub struct GameObject;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnExit(GameState::Gameplay), cleanup_game_objects);
}

fn cleanup_game_objects(mut commands: Commands, menu: Query<Entity, With<GameObject>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}
