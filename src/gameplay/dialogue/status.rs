use bevy::prelude::*;
use bevy_yarnspinner::prelude::{DialogueRunner, YarnValue};

use super::DialogueTarget;
use crate::gameplay::{
    BALANCED, Health, Met, Visitor,
    herbs::Element,
    upgrades::Trader,
    visitors::{Ailing, Cured, Died, UNWELL},
};

/// For dialogues
pub(super) fn refresh_visitor(
    target: Res<DialogueTarget>,
    visitors: Query<(
        &Visitor,
        &Health,
        Has<Cured>,
        Has<Died>,
        Option<&Ailing>,
        Has<Met>,
        Option<&Trader>,
    )>,
    mut runners: Query<&mut DialogueRunner>,
) {
    let Some(visitor) = target.0 else {
        return;
    };
    let Ok((who, health, cured, dead, ailing, met, trader)) = visitors.get(visitor) else {
        return;
    };
    let Ok(mut runner) = runners.single_mut() else {
        return;
    };

    let (ailment, wants) = ailing
        .map(|ailing| (ailing.ailment.name(), ailing.ailment.wants()))
        .unwrap_or_default();

    let variables = [
        ("$visitor_cured", YarnValue::Boolean(cured)),
        ("$visitor_dead", YarnValue::Boolean(dead)),
        ("$visitor_ailment", YarnValue::String(ailment)),
        ("$visitor_wants", YarnValue::String(wants)),
        ("$symptoms", YarnValue::String(symptoms(*health))),
        ("$met", YarnValue::Boolean(met)),
        ("$trader_name", YarnValue::String(who.name().to_string())),
        (
            "$trader_stock",
            YarnValue::Number(f32::from(trader.map_or(0, |trader| trader.stock))),
        ),
    ];
    for (name, value) in variables {
        let _ = runner.variable_storage_mut().set(name.to_string(), value);
    }
}

/// Marks the current visitor as met, so they show up in the journal.
pub(super) fn mark_met(target: Res<DialogueTarget>, mut commands: Commands) {
    if let Some(visitor) = target.0 {
        commands.entity(visitor).insert(Met);
    }
}

fn symptoms(health: Health) -> String {
    let mut deviations = Element::ALL
        .into_iter()
        .map(|element| {
            let offset = i16::from(health.get(element)) - i16::from(BALANCED);
            (element, offset)
        })
        .filter(|&(_, offset)| offset.unsigned_abs() >= u16::from(UNWELL))
        .collect::<Vec<_>>();
    deviations.sort_by_key(|&(_, offset)| std::cmp::Reverse(offset.unsigned_abs()));

    let mut words = deviations
        .into_iter()
        .map(|(element, offset)| symptom(element, offset > 0))
        .collect::<Vec<_>>();
    match words.len() {
        0 => "out of sorts".to_string(),
        1 => words.pop().unwrap().to_string(),
        _ => {
            let last = words.pop().unwrap();
            format!("{} and {last}", words.join(", "))
        }
    }
}

/// The word a visitor uses for an element being too high or too low.
fn symptom(element: Element, excess: bool) -> &'static str {
    match (element, excess) {
        (Element::Wood, true) => "restless",
        (Element::Wood, false) => "weary",
        (Element::Fire, true) => "hot",
        (Element::Fire, false) => "cold",
        (Element::Earth, true) => "heavy",
        (Element::Earth, false) => "floaty",
        (Element::Metal, true) => "stiff",
        (Element::Metal, false) => "limp",
        (Element::Water, true) => "damp",
        (Element::Water, false) => "dry",
    }
}
