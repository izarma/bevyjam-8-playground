use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_yarnspinner::events::{DialogueCompleted, DialogueStarted};

use crate::{
    GameplayScene,
    loading::{AmbienceAssets, NPCAssets},
};

mod footsteps;
mod harvesting;

#[derive(Resource)]
struct Ambience;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AudioPlugin)
        .add_audio_channel::<Ambience>()
        .add_plugins((footsteps::plugin, harvesting::plugin))
        .add_observer(play_dialogue_open)
        .add_observer(play_dialogue_close)
        .add_systems(OnEnter(GameplayScene::Outdoors), start_outdoor_ambience)
        .add_systems(OnExit(GameplayScene::Outdoors), stop_ambience);
}

fn play_dialogue_open(_: On<DialogueStarted>, npc_assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(npc_assets.dialogue_open.clone());
}

fn play_dialogue_close(_: On<DialogueCompleted>, npc_assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(npc_assets.dialogue_close.clone());
}

fn start_outdoor_ambience(assets: Res<AmbienceAssets>, ambience: Res<AudioChannel<Ambience>>) {
    ambience.play(assets.outdoors.clone()).looped();
    ambience
        .play(assets.outdoor_music.clone())
        .with_volume(6.0)
        .looped();
}

fn stop_ambience(ambience: Res<AudioChannel<Ambience>>) {
    ambience.stop();
}
