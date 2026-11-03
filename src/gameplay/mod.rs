mod animation;
mod base;
mod outdoors;
mod player;

use bevy_enhanced_input::EnhancedInputPlugin;
pub(crate) use player::{PlayerAnimation, tick_player_animation};

use bevy::prelude::*;
use bevy_sprite3d::Sprite3dPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        EnhancedInputPlugin,
        Sprite3dPlugin,
        animation::plugin,
        player::plugin,
        base::plugin,
        outdoors::plugin,
    ));
}
