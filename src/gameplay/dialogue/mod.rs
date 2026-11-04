mod view;

use bevy::prelude::*;
use bevy_yarnspinner::prelude::{
    DialogueRunner, YarnFileSource, YarnProject, YarnSpinnerPlugin, YarnSpinnerSystemSet,
};

use crate::{GameState, GameplayScene, Pause};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        YarnSpinnerPlugin::with_yarn_sources([YarnFileSource::file("dialogue/npcs.yarn")]),
        view::plugin,
    ))
    .configure_sets(
        Update,
        YarnSpinnerDialogueViewSystemSet
            .after(YarnSpinnerSystemSet)
            .run_if(in_state(Pause(false))),
    )
    .add_systems(
        Update,
        spawn_dialogue_runner.run_if(
            in_state(GameplayScene::Base)
                .and_then(resource_exists::<YarnProject>)
                .and_then(not(any_with_component::<DialogueRunner>)),
        ),
    )
    .add_systems(OnExit(GameplayScene::Base), stop_dialogue_runner)
    .add_systems(OnExit(GameState::Gameplay), despawn_dialogue_runner);
}

/// The [`SystemSet`] containing all systems added by the dialogue view plugin.
/// Is run after the [`YarnSpinnerSystemSet`](bevy_yarnspinner::prelude::YarnSpinnerSystemSet).
#[derive(Debug, Default, Clone, Copy, SystemSet, Eq, PartialEq, Hash)]
pub struct YarnSpinnerDialogueViewSystemSet;

/// Spawns the single [`DialogueRunner`] the game uses once the Yarn project has
/// finished compiling. The runner is left idle until an interaction starts it.
fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    let dialogue_runner = project.create_dialogue_runner(&mut commands);
    commands.spawn(dialogue_runner);
}

fn stop_dialogue_runner(mut commands: Commands, mut runners: Query<&mut DialogueRunner>) {
    for mut runner in &mut runners {
        if runner.is_running() {
            runner.stop();
        }
    }
    commands.remove_resource::<view::PresentedOptions>();
}

fn despawn_dialogue_runner(mut commands: Commands, runners: Query<Entity, With<DialogueRunner>>) {
    for runner in &runners {
        commands.entity(runner).despawn();
    }
}
