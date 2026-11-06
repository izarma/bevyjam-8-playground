mod concoction;
mod speaker;
mod view;

pub(crate) use concoction::Concoction;

use bevy::prelude::*;
use bevy_yarnspinner::prelude::{
    DialogueRunner, YarnFileSource, YarnProject, YarnSpinnerPlugin, YarnSpinnerSystemSet,
};

use crate::{GameState, GameplayScene, Pause};

/// Signals that the character speaking in the dialogue has changed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Message)]
#[reflect(Debug, PartialEq, Hash)]
pub struct SpeakerChangeEvent {
    pub character_name: String,
    pub speaking: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        YarnSpinnerPlugin::with_yarn_sources([YarnFileSource::file("dialogue/npcs.yarn")]),
        view::plugin,
        speaker::plugin,
        concoction::plugin,
    ))
    .add_message::<SpeakerChangeEvent>()
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
    let mut dialogue_runner = project.create_dialogue_runner(&mut commands);
    let offer_concoction = commands.register_system(concoction::offer_concoction);
    commands
        .entity(offer_concoction.entity())
        .insert(DespawnOnExit(GameState::Gameplay));
    dialogue_runner
        .commands_mut()
        .add_command("offer_concoction", offer_concoction);
    commands.spawn(dialogue_runner);
}

fn stop_dialogue_runner(mut commands: Commands, mut runners: Query<&mut DialogueRunner>) {
    for mut runner in &mut runners {
        if runner.is_running() {
            runner.stop();
        }
    }
    commands.remove_resource::<view::PresentedOptions>();
    commands.remove_resource::<view::Typewriter>();
}

fn despawn_dialogue_runner(mut commands: Commands, runners: Query<Entity, With<DialogueRunner>>) {
    for runner in &runners {
        commands.entity(runner).despawn();
    }
}
