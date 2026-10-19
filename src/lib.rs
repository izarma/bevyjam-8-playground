#![allow(clippy::type_complexity)]

mod gameplay;
mod loading;
mod ui;

use crate::gameplay::GameplayPlugin;
use crate::loading::LoadingPlugin;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
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
            .add_plugins((LoadingPlugin, GameplayPlugin, ui::plugin));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                LogDiagnosticsPlugin::default(),
            ));
        }

        app.add_systems(Startup, setup_camera.spawn());

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));
    }
}

fn setup_camera() -> impl Scene {
    bsn! {
        #Camera
        Camera3d
        Camera {
            // Use the global ClearColor resource so splash/title/gameplay can control the background.
            clear_color: ClearColorConfig::Default,
        }
        //Hdr,
        Msaa::Off
        // Bloom {
        //     intensity: 0.3,
        //     ..default()
        // },
        bevy::prelude::Projection::Perspective(PerspectiveProjection {
            fov: { std::f32::consts::PI / 6.0 },
        })
        // ScreenSpaceAmbientOcclusion::default(),
        // TemporalAntiAliasing::default(),
        // Tonemapping::AcesFitted,
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;
