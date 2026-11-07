use bevy::prelude::*;

use crate::loading_screen::{Arrived, Travel};

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Entrance;

pub(crate) fn through_door(arrived: &Arrived) -> bool {
    matches!(arrived.0, Travel::EnterHouse | Travel::LeaveHouse)
}
