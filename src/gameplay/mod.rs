mod animation;
mod base;
pub(crate) mod clock;
mod dialogue;
pub(crate) mod herbs;
pub(crate) mod inventory;
mod outdoors;
mod remedy;
mod visitors;

pub(crate) use animation::{PlayerAnimation, PlayerAnimationSystems};
pub(crate) use base::{BodyOf, InDialogue};
pub(crate) use dialogue::Concoction;
pub(crate) use outdoors::HarvestScratch;
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
        visitors::plugin,
        dialogue::plugin,
        outdoors::plugin,
    ));
}

/// Marker for both indoor & outdoor player
#[derive(Component, Clone, Default)]
pub struct Player;

/// The name a character is addressed by in Yarn dialogue (their speaker label).
#[derive(Component, Clone, Copy)]
pub(crate) struct CharacterName(pub(crate) &'static str);

/// Marks the visual child entity of a character that is lifted while it speaks.
#[derive(Component, Default)]
pub(crate) struct SpeakerSprite;
