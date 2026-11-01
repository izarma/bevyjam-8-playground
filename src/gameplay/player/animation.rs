use std::time::Duration;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::Action;

use crate::{GameState, PausableSystems, camera::face_camera};

use super::{
    FRAME_INTERVAL, Player,
    controller::{Grounded, Velocity},
    input::Run,
};

/// Squared horizontal speed below which the character counts as idle: (0.1 m/s)².
const MOVING_SPEED_SQ: f32 = 0.01;

/// Animation state for the player character.
///
/// Drives the 35x7 character sheet (`assets/textures/character/README.md`):
/// Rows are animation states; each directional group has the state's frame count.
#[derive(Component, Debug, Clone)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: PlayerAnimationState,
    /// World-space heading (XZ plane) the character last moved along.
    heading: Vec2,
    facing: Facing,
    flip: bool,
}

#[derive(Reflect, PartialEq, Eq, Clone, Copy, Debug, Default)]
enum PlayerAnimationState {
    Rotate,
    #[default]
    Idle,
    Walk,
    Run,
    Interact,
    Jump,
    Attack,
}

impl PlayerAnimationState {
    /// Sprite sheet row for this animation state (0-based).
    fn row(self) -> usize {
        match self {
            PlayerAnimationState::Rotate => 0,
            PlayerAnimationState::Idle => 1,
            PlayerAnimationState::Walk => 2,
            PlayerAnimationState::Run => 3,
            PlayerAnimationState::Interact => 4,
            PlayerAnimationState::Jump => 5,
            PlayerAnimationState::Attack => 6,
        }
    }

    /// Number of frames in this animation state.
    fn frame_count(self) -> usize {
        match self {
            PlayerAnimationState::Rotate => 8,
            PlayerAnimationState::Idle => 4,
            PlayerAnimationState::Walk => 4,
            PlayerAnimationState::Run => 6,
            PlayerAnimationState::Interact => 4,
            PlayerAnimationState::Jump => 5,
            PlayerAnimationState::Attack => 7,
        }
    }
}

/// Which of the sheet's 5 facing groups the sprite shows.
///
/// The sheet only draws one side view ("Side", facing screen-right); the
/// left-facing variants are the same frames mirrored (see `apply_player_animation`).
/// Facing is expressed relative to the camera, not the world.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
enum Facing {
    Down,
    DownSide,
    Side,
    UpSide,
    #[default]
    Up,
}

impl Facing {
    /// Sprite sheet columns per row.
    const COLUMNS: usize = 35;

    /// Directional group index, independent of the animation's frame count.
    fn group_index(self) -> usize {
        match self {
            Facing::Down => 0,
            Facing::DownSide => 1,
            Facing::Side => 2,
            Facing::UpSide => 3,
            Facing::Up => 4,
        }
    }

    /// Pick the facing group for a direction given as `(right, away)` components
    /// relative to the camera yaw (`away` = in the direction the camera looks).
    /// Returns `(facing, mirrored)`.
    ///
    /// ```text
    ///       away
    ///      3  2  1
    ///     4   ·   0     → right
    ///      5  6  7
    ///       toward
    /// ```
    /// Left-hand octants (3, 4, 5) reuse the right-hand art, mirrored.
    fn from_camera_relative(right: f32, away: f32) -> (Self, bool) {
        const TABLE: [(Facing, bool); 8] = [
            (Facing::Side, false),
            (Facing::UpSide, false),
            (Facing::Up, false),
            (Facing::UpSide, true),
            (Facing::Side, true),
            (Facing::DownSide, true),
            (Facing::Down, false),
            (Facing::DownSide, false),
        ];
        // Angle in the camera-relative plane: 0 = right, >0 = away from the camera.
        let octant = (away.atan2(right) / (std::f32::consts::TAU / 8.0)).round() as i32;
        TABLE[octant.rem_euclid(8) as usize]
    }
}

impl Default for PlayerAnimation {
    fn default() -> Self {
        Self {
            timer: Timer::new(FRAME_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Idle,
            heading: Vec2::new(0.0, -1.0),
            facing: Facing::default(),
            flip: false,
        }
    }
}

impl PlayerAnimation {
    /// Current atlas index for the animation.
    pub(crate) fn frame_index(&self) -> usize {
        let frame_count = self.state.frame_count();
        // Rotate is a single eight-frame sequence, not five directional groups.
        let start_column = if self.state == PlayerAnimationState::Rotate {
            0
        } else {
            self.facing.group_index() * frame_count
        };
        self.state.row() * Facing::COLUMNS + start_column + (self.frame % frame_count)
    }

    /// Whether the animation advanced to a footfall frame of the walk/run
    /// cycle this tick. Used to synchronize step sounds with the animation.
    pub fn step_frame(&self) -> bool {
        let footfall = match self.state {
            PlayerAnimationState::Walk => self.frame == 1 || self.frame == 3,
            PlayerAnimationState::Run => self.frame == 1 || self.frame == 4,
            _ => return false,
        };
        self.timer.just_finished() && footfall
    }

    /// Advance the animation timer.
    fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if self.timer.just_finished() {
            let count = self.state.frame_count();
            self.frame = if self.state == PlayerAnimationState::Jump {
                // Jump is a one-shot: the airtime is longer than the animation,
                // so hold the last frame until landing instead of wrapping back
                // to the takeoff pose.
                (self.frame + 1).min(count - 1)
            } else {
                (self.frame + 1) % count
            };
        }
    }

    /// Update state and facing from the world-space velocity, camera yaw,
    /// whether the player is running, and whether the player is on the ground.
    pub(crate) fn update(
        &mut self,
        velocity: Vec3,
        camera_rotation: Quat,
        running: bool,
        grounded: bool,
    ) {
        let previous_state = self.state;
        if !grounded {
            self.state = PlayerAnimationState::Jump;
        } else {
            // Test horizontal speed only; `velocity.y` is zeroed by ground
            // contact in the physics, which is a hidden dependency we don't
            // want to rely on here.
            let horizontal = Vec2::new(velocity.x, velocity.z);
            if horizontal.length_squared() < MOVING_SPEED_SQ {
                self.state = PlayerAnimationState::Idle;
            } else {
                self.state = if running {
                    PlayerAnimationState::Run
                } else {
                    PlayerAnimationState::Walk
                };
                self.heading = horizontal.normalize();
            }
        }

        // Decompose the stored world-space heading into camera-relative
        // components, since the billboarded sprite's "screen space" is defined
        // by the camera's yaw.
        let (yaw, _, _) = camera_rotation.to_euler(EulerRot::YXZ);
        let rotation = Quat::from_rotation_y(yaw);
        let forward = rotation * Vec3::NEG_Z;
        let right = rotation * Vec3::X;
        let (facing, flip) = Facing::from_camera_relative(
            self.heading.dot(right.xz()),
            self.heading.dot(forward.xz()),
        );
        self.facing = facing;
        self.flip = flip;

        if self.state != previous_state {
            self.frame = 0;
            self.timer.reset();
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            tick_player_animation,
            update_player_animation_state,
            apply_player_animation.after(face_camera),
        )
            .chain()
            .in_set(PausableSystems)
            .run_if(in_state(GameState::Gameplay)),
    );
}

pub(crate) fn tick_player_animation(time: Res<Time>, mut query: Query<&mut PlayerAnimation>) {
    for mut animation in &mut query {
        animation.tick(time.delta());
    }
}

pub(crate) fn update_player_animation_state(
    camera: Single<&Transform, With<Camera>>,
    run: Query<&Action<Run>>,
    mut query: Query<(&Velocity, &Grounded, &mut PlayerAnimation), With<Player>>,
) {
    let running = run.iter().next().map(|run| **run).unwrap_or(false);
    for (velocity, grounded, mut animation) in &mut query {
        animation.update(**velocity, camera.rotation, running, grounded.0);
    }
}

pub(crate) fn apply_player_animation(
    mut query: Query<(&PlayerAnimation, &mut Sprite, &mut Transform)>,
) {
    for (animation, mut sprite, mut transform) in &mut query {
        let index = animation.frame_index();
        // Read before writing: calling `as_mut()` on `Mut<Sprite>` would mark the
        // sprite as changed every frame, waking bevy_sprite3d's `Changed<Sprite>`
        // systems even when the index didn't actually change.
        if sprite
            .texture_atlas
            .as_ref()
            .is_some_and(|atlas| atlas.index != index)
            && let Some(atlas) = sprite.texture_atlas.as_mut()
        {
            atlas.index = index;
        }
        // Mirror the texture for left facings by showing the quad's double-sided
        // back face.
        if animation.flip {
            transform.rotate_y(std::f32::consts::PI);
        }
    }
}
