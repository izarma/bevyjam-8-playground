use crate::GameState;
use crate::engine::PausableSystems;
use crate::engine::loading::PlayerAssets;
use crate::gameplay::player::{PlayerAnimation, update_player_animation_state};
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::prelude::*;

pub struct InternalAudioPlugin;

// This plugin is responsible to control the game audio
impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin).add_systems(
            Update,
            trigger_step_sound
                .after(update_player_animation_state)
                .in_set(PausableSystems)
                .run_if(in_state(GameState::Gameplay)),
        );
    }
}

/// If the player is moving, play a random step sound effect synchronized with
/// the walk/run animation.
fn trigger_step_sound(
    player_assets: Res<PlayerAssets>,
    audio: Res<Audio>,
    mut step_query: Query<&PlayerAnimation>,
) {
    for animation in &mut step_query {
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
