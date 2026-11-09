use bevy::prelude::*;
use bevy_yarnspinner::prelude::{DialogueRunner, YarnValue};

use crate::{
    GameState,
    gameplay::{
        clock::Clock,
        visitors::{Cured, Visitor},
    },
};

pub(crate) const RADAR_CHARGES: u8 = 2;
const RADAR_RECHARGE_HOURS: f32 = 6.0;
pub(crate) const TRADER_STOCK: u8 = 2;
const TRADER_RESTOCK_HOURS: f32 = 24.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), reset)
        .add_observer(reward)
        .add_systems(
            Update,
            (recharge_radar, restock_trader).run_if(in_state(GameState::Gameplay)),
        );
}

#[derive(Resource, Default)]
pub(crate) struct Rewards {
    cures: u8,
    pub(crate) radar_charges: u8,
    radar_next: f32,
}

impl Rewards {
    pub(crate) fn radar(&self) -> bool {
        self.cures >= 1
    }

    pub(crate) fn hours_to_charge(&self, hours: f32) -> u32 {
        (self.radar_next - hours).ceil().max(1.0) as u32
    }

    pub(crate) fn double_chance(&self) -> f64 {
        match self.cures {
            0..=2 => 0.0,
            3 | 4 => 0.5,
            _ => 1.0,
        }
    }

    pub(crate) fn trader_chooses(&self) -> bool {
        self.cures >= 4
    }
}

#[derive(Component)]
pub(crate) struct Trader {
    pub(crate) stock: u8,
    next: f32,
}

impl Trader {
    pub(crate) fn traded(&mut self, hours: f32) {
        if self.stock == TRADER_STOCK {
            self.next = hours + TRADER_RESTOCK_HOURS;
        }
        self.stock = self.stock.saturating_sub(1);
    }
}

fn reset(mut commands: Commands) {
    commands.insert_resource(Rewards::default());
}

fn reward(
    cured: On<Add<Cured>>,
    mut commands: Commands,
    mut rewards: ResMut<Rewards>,
    clock: Single<&Clock>,
    visitors: Query<&Visitor>,
    mut runners: Query<&mut DialogueRunner>,
) {
    rewards.cures += 1;
    let name = visitors
        .get(cured.entity)
        .map(Visitor::name)
        .unwrap_or("Your patient");
    let message = match rewards.cures {
        1 => {
            rewards.radar_charges = RADAR_CHARGES;
            rewards.radar_next = clock.hours + RADAR_RECHARGE_HOURS;
            "You earned a herb radar. Press R outdoors to find the nearest herb.".to_string()
        }
        2 => {
            commands.entity(cured.entity).insert(Trader {
                stock: TRADER_STOCK,
                next: clock.hours + TRADER_RESTOCK_HOURS,
            });
            format!("The {name} will now trade herbs with you.")
        }
        3 => "You earned a scythe. Herbs sometimes drop double.".to_string(),
        4 => "The trader now lets you choose the herbs you get.".to_string(),
        5 => "You earned a sharper scythe. Herbs always drop double.".to_string(),
        _ => String::new(),
    };
    for mut runner in &mut runners {
        if let Err(error) = runner
            .variable_storage_mut()
            .set("$reward".into(), YarnValue::String(message.clone()))
        {
            error!("Couldn't tell the dialogue about the reward: {error}");
        }
    }
}

fn recharge_radar(mut rewards: ResMut<Rewards>, clock: Single<&Clock>) {
    if !rewards.radar() {
        return;
    }
    while rewards.radar_charges < RADAR_CHARGES && rewards.radar_next <= clock.hours {
        rewards.radar_charges += 1;
        rewards.radar_next += RADAR_RECHARGE_HOURS;
    }
    if rewards.radar_charges == RADAR_CHARGES && rewards.radar_next <= clock.hours {
        rewards.radar_next = clock.hours + RADAR_RECHARGE_HOURS;
    }
}

fn restock_trader(mut traders: Query<&mut Trader>, clock: Single<&Clock>) {
    for mut trader in &mut traders {
        while trader.stock < TRADER_STOCK && trader.next <= clock.hours {
            trader.stock += 1;
            trader.next += TRADER_RESTOCK_HOURS;
        }
    }
}
