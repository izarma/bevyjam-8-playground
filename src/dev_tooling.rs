//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
};

use crate::{GameResult, GameState};

pub(super) fn plugin(app: &mut App) {
    // Log `GameState` state transitions.
    app.add_systems(Update, log_transitions::<GameState>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );

    // Debug-only win/lose triggers
    app.add_systems(
        Update,
        win.run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F1))),
    );
    app.add_systems(
        Update,
        lose.run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F2))),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<GlobalUiDebugOptions>) {
    println!("Debug UI toggled");
    options.toggle();
}

fn win(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    commands.insert_resource(GameResult::Won);
    next_state.set(GameState::GameOver);
}

fn lose(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    commands.insert_resource(GameResult::Lost);
    next_state.set(GameState::GameOver);
}
