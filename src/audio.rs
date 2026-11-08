use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_yarnspinner::events::{DialogueCompleted, DialogueStarted};

use crate::{
    GameplayScene,
    loading::{AmbienceAssets, NPCAssets},
    loading_screen::Revealed,
};

mod footsteps;
mod harvesting;

const FADE_IN: Duration = Duration::from_secs(1);
const FADE_OUT: Duration = Duration::from_millis(600);

#[derive(Resource)]
struct Ambience;

#[derive(Resource)]
struct Music;

#[derive(Resource, Clone, Copy)]
pub(crate) struct Volume {
    pub(crate) sound: u32,
    pub(crate) music: u32,
}

impl Default for Volume {
    fn default() -> Self {
        Self {
            sound: 100,
            music: 100,
        }
    }
}

pub(crate) enum Mix {
    Sound,
    Music,
}

#[derive(Resource, Default)]
struct MixControls {
    sound: Vec<VolumeControlHandle>,
    music: Vec<VolumeControlHandle>,
}

pub(crate) fn add_mixed_channel<T: Resource>(
    app: &mut App,
    mix: Mix,
    mut track: AudioTrack,
) -> &mut App {
    let control = track.add_effect(VolumeControlBuilder::new(Decibels::IDENTITY));
    let mut controls = app.world_mut().get_resource_or_init::<MixControls>();
    match mix {
        Mix::Sound => controls.sound.push(control),
        Mix::Music => controls.music.push(control),
    }
    app.add_audio_channel_with_track::<T>(track)
}

fn decibels(percent: u32) -> Decibels {
    if percent == 0 {
        return Decibels::SILENCE;
    }
    Decibels(20.0 * (percent as f32 / 100.0).log10())
}

fn apply_volume(volume: Res<Volume>, mut controls: ResMut<MixControls>) {
    let MixControls { sound, music } = &mut *controls;
    for control in sound {
        control.set_volume(decibels(volume.sound), AudioTween::default());
    }
    for control in music {
        control.set_volume(decibels(volume.music), AudioTween::default());
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AudioPlugin).init_resource::<Volume>();
    add_mixed_channel::<MainTrack>(app, Mix::Sound, AudioTrack::new());
    add_mixed_channel::<Ambience>(app, Mix::Sound, AudioTrack::new());
    add_mixed_channel::<Music>(app, Mix::Music, AudioTrack::new());
    app.add_systems(Update, apply_volume.run_if(resource_changed::<Volume>))
        .add_plugins((footsteps::plugin, harvesting::plugin))
        .add_observer(play_dialogue_open)
        .add_observer(play_dialogue_close)
        .add_observer(start_outdoor_sound)
        .add_systems(OnExit(GameplayScene::Outdoors), stop_ambience);
}

fn play_dialogue_open(_: On<DialogueStarted>, npc_assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(npc_assets.dialogue_open.clone());
}

fn play_dialogue_close(_: On<DialogueCompleted>, npc_assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(npc_assets.dialogue_close.clone());
}

fn start_outdoor_sound(
    _: On<Revealed>,
    scene: Option<Res<State<GameplayScene>>>,
    assets: Res<AmbienceAssets>,
    ambience: Res<AudioChannel<Ambience>>,
    music: Res<AudioChannel<Music>>,
) {
    if scene.is_none_or(|scene| *scene.get() != GameplayScene::Outdoors) {
        return;
    }
    if !ambience.is_playing_sound() {
        ambience
            .play(assets.outdoors.clone())
            .linear_fade_in(FADE_IN)
            .looped();
    }
    if !music.is_playing_sound() {
        music
            .play(assets.outdoor_music.clone())
            .with_volume(6.0)
            .linear_fade_in(FADE_IN)
            .looped();
    }
}

fn stop_ambience(ambience: Res<AudioChannel<Ambience>>, music: Res<AudioChannel<Music>>) {
    ambience.stop().linear_fade_out(FADE_OUT);
    music.stop().linear_fade_out(FADE_OUT);
}
