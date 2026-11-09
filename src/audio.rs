use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_yarnspinner::events::{DialogueCompleted, DialogueStarted};

use crate::{
    GameState, GameplayScene,
    gameplay::{Cured, clock::Clock},
    loading::{AmbienceAssets, NPCAssets, UIAssets},
    loading_screen::{Revealed, Travel},
};

mod footsteps;
mod harvesting;

#[derive(Resource)]
struct Ambience;

#[derive(Resource)]
struct Music;

const OUTDOOR_FADE: Duration = Duration::from_secs(2);
const FADE_OUT: Duration = Duration::from_millis(600);

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutdoorPeriod {
    Day,
    Night,
}

impl OutdoorPeriod {
    fn at(hours: f32) -> Self {
        if (6.0..18.0).contains(&hours.rem_euclid(24.0)) {
            Self::Day
        } else {
            Self::Night
        }
    }
}

#[derive(Resource, Default)]
struct OutdoorPlayback {
    period: Option<OutdoorPeriod>,
    revealed: bool,
}

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
    app.init_resource::<OutdoorPlayback>()
        .add_systems(Update, apply_volume.run_if(resource_changed::<Volume>))
        .add_plugins((footsteps::plugin, harvesting::plugin))
        .add_observer(play_dialogue_open)
        .add_observer(play_dialogue_close)
        .add_observer(play_healed)
        .add_observer(play_transition)
        .add_observer(reveal_outdoor_audio)
        .add_observer(start_title_ambience)
        .add_systems(OnExit(GameState::Title), stop_title_music)
        .add_systems(
            PostUpdate,
            update_outdoor_audio.run_if(in_state(GameplayScene::Outdoors)),
        )
        .add_systems(OnExit(GameplayScene::Outdoors), stop_outdoor_audio);
}

fn play_transition(
    travel: On<Travel>,
    assets: Res<UIAssets>,
    audio: Res<Audio>,
    mut next: Local<usize>,
) {
    if matches!(*travel, Travel::EnterHouse | Travel::LeaveHouse) && !assets.transitions.is_empty()
    {
        audio.play(assets.transitions[*next % assets.transitions.len()].clone());
        *next = (*next + 1) % assets.transitions.len();
    }
}

fn play_dialogue_open(_: On<DialogueStarted>, assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(assets.dialogue_open.clone()).with_volume(3.0);
}

fn play_healed(_: On<Add<Cured>>, assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(assets.healed.clone());
}

fn play_dialogue_close(_: On<DialogueCompleted>, npc_assets: Res<NPCAssets>, audio: Res<Audio>) {
    audio.play(npc_assets.dialogue_close.clone());
}

fn start_title_ambience(
    _: On<Revealed>,
    state: Res<State<GameState>>,
    assets: Res<AmbienceAssets>,
    music: Res<AudioChannel<Music>>,
) {
    if *state.get() != GameState::Title {
        return;
    }
    music
        .play(assets.title.clone())
        .linear_fade_in(Duration::from_millis(500))
        .looped();
}

fn reveal_outdoor_audio(
    _: On<Revealed>,
    scene: Option<Res<State<GameplayScene>>>,
    mut playback: ResMut<OutdoorPlayback>,
) {
    if scene.is_some_and(|scene| *scene.get() == GameplayScene::Outdoors) {
        playback.revealed = true;
    }
}

fn update_outdoor_audio(
    clock: Single<&Clock>,
    assets: Res<AmbienceAssets>,
    music: Res<AudioChannel<Music>>,
    ambience: Res<AudioChannel<Ambience>>,
    mut playback: ResMut<OutdoorPlayback>,
) {
    let period = OutdoorPeriod::at(clock.hours);
    if !playback.revealed || playback.period == Some(period) {
        return;
    }
    music.stop().linear_fade_out(OUTDOOR_FADE);
    ambience.stop().linear_fade_out(OUTDOOR_FADE);
    let background = match period {
        OutdoorPeriod::Day => &assets.outdoor_daytime,
        OutdoorPeriod::Night => &assets.outdoor_night,
    };
    ambience
        .play(background.clone())
        .linear_fade_in(OUTDOOR_FADE)
        .looped();
    let track = match period {
        OutdoorPeriod::Day => &assets.outdoor_music,
        OutdoorPeriod::Night => &assets.outdoor_night_music,
    };
    music
        .play(track.clone())
        .with_volume(6.0)
        .linear_fade_in(OUTDOOR_FADE)
        .looped();
    playback.period = Some(period);
}

fn stop_outdoor_audio(
    music: Res<AudioChannel<Music>>,
    ambience: Res<AudioChannel<Ambience>>,
    mut playback: ResMut<OutdoorPlayback>,
) {
    music.stop().linear_fade_out(FADE_OUT);
    ambience.stop().linear_fade_out(FADE_OUT);
    *playback = OutdoorPlayback::default();
}

fn stop_title_music(music: Res<AudioChannel<Music>>) {
    music.stop();
}
