use bevy::prelude::*;

mod animation;

pub(crate) use animation::{PlayerAnimation, tick_player_animation};

#[derive(Component, Clone, Default)]
pub struct Player;

/// This plugin handles player related stuff like movement.
/// Player logic is only active in the `GameplayScene::Base` substate of `GameState::Gameplay`.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(animation::plugin);
}
