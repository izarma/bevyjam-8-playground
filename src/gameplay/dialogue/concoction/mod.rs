mod panel;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bevy::prelude::*;
use bevy_yarnspinner::prelude::{DialogueRunner, YarnValue};
use plants::Plant;

use super::{DialogueTarget, YarnSpinnerDialogueViewSystemSet, view::DialogueOptionsNode};
use crate::gameplay::{
    Health,
    clock::Clock,
    inventory::Inventory,
    remedy::{Mark, Remedy, SLOTS, grade},
    visitors::{Brew, BrewHistory, Cured, Visitor},
};

const HOURS_PER_DOSE: f32 = 2.0;

// each slot's brew: lift, shake, drop into the stamp (mark + dose), settle,
// then hold until the next slot
const LIFT: f32 = 0.18;
const SHAKE: f32 = 0.45;
const DROP: f32 = 0.08;
const STAMP_AT: f32 = LIFT + SHAKE + DROP;
const SETTLE: f32 = 0.12;
const SLOT_SECONDS: f32 = 1.3;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(panel::plugin)
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
    hurried: bool,
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
    stamped: Vec<Plant>,
    drunk: bool,
}

impl Concoction {
    fn new(visitor: Entity, done: Arc<AtomicBool>) -> Self {
        Self {
            visitor,
            mix: [None; SLOTS],
            stage: Stage::Mixing,
            hurried: false,
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

    fn cures(&self) -> bool {
        match &self.stage {
            Stage::Mixing => false,
            Stage::Brewing { marks, .. } | Stage::Drunk { marks } => {
                *marks == [Mark::Correct; SLOTS]
            }
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
        let Concoction {
            mix, stage, done, ..
        } = self;
        let mut step = Step::default();
        let mut left = seconds;
        // as many slots as `seconds` covers
        while let Stage::Brewing {
            marks,
            slot,
            elapsed,
        } = stage
        {
            let before = *elapsed;
            let spent = left.min(SLOT_SECONDS - before);
            *elapsed += spent;
            left -= spent;
            if before < STAMP_AT
                && *elapsed >= STAMP_AT
                && let Some(plant) = mix[*slot]
            {
                step.stamped.push(plant);
            }
            if *elapsed < SLOT_SECONDS {
                break;
            }
            *slot += 1;
            *elapsed = 0.0;
            if *slot == SLOTS {
                *stage = Stage::Drunk { marks: *marks };
                done.store(true, Ordering::Relaxed);
                step.drunk = true;
            }
        }
        step
    }

    // finish brewing instantly
    pub(crate) fn hurry(&mut self) {
        self.hurried = true;
    }
}

// hold the dialogue open while we brew
pub(super) fn offer_concoction(
    mut commands: Commands,
    target: Res<DialogueTarget>,
    options: Query<Entity, With<DialogueOptionsNode>>,
    inventory: Query<&Inventory>,
) -> Arc<AtomicBool> {
    let (Some(visitor), Some(options), Some(inventory)) =
        (target.0, options.iter().next(), inventory.iter().next())
    else {
        error!("offer_concoction ran without a visitor in dialogue");
        return Arc::new(AtomicBool::new(true));
    };
    let done = Arc::new(AtomicBool::new(false));
    panel::spawn(&mut commands, options, inventory).insert(Concoction::new(visitor, done.clone()));
    done
}

fn release_dialogue(remove: On<Remove<Concoction>>, concoctions: Query<&Concoction>) {
    if let Ok(concoction) = concoctions.get(remove.entity) {
        concoction.done.store(true, Ordering::Relaxed);
    }
}

fn brew(
    mut commands: Commands,
    time: Res<Time>,
    mut concoction: Single<&mut Concoction>,
    mut visitors: Query<(&Visitor, &mut Health, &mut BrewHistory)>,
    mut clock: Single<&mut Clock>,
    mut runner: Single<&mut DialogueRunner>,
) {
    let seconds = if concoction.hurried {
        f32::INFINITY
    } else {
        time.delta_secs()
    };
    let step = concoction.advance(seconds);
    let cures = concoction.cures();
    let Ok((visitor, mut health, mut history)) = visitors.get_mut(concoction.visitor) else {
        return;
    };
    if !cures {
        for plant in step.stamped {
            health.dose(plant);
        }
    }
    if step.drunk {
        clock.bank += HOURS_PER_DOSE;
        // only record a brew once it has actually been drunk
        let Stage::Drunk { marks } = &concoction.stage else {
            unreachable!("a drunk brew is always in the drunk stage");
        };
        history.brews.push(Brew {
            mix: concoction.mix,
            marks: *marks,
        });
        if cures {
            commands.entity(concoction.visitor).insert(Cured);
        }
        if visitor.guardian()
            && let Err(error) = runner
                .variable_storage_mut()
                .set("$treated_guardian".into(), YarnValue::Boolean(true))
        {
            error!("Couldn't tell the dialogue the guardian was treated: {error}");
        }
    }
}
