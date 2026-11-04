//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{GameResult, GameState};

pub(super) fn plugin(app: &mut App) {
    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );

    // Debug-only win/lose triggers
    app.add_systems(
        Update,
        win.run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F1))),
    );
    app.add_systems(
        Update,
        lose.run_if(in_state(GameState::Gameplay).and_then(input_just_pressed(KeyCode::F2))),
    );

    app.add_systems(
        Update,
        cycle_time_of_day.run_if(input_just_pressed(KeyCode::F3)),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<GlobalUiDebugOptions>) {
    println!("Debug UI toggled");
    options.toggle();
}

fn win(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    commands.insert_resource(GameResult::Won);
    next_state.set(GameState::GameOver);
}

fn lose(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    commands.insert_resource(GameResult::Lost);
    next_state.set(GameState::GameOver);
}

/// how far above the horizon the sun sits at dusk
const DUSK_ELEVATION: f32 = 8.0_f32.to_radians();
const DUSK_SUN: Color = Color::srgb(1.0, 0.55, 0.3);
// fog is a flat color, not lit by anything, so it has to follow the light
const DUSK_FOG: Color = Color::srgb(0.3, 0.22, 0.2);
const NIGHT_FOG: Color = Color::srgb(0.02, 0.03, 0.06);

#[derive(Default, Clone, Copy, Debug)]
enum TimeOfDay {
    #[default]
    Day,
    Dusk,
    Night,
}

/// a sun as it was during the day, kept while F3 has it at dusk or night
#[derive(Component)]
struct DaySun {
    rotation: Quat,
    color: Color,
    illuminance: f32,
}

/// a camera's fog color as it was during the day
#[derive(Component)]
struct DayFog(Color);

// F3 steps day -> dusk -> night -> day. night turns the sun off, leaving only
// ambient and point/spot lights
fn cycle_time_of_day(
    mut commands: Commands,
    mut time: Local<TimeOfDay>,
    mut suns: Query<(
        Entity,
        &mut Transform,
        &mut DirectionalLight,
        Option<&DaySun>,
    )>,
    mut fogs: Query<(Entity, &mut DistanceFog, Option<&DayFog>)>,
) {
    *time = match *time {
        TimeOfDay::Day => TimeOfDay::Dusk,
        TimeOfDay::Dusk => TimeOfDay::Night,
        TimeOfDay::Night => TimeOfDay::Day,
    };
    info!("F3: {:?}", *time);

    for (entity, mut transform, mut light, day) in &mut suns {
        let day = match day {
            Some(day) => DaySun { ..*day },
            None => {
                let day = DaySun {
                    rotation: transform.rotation,
                    color: light.color,
                    illuminance: light.illuminance,
                };
                commands.entity(entity).insert(DaySun { ..day });
                day
            }
        };
        match *time {
            TimeOfDay::Day => {
                transform.rotation = day.rotation;
                light.color = day.color;
                light.illuminance = day.illuminance;
                commands.entity(entity).remove::<DaySun>();
            }
            TimeOfDay::Dusk => {
                // same compass heading, sun low on the horizon
                let forward = day.rotation * Vec3::NEG_Z;
                let heading = Vec3::new(forward.x, 0.0, forward.z).normalize_or(Vec3::NEG_Z);
                transform.look_to(
                    heading * DUSK_ELEVATION.cos() - Vec3::Y * DUSK_ELEVATION.sin(),
                    Vec3::Y,
                );
                light.color = DUSK_SUN;
                light.illuminance = day.illuminance * 0.25;
            }
            TimeOfDay::Night => light.illuminance = 0.0,
        }
    }

    for (entity, mut fog, day) in &mut fogs {
        let day = match day {
            Some(DayFog(color)) => *color,
            None => {
                commands.entity(entity).insert(DayFog(fog.color));
                fog.color
            }
        };
        fog.color = match *time {
            TimeOfDay::Day => {
                commands.entity(entity).remove::<DayFog>();
                day
            }
            TimeOfDay::Dusk => DUSK_FOG,
            TimeOfDay::Night => NIGHT_FOG,
        };
    }
}
