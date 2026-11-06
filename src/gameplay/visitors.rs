use bevy::prelude::*;
use plants::Plant;

use crate::{
    GameState,
    gameplay::{
        clock::Clock,
        herbs::{self, Element},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_visitors)
        .add_systems(Update, record_deaths.run_if(in_state(GameState::Gameplay)));
}

pub(crate) const MIN_HEALTH: u8 = 0;
pub(crate) const MAX_HEALTH: u8 = 10;
pub(crate) const BALANCED: u8 = (MIN_HEALTH + MAX_HEALTH) / 2;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Health {
    wood: u8,
    fire: u8,
    earth: u8,
    metal: u8,
    water: u8,
}

impl Health {
    pub(crate) fn get(self, element: Element) -> u8 {
        match element {
            Element::Wood => self.wood,
            Element::Fire => self.fire,
            Element::Earth => self.earth,
            Element::Metal => self.metal,
            Element::Water => self.water,
        }
    }

    fn get_mut(&mut self, element: Element) -> &mut u8 {
        match element {
            Element::Wood => &mut self.wood,
            Element::Fire => &mut self.fire,
            Element::Earth => &mut self.earth,
            Element::Metal => &mut self.metal,
            Element::Water => &mut self.water,
        }
    }

    fn shift(&mut self, element: Element, delta: i8) {
        let value = self.get_mut(element);
        *value = (i16::from(*value) + i16::from(delta)).clamp(MIN_HEALTH.into(), MAX_HEALTH.into())
            as u8;
    }

    pub(crate) fn offset(&mut self, delta: i8) {
        for element in Element::ALL {
            self.shift(element, delta);
        }
    }

    pub(crate) fn dose(&mut self, plant: Plant) {
        for &(element, delta) in herbs::yields(plant) {
            self.shift(element, delta);
        }
    }

    pub(crate) fn dead(self) -> bool {
        Element::ALL
            .into_iter()
            .any(|element| matches!(self.get(element), MIN_HEALTH | MAX_HEALTH))
    }
}

impl Default for Health {
    fn default() -> Self {
        Self {
            wood: BALANCED,
            fire: BALANCED,
            earth: BALANCED,
            metal: BALANCED,
            water: BALANCED,
        }
    }
}

// hardcoded schedules for now
// maybe i couldve done some data thing idk
struct Schedule {
    name: &'static str,
    first_day: u32,
    arrive: f32,
    leave: f32,
}

const SCHEDULES: [Schedule; 5] = [
    Schedule {
        name: "Monk",
        first_day: 0,
        arrive: 6.0,
        leave: 18.0,
    },
    Schedule {
        name: "Woman",
        first_day: 0,
        arrive: 6.0,
        leave: 13.0,
    },
    Schedule {
        name: "OldMan",
        first_day: 0,
        arrive: 10.0,
        leave: 16.0,
    },
    Schedule {
        name: "OldWoman",
        first_day: 1,
        arrive: 7.0,
        leave: 15.0,
    },
    Schedule {
        name: "Kid",
        first_day: 2,
        arrive: 9.0,
        leave: 14.0,
    },
];

pub(crate) const VISITORS: usize = SCHEDULES.len();

#[derive(Component)]
#[require(Health)]
pub(crate) struct Visitor(pub(crate) usize);

impl Visitor {
    pub(crate) fn name(&self) -> &'static str {
        SCHEDULES[self.0].name
    }
}

#[derive(Component)]
pub(crate) struct Died(pub(crate) f32);

fn spawn_visitors(mut commands: Commands) {
    for (index, schedule) in SCHEDULES.iter().enumerate() {
        commands.spawn((
            Visitor(index),
            Name::new(schedule.name),
            DespawnOnExit(GameState::Gameplay),
        ));
    }
}

fn record_deaths(
    mut commands: Commands,
    clock: Single<&Clock>,
    visitors: Query<(Entity, &Health), (Changed<Health>, Without<Died>)>,
) {
    for (visitor, health) in &visitors {
        if health.dead() {
            commands.entity(visitor).insert(Died(clock.hours));
        }
    }
}

pub(crate) fn lineup(hours: f32, deaths: [Option<f32>; VISITORS]) -> [Option<usize>; VISITORS] {
    let day = (hours / 24.0).floor();
    let hour = hours - day * 24.0;

    let mut events = Vec::new();
    for (index, schedule) in SCHEDULES.iter().enumerate() {
        if schedule.first_day as f32 > day {
            continue;
        }
        let mut leave = schedule.leave;
        if let Some(died) = deaths[index] {
            let death_day = (died / 24.0).floor();
            if death_day < day {
                continue;
            }
            if death_day == day {
                leave = leave.min(died - day * 24.0);
            }
        }
        events.push((schedule.arrive, true, index));
        events.push((leave, false, index));
    }
    events.retain(|&(at, ..)| at <= hour);
    events.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut taken = Vec::<bool>::new();
    let mut lineup = [None; VISITORS];
    for (_, arriving, visitor) in events {
        if arriving {
            let slot = taken.iter().position(|taken| !taken).unwrap_or_else(|| {
                taken.push(false);
                taken.len() - 1
            });
            taken[slot] = true;
            lineup[visitor] = Some(slot);
        } else if let Some(slot) = lineup[visitor].take() {
            taken[slot] = false;
        }
    }
    lineup
}
