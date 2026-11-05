// ui scaling for pixel art ui
use bevy::{prelude::*, window::PrimaryWindow};

const WINDOW_TEXELS_TALL: f32 = 240.0;

// logical ui pixels per texel
pub(crate) const TEXEL: f32 = 2.0;

// size all ui in whole texels to keep it on the grid
pub(crate) fn texels(n: u16) -> Val {
    px(f32::from(n) * TEXEL)
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, fit_ui_scale);
}

pub(crate) fn pixels_per_texel(window: &Window) -> f32 {
    (window.physical_height() as f32 / WINDOW_TEXELS_TALL)
        .floor()
        .max(1.0)
}

fn fit_ui_scale(window: Single<&Window, With<PrimaryWindow>>, mut ui_scale: ResMut<UiScale>) {
    let scale = pixels_per_texel(&window) / (TEXEL * window.scale_factor());
    if ui_scale.0 != scale {
        ui_scale.0 = scale;
    }
}
