use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::prelude::*;

use crate::{
    GameState, PausableSystems,
    gameplay::{PlayerAnimation, update_player_animation_state},
    loading::{MusicAssets, PlayerAssets},
};

/// Dedicated background music audio channel.
#[derive(Resource)]
struct Music;

/// This plugin controls all game audio: the soundtrack and gameplay sound
/// effects.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AudioPlugin)
        .add_audio_channel::<Music>()
        // The soundtrack follows the application state.
        .add_systems(OnEnter(GameState::Title), start_title_music)
        .add_systems(OnEnter(GameState::Gameplay), start_gameplay_music)
        .add_systems(OnEnter(GameState::GameOver), start_gameover_music)
        .add_systems(OnExit(GameState::Title), stop_music)
        .add_systems(OnExit(GameState::GameOver), stop_music)
        .add_systems(OnExit(GameState::Gameplay), stop_music)
        // Footsteps follow the player animation.
        .add_systems(
            Update,
            trigger_step_sound
                .after(update_player_animation_state)
                .in_set(PausableSystems)
                .run_if(in_state(GameState::Gameplay)),
        );
}

fn start_title_music(music_assets: Res<MusicAssets>, music: Res<AudioChannel<Music>>) {
    music.play(music_assets.title_theme.clone()).looped();
}

fn start_gameplay_music(music_assets: Res<MusicAssets>, music: Res<AudioChannel<Music>>) {
    music.play(music_assets.game_bg1.clone()).looped();
}

fn start_gameover_music(music_assets: Res<MusicAssets>, music: Res<AudioChannel<Music>>) {
    music.play(music_assets.game_over_bg.clone()).looped();
}

fn stop_music(music: Res<AudioChannel<Music>>) {
    music
        .stop()
        .fade_out(AudioTween::linear(Duration::from_millis(500)));
}

/// If the player is moving, play a random step sound effect synchronized with
/// the walk/run animation.
fn trigger_step_sound(
    player_assets: Res<PlayerAssets>,
    audio: Res<Audio>,
    step_query: Query<&PlayerAnimation>,
) {
    for animation in &step_query {
        if animation.step_frame() {
            let rng = &mut rand::rng();
            let steps = [
                &player_assets.step1,
                &player_assets.step2,
                &player_assets.step3,
                &player_assets.step4,
            ];
            audio.play(steps.choose(rng).copied().unwrap().clone());
        }
    }
}
