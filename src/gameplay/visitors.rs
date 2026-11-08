use bevy::prelude::*;
use plants::Plant;
use rand::{RngExt, seq::SliceRandom};

use crate::{
    GameResult, GameState,
    gameplay::{
        clock::{Clock, START_HOUR},
        herbs::{self, Element},
        remedy::Remedy,
    },
    loading_screen::Travel,
};

// an ailment worsens once every this many hours
const AILMENT_HOURS: f32 = 24.0;
// how far off balanced any element has to be before someone feels unwell
const UNWELL: u8 = 2;
// unwell visitors come to the lab between these hours
const DAY_START: f32 = 6.0;
const DAY_END: f32 = 20.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_visitors)
        .add_observer(cure)
        .add_systems(
            Update,
            (progress_ailments, record_deaths, update_lineup, end_game)
                .chain()
                .run_if(in_state(GameState::Gameplay)),
        );
}

pub(crate) const MIN_HEALTH: u8 = 0;
pub(crate) const MAX_HEALTH: u8 = 20;
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
        for (element, delta) in herbs::effect(plant).shifts() {
            self.shift(element, delta);
        }
    }

    fn imbalance(self) -> u16 {
        Element::ALL
            .into_iter()
            .map(|element| u16::from(self.get(element).abs_diff(BALANCED)))
            .sum()
    }

    fn unwell(self) -> bool {
        Element::ALL
            .into_iter()
            .any(|element| self.get(element).abs_diff(BALANCED) >= UNWELL)
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

const NAMES: [&str; 6] = ["Guardian", "Monk", "Woman", "OldMan", "OldWoman", "Kid"];
// already at the house when the game starts, further gone than the others
const GUARDIAN: usize = 0;
// days ill before the game starts
const GUARDIAN_DAYS_ILL: u8 = 4;

#[derive(Component)]
#[require(Health)]
pub(crate) struct Visitor(pub(crate) usize);

impl Visitor {
    pub(crate) fn name(&self) -> &'static str {
        NAMES[self.0]
    }

    pub(crate) fn guardian(&self) -> bool {
        self.0 == GUARDIAN
    }
}

// the herbs the guardian hands over to be brewed foor the tutorial
#[derive(Component)]
pub(crate) struct Gift(pub(crate) [Plant; 3]);

impl Gift {
    fn roll(signature: Plant, sick: Health, remedy: &Remedy, rng: &mut impl rand::Rng) -> Self {
        let in_cure = remedy.0.into_iter().filter(|&herb| herb != signature);
        let outside = Plant::ALL
            .into_iter()
            .filter(|herb| *herb != signature && !remedy.0.contains(herb))
            .collect::<Vec<_>>();
        let mut pairs = in_cure
            .flat_map(|cure| outside.iter().map(move |&other| (cure, other)))
            .collect::<Vec<_>>();
        pairs.shuffle(rng);
        let helps = |&(cure, other): &(Plant, Plant)| {
            let mut after = sick;
            for herb in [signature, cure, other] {
                after.dose(herb);
            }
            !after.dead() && after.imbalance() < sick.imbalance()
        };
        let (cure, other) = pairs
            .iter()
            .copied()
            .find(helps)
            .unwrap_or(pairs[0]);
        let mut herbs = [signature, cure, other];
        herbs.shuffle(rng);
        Gift(herbs)
    }
}

#[derive(Component)]
pub(crate) struct Died;

// drank their remedy
#[derive(Component)]
pub(crate) struct Cured;


#[derive(Clone, Copy)]
pub(crate) enum Ailment {
    Deficiency(Element),
    Excess(Element),
}

impl Ailment {
    fn undone_by(plant: Plant) -> Ailment {
        let effect = herbs::effect(plant);
        if effect.by > 0 {
            Ailment::Deficiency(effect.element)
        } else {
            Ailment::Excess(effect.element)
        }
    }

    pub(crate) fn name(self) -> String {
        match self {
            Ailment::Deficiency(element) => format!("{} Deficiency", element.name()),
            Ailment::Excess(element) => format!("{} Excess", element.name()),
        }
    }

    // what a herb has to do to undo it, as the dialogue says it
    pub(crate) fn wants(self) -> String {
        match self {
            Ailment::Deficiency(element) => format!("raise {}", element.name()),
            Ailment::Excess(element) => format!(
                "lower {} and raise {}",
                element.name(),
                element.next().name()
            ),
        }
    }

    // what each worsening does
    pub(crate) fn effects(self) -> Vec<(Element, i8)> {
        match self {
            Ailment::Deficiency(element) => vec![(element, -1)],
            Ailment::Excess(element) => vec![(element, 1), (element.next(), -1)],
        }
    }

    fn worsen(self, health: &mut Health) {
        for (element, delta) in self.effects() {
            health.shift(element, delta);
        }
    }
}

#[derive(Component)]
pub(crate) struct Ailing {
    pub(crate) ailment: Ailment,
    pub(crate) next: f32,
}

#[derive(Component)]
pub(crate) struct InLineup(pub(crate) usize);

fn spawn_visitors(mut commands: Commands) {
    let mut rng = rand::rng();
    // the guardian is there ont he first day, everyone else turns up unwell on their own day
    let mut days = (1..NAMES.len()).collect::<Vec<_>>();
    days.shuffle(&mut rng);
    let mut days = days.into_iter();
    for (index, name) in NAMES.into_iter().enumerate() {
        let guardian = index == GUARDIAN;
        let arrives = if guardian {
            START_HOUR
        } else {
            let day = days.next().expect("a day for everyone but the guardian");
            day as f32 * 24.0 + rng.random_range(DAY_START..DAY_END)
        };
        // the guardian is sick before the game starts
        let days_ill = if guardian { GUARDIAN_DAYS_ILL } else { UNWELL };
        let signature = Plant::ALL[rng.random_range(0..Plant::ALL.len())];
        let ailment = Ailment::undone_by(signature);
        let remedy = Remedy::roll(&mut rng);
        let mut visitor = commands.spawn((
            Visitor(index),
            Name::new(name),
            Ailing {
                ailment,
                next: arrives - AILMENT_HOURS * f32::from(days_ill - 1),
            },
            DespawnOnExit(GameState::Gameplay),
        ));
        if guardian {
            let mut sick = Health::default();
            for _ in 0..days_ill {
                ailment.worsen(&mut sick);
            }
            visitor.insert(Gift::roll(signature, sick, &remedy, &mut rng));
        }
        visitor.insert(remedy);
    }
}

fn progress_ailments(
    clock: Single<&Clock>,
    mut visitors: Query<(&mut Health, &mut Ailing), Without<Died>>,
) {
    for (mut health, mut ailing) in &mut visitors {
        while ailing.next <= clock.hours && !health.dead() {
            ailing.ailment.worsen(&mut health);
            ailing.next += AILMENT_HOURS;
        }
    }
}

fn record_deaths(
    mut commands: Commands,
    visitors: Query<(Entity, &Health), (Changed<Health>, Without<Died>)>,
) {
    for (visitor, health) in &visitors {
        if health.dead() {
            commands.entity(visitor).insert(Died);
        }
    }
}

// unwell visitors come in during the day, taking the first free spot
fn update_lineup(
    mut commands: Commands,
    clock: Single<&Clock>,
    visitors: Query<(Entity, &Health, Has<Died>, Option<&InLineup>), With<Visitor>>,
) {
    let hour = clock.hours % 24.0;
    let daytime = (DAY_START..DAY_END).contains(&hour);
    let mut taken = visitors
        .iter()
        .filter_map(|(.., spot)| spot.map(|spot| spot.0))
        .collect::<Vec<_>>();
    for (visitor, health, died, spot) in &visitors {
        let here = daytime && !died && health.unwell();
        match (here, spot) {
            (true, None) => {
                let free = (0..).find(|spot| !taken.contains(spot)).unwrap_or_default();
                taken.push(free);
                commands.entity(visitor).insert(InLineup(free));
            }
            (false, Some(spot)) => {
                taken.retain(|&taken| taken != spot.0);
                commands.entity(visitor).remove::<InLineup>();
            }
            _ => {}
        }
    }
}

fn cure(cured: On<Add<Cured>>, mut commands: Commands, mut health: Query<&mut Health>) {
    if let Ok(mut health) = health.get_mut(cured.entity) {
        *health = Health::default();
    }
    commands.entity(cured.entity).remove::<Ailing>();
}

// over once everyone is cured or dead: won if anyone was saved
fn end_game(
    mut commands: Commands,
    settled: Query<(), Or<(Added<Died>, Added<Cured>)>>,
    visitors: Query<(Has<Died>, Has<Cured>), With<Visitor>>,
) {
    if settled.is_empty() || !visitors.iter().all(|(died, cured)| died || cured) {
        return;
    }
    let saved = visitors.iter().any(|(_, cured)| cured);
    commands.insert_resource(if saved {
        GameResult::Won
    } else {
        GameResult::Lost
    });
    commands.trigger(Travel::GameOver);
}

