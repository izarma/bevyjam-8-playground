use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_yarnspinner::events::{DialogueCompleted, DialogueStarted};

use crate::{
    GameState, GameplayScene,
    loading::{AmbienceAssets, NPCAssets},
    ui::menus::Menu,
};

mod footsteps;
mod harvesting;

#[derive(Resource)]
struct Ambience;

#[derive(Resource)]
struct MenuAudio;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AudioPlugin)
        .add_audio_channel::<Ambience>()
        .add_audio_channel::<MenuAudio>()
        .add_plugins((footsteps::plugin, harvesting::plugin))
        .add_observer(play_dialogue_open)
        .add_observer(play_dialogue_close)
        .add_systems(OnEnter(GameplayScene::Outdoors), start_outdoor_ambience)
        .add_systems(OnExit(GameplayScene::Outdoors), stop_ambience)
        .add_systems(
            Update,
            update_menu_audio.run_if(resource_exists::<AmbienceAssets>),
        );
}

fn play_dialogue_open(_: On<DialogueStarted>, npc_assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(npc_assets.dialogue_open.clone());
}

fn play_dialogue_close(_: On<DialogueCompleted>, npc_assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(npc_assets.dialogue_close.clone());
}

fn start_outdoor_ambience(assets: Res<AmbienceAssets>, ambience: Res<AudioChannel<Ambience>>) {
    ambience
        .play(assets.outdoors.clone())
        .with_volume(-3.0)
        .looped();
}

fn stop_ambience(ambience: Res<AudioChannel<Ambience>>) {
    ambience.stop();
}

fn update_menu_audio(
    state: Res<State<GameState>>,
    menu: Res<State<Menu>>,
    assets: Res<AmbienceAssets>,
    audio: Res<AudioChannel<MenuAudio>>,
    mut playing: Local<bool>,
) {
    let should_play = matches!(state.get(), GameState::Title | GameState::GameOver)
        || (*state.get() == GameState::Gameplay && *menu.get() != Menu::None);
    if should_play == *playing {
        return;
    }
    if should_play {
        audio
            .play(assets.menus.clone())
            .looped()
            .fade_in(AudioTween::linear(Duration::from_secs(5)));
    } else {
        audio.stop();
    }
    *playing = should_play;
}
