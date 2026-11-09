mod animation;
mod base;
mod candles;
pub(crate) mod clock;
mod dialogue;
pub(crate) mod entrance;
pub(crate) mod herbs;
pub(crate) mod inventory;
mod outdoors;
mod remedy;
mod time_of_day;
mod visitors;

pub(crate) use animation::{
    Facing, PlayerAnimation, PlayerAnimationSystems, apply_player_animation, tick_player_animation,
};
pub(crate) use dialogue::{Concoction, DialogueTarget};
pub(crate) use outdoors::HarvestScratch;
pub(crate) use remedy::Mark;
pub(crate) use visitors::{
    Ailing, Ailment, BALANCED, Brew, BrewHistory, Died, Health, InLineup, MAX_HEALTH, MIN_HEALTH,
    Met, Returning, ShopClosed, ShopOpened, Visitor,
};
// for the dev tools
#[cfg(debug_assertions)]
pub(crate) use visitors::{DAY_END, DAY_START};

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        pixel_art::PixelArtPlugin,
        animation::plugin,
        base::plugin,
        clock::plugin,
        inventory::plugin,
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
