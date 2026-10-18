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
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Title,
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

        app.add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            // Use the global ClearColor resource so splash/title/gameplay can control the background.
            clear_color: ClearColorConfig::Default,
            ..default()
        },
        //Hdr,
        Msaa::Off,
        // Bloom {
        //     intensity: 0.3,
        //     ..default()
        // },
        bevy::prelude::Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::PI / 6.0,
            ..default()
        }),
        // ScreenSpaceAmbientOcclusion::default(),
        // TemporalAntiAliasing::default(),
        // Tonemapping::AcesFitted,
    ));
}
