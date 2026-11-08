use bevy::{platform::collections::HashMap, prelude::*};
use plants::Plant;

use crate::GameState;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_inventory);
}

#[derive(Component, Default, Debug)]
pub(crate) struct Inventory {
    pub(crate) plants: HashMap<Plant, u32>,
}

fn spawn_inventory(mut commands: Commands) {
    commands.spawn((
        Name::new("Inventory"),
        Inventory::default(),
        DespawnOnExit(GameState::Gameplay),
    ));
}
