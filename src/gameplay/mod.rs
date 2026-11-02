mod animation;
mod base;
mod game_object;
mod outdoors;
mod player;

pub(crate) use player::{PlayerAnimation, tick_player_animation, update_player_animation_state};

use bevy::prelude::*;
use bevy_sprite3d::Sprite3dPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        Sprite3dPlugin,
        animation::plugin,
        game_object::plugin,
        player::plugin,
        base::plugin,
        outdoors::plugin,
    ));
}
