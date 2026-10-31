#![allow(clippy::type_complexity)]

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod audio;
mod camera;
#[cfg(debug_assertions)]
mod dev_tooling;
mod gameplay;
mod loading;
mod ui;

/// Application-level states. See `gameplay` for gameplay-owned state.
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    /// During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    /// During this State the actual game logic is executed
    Gameplay,
    /// Here the menu is drawn and waiting for player interaction
    Title,
    /// Game over state with `GameResult`
    GameOver,
}

/// The outcome of the last round of gameplay. Only available after the player won or lost once.
#[derive(Resource, Clone, Copy, Eq, PartialEq, Debug)]
enum GameResult {
    Won,
    Lost,
}

/// Whether or not the game is paused.
#[derive(SubStates, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(GameState = GameState::Gameplay)]
pub(crate) struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct PausableSystems;

/// Composes the whole game. The single entry point used by the desktop/web and
/// mobile launchers.
pub fn plugin(app: &mut App) {
    app.init_state::<GameState>().add_sub_state::<Pause>();

    app.add_plugins((
        loading::plugin,
        camera::plugin,
        audio::plugin,
        ui::plugin,
        gameplay::plugin,
    ));

    // Gameplay simulation runs across several schedules, and all of them must
    // respect pause.
    app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))))
        .configure_sets(FixedUpdate, PausableSystems.run_if(in_state(Pause(false))))
        .configure_sets(
            RunFixedMainLoop,
            PausableSystems.run_if(in_state(Pause(false))),
        );

    #[cfg(debug_assertions)]
    {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            dev_tooling::plugin,
        ));
    }
}
