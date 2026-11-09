#![allow(clippy::type_complexity)]

use avian3d::prelude::{Physics, PhysicsTime};
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ahoy::{AhoyPlugins, AhoySystems};

mod audio;
mod camera;
#[cfg(debug_assertions)]
mod dev_tooling;
mod gameplay;
mod graphics;
mod loading;
mod loading_screen;
mod render_scale;
mod scene_ready;
mod sky_ambient;
mod terrain_variety;
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

#[derive(SubStates, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(GameState = GameState::Gameplay)]
pub(crate) enum GameplayScene {
    /// Base / Lab / Hub area
    #[default]
    Base,
    /// Open gathering scene
    Outdoors,
}

/// The outcome of the last round of gameplay. Only available after the player won or lost once.
#[derive(Resource, Clone, Default, Debug)]
pub(crate) struct Survivors(pub(crate) Vec<&'static str>);

/// Whether or not the game is paused.
#[derive(SubStates, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(GameState = GameState::Gameplay)]
pub(crate) struct Pause(pub bool);

#[derive(SubStates, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(GameplayScene = GameplayScene::Outdoors)]
pub(crate) struct Harvesting(pub bool);

// not harvesting, not paused
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct InControl;

impl ComputedStates for InControl {
    type SourceStates = (Pause, Option<Harvesting>);

    fn compute((pause, harvesting): (Pause, Option<Harvesting>)) -> Option<Self> {
        (!pause.0 && harvesting != Some(Harvesting(true))).then_some(Self)
    }
}

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct PausableSystems;

/// Composes the whole game. The single entry point used by the desktop/web and
/// mobile launchers.
pub fn plugin(app: &mut App) {
    app.init_state::<GameState>();
    app.add_sub_state::<Pause>();
    app.add_sub_state::<GameplayScene>();
    app.add_sub_state::<Harvesting>();
    app.add_computed_state::<InControl>();

    app.add_plugins((
        loading::plugin,
        camera::plugin,
        loading_screen::plugin,
        graphics::plugin,
        render_scale::plugin,
        scene_ready::plugin,
        (sky_ambient::plugin, terrain_variety::plugin),
        audio::plugin,
        bevy_enhanced_input::EnhancedInputPlugin,
        ui::plugin,
        AhoyPlugins::default(),
        gameplay::plugin,
        jackdaw_runtime::JackdawPlugin,
        avian3d::prelude::PhysicsPlugins::default(),
        jackdaw_avian_integration::AvianColliderBridgePlugin,
    ));

    // Gameplay simulation runs across several schedules, and all of them must
    // respect pause or loading states.
    app.configure_sets(Update, PausableSystems.run_if(simulating))
        .configure_sets(FixedUpdate, PausableSystems.run_if(simulating))
        .configure_sets(RunFixedMainLoop, PausableSystems.run_if(simulating))
        .configure_sets(
            FixedPostUpdate,
            (
                PausableSystems,
                AhoySystems::MoveCharacters,
                AhoySystems::ApplyForcesToDynamicRigidBodies,
            )
                .run_if(simulating),
        );
    app.add_systems(
        RunFixedMainLoop,
        hold_physics_time.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
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

fn simulating(
    pause: Option<Res<State<Pause>>>,
    loading: Query<(), scene_ready::Loading>,
    trips: Query<(), With<loading_screen::Trip>>,
) -> bool {
    pause.is_some_and(|pause| !pause.0) && loading.is_empty() && trips.is_empty()
}

fn hold_physics_time(
    pause: Option<Res<State<Pause>>>,
    loading: Query<(), scene_ready::Loading>,
    trips: Query<(), With<loading_screen::Trip>>,
    mut time: ResMut<Time<Physics>>,
) {
    let held = !simulating(pause, loading, trips);
    if held != time.is_paused() {
        if held {
            time.pause();
        } else {
            time.unpause();
        }
    }
}
