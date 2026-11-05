use std::time::Duration;

use bevy::prelude::*;
use pixel_art::PixelSprite;

use crate::{GameState, PausableSystems, loading::PLAYER_SHEET_COLUMNS};

/// Interval between animation frames for every sprite animation.
pub(crate) const FRAME_INTERVAL: Duration = Duration::from_millis(250);

/// Squared horizontal speed below which the character counts as idle: (0.1 m/s)².
pub(crate) const MOVING_SPEED_SQ: f32 = 0.01;

/// Ordered sets of the character-animation pipeline, shared by every scene.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PlayerAnimationSystems {
    Tick,
    /// Update animation state from the character's movement.
    UpdateState,
    /// Write the current frame to the character sprites.
    Apply,
}

pub(super) fn plugin(app: &mut App) {
    app.configure_sets(
        Update,
        (
            PlayerAnimationSystems::Tick,
            PlayerAnimationSystems::UpdateState,
            PlayerAnimationSystems::Apply,
        )
            .chain()
            .in_set(PausableSystems)
            .run_if(in_state(GameState::Gameplay)),
    )
    .add_systems(
        Update,
        tick_player_animation.in_set(PlayerAnimationSystems::Tick),
    )
    .add_systems(
        Update,
        apply_player_animation.in_set(PlayerAnimationSystems::Apply),
    );
}

/// Animation state for the player character.
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
    #[default]
    Idle,
    Walk,
    Jump,
}

impl PlayerAnimationState {
    /// First column of this state's frames.
    fn start_column(self) -> usize {
        match self {
            PlayerAnimationState::Idle => 0,
            PlayerAnimationState::Walk => 4,
            PlayerAnimationState::Jump => 8,
        }
    }

    /// Number of frames in this animation state.
    fn frame_count(self) -> usize {
        match self {
            PlayerAnimationState::Idle => 4,
            PlayerAnimationState::Walk => 4,
            PlayerAnimationState::Jump => 5,
        }
    }
}

/// Facing is expressed relative to the camera, not the world.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
enum Facing {
    #[default]
    Front,
    Back,
    Side,
}

impl Facing {
    /// Sprite sheet columns per row.
    const COLUMNS: usize = PLAYER_SHEET_COLUMNS as usize;

    /// Sprite sheet row for this facing (0-based).
    fn row(self) -> usize {
        match self {
            Facing::Front => 0,
            Facing::Back => 1,
            Facing::Side => 2,
        }
    }

    /// Pick the facing group for a direction given as `(right, away)` components
    /// relative to the camera yaw (`away` = in the direction the camera looks).
    /// Returns `(facing, mirrored)`.
    fn from_camera_relative(right: f32, away: f32) -> (Self, bool) {
        if away.abs() >= right.abs() {
            (
                if away >= 0.0 {
                    Facing::Back
                } else {
                    Facing::Front
                },
                false,
            )
        } else {
            (Facing::Side, right < 0.0)
        }
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
    fn flipped(&self) -> bool {
        self.flip
    }

    /// Current sheet cell for the animation.
    pub(crate) fn frame_index(&self) -> usize {
        let start_column = self.state.start_column();
        self.facing.row() * Facing::COLUMNS + start_column + (self.frame % self.state.frame_count())
    }

    /// Whether the animation advanced to a footfall frame of the walk cycle
    /// this tick. Used to synchronize step sounds with the animation.
    pub fn step_frame(&self) -> bool {
        self.timer.just_finished()
            && self.state == PlayerAnimationState::Walk
            && (self.frame == 1 || self.frame == 3)
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

    /// Update state and facing from the world-space velocity, camera rotation,
    /// and whether the player is on the ground.
    pub(crate) fn update(&mut self, velocity: Vec3, camera_rotation: Quat, grounded: bool) {
        let previous_state = self.state;
        if !grounded {
            self.state = PlayerAnimationState::Jump;
        } else {
            let horizontal = Vec2::new(velocity.x, velocity.z);
            if horizontal.length_squared() < MOVING_SPEED_SQ {
                self.state = PlayerAnimationState::Idle;
            } else {
                self.state = PlayerAnimationState::Walk;
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

    /// Point the sprite toward a world-space direction (XZ plane).
    pub(crate) fn face_towards(&mut self, direction: Vec2) {
        if direction.length_squared() > 0.0 {
            self.heading = direction.normalize();
        }
    }
}

pub(crate) fn tick_player_animation(time: Res<Time>, mut query: Query<&mut PlayerAnimation>) {
    for mut animation in &mut query {
        animation.tick(time.delta());
    }
}

pub(crate) fn apply_player_animation(mut sprites: Query<(&PlayerAnimation, &mut PixelSprite)>) {
    for (animation, mut sprite) in &mut sprites {
        let cell = animation.frame_index() as u32;
        let flip_x = animation.flipped();

        if sprite.cell != cell || sprite.flip_x != flip_x {
            sprite.cell = cell;
            sprite.flip_x = flip_x;
        }
    }
}
