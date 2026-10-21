use std::time::Duration;

use crate::GameState;
use crate::engine::loading::{LevelAssets, UIAssets};
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// Dedicated background music audio channel.
#[derive(Resource)]
struct Music;

pub(super) fn plugin(app: &mut App) {
    app.add_audio_channel::<Music>()
        .add_systems(OnEnter(GameState::Title), start_title_music)
        .add_systems(OnEnter(GameState::Gameplay), start_gameplay_music)
        .add_systems(OnEnter(GameState::GameOver), start_gameover_music)
        .add_systems(OnExit(GameState::Title), stop_music)
        .add_systems(OnExit(GameState::GameOver), stop_music)
        .add_systems(OnExit(GameState::Gameplay), stop_music);
}

fn start_title_music(ui_assets: Res<UIAssets>, music: Res<AudioChannel<Music>>) {
    music.play(ui_assets.title_theme.clone()).looped();
}

fn start_gameplay_music(level_assets: Res<LevelAssets>, music: Res<AudioChannel<Music>>) {
    music.play(level_assets.game_bg1.clone()).looped();
}

fn start_gameover_music(ui_assets: Res<UIAssets>, music: Res<AudioChannel<Music>>) {
    music.play(ui_assets.game_over_bg.clone()).looped();
}

fn stop_music(music: Res<AudioChannel<Music>>) {
    music
        .stop()
        .fade_out(AudioTween::linear(Duration::from_millis(500)));
}
