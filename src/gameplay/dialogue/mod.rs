mod concoction;
mod speaker;
mod view;

pub(crate) use concoction::Concoction;

use bevy::prelude::*;
use bevy_yarnspinner::prelude::{
    DialogueRunner, YarnFileSource, YarnProject, YarnSpinnerPlugin, YarnSpinnerSystemSet, YarnValue,
};

use crate::{
    GameState, GameplayScene, Pause,
    gameplay::{
        herbs,
        inventory::Inventory,
        visitors::{Ailing, Gift},
    },
};

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
    .init_resource::<DialogueTarget>()
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

#[derive(Resource, Default)]
pub(crate) struct DialogueTarget(pub(crate) Option<Entity>);

/// Spawns the single [`DialogueRunner`] the game uses once the Yarn project has
/// finished compiling. The runner is left idle until an interaction starts it.
fn spawn_dialogue_runner(
    mut commands: Commands,
    project: Res<YarnProject>,
    guardian: Single<(&Gift, &Ailing)>,
) {
    let (gift, ailing) = *guardian;
    let mut dialogue_runner = project.create_dialogue_runner(&mut commands);
    let offer_concoction = commands.register_system(concoction::offer_concoction);
    let give_guardian_herbs = commands.register_system(give_guardian_herbs);
    for system in [offer_concoction.entity(), give_guardian_herbs.entity()] {
        commands
            .entity(system)
            .insert(DespawnOnExit(GameState::Gameplay));
    }
    dialogue_runner
        .commands_mut()
        .add_command("offer_concoction", offer_concoction)
        .add_command("give_guardian_herbs", give_guardian_herbs);

    let gifts = gift.0.into_iter().enumerate().map(|(index, herb)| {
        (
            format!("$gift_{}", index + 1),
            herbs::name(herb).to_string(),
        )
    });
    let need = ("$guardian_wants".to_string(), ailing.ailment.wants());
    for (variable, text) in gifts.chain([need]) {
        if let Err(error) = dialogue_runner
            .variable_storage_mut()
            .set(variable, YarnValue::String(text))
        {
            error!("Couldn't give the guardian's dialogue its words: {error}");
        }
    }
    commands.spawn(dialogue_runner);
}

fn give_guardian_herbs(gift: Single<&Gift>, mut inventory: Single<&mut Inventory>) {
    for herb in gift.0 {
        *inventory.plants.entry(herb).or_default() += 1;
    }
}

fn stop_dialogue_runner(
    mut commands: Commands,
    mut runners: Query<&mut DialogueRunner>,
    mut target: ResMut<DialogueTarget>,
) {
    for mut runner in &mut runners {
        if runner.is_running() {
            runner.stop();
        }
    }
    target.0 = None;
    commands.remove_resource::<view::PresentedOptions>();
    commands.remove_resource::<view::Typewriter>();
}

fn despawn_dialogue_runner(
    mut commands: Commands,
    runners: Query<Entity, With<DialogueRunner>>,
    mut target: ResMut<DialogueTarget>,
) {
    for runner in &runners {
        commands.entity(runner).despawn();
    }
    target.0 = None;
}
