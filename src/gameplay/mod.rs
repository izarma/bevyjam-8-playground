mod animation;
mod collision;
mod game_object;
mod player;
mod world;

pub(crate) use player::{PlayerAnimation, update_player_animation_state};

use bevy::prelude::*;
use bevy_sprite3d::Sprite3dPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        Sprite3dPlugin,
        animation::plugin,
        game_object::plugin,
        player::plugin,
        world::plugin,
    ));
}
