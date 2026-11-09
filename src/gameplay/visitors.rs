use bevy::prelude::*;
use bevy_kira_audio::prelude::{Audio, AudioControl};
use plants::Plant;
use rand::{
    RngExt,
    seq::{IteratorRandom, SliceRandom},
};

use crate::{
    GameResult, GameState,
    gameplay::{
        clock::{Clock, START_HOUR},
        herbs::{self, Element},
        remedy::{Mark, Remedy, SLOTS},
    },
    loading::UIAssets,
    loading_screen::Travel,
};

// an ailment worsens once every this many hours
const AILMENT_HOURS: f32 = 24.0;
// how far off balanced any element has to be before someone feels unwell
pub(crate) const UNWELL: u8 = 2;
// unwell visitors come to the lab between these hours
pub(crate) const DAY_START: f32 = 6.0;
pub(crate) const DAY_END: f32 = 20.0;
// visitors newly arrive no later than this
const ARRIVE_BY: f32 = 12.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_visitors)
        .add_observer(cure)
        .add_systems(
            Update,
            (
                progress_ailments,
                record_deaths,
                update_lineup,
                open_and_close_shop,
                end_game,
            )
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

    // for the dev tools
    #[cfg(debug_assertions)]
    pub(crate) fn offset(&mut self, delta: i8) {
        for element in Element::ALL {
            self.shift(element, delta);
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn make_unwell(&mut self) {
        self.wood = BALANCED - UNWELL;
    }

    pub(crate) fn dose(&mut self, plant: Plant) {
        match herbs::shift(plant) {
            Some(shift) => {
                for (element, delta) in shift.shifts() {
                    self.shift(element, delta);
                }
            }
            None => {
                for element in Element::ALL {
                    let toward = BALANCED.cmp(&self.get(element)) as i8;
                    self.shift(element, toward);
                }
            }
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

const NAMES: [&str; 6] = [
    "Guardian",
    "Monk",
    "Bride",
    "Beggar",
    "Old Beauty",
    "Orphan",
];

// Yarn node titles these names map to.
const NODES: [&str; 6] = ["Guardian", "Monk", "Bride", "Beggar", "OldBeauty", "Orphan"];

const DESCRIPTIONS: [&str; 6] = [
    "Your mentor grows weaker by the day, now too frail to handle the flood of visitors.",
    "Monk of the West. Strong. Bereft of his once great wealth",
    "Celebrated bride. Forgotten widow.",
    "Once he invaded kingdoms for glory; now takes pennies for a living.",
    "Her beauty was once unparalleled under the heavens. Now she's just old.",
    "Unmothered, unfathered, uneducated.",
];
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

    pub(crate) fn node(&self) -> &'static str {
        NODES[self.0]
    }

    pub(crate) fn description(&self) -> &'static str {
        DESCRIPTIONS[self.0]
    }

    pub(crate) fn guardian(&self) -> bool {
        self.0 == GUARDIAN
    }
}

/// A visitor the player has started talking to, and so should appear in the journal.
#[derive(Component)]
pub(crate) struct Met;

// the herbs the guardian hands over to be brewed foor the tutorial
#[derive(Component)]
pub(crate) struct Gift(pub(crate) [Plant; 3]);

impl Gift {
    fn roll(signature: Plant, sick: Health, remedy: &Remedy, rng: &mut impl rand::Rng) -> Self {
        let in_cure = remedy.0.into_iter().filter(|&herb| herb != signature);
        let outside = herbs::shifting()
            .map(|(herb, _)| herb)
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
        let (cure, other) = pairs.iter().copied().find(helps).unwrap_or(pairs[0]);
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

/// One brew a visitor actually drank: what went in and how it graded.
#[derive(Clone, Copy)]
pub(crate) struct Brew {
    pub(crate) mix: [Option<Plant>; SLOTS],
    pub(crate) marks: [Mark; SLOTS],
}

/// Every brew a visitor has drunk, in the order they drank them.
#[derive(Component, Default, Clone)]
pub(crate) struct BrewHistory {
    pub(crate) brews: Vec<Brew>,
}

#[derive(Clone, Copy)]
pub(crate) enum Ailment {
    Deficiency(Element),
    Excess(Element),
}

impl Ailment {
    fn undone_by(shift: herbs::Shift) -> Ailment {
        if shift.by > 0 {
            Ailment::Deficiency(shift.element)
        } else {
            Ailment::Excess(shift.element)
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
    commands.spawn((
        Name::new("Shop"),
        Shop { open: true },
        DespawnOnExit(GameState::Gameplay),
    ));
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
            day as f32 * 24.0 + rng.random_range(DAY_START..ARRIVE_BY)
        };
        // the guardian is sick before the game starts
        let days_ill = if guardian { GUARDIAN_DAYS_ILL } else { UNWELL };
        let (signature, shift) = herbs::shifting()
            .choose(&mut rng)
            .expect("herbs that shift an element");
        let ailment = Ailment::undone_by(shift);
        let remedy = Remedy::roll(&mut rng);
        let mut visitor = commands.spawn((
            Visitor(index),
            Name::new(name),
            Ailing {
                ailment,
                next: arrives - AILMENT_HOURS * f32::from(days_ill - 1),
            },
            BrewHistory::default(),
            DespawnOnExit(GameState::Gameplay),
        ));
        if guardian {
            let mut sick = Health::default();
            for _ in 0..days_ill {
                ailment.worsen(&mut sick);
            }
            visitor.insert((Gift::roll(signature, sick, &remedy, &mut rng), Returning));
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
#[derive(Component)]
pub(crate) struct Returning;

#[derive(Event)]
pub(crate) struct ShopOpened;

#[derive(Event)]
pub(crate) struct ShopClosed;

fn update_lineup(
    mut commands: Commands,
    clock: Single<&Clock>,
    visitors: Query<
        (
            Entity,
            &Health,
            Has<Died>,
            Has<Cured>,
            Has<Returning>,
            Option<&InLineup>,
        ),
        With<Visitor>,
    >,
    assets: Res<UIAssets>,
    audio: Res<Audio>,
) {
    let daytime = shop_open(&clock);
    let mut taken = visitors
        .iter()
        .filter_map(|(.., spot)| spot.map(|spot| spot.0))
        .collect::<Vec<_>>();
    for (visitor, health, died, cured, returning, spot) in &visitors {
        let here = daytime && !died && !cured && (returning || health.unwell());
        match (here, spot) {
            (true, None) => {
                let free = (0..).find(|spot| !taken.contains(spot)).unwrap_or_default();
                taken.push(free);
                commands.entity(visitor).insert(InLineup(free));
                if !returning {
                    commands.entity(visitor).insert(Returning);
                    audio.play(assets.bell.clone());
                }
            }
            (false, Some(spot)) => {
                taken.retain(|&taken| taken != spot.0);
                commands.entity(visitor).remove::<InLineup>();
            }
            _ => {}
        }
    }
}

fn shop_open(clock: &Clock) -> bool {
    (DAY_START..DAY_END).contains(&(clock.hours % 24.0))
}

#[derive(Component)]
struct Shop {
    open: bool,
}

fn open_and_close_shop(
    mut commands: Commands,
    clock: Single<&Clock>,
    mut shop: Single<&mut Shop>,
    assets: Res<UIAssets>,
    audio: Res<Audio>,
) {
    let open = shop_open(&clock);
    if shop.open == open {
        return;
    }
    shop.open = open;
    if open {
        audio.play(assets.bell.clone());
        commands.trigger(ShopOpened);
    } else {
        audio.play(assets.gong.clone());
        commands.trigger(ShopClosed);
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
