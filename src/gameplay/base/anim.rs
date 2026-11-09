use std::time::Duration;

use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_ahoy::prelude::CharacterController;
use bevy_yarnspinner::prelude::DialogueRunner;
use pixel_art::PixelSprite;

use super::npc::InDialogue;
use crate::{
    GameplayScene,
    gameplay::{
        SpeakerSprite,
        animation::{FRAME_INTERVAL, MOVING_SPEED_SQ, PlayerAnimationSystems},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            select_animation_state
                .in_set(PlayerAnimationSystems::UpdateState)
                .run_if(in_state(GameplayScene::Base)),
            animate
                .in_set(PlayerAnimationSystems::Apply)
                .run_if(in_state(GameplayScene::Base)),
        ),
    );
}

#[derive(Component, Debug, Clone)]
#[require(SpeakerSprite)]
pub(super) struct IndoorCharAnimation {
    timer: Timer,
    frame: usize,
    state: IndoorCharAnimationState,
    facing: IndoorCharFacing,
    flip: bool,
}

#[derive(Reflect, PartialEq, Eq, Clone, Copy, Debug, Default)]
pub(super) enum IndoorCharAnimationState {
    #[default]
    Idle,
    Walk,
}

#[derive(Reflect, PartialEq, Eq, Clone, Copy, Debug, Default)]
pub(super) enum IndoorCharFacing {
    #[default]
    Front,
    Back,
    Side,
}

impl Default for IndoorCharAnimation {
    fn default() -> Self {
        Self {
            timer: Timer::new(FRAME_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: IndoorCharAnimationState::default(),
            facing: IndoorCharFacing::default(),
            flip: false,
        }
    }
}

impl IndoorCharAnimation {
    fn frames(&self) -> &'static [usize] {
        match (self.facing, self.state) {
            // Love you cat.
            (IndoorCharFacing::Front, IndoorCharAnimationState::Idle) => &[0, 1],
            (IndoorCharFacing::Front, IndoorCharAnimationState::Walk) => &[6, 7],
            (IndoorCharFacing::Back, IndoorCharAnimationState::Walk) => &[8, 9],
            (IndoorCharFacing::Back, IndoorCharAnimationState::Idle) => &[2, 3],
            (IndoorCharFacing::Side, IndoorCharAnimationState::Idle) => &[4, 5],
            (IndoorCharFacing::Side, IndoorCharAnimationState::Walk) => &[10, 11],
        }
    }

    fn frame_index(&self) -> usize {
        let frames = self.frames();
        frames[self.frame % frames.len()]
    }

    fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if self.timer.just_finished() {
            self.frame = (self.frame + 1) % self.frames().len();
        }
    }

    pub(super) fn flipped(&self) -> bool {
        self.flip
    }

    fn update_state(&mut self, velocity: Vec3) {
        let horizontal = Vec2::new(velocity.x, velocity.z);
        let next = if horizontal.length_squared() < MOVING_SPEED_SQ {
            IndoorCharAnimationState::Idle
        } else {
            let (facing, flip) = if horizontal.y.abs() >= horizontal.x.abs() {
                (
                    if horizontal.y >= 0.0 {
                        IndoorCharFacing::Front
                    } else {
                        IndoorCharFacing::Back
                    },
                    false,
                )
            } else {
                (IndoorCharFacing::Side, horizontal.x < 0.0)
            };
            self.facing = facing;
            self.flip = flip;
            IndoorCharAnimationState::Walk
        };
        if next != self.state {
            self.state = next;
            self.frame = 0;
            self.timer.reset();
        }
    }

    /// Point the sprite toward a world-space direction (XZ plane), using the
    /// same facing rules as movement.
    pub(super) fn face_towards(&mut self, direction: Vec2) {
        if direction.length_squared() <= 0.0 {
            return;
        }
        let direction = direction.normalize();
        if direction.y.abs() >= direction.x.abs() {
            self.facing = if direction.y >= 0.0 {
                IndoorCharFacing::Front
            } else {
                IndoorCharFacing::Back
            };
            self.flip = false;
        } else {
            self.facing = IndoorCharFacing::Side;
            self.flip = direction.x < 0.0;
        }
    }
}

/// Selects animation state and facing from the owning character's velocity.
/// While a dialogue is running, the speaking NPC is held in its idle pose.
fn select_animation_state(
    roots: Query<(&LinearVelocity, &Children, Has<InDialogue>), With<CharacterController>>,
    mut animations: Query<&mut IndoorCharAnimation>,
    runners: Query<&DialogueRunner>,
) {
    let dialogue_active = runners.iter().any(|runner| runner.is_running());
    for (velocity, children, in_dialogue) in &roots {
        let velocity = if dialogue_active && in_dialogue {
            Vec3::ZERO
        } else {
            **velocity
        };
        for child in children {
            let Ok(mut animation) = animations.get_mut(*child) else {
                continue;
            };
            animation.update_state(velocity);
        }
    }
}

/// The visual components shared by every indoor character
pub(super) fn indoor_visual_bundle(
    image: &Handle<Image>,
    frame: UVec2,
) -> (IndoorCharAnimation, PixelSprite) {
    let animation = IndoorCharAnimation::default();
    let sprite = PixelSprite {
        image: image.clone(),
        size: Some(frame),
        cell: animation.frame_index() as u32,
        ..default()
    };
    (animation, sprite)
}

fn animate(time: Res<Time>, mut query: Query<(&mut IndoorCharAnimation, &mut PixelSprite)>) {
    for (mut animation, mut sprite) in &mut query {
        animation.tick(time.delta());
        let cell = animation.frame_index() as u32;
        let flip_x = animation.flipped();
        if sprite.cell != cell || sprite.flip_x != flip_x {
            sprite.cell = cell;
            sprite.flip_x = flip_x;
        }
    }
}
