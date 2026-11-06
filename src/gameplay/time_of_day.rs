use bevy::prelude::*;

use jackdaw_scene_types::environment::Environment;

use crate::{GameState, gameplay::clock::Clock};

// i want to make these editable in jackdaw but it's a huge pain
const LOW_SUN_ELEVATION: f32 = 8.0_f32.to_radians();
const NIGHT_SUN_ELEVATION: f32 = -10.0_f32.to_radians();
const DAWN_SUN: Color = Color::srgb(1.0, 0.7, 0.55);
const DUSK_SUN: Color = Color::srgb(1.0, 0.55, 0.3);
const LOW_SUN_BRIGHTNESS: f32 = 0.25;
const DAWN_FOG: Color = Color::srgb(0.35, 0.28, 0.3);
const DUSK_FOG: Color = Color::srgb(0.3, 0.22, 0.2);
const NIGHT_FOG: Color = Color::srgb(0.02, 0.03, 0.06);

// (zenith, horizon, fraction of the day sky's brightness)
const DAWN_SKY: (Color, Color, f32) = (
    Color::srgb(0.32, 0.38, 0.6),
    Color::srgb(0.95, 0.66, 0.52),
    0.5,
);
const DUSK_SKY: (Color, Color, f32) = (
    Color::srgb(0.28, 0.27, 0.48),
    Color::srgb(1.0, 0.5, 0.3),
    0.4,
);
const NIGHT_SKY: (Color, Color, f32) = (
    Color::srgb(0.01, 0.015, 0.04),
    Color::srgb(0.03, 0.04, 0.09),
    0.15,
);

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
            .run_if(in_state(GameState::Gameplay)),
    );
}

#[derive(Clone, Copy)]
enum Look {
    Night,
    Dawn,
    Day,
    Dusk,
}

struct Moment {
    from: Look,
    to: Look,
    t: f32,
}

impl Moment {
    fn at(hours: f32) -> Self {
        let hour = hours % 24.0;
        let next = KEYFRAMES
            .iter()
            .position(|(at, _)| *at > hour)
            .expect("the last key is past every hour of the day");
        let (from_hour, from) = KEYFRAMES[next - 1];
        let (to_hour, to) = KEYFRAMES[next];
        Self {
            from,
            to,
            t: (hour - from_hour) / (to_hour - from_hour),
        }
    }

    fn blend<T>(&self, of: impl Fn(Look) -> T, mix: impl Fn(T, T, f32) -> T) -> T {
        mix(of(self.from), of(self.to), self.t)
    }
}

fn daylight(look: Look) -> f32 {
    match look {
        Look::Day => 1.0,
        Look::Dawn | Look::Dusk => LOW_SUN_BRIGHTNESS,
        Look::Night => 0.0,
    }
}

// the sun in the editor scene - how it looks by day.
#[derive(Component)]
struct DaySun {
    rotation: Quat,
    color: Color,
    illuminance: f32,
}

#[derive(Component)]
struct DayEnvironment(Environment);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Daylit {
    pub intensity: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct DaylitAmbient {
    pub night: f32,
}

fn remember_day(
    mut commands: Commands,
    suns: Query<(Entity, &Transform, &DirectionalLight), Without<DaySun>>,
    environments: Query<(Entity, &Environment), Without<DayEnvironment>>,
) {
    for (entity, transform, light) in &suns {
        commands.entity(entity).insert(DaySun {
            rotation: transform.rotation,
            color: light.color,
            illuminance: light.illuminance,
        });
    }
    for (entity, environment) in &environments {
        commands
            .entity(entity)
            .insert(DayEnvironment(environment.clone()));
    }
}

fn follow_clock(
    clock: Single<&Clock>,
    mut suns: Query<(&DaySun, &mut Transform, &mut DirectionalLight)>,
    mut environments: Query<(&DayEnvironment, &mut Environment, Option<&DaylitAmbient>)>,
    mut spots: Query<(&Daylit, &mut SpotLight)>,
    mut points: Query<(&Daylit, &mut PointLight)>,
) {
    let moment = Moment::at(clock.hours);
    let daylight = moment.blend(daylight, f32::lerp);
    let mix = |from: LinearRgba, to: LinearRgba, t| from.mix(&to, t);

    let mut sun_color = None;
    for (day, mut transform, mut light) in &mut suns {
        let forward = day.rotation * Vec3::NEG_Z;
        let heading = Vec3::new(forward.x, 0.0, forward.z).normalize_or(Vec3::NEG_Z);
        let elevation = moment.blend(|look| sun_elevation(look, day), f32::lerp);
        transform.look_to(
            heading * elevation.cos() - Vec3::Y * elevation.sin(),
            Vec3::Y,
        );
        light.color = moment.blend(|look| sun_color_of(look, day), mix).into();
        light.illuminance = day.illuminance * daylight;
        sun_color = Some(light.color);
    }

    for (day, mut environment, ambient) in &mut environments {
        let day = &day.0;
        let (zenith, horizon, brightness) = moment.blend(
            |look| sky(look, day),
            |from, to, t| {
                (
                    from.0.mix(&to.0, t),
                    from.1.mix(&to.1, t),
                    from.2.lerp(to.2, t),
                )
            },
        );
        environment.sky.zenith = zenith.into();
        environment.sky.horizon = horizon.into();
        environment.sky.brightness = brightness;
        environment.fog.color = moment
            .blend(|look| fog_color(look, day.fog.color), mix)
            .into();
        if let Some(ambient) = ambient {
            environment.ambient.brightness =
                ambient.night + (day.ambient.brightness - ambient.night) * daylight;
        }
    }

    let Some(sun_color) = sun_color else {
        return;
    };
    for (lit, mut light) in &mut spots {
        light.intensity = lit.intensity * daylight;
        light.color = sun_color;
    }
    for (lit, mut light) in &mut points {
        light.intensity = lit.intensity * daylight;
        light.color = sun_color;
    }
}

fn sun_elevation(look: Look, day: &DaySun) -> f32 {
    match look {
        Look::Day => (-(day.rotation * Vec3::NEG_Z).y).asin(),
        Look::Dawn | Look::Dusk => LOW_SUN_ELEVATION,
        Look::Night => NIGHT_SUN_ELEVATION,
    }
}

fn sun_color_of(look: Look, day: &DaySun) -> LinearRgba {
    match look {
        Look::Day => day.color,
        Look::Dawn => DAWN_SUN,
        Look::Dusk | Look::Night => DUSK_SUN,
    }
    .into()
}

fn sky(look: Look, day: &Environment) -> (LinearRgba, LinearRgba, f32) {
    let (zenith, horizon, fraction) = match look {
        Look::Day => (day.sky.zenith, day.sky.horizon, 1.0),
        Look::Dawn => DAWN_SKY,
        Look::Dusk => DUSK_SKY,
        Look::Night => NIGHT_SKY,
    };
    (zenith.into(), horizon.into(), day.sky.brightness * fraction)
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
