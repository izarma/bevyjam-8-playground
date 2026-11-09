use bevy::prelude::*;
use plants::Plant;
use rand::{Rng, seq::IndexedRandom};

use crate::gameplay::herbs;

// how many herbs are used to make the remedy?
pub(crate) const SLOTS: usize = 3;

// a visitor's cure: three different herbs, in order
#[derive(Component)]
pub(crate) struct Remedy(pub(crate) [Plant; SLOTS]);

impl Remedy {
    pub(crate) fn roll(rng: &mut impl Rng) -> Self {
        let herbs = herbs::shifting()
            .map(|(herb, _)| herb)
            .collect::<Vec<_>>()
            .sample_array::<_, SLOTS>(rng);
        Remedy(herbs.expect("there are more herbs than slots"))
    }
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
