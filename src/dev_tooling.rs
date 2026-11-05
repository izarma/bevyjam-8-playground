//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    GameResult, GameState,
    gameplay::{Health, InDialogue},
};

pub(super) fn plugin(app: &mut App) {
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

    // Debug-only NPC health editing.
    app.add_systems(
        Update,
        decrease_npc_health
            .run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F3))),
    );
    app.add_systems(
        Update,
        increase_npc_health
            .run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F4))),
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

fn decrease_npc_health(npcs: Query<&mut Health, With<InDialogue>>) {
    adjust_npc_health(npcs, -1);
}

fn increase_npc_health(npcs: Query<&mut Health, With<InDialogue>>) {
    adjust_npc_health(npcs, 1);
}

fn adjust_npc_health(mut npcs: Query<&mut Health, With<InDialogue>>, delta: i8) {
    let Some(mut health) = npcs.iter_mut().next() else {
        println!("Health editor: no NPC in dialogue");
        return;
    };
    health.offset(delta);
    println!("Health editor: all stats {delta:+}");
}
