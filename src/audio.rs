use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::prelude::*;

use crate::{
    GameState, PausableSystems,
    gameplay::{PlayerAnimation, PlayerAnimationSystems},
    loading::{MusicAssets, PlayerAssets},
    scene_ready::Loading,
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
        .add_systems(Update, start_music.run_if(resource_exists::<MusicAssets>))
        .add_systems(OnExit(GameState::Title), stop_music)
        .add_systems(OnExit(GameState::GameOver), stop_music)
        .add_systems(OnExit(GameState::Gameplay), stop_music)
        // Footsteps follow the player animation.
        .add_systems(
            Update,
            trigger_step_sound
                .after(PlayerAnimationSystems::UpdateState)
                .in_set(PausableSystems)
                .run_if(in_state(GameState::Gameplay)),
        );
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
