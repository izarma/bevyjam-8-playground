// the ambient light is what the sky shines on things: each scene's ambient is the
// light its PixelSky's gradient sends onto faces turned up, sideways and down, as
// bright as the sky looks. the sky is drawn as its colours, unexposed, while lit
// surfaces are scaled by the camera's exposure, so the sky's luminance is its
// colours over that exposure
use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{camera::Exposure, prelude::*};
use jackdaw_scene_types::environment::{AmbientMode, Environment};
use pixel_art::PixelSky;

use crate::camera::SceneCamera;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, light_from_sky);
}

fn light_from_sky(
    camera: Single<Ref<Exposure>, SceneCamera>,
    mut skies: Query<(Ref<PixelSky>, &mut Environment)>,
) {
    for (sky, mut environment) in &mut skies {
        if !sky.is_changed() && !camera.is_changed() {
            continue;
        }
        let ambient = &mut environment.ambient;
        ambient.mode = AmbientMode::Trilight;
        ambient.brightness = 1.0 / camera.exposure();
        ambient.sky = facing(&sky, Vec3::Y).into();
        ambient.equator = facing(&sky, Vec3::X).into();
        ambient.ground = facing(&sky, Vec3::NEG_Y).into();
    }
}

// the sky's colour in a direction, as its gradient runs (bands and sun aside)
fn sky_color(sky: &PixelSky, up: f32) -> LinearRgba {
    let g = (1.0 - up.abs()).powf(1.0 / sky.horizon_reach.max(0.05));
    let horizon = sky.horizon.to_linear();
    if up < 0.0 {
        return sky.ground.to_linear().mix(&horizon, g);
    }
    let middle = sky.middle.to_linear();
    if g < 0.5 {
        sky.zenith.to_linear().mix(&middle, g * 2.0)
    } else {
        middle.mix(&horizon, (g - 0.5) * 2.0)
    }
}

// the average colour of the sky a face sees, each direction counted by how
// squarely the face meets it
fn facing(sky: &PixelSky, normal: Vec3) -> LinearRgba {
    const ROWS: u32 = 32;
    const COLUMNS: u32 = 64;
    let mut sum = Vec3::ZERO;
    let mut weights = 0.0;
    for row in 0..ROWS {
        let elevation = ((row as f32 + 0.5) / ROWS as f32) * PI - FRAC_PI_2;
        for column in 0..COLUMNS {
            let around = (column as f32 + 0.5) / COLUMNS as f32 * TAU;
            let direction = Vec3::new(
                elevation.cos() * around.cos(),
                elevation.sin(),
                elevation.cos() * around.sin(),
            );
            // cos(elevation) is the patch's share of the sphere
            let weight = normal.dot(direction).max(0.0) * elevation.cos();
            let color = sky_color(sky, direction.y);
            sum += Vec3::new(color.red, color.green, color.blue) * weight;
            weights += weight;
        }
    }
    let average = sum / weights;
    LinearRgba::rgb(average.x, average.y, average.z)
}
