mod panel;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use plants::Plant;

use super::{YarnSpinnerDialogueViewSystemSet, view::DialogueOptionsNode};
use crate::gameplay::{
    BodyOf, Health, InDialogue,
    clock::Clock,
    inventory::Inventory,
    remedy::{Mark, Remedy, SLOTS, grade},
};
use crate::loading::ConcoctionAssets;

const HOURS_PER_DOSE: f32 = 2.0;

// each slot's brew: lift, shake, drop into the stamp (mark + dose), settle,
// then hold until the next slot
const LIFT: f32 = 0.18;
const SHAKE: f32 = 0.45;
const DROP: f32 = 0.08;
const STAMP_AT: f32 = LIFT + SHAKE + DROP;
const SETTLE: f32 = 0.12;
const SLOT_SECONDS: f32 = 1.3;

#[derive(Resource)]
struct Anticipation;

pub(super) fn plugin(app: &mut App) {
    app.add_audio_channel::<Anticipation>()
        .add_plugins(panel::plugin)
        .add_observer(release_dialogue)
        .add_systems(
            Update,
            brew.in_set(YarnSpinnerDialogueViewSystemSet)
                .before(panel::PanelSystems)
                .run_if(any_with_component::<Concoction>),
        );
}

#[derive(Component)]
pub(crate) struct Concoction {
    visitor: Entity,
    mix: [Option<Plant>; SLOTS],
    stage: Stage,
    // for keeping the yarn dialogue open while we do the brew event
    done: Arc<AtomicBool>,
}

enum Stage {
    Mixing,
    Brewing {
        marks: [Mark; SLOTS],
        slot: usize,
        elapsed: f32,
    },
    Drunk {
        marks: [Mark; SLOTS],
    },
}

/// What one [`Concoction::advance`] did.
#[derive(Default)]
struct Step {
    anticipating: bool,
    stamped: Option<Plant>,
    revealed: Option<Mark>,
    drunk: bool,
}

impl Concoction {
    fn new(visitor: Entity, done: Arc<AtomicBool>) -> Self {
        Self {
            visitor,
            mix: [None; SLOTS],
            stage: Stage::Mixing,
            done,
        }
    }

    pub(super) fn mixing(&self) -> bool {
        matches!(self.stage, Stage::Mixing)
    }

    pub(super) fn herb(&self, slot: usize) -> Option<Plant> {
        self.mix[slot]
    }

    pub(super) fn available(&self, inventory: &Inventory, plant: Plant) -> u32 {
        let held = inventory.plants.get(&plant).copied().unwrap_or(0);
        let used = self
            .mix
            .iter()
            .filter(|&&mixed| mixed == Some(plant))
            .count() as u32;
        held.saturating_sub(used)
    }

    pub(super) fn put(&mut self, slot: usize, plant: Plant, inventory: &Inventory) -> bool {
        let freed = u32::from(self.mix[slot] == Some(plant));
        if !self.mixing() || self.available(inventory, plant) + freed == 0 {
            return false;
        }
        self.mix[slot] = Some(plant);
        true
    }

    pub(super) fn clear(&mut self, slot: usize) {
        if self.mixing() {
            self.mix[slot] = None;
        }
    }

    pub(super) fn start_brewing(&mut self, remedy: &Remedy, inventory: &mut Inventory) -> bool {
        if !self.mixing() || self.mix.iter().all(Option::is_none) {
            return false;
        }
        for &plant in self.mix.iter().flatten() {
            if let Some(count) = inventory.plants.get_mut(&plant) {
                *count -= 1;
                if *count == 0 {
                    inventory.plants.remove(&plant);
                }
            }
        }
        self.stage = Stage::Brewing {
            marks: grade(self.mix, remedy.0),
            slot: 0,
            elapsed: 0.0,
        };
        true
    }

    pub(super) fn preview(&self, mut health: Health) -> Health {
        if self.mixing() {
            for &plant in self.mix.iter().flatten() {
                health.dose(plant);
            }
        }
        health
    }

    pub(crate) fn brewing(&self) -> Option<(usize, f32)> {
        match self.stage {
            Stage::Brewing { slot, elapsed, .. } => Some((slot, elapsed)),
            _ => None,
        }
    }

    pub(super) fn marks(&self) -> [Option<Mark>; SLOTS] {
        match &self.stage {
            Stage::Mixing => [None; SLOTS],
            Stage::Brewing {
                marks,
                slot,
                elapsed,
            } => {
                let stamped = slot + usize::from(*elapsed >= STAMP_AT);
                std::array::from_fn(|index| (index < stamped).then_some(marks[index]))
            }
            Stage::Drunk { marks } => marks.map(Some),
        }
    }

    // moves through the brewing timeline in `Stage`s and `Step`s.
    fn advance(&mut self, seconds: f32) -> Step {
        let Stage::Brewing {
            marks,
            slot,
            elapsed,
        } = &mut self.stage
        else {
            return Step::default();
        };
        let mut step = Step::default();
        let before = *elapsed;
        *elapsed += seconds;
        step.anticipating = before < LIFT && *elapsed >= LIFT && *elapsed < STAMP_AT;
        if before < STAMP_AT && *elapsed >= STAMP_AT {
            step.stamped = self.mix[*slot];
            step.revealed = Some(marks[*slot]);
        }
        if *elapsed >= SLOT_SECONDS {
            *slot += 1;
            *elapsed = 0.0;
            if *slot == SLOTS {
                self.stage = Stage::Drunk { marks: *marks };
                self.done.store(true, Ordering::Relaxed);
                step.drunk = true;
            }
        }
        step
    }
}

// hold the dialogue open while we brew
pub(super) fn offer_concoction(
    mut commands: Commands,
    npc: Query<&BodyOf, With<InDialogue>>,
    options: Query<Entity, With<DialogueOptionsNode>>,
    inventory: Query<&Inventory>,
) -> Arc<AtomicBool> {
    let (Some(body), Some(options), Some(inventory)) = (
        npc.iter().next(),
        options.iter().next(),
        inventory.iter().next(),
    ) else {
        error!("offer_concoction ran without a visitor in dialogue");
        return Arc::new(AtomicBool::new(true));
    };
    let done = Arc::new(AtomicBool::new(false));
    panel::spawn(&mut commands, options, inventory).insert(Concoction::new(body.0, done.clone()));
    done
}

fn release_dialogue(
    remove: On<Remove<Concoction>>,
    concoctions: Query<&Concoction>,
    anticipation: Res<AudioChannel<Anticipation>>,
) {
    anticipation
        .stop()
        .fade_out(AudioTween::linear(Duration::from_millis(10)));
    if let Ok(concoction) = concoctions.get(remove.entity) {
        concoction.done.store(true, Ordering::Relaxed);
    }
}

// if one of the stamp steps happened, dose the npc
fn brew(
    time: Res<Time>,
    mut concoction: Single<&mut Concoction>,
    mut visitors: Query<&mut Health>,
    mut clock: Single<&mut Clock>,
    assets: Res<ConcoctionAssets>,
    audio: Res<Audio>,
    mut next_result: Local<[usize; 3]>,
    anticipation: Res<AudioChannel<Anticipation>>,
    mut next_anticipation: Local<usize>,
) {
    let step = concoction.advance(time.delta_secs());
    if step.anticipating && !assets.herb_anticipation.is_empty() {
        let clips = &assets.herb_anticipation;
        anticipation.play(clips[*next_anticipation % clips.len()].clone());
        *next_anticipation = (*next_anticipation + 1) % clips.len();
    }
    if let Some(mark) = step.revealed {
        anticipation
            .stop()
            .fade_out(AudioTween::linear(Duration::from_millis(10)));
        let (clips, index) = match mark {
            Mark::Correct => (&assets.herb_good, 0),
            Mark::Misplaced => (&assets.herb_neutral, 1),
            Mark::Absent => (&assets.herb_bad, 2),
        };
        if !clips.is_empty() {
            audio.play(clips[next_result[index] % clips.len()].clone());
            next_result[index] = (next_result[index] + 1) % clips.len();
        }
    }
    if let Some(plant) = step.stamped
        && let Ok(mut health) = visitors.get_mut(concoction.visitor)
    {
        health.dose(plant);
    }
    if step.drunk {
        clock.bank += HOURS_PER_DOSE;
    }
}
