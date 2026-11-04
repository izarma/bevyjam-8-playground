mod animation;
mod base;
pub(crate) mod herbs;
pub(crate) mod inventory;
mod outdoors;

pub(crate) use animation::{PlayerAnimation, PlayerAnimationSystems};

use bevy::prelude::*;
use bevy_sprite3d::Sprite3dPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        Sprite3dPlugin,
        pixel_art::PixelArtPlugin,
        animation::plugin,
        base::plugin,
        inventory::plugin,
        outdoors::plugin,
    ));
}

/// Marker for both indoor & outdoor player
#[derive(Component, Clone, Default)]
pub struct Player;
