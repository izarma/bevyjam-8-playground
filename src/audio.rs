use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_yarnspinner::events::{DialogueCompleted, DialogueStarted};

use crate::{
    GameState,
    loading::{MusicAssets, NPCAssets},
    scene_ready::Loading,
};

mod footsteps;
mod harvesting;

/// Dedicated background music audio channel.
#[derive(Resource)]
struct Music;

/// This plugin controls all game audio: the soundtrack and gameplay sound
/// effects.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AudioPlugin)
        .add_audio_channel::<Music>()
        .add_plugins((footsteps::plugin, harvesting::plugin))
        .add_observer(play_dialogue_open)
        .add_observer(play_dialogue_close)
        // The soundtrack follows the application state.
        .add_systems(Update, start_music.run_if(resource_exists::<MusicAssets>))
        .add_systems(OnExit(GameState::Title), stop_music)
        .add_systems(OnExit(GameState::GameOver), stop_music)
        .add_systems(OnExit(GameState::Gameplay), stop_music);
}

fn play_dialogue_open(_: On<DialogueStarted>, npc_assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(npc_assets.dialogue_open.clone());
}

fn play_dialogue_close(_: On<DialogueCompleted>, npc_assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(npc_assets.dialogue_close.clone());
}

// dont start music until after loading
fn start_music(
    state: Res<State<GameState>>,
    loading: Query<(), Loading>,
    music_assets: Res<MusicAssets>,
    music: Res<AudioChannel<Music>>,
    mut started_for: Local<Option<GameState>>,
) {
    if !loading.is_empty() || started_for.as_ref() == Some(state.get()) {
        return;
    }
    *started_for = Some(state.get().clone());
    let track = match state.get() {
        GameState::Loading => return,
        GameState::Title => &music_assets.title_theme,
        GameState::Gameplay => &music_assets.game_bg1,
        GameState::GameOver => &music_assets.game_over_bg,
    };
    music.play(track.clone()).looped();
}

fn stop_music(music: Res<AudioChannel<Music>>) {
    music
        .stop()
        .fade_out(AudioTween::linear(Duration::from_millis(500)));
}
