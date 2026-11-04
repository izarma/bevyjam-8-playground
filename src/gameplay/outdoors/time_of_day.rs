use bevy::prelude::*;

use crate::{GameplayScene, gameplay::clock::Clock};

// i want to make these editable in jackdaw but it's a huge pain
const LOW_SUN_ELEVATION: f32 = 8.0_f32.to_radians();
const NIGHT_SUN_ELEVATION: f32 = -10.0_f32.to_radians();
const DAWN_SUN: Color = Color::srgb(1.0, 0.7, 0.55);
const DUSK_SUN: Color = Color::srgb(1.0, 0.55, 0.3);
const LOW_SUN_BRIGHTNESS: f32 = 0.25;
const DAWN_FOG: Color = Color::srgb(0.35, 0.28, 0.3);
const DUSK_FOG: Color = Color::srgb(0.3, 0.22, 0.2);
const NIGHT_FOG: Color = Color::srgb(0.02, 0.03, 0.06);

const KEYFRAMES: [(f32, Look); 8] = [
    (0.0, Look::Night),
    (4.0, Look::Night),
    (6.0, Look::Dawn),
    (9.0, Look::Day),
    (16.0, Look::Day),
    (18.0, Look::Dusk),
    (20.0, Look::Night),
    (24.0, Look::Night),
];

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (remember_day, follow_clock)
            .chain()
            .run_if(in_state(GameplayScene::Outdoors)),
    );
}

#[derive(Clone, Copy)]
enum Look {
    Night,
    Dawn,
    Day,
    Dusk,
}

#[derive(Component)]
struct DaySun {
    rotation: Quat,
    color: Color,
    illuminance: f32,
}

#[derive(Component)]
struct DayFog(Color);

struct Lighting {
    elevation: f32,
    color: LinearRgba,
    illuminance: f32,
}

fn remember_day(
    mut commands: Commands,
    suns: Query<(Entity, &Transform, &DirectionalLight), Without<DaySun>>,
    fogs: Query<(Entity, &DistanceFog), Without<DayFog>>,
) {
    for (entity, transform, light) in &suns {
        commands.entity(entity).insert(DaySun {
            rotation: transform.rotation,
            color: light.color,
            illuminance: light.illuminance,
        });
    }
    for (entity, fog) in &fogs {
        commands.entity(entity).insert(DayFog(fog.color));
    }
}

fn follow_clock(
    clock: Single<&Clock>,
    mut suns: Query<(&DaySun, &mut Transform, &mut DirectionalLight)>,
    mut fogs: Query<(&DayFog, &mut DistanceFog)>,
) {
    let hour = clock.hours % 24.0;
    let next = KEYFRAMES
        .iter()
        .position(|(at, _)| *at > hour)
        .expect("the last key is past every hour of the day");
    let (from_hour, from) = KEYFRAMES[next - 1];
    let (to_hour, to) = KEYFRAMES[next];
    let t = (hour - from_hour) / (to_hour - from_hour);

    for (day, mut transform, mut light) in &mut suns {
        let from = lighting(from, day);
        let to = lighting(to, day);
        let forward = day.rotation * Vec3::NEG_Z;
        let heading = Vec3::new(forward.x, 0.0, forward.z).normalize_or(Vec3::NEG_Z);
        let elevation = from.elevation.lerp(to.elevation, t);
        transform.look_to(
            heading * elevation.cos() - Vec3::Y * elevation.sin(),
            Vec3::Y,
        );
        light.color = from.color.mix(&to.color, t).into();
        light.illuminance = from.illuminance.lerp(to.illuminance, t);
    }
    for (day, mut fog) in &mut fogs {
        let from = fog_color(from, day.0);
        let to = fog_color(to, day.0);
        fog.color = from.mix(&to, t).into();
    }
}

fn lighting(look: Look, day: &DaySun) -> Lighting {
    let forward = day.rotation * Vec3::NEG_Z;
    let (color, elevation, illuminance) = match look {
        Look::Day => (day.color, (-forward.y).asin(), day.illuminance),
        Look::Dawn => (
            DAWN_SUN,
            LOW_SUN_ELEVATION,
            day.illuminance * LOW_SUN_BRIGHTNESS,
        ),
        Look::Dusk => (
            DUSK_SUN,
            LOW_SUN_ELEVATION,
            day.illuminance * LOW_SUN_BRIGHTNESS,
        ),
        Look::Night => (DUSK_SUN, NIGHT_SUN_ELEVATION, 0.0),
    };
    Lighting {
        elevation,
        color: color.into(),
        illuminance,
    }
}

fn fog_color(look: Look, day: Color) -> LinearRgba {
    match look {
        Look::Day => day,
        Look::Dawn => DAWN_FOG,
        Look::Dusk => DUSK_FOG,
        Look::Night => NIGHT_FOG,
    }
    .into()
}
