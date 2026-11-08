use bevy::prelude::*;

const CANDLE_COLOR: Color = Color::srgb(1.0, 0.72, 0.4);
const CANDLE_INTENSITY: f32 = 600_000.0;
// point lights are expensive so make them tiny
// this is still expensive tbh
const CANDLE_RANGE: f32 = 3.0;
const FLICKER: f32 = 0.15;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(light_candle).add_systems(Update, flicker);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Visibility)]
pub struct Candle {
    pub flame: Vec3,
}

#[derive(Component)]
struct Flicker {
    phase: f32,
}

fn light_candle(add: On<Add<Candle>>, candles: Query<&Candle>, mut commands: Commands) {
    let Ok(candle) = candles.get(add.entity) else {
        return;
    };
    commands.spawn((
        Name::new("Candle Light"),
        PointLight {
            color: CANDLE_COLOR,
            intensity: CANDLE_INTENSITY,
            range: CANDLE_RANGE,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(candle.flame),
        Flicker {
            phase: rand::random_range(0.0..std::f32::consts::TAU),
        },
        ChildOf(add.entity),
    ));
}

fn flicker(time: Res<Time>, mut lights: Query<(&Flicker, &mut PointLight)>) {
    let t = time.elapsed_secs();
    for (flicker, mut light) in &mut lights {
        let p = flicker.phase;
        let wobble = 0.5 * (t * 7.3 + p).sin()
            + 0.3 * (t * 13.1 + p * 1.7).sin()
            + 0.2 * (t * 23.7 + p * 2.3).sin();
        light.intensity = CANDLE_INTENSITY * (1.0 + FLICKER * wobble);
    }
}
