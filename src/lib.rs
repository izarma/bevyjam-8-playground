#![allow(clippy::type_complexity)]

pub(crate) mod engine;
mod gameplay;
mod ui;

use crate::gameplay::GameplayPlugin;

use bevy::app::App;
use bevy::prelude::*;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Gameplay,
    // Here the menu is drawn and waiting for player interaction
    Title,
    // Game over state with `GameResult`
    GameOver,
}

// The outcome of the last round of gameplay. Only available after the player won or lost once.
#[derive(Resource, Clone, Copy, Eq, PartialEq, Debug)]
enum GameResult {
    Won,
    Lost,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((engine::plugin, ui::plugin, GameplayPlugin));
    }
}
