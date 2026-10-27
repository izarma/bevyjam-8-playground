//! https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs
//!
//! Using a fixed timestep: means that we advance the physics simulation by a fixed amount
//! at a time. If the real time that passed between two frames is less than the fixed timestep, we simply
//! don't advance the physics simulation at all.
//! If it is more, we advance the physics simulation multiple times until we catch up.
//! You can read more about how Bevy implements this in the documentation for
//! [`bevy::time::Fixed`](https://docs.rs/bevy/latest/bevy/time/struct.Fixed.html).
//!
//! This leaves us with a last problem, however. If our physics simulation may advance zero or multiple times
//! per frame, there may be frames in which the player's position did not need to be updated at all,
//! and some where it is updated by a large amount that resulted from running the physics simulation multiple times.
//! This is physically correct, but visually jarring. Imagine a player moving in a straight line, but depending on the frame rate,
//! they may sometimes advance by a large amount and sometimes not at all. Visually, we want the player to move smoothly.
//! This is why we need to separate the player's position in the physics simulation from the player's position in the visual representation.
//! The visual representation can then be interpolated smoothly based on the previous and current actual player position in the physics simulation.
//!
//! This is a tradeoff: every visual frame is now slightly lagging behind the actual physical frame,
//! but in return, the player's movement will appear smooth.
//! There are other ways to compute the visual representation of the player, such as extrapolation.
//! See the [documentation of the lightyear crate](https://cbournhonesque.github.io/lightyear/book/concepts/advanced_replication/visual_interpolation.html)
//! for a nice overview of the different methods and their respective tradeoffs.
//!
//! If we decide to use a fixed timestep, our game logic should mostly go in the `FixedUpdate` schedule.
//! One notable exception is the camera. Cameras should update as often as possible, or the player will very quickly
//! notice choppy movement if it's only updated at the same rate as the physics simulation. So, we use a variable timestep for the camera,
//! updating its transform every frame. The question now is which schedule to use. That depends on whether the camera data is required
//! for the physics simulation to run or not.
//! For example, in 3D games, the camera rotation often determines which direction the player moves when pressing "W",
//! so we need to rotate the camera *before* the fixed timestep. In contrast, the translation of the camera depends on what the physics simulation
//! has calculated for the player's position. Therefore, we need to update the camera's translation *after* the fixed timestep. Fortunately,
//! we can get smooth movement by simply using the interpolated player translation for the camera as well.

use bevy::prelude::*;

use crate::{
    GameState,
    engine::{PausableSystems, Pause, camera::rotate_camera},
};

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<DidFixedTimestepRunThisFrame>();
    // `FixedUpdate` and `RunFixedMainLoop` run regardless of our game states,
    // so gate them on `Pause` in addition to `GameState` below.
    app.configure_sets(FixedUpdate, PausableSystems.run_if(in_state(Pause(false))))
        .configure_sets(
            RunFixedMainLoop,
            PausableSystems.run_if(in_state(Pause(false))),
        );
    // At the beginning of each frame, clear the flag that indicates whether the fixed timestep has run this frame.
    app.add_systems(PreUpdate, clear_fixed_timestep_flag)
        // At the beginning of each fixed timestep, set the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(FixedPreUpdate, set_fixed_time_step_flag)
        // Advance the physics simulation using a fixed timestep.
        .add_systems(
            FixedUpdate,
            advance_physics
                .in_set(PausableSystems)
                .run_if(in_state(GameState::Gameplay)),
        );
    app.add_systems(
        // The `RunFixedMainLoop` schedule allows us to schedule systems to run before and after the fixed timestep loop.
        RunFixedMainLoop,
        (
            // Accumulate our input before the fixed timestep loop to tell the physics simulation what it should do during the fixed timestep.
            // It reads the camera's rotation, so it must run after `camera::rotate_camera`
            // (which also runs before the fixed timestep loop).
            accumulate_input
                .after(rotate_camera)
                .in_set(PausableSystems)
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            (
                // Clear our accumulated input after it was processed during the fixed timestep.
                // By clearing the input *after* the fixed timestep, we can still use `AccumulatedInput` inside `FixedUpdate` if we need it.
                clear_input.run_if(did_fixed_timestep_run_this_frame),
                // The player's visual representation needs to be updated after the physics simulation has been advanced.
                // This could be run in `Update`, but if we run it here instead, the systems in `Update`
                // will be working with the `Transform` that will actually be shown on screen.
                // The camera's position is then synced with the interpolated transform in `Update`
                // by `camera::follow_target`.
                interpolate_rendered_transform,
            )
                .chain()
                .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
        ),
    );
}

/// A simple resource that tells us whether the fixed timestep ran this frame.
#[derive(Resource, Debug, Deref, DerefMut, Default)]
pub(crate) struct DidFixedTimestepRunThisFrame(bool);

/// A vector representing the player's input, accumulated over all frames that ran
/// since the last time the physics simulation was advanced.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub(crate) struct AccumulatedInput {
    /// The direction the character wants to move in on the XZ plane.
    /// `x` maps to world X, `y` maps to world Z.
    movement: Vec2,
    // Other input that could make sense would be e.g.
    // boost: bool
}

/// A vector representing the player's velocity in the physics simulation.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub(crate) struct Velocity(Vec3);

/// The actual position of the player in the physics simulation.
/// This is separate from the `Transform`, which is merely a visual representation.
///
/// If you want to make sure that this component is always initialized
/// with the same value as the `Transform`'s translation, you can
/// use a [component lifecycle hook](https://docs.rs/bevy/latest/bevy/ecs/component/struct.ComponentHooks.html)
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub(crate) struct PhysicalTranslation(pub(crate) Vec3);

/// The value [`PhysicalTranslation`] had in the last fixed timestep.
/// Used for interpolation in the `interpolate_rendered_transform` system.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub(crate) struct PreviousPhysicalTranslation(pub(crate) Vec3);

/// Reset the flag at the start of every frame.
fn clear_fixed_timestep_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = false;
}

/// Set the flag during each fixed timestep.
fn set_fixed_time_step_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = true;
}

fn did_fixed_timestep_run_this_frame(
    did_fixed_timestep_run_this_frame: Res<DidFixedTimestepRunThisFrame>,
) -> bool {
    did_fixed_timestep_run_this_frame.0
}

// Clear the input after it was processed in the fixed timestep.
fn clear_input(mut input: Single<&mut AccumulatedInput>) {
    **input = AccumulatedInput::default();
}

/// Advance the physics simulation by one fixed timestep. This may run zero or multiple times per frame.
///
/// Note that since this runs in `FixedUpdate`, `Res<Time>` would be `Res<Time<Fixed>>` automatically.
/// We are being explicit here for clarity.
fn advance_physics(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut PhysicalTranslation,
        &mut PreviousPhysicalTranslation,
        &Velocity,
    )>,
) {
    for (mut current_physical_translation, mut previous_physical_translation, velocity) in
        query.iter_mut()
    {
        previous_physical_translation.0 = current_physical_translation.0;
        current_physical_translation.0 += velocity.0 * fixed_time.delta_secs();
    }
}

/// Handle keyboard input and accumulate it in the `AccumulatedInput` component.
///
/// There are many strategies for how to handle all the input that happened since the last fixed timestep.
/// This is a very simple one: we just use the last available input.
/// That strategy works fine for us since the user continuously presses the input keys in this example.
/// If we had some kind of instantaneous action like activating a boost ability, we would need to remember that that input
/// was pressed at some point since the last fixed timestep.
fn accumulate_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut AccumulatedInput, &mut Velocity)>,
    camera: Single<&Transform, With<Camera>>,
) {
    /// Since Bevy's 3D renderer assumes SI units, this has the unit of meters per second.
    /// Note that about 1.5 is the average walking speed of a human.
    const SPEED: f32 = 4.0;
    let (mut input, mut velocity) = player.into_inner();
    // Reset the input to zero before reading the new input. As mentioned above, we can only do this
    // because this is continuously pressed by the user. Do not reset e.g. whether the user wants to boost.
    input.movement = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        input.movement.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        input.movement.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.movement.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        input.movement.x += 1.0;
    }

    // Remap the 2D input to Bevy's 3D coordinate system.
    // Pressing W makes `input.y` go up. Since Bevy assumes that -Z is forward, we make our new Z equal to -input.y
    let input_3d = Vec3 {
        x: input.movement.x,
        y: 0.0,
        z: -input.movement.y,
    };

    // The character is grounded (XZ-plane movement only), so rotate the input by the camera's
    // yaw only. Using the full rotation would inject a Y component into the velocity whenever
    // the camera is pitched, pushing the character off the ground plane.
    let (yaw, _, _) = camera.rotation.to_euler(EulerRot::YXZ);
    let rotated_input = Quat::from_rotation_y(yaw) * input_3d;

    // We need to normalize and scale because otherwise
    // diagonal movement would be faster than horizontal or vertical movement.
    // We use `clamp_length_max` instead of `.normalize_or_zero()` because gamepad input
    // may be smaller than 1.0 when the player is pushing the stick just a little bit.
    velocity.0 = rotated_input.clamp_length_max(1.0) * SPEED;
}

fn interpolate_rendered_transform(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut Transform,
        &PhysicalTranslation,
        &PreviousPhysicalTranslation,
    )>,
) {
    for (mut transform, current_physical_translation, previous_physical_translation) in
        query.iter_mut()
    {
        let previous = previous_physical_translation.0;
        let current = current_physical_translation.0;
        // The overstep fraction is a value between 0 and 1 that tells us how far we are between two fixed timesteps.
        let alpha = fixed_time.overstep_fraction();

        let rendered_translation = previous.lerp(current, alpha);
        transform.translation = rendered_translation;
    }
}
