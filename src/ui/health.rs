use bevy::prelude::*;

use crate::{
    gameplay::{BALANCED, MAX_HEALTH, MIN_HEALTH},
    ui::palette::{GAIN_TEXT, LOSS_TEXT},
};

pub(crate) fn bar_width(value: u8) -> f32 {
    f32::from(value - MIN_HEALTH) / f32::from(MAX_HEALTH - MIN_HEALTH) * 100.0
}

// green or red based on distance from balanced
pub(crate) fn health_bar_color(value: u8) -> Color {
    let value = value.clamp(MIN_HEALTH, MAX_HEALTH);
    let t = if value >= BALANCED {
        f32::from(value - BALANCED) / f32::from(MAX_HEALTH - BALANCED)
    } else {
        f32::from(BALANCED - value) / f32::from(BALANCED - MIN_HEALTH)
    };
    GAIN_TEXT.mix(&LOSS_TEXT, t)
}
