use bevy::prelude::*;
use plants::Plant;
use rand::{RngExt, seq::SliceRandom};

use crate::GameState;

// how many herbs are used to make the remedy?
pub(crate) const SLOTS: usize = 3;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_remedy);
}

// answer to the question: what is the cure?
// for now, it ALWAYS includes blue hibiscus in a random slot
// then is two other random herbs
#[derive(Component)]
pub(crate) struct Remedy(pub(crate) [Plant; SLOTS]);

fn spawn_remedy(mut commands: Commands) {
    let mut rng = rand::rng();
    let mut herbs = Plant::ALL
        .into_iter()
        .filter(|&plant| plant != Plant::BlueHibiscus)
        .collect::<Vec<_>>();
    herbs.shuffle(&mut rng);
    let mut remedy = [Plant::BlueHibiscus; SLOTS];
    let hibiscus = rng.random_range(0..SLOTS);
    let others = (0..SLOTS).filter(|&slot| slot != hibiscus);
    for (slot, herb) in others.zip(herbs) {
        remedy[slot] = herb;
    }
    commands.spawn((
        Name::new("Remedy"),
        Remedy(remedy),
        DespawnOnExit(GameState::Gameplay),
    ));
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Mark {
    Absent,
    Misplaced,
    Correct,
}

pub(crate) fn grade(mix: [Option<Plant>; SLOTS], remedy: [Plant; SLOTS]) -> [Mark; SLOTS] {
    let mut marks = [Mark::Absent; SLOTS];
    let mut unmatched = Vec::new();
    for slot in 0..SLOTS {
        if mix[slot] == Some(remedy[slot]) {
            marks[slot] = Mark::Correct;
        } else {
            unmatched.push(remedy[slot]);
        }
    }
    for slot in 0..SLOTS {
        if marks[slot] == Mark::Correct {
            continue;
        }
        if let Some(plant) = mix[slot]
            && let Some(index) = unmatched.iter().position(|&left| left == plant)
        {
            unmatched.swap_remove(index);
            marks[slot] = Mark::Misplaced;
        }
    }
    marks
}
