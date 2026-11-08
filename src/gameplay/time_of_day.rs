use std::f32::consts::PI;

use bevy::prelude::*;

use jackdaw_scene_types::environment::Environment;
use pixel_art::PixelSky;

use crate::gameplay::clock::Clock;

// i want to make these editable in jackdaw but it's a huge pain
const DAWN_SUN: Color = Color::srgb(1.0, 0.7, 0.55);
const DUSK_SUN: Color = Color::srgb(1.0, 0.55, 0.3);
const LOW_SUN_BRIGHTNESS: f32 = 0.6;
const MOONLIGHT: Color = Color::srgb(0.55, 0.65, 0.9);
const MOON_BRIGHTNESS: f32 = 0.2;
const MOON_RISES: f32 = -0.04;
const ATMOSPHERE: f32 = 0.2;

struct SkyTint {
    zenith: Color,
    middle: Color,
    horizon: Color,
    ground: Color,
    glow: Color,
    sunset: Color,
    stars: f32,
}

const DAWN_SKY: SkyTint = SkyTint {
    zenith: Color::srgb(0.165, 0.208, 0.439),
    middle: Color::srgb(0.541, 0.361, 0.557),
    horizon: Color::srgb(0.953, 0.604, 0.388),
    ground: Color::srgb(0.227, 0.165, 0.243),
    glow: Color::srgb(1.0, 0.769, 0.478),
    sunset: Color::srgb(1.0, 0.541, 0.314),
    stars: 0.15,
};
const DUSK_SKY: SkyTint = SkyTint {
    zenith: Color::srgb(0.165, 0.208, 0.439),
    middle: Color::srgb(0.541, 0.361, 0.557),
    horizon: Color::srgb(0.941, 0.541, 0.294),
    ground: Color::srgb(0.227, 0.165, 0.243),
    glow: Color::srgb(1.0, 0.706, 0.416),
    sunset: Color::srgb(1.0, 0.478, 0.251),
    stars: 0.15,
};
const NIGHT_SKY: SkyTint = SkyTint {
    zenith: Color::srgb(0.027, 0.039, 0.11),
    middle: Color::srgb(0.071, 0.094, 0.22),
    horizon: Color::srgb(0.137, 0.165, 0.322),
    ground: Color::srgb(0.047, 0.059, 0.118),
    glow: Color::srgb(0.227, 0.267, 0.471),
    sunset: Color::srgb(0.227, 0.227, 0.431),
    stars: 1.0,
};

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
    // wherever there's a clock: gameplay's, and the title's that stands still
    app.add_systems(Update, (remember_day, follow_clock).chain());
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

// the sun as the scene places it: how it looks by day, and where it stands at noon
#[derive(Component)]
struct DaySun {
    rotation: Quat,
    color: Color,
    illuminance: f32,
}

#[derive(Component)]
struct DayAmbient(f32);

#[derive(Component)]
struct DaySky(PixelSky);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Daylit {
    pub intensity: f32,
}

#[derive(Component)]
pub(crate) struct Nightlit {
    pub(crate) intensity: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct DaylitAmbient {
    pub night: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct DaylitExposure {
    pub day: f32,
}

#[derive(Component)]
struct NightExposure(f32);

fn remember_day(
    mut commands: Commands,
    suns: Query<(Entity, &Transform, &DirectionalLight), Without<DaySun>>,
    ambient_lit: Query<(Entity, &Environment), (With<DaylitAmbient>, Without<DayAmbient>)>,
    exposed: Query<(Entity, &Environment), (With<DaylitExposure>, Without<NightExposure>)>,
    skies: Query<(Entity, &PixelSky), Without<DaySky>>,
) {
    for (entity, transform, light) in &suns {
        commands.entity(entity).insert(DaySun {
            rotation: transform.rotation,
            color: light.color,
            illuminance: light.illuminance,
        });
    }
    for (entity, environment) in &ambient_lit {
        commands
            .entity(entity)
            .insert(DayAmbient(environment.ambient.brightness));
    }
    for (entity, environment) in &exposed {
        commands
            .entity(entity)
            .insert(NightExposure(environment.post.exposure));
    }
    for (entity, sky) in &skies {
        commands.entity(entity).insert(DaySky(sky.clone()));
    }
}

fn follow_clock(
    clock: Single<&Clock>,
    mut suns: Query<(&DaySun, &mut Transform, &mut DirectionalLight)>,
    mut environments: Query<(
        &mut Environment,
        Option<(&DayAmbient, &DaylitAmbient)>,
        Option<(&DaylitExposure, &NightExposure)>,
    )>,
    mut skies: Query<(&DaySky, &mut PixelSky)>,
    mut spots: Query<(&Daylit, &mut SpotLight)>,
    mut points: Query<(&Daylit, &mut PointLight), Without<Nightlit>>,
    mut lanterns: Query<(&Nightlit, &mut PointLight), Without<Daylit>>,
) {
    let moment = Moment::at(clock.hours);
    let daylight = moment.blend(daylight, f32::lerp);
    let mix = |from: LinearRgba, to: LinearRgba, t| from.mix(&to, t);

    let mut shining = None;
    let mut sun = None;
    // the scene's sun is where it stands at noon. it crosses the sky on the circle
    // through there that meets the horizon at 06:00 and 18:00, square to its heading
    let turn = (clock.hours % 24.0 - 12.0) / 12.0 * PI;
    for (day, mut transform, mut light) in &mut suns {
        let noon = day.rotation * Vec3::Z;
        let heading = Vec3::new(noon.x, 0.0, noon.z).normalize_or(Vec3::Z);
        let across = Vec3::Y.cross(heading);
        let toward = noon * turn.cos() + across * turn.sin();
        let (from, color, brightness) = if toward.y < MOON_RISES {
            (-toward, MOONLIGHT, MOON_BRIGHTNESS)
        } else {
            (
                toward,
                moment.blend(|look| sun_color_of(look, day), mix).into(),
                daylight,
            )
        };
        let brightness = brightness * through_air(from.y);
        transform.look_to(-from, Vec3::Y);
        light.color = color;
        light.illuminance = day.illuminance * brightness;
        shining = Some((color, brightness));
        sun = Some(toward);
    }

    for (day, mut sky) in &mut skies {
        let mut now = moment.blend(|look| sky_at(look, &day.0), mix_skies);
        if let Some(sun) = sun {
            now.sun = sun;
        }
        sky.set_if_neq(now);
    }

    for (mut environment, ambient, exposure) in &mut environments {
        if let Some((&DayAmbient(day), ambient)) = ambient {
            let brightness = ambient.night + (day - ambient.night) * daylight;
            if environment.ambient.brightness != brightness {
                environment.ambient.brightness = brightness;
            }
        }
        if let Some((exposure, &NightExposure(night))) = exposure {
            let ev100 = night + (exposure.day - night) * daylight;
            if environment.post.exposure != ev100 {
                environment.post.exposure = ev100;
            }
        }
    }

    for (lantern, mut light) in &mut lanterns {
        light.intensity = lantern.intensity * (1.0 - daylight);
    }

    let Some((color, brightness)) = shining else {
        return;
    };
    for (lit, mut light) in &mut spots {
        light.intensity = lit.intensity * brightness;
        light.color = color;
    }
    for (lit, mut light) in &mut points {
        light.intensity = lit.intensity * brightness;
        light.color = color;
    }
}

fn through_air(height: f32) -> f32 {
    let degrees = height.clamp(0.0, 1.0).asin().to_degrees();
    let air = 1.0 / (height.max(0.0) + 0.50572 * (degrees + 6.07995).powf(-1.6364));
    (-ATMOSPHERE * (air - 1.0)).exp()
}

fn sun_color_of(look: Look, day: &DaySun) -> LinearRgba {
    match look {
        Look::Day => day.color,
        Look::Dawn => DAWN_SUN,
        Look::Dusk | Look::Night => DUSK_SUN,
    }
    .into()
}

fn sky_at(look: Look, day: &PixelSky) -> PixelSky {
    let tint = match look {
        Look::Day => return day.clone(),
        Look::Dawn => &DAWN_SKY,
        Look::Dusk => &DUSK_SKY,
        Look::Night => &NIGHT_SKY,
    };
    PixelSky {
        zenith: tint.zenith,
        middle: tint.middle,
        horizon: tint.horizon,
        ground: tint.ground,
        glow_color: tint.glow,
        sunset_color: tint.sunset,
        stars: tint.stars,
        ..day.clone()
    }
}

fn mix_skies(from: PixelSky, to: PixelSky, t: f32) -> PixelSky {
    let color = |a: Color, b: Color| -> Color { a.to_linear().mix(&b.to_linear(), t).into() };
    PixelSky {
        zenith: color(from.zenith, to.zenith),
        middle: color(from.middle, to.middle),
        horizon: color(from.horizon, to.horizon),
        ground: color(from.ground, to.ground),
        glow_color: color(from.glow_color, to.glow_color),
        sunset_color: color(from.sunset_color, to.sunset_color),
        stars: from.stars.lerp(to.stars, t),
        ..to
    }
}
