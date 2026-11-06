mod animation;
mod base;
mod candles;
pub(crate) mod clock;
mod dialogue;
pub(crate) mod herbs;
pub(crate) mod inventory;
mod outdoors;
mod remedy;
mod time_of_day;
mod visitors;

pub(crate) use animation::{PlayerAnimation, PlayerAnimationSystems};
pub(crate) use base::{BodyOf, InDialogue};
pub(crate) use dialogue::Concoction;
pub(crate) use visitors::{BALANCED, Health, MAX_HEALTH, MIN_HEALTH};

use bevy::prelude::*;
use bevy_sprite3d::Sprite3dPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        Sprite3dPlugin,
        pixel_art::PixelArtPlugin,
        animation::plugin,
        base::plugin,
        clock::plugin,
        inventory::plugin,
        remedy::plugin,
        candles::plugin,
        time_of_day::plugin,
        visitors::plugin,
        dialogue::plugin,
        outdoors::plugin,
    ));
}

/// Marker for both indoor & outdoor player
#[derive(Component, Clone, Default)]
// its sprite is a child, so it has to take part in visibility
#[require(Visibility)]
pub struct Player;

/// The name a character is addressed by in Yarn dialogue (their speaker label).
#[derive(Component, Clone, Copy)]
pub(crate) struct CharacterName(pub(crate) &'static str);

/// Marks the visual child entity of a character that is lifted while it speaks.
#[derive(Component, Default)]
pub(crate) struct SpeakerSprite;
