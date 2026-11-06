use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{Harvesting, Pause, gameplay::HarvestScratch, loading::HarvestingAssets};

const SCRATCH_INTERVAL: Duration = Duration::from_millis(150);
const SCRATCH_GRACE: Duration = Duration::from_millis(100);
const SCRATCH_FADE: Duration = Duration::from_millis(40);

#[derive(Resource)]
struct Scratching;

#[derive(Resource, Default)]
struct ScratchPlayback {
    last_scratch: Option<Duration>,
    next_clip: usize,
    instance: Option<Handle<AudioInstance>>,
    next_allowed: Duration,
}

pub(super) fn plugin(app: &mut App) {
    let mut track = AudioTrack::new();
    track.add_effect(
        CompressorBuilder::new()
            .threshold(-18.0)
            .ratio(4.0)
            .attack_duration(Duration::from_millis(5))
            .release_duration(Duration::from_millis(100)),
    );
    app.add_audio_channel_with_track::<Scratching>(track)
        .init_resource::<ScratchPlayback>()
        .add_observer(record_scratch)
        .add_systems(PostUpdate, update_playback);
}

fn record_scratch(_: On<HarvestScratch>, time: Res<Time>, mut playback: ResMut<ScratchPlayback>) {
    playback.last_scratch = Some(time.elapsed());
}

fn update_playback(
    time: Res<Time>,
    harvesting: Option<Res<State<Harvesting>>>,
    pause: Option<Res<State<Pause>>>,
    assets: Option<Res<HarvestingAssets>>,
    audio: Res<AudioChannel<Scratching>>,
    mut playback: ResMut<ScratchPlayback>,
) {
    let harvesting = harvesting.is_some_and(|state| state.0) && pause.is_some_and(|state| !state.0);
    if !harvesting {
        if playback.instance.take().is_some() {
            audio.stop().fade_out(AudioTween::linear(SCRATCH_FADE));
        }
        playback.last_scratch = None;
        return;
    }
    let scratching = playback
        .last_scratch
        .is_some_and(|last| time.elapsed().saturating_sub(last) <= SCRATCH_GRACE);
    if !scratching || time.elapsed() < playback.next_allowed {
        return;
    }
    let Some(assets) = assets else {
        return;
    };
    if assets.scratches.is_empty() {
        return;
    }
    let index = playback.next_clip % assets.scratches.len();
    playback.instance = Some(audio.play(assets.scratches[index].clone()).handle());
    playback.next_allowed = time.elapsed() + SCRATCH_INTERVAL;
    playback.next_clip = (index + 1) % assets.scratches.len();
}
