use bevy::prelude::*;

use crate::{
    GameState, PausableSystems,
    ui::{
        font,
        scale::{TEXEL, texels},
    },
};

const START_HOUR: f32 = 6.0;
// for clock progress smoothing
// how many seconds to spend 2/3 of the time bank in
const SPEND_SECONDS: f32 = 0.5;
const MINUTE: f32 = 1.0 / 60.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_clock)
        .add_systems(
            Update,
            (spend_bank.in_set(PausableSystems), show_time)
                .chain()
                .run_if(in_state(GameState::Gameplay)),
        );
}

#[derive(Component, Default, Clone)]
pub(crate) struct Clock {
    pub(crate) hours: f32,
    pub(crate) bank: f32,
}

fn spawn_clock(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #Clock
        Clock { hours: START_HOUR }
        Text("")
        @font::text(1)
        TextColor(Color::WHITE)
        TextShadow {
            offset: Vec2::splat(TEXEL),
            color: Color::BLACK,
        }
        Node {
            position_type: PositionType::Absolute,
            right: texels(4),
            bottom: texels(4),
        }
        Pickable::IGNORE
        DespawnOnExit<GameState>(GameState::Gameplay)
    });
}

fn spend_bank(time: Res<Time>, mut clocks: Query<&mut Clock>) {
    for mut clock in &mut clocks {
        if clock.bank == 0.0 {
            continue;
        }
        let mut spent = clock.bank * (1.0 - (-time.delta_secs() / SPEND_SECONDS).exp());
        if clock.bank - spent < MINUTE {
            spent = clock.bank;
        }
        clock.bank -= spent;
        clock.hours += spent;
    }
}

fn show_time(mut clocks: Query<(&Clock, &mut Text), Changed<Clock>>) {
    for (clock, mut text) in &mut clocks {
        let minutes = (clock.hours * 60.0) as u32 % (24 * 60);
        let (hour, minute) = (minutes / 60, minutes % 60);
        let half = if hour < 12 { "am" } else { "pm" };
        let shown = match hour % 12 {
            0 => 12,
            hour => hour,
        };
        text.0 = format!("{shown}:{minute:02} {half}");
    }
}
