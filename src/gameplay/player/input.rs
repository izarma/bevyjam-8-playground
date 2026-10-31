use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::Pause;

use super::{Player, controller::AccumulatedInput};

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub(crate) struct Move;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(crate) struct Run;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(crate) struct Jump;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(crate) struct Interact;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(crate) struct Attack;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<Player>()
        // Keep the `Player` context active only while the `Pause` substate of `GameState::Gameplay`
        // is `Pause(false)` (the player can act).
        .sync_context_to_state::<Player, Pause>()
        .add_observer(on_jump_start)
        .add_observer(on_interact_start)
        .add_observer(on_attack_start);
}

/// The player's action bindings, spawned as [`Actions`] related to the player entity.
pub(crate) fn player_input_scenes() -> impl SceneList {
    bsn_list! {
        Action::<Move>::new()
        DeadZone
        Bindings [
            {preset::cardinal::wasd_keys()}
            --
            {preset::axial::left_stick()}
        ]
        --
        Action::<Run>::new()
        Bindings [
            Binding(KeyCode::ShiftLeft)
            --
            Binding(KeyCode::ShiftRight)
            --
            Binding(GamepadButton::LeftThumb)
        ]
        --
        Action::<Jump>::new()
        Press
        Bindings [
            Binding(KeyCode::Space)
            --
            Binding(GamepadButton::South)
        ]
        --
        Action::<Interact>::new()
        Press
        Bindings [
            Binding(KeyCode::KeyE)
            --
            Binding(GamepadButton::East)
        ]
        --
        Action::<Attack>::new()
        Press
        Bindings [
            Binding(KeyCode::Enter)
            --
            Binding(GamepadButton::West)
        ]
    }
}

fn on_jump_start(start: On<Start<Jump>>, mut inputs: Query<&mut AccumulatedInput, With<Player>>) {
    if let Ok(mut input) = inputs.get_mut(start.context) {
        input.jump = true;
    }
}

fn on_interact_start(
    start: On<Start<Interact>>,
    mut inputs: Query<&mut AccumulatedInput, With<Player>>,
) {
    if let Ok(mut input) = inputs.get_mut(start.context) {
        input.interact = true;
    }
}

fn on_attack_start(
    start: On<Start<Attack>>,
    mut inputs: Query<&mut AccumulatedInput, With<Player>>,
) {
    if let Ok(mut input) = inputs.get_mut(start.context) {
        input.attack = true;
    }
}
