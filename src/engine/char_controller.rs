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
use bevy_enhanced_input::prelude::Action;

use crate::{
    GameState,
    engine::{PausableSystems, Pause, camera::rotate_camera},
    gameplay::input::{Move, Run},
};

pub(crate) fn plugin(app: &mut App) {
    // `FixedUpdate` and `RunFixedMainLoop` run regardless of our game states,
    // so gate them on `Pause` in addition to `GameState` below.
    app.configure_sets(FixedUpdate, PausableSystems.run_if(in_state(Pause(false))))
        .configure_sets(
            RunFixedMainLoop,
            PausableSystems.run_if(in_state(Pause(false))),
        );
    // Advance the physics simulation using a fixed timestep.
    app.add_systems(
        FixedUpdate,
        advance_physics
            .in_set(PausableSystems)
            .run_if(in_state(GameState::Gameplay)),
    );
    // Drop pending one-shot requests when pausing so they don't fire on resume.
    // (`Pause` is a substate of `GameState::Gameplay`; exiting gameplay despawns
    // the player, which clears its `AccumulatedInput` along with it.)
    app.add_systems(OnEnter(Pause(true)), clear_pending_requests);
    app.add_systems(
        // The `RunFixedMainLoop` schedule allows us to schedule systems to run before and after the fixed timestep loop.
        RunFixedMainLoop,
        (
            // Read held movement input before the fixed timestep loop to tell the physics simulation what it should do during the fixed timestep.
            // It reads the camera's rotation, so it must run after `camera::rotate_camera`
            // (which also runs before the fixed timestep loop).
            apply_movement_input
                .after(rotate_camera)
                .in_set(PausableSystems)
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop)
                .run_if(in_state(GameState::Gameplay)),
            // The player's visual representation needs to be updated after the physics simulation has been advanced.
            // This could be run in `Update`, but if we run it here instead, the systems in `Update`
            // will be working with the `Transform` that will actually be shown on screen.
            // The camera's position is then synced with the interpolated transform in `Update`
            // by `camera::follow_target`.
            interpolate_rendered_transform
                .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop)
                .in_set(PausableSystems),
        ),
    );
}

/// One-shot input requests buffered by action observers in `PreUpdate`.
///
/// A press during a frame with zero fixed ticks stays buffered until a tick
/// consumes it, and a request consumed by one tick is not visible to later
/// catch-up ticks in the same frame. `jump` is consumed by `advance_physics`;
/// `interact` and `attack` are buffered here but not yet consumed.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
pub(crate) struct AccumulatedInput {
    pub(crate) jump: bool,
    pub(crate) interact: bool,
    pub(crate) attack: bool,
}

/// A vector representing the player's velocity in the physics simulation.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub(crate) struct Velocity(Vec3);

/// Whether the player is standing on the ground, as opposed to airborne.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub(crate) struct Grounded(pub(crate) bool);

/// Gravity acceleration in m/s² (SI units, negative Y is down).
const GRAVITY: f32 = 20.0;
/// Upward velocity imparted by a jump in m/s.
const JUMP_VELOCITY: f32 = 7.0;
/// Height of the ground plane in the physics simulation.
const GROUND_LEVEL: f32 = 0.0;
/// Y translation at which the player's feet touch the ground plane.
/// The player sprite is 1 m tall with its pivot at the center.
const GROUNDED_TRANSLATION_Y: f32 = GROUND_LEVEL + 0.5;

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

fn clear_pending_requests(mut query: Query<&mut AccumulatedInput>) {
    for mut input in &mut query {
        *input = AccumulatedInput::default();
    }
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
        &mut Velocity,
        &mut Grounded,
        &mut AccumulatedInput,
    )>,
) {
    for (
        mut current_physical_translation,
        mut previous_physical_translation,
        mut velocity,
        mut grounded,
        mut input,
    ) in query.iter_mut()
    {
        // Consume the jump request in this tick so it is not visible to
        // subsequent catch-up ticks in the same frame.
        let jump = input.jump;
        input.jump = false;

        // Jump only from the ground
        if jump && grounded.0 {
            velocity.y = JUMP_VELOCITY;
            grounded.0 = false;
        }

        // Gravity pulls the player down every tick, even while grounded.
        velocity.y -= GRAVITY * fixed_time.delta_secs();

        previous_physical_translation.0 = current_physical_translation.0;
        current_physical_translation.0 += velocity.0 * fixed_time.delta_secs();

        // Check for ground collision.
        if current_physical_translation.y <= GROUNDED_TRANSLATION_Y {
            current_physical_translation.y = GROUNDED_TRANSLATION_Y;
            velocity.y = velocity.y.max(0.0);
            grounded.0 = true;
        }
    }
}

/// Convert held movement/run input into the player's horizontal velocity.
fn apply_movement_input(
    movement: Query<&Action<Move>>,
    run: Query<&Action<Run>>,
    player: Single<(&mut Velocity, &Grounded)>,
    camera: Single<&Transform, With<Camera>>,
) {
    /// Since Bevy's 3D renderer assumes SI units, this has the unit of meters per second.
    /// Note that about 1.5 is the average walking speed of a human.
    const WALK_SPEED: f32 = 4.0;
    const RUN_SPEED: f32 = 7.0;

    let (Some(movement), Some(run)) = (movement.iter().next(), run.iter().next()) else {
        return;
    };
    let input: Vec2 = **movement;
    let running: bool = **run;

    // Remap the 2D input to Bevy's 3D coordinate system.
    // `input.y` is forward (from the cardinal preset), and Bevy's -Z is forward.
    let input_3d = Vec3 {
        x: input.x,
        y: 0.0,
        z: -input.y,
    };

    // The character is grounded (XZ-plane movement only), so rotate the input by the camera's
    // yaw only.
    let (yaw, _, _) = camera.rotation.to_euler(EulerRot::YXZ);
    let rotated_input = Quat::from_rotation_y(yaw) * input_3d;

    let speed = if running { RUN_SPEED } else { WALK_SPEED };
    let (mut velocity, grounded) = player.into_inner();
    if grounded.0 {
        let scaled_input = rotated_input.clamp_length_max(1.0) * speed;
        velocity.x = scaled_input.x;
        velocity.z = scaled_input.z;
    }
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
