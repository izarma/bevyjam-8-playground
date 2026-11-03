use std::f32::consts::PI;
use std::time::Duration;

use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_ahoy::prelude::CharacterController;
use bevy_sprite3d::Sprite3d;

use crate::{
    GameplayScene, PausableSystems,
    gameplay::animation::{FRAME_INTERVAL, MOVING_SPEED_SQ},
    loading::NPCAssets,
};
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (select_animation_state, animate)
            .chain()
            .in_set(PausableSystems)
            .run_if(in_state(GameplayScene::Base)),
    );
}

#[derive(Component, Debug, Clone)]
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
    Sit,
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
            // Fuck you cat.
            (IndoorCharFacing::Front, IndoorCharAnimationState::Idle) => &[0, 1, 2, 3],
            (IndoorCharFacing::Front, IndoorCharAnimationState::Walk) => &[4, 5],
            (IndoorCharFacing::Front, IndoorCharAnimationState::Sit) => &[6],
            (IndoorCharFacing::Back, IndoorCharAnimationState::Sit) => &[7],
            (IndoorCharFacing::Back, IndoorCharAnimationState::Walk) => &[8, 9],
            (IndoorCharFacing::Back, IndoorCharAnimationState::Idle) => &[10, 11, 12, 13],
            (IndoorCharFacing::Side, IndoorCharAnimationState::Idle) => &[14, 15, 16, 17],
            (IndoorCharFacing::Side, IndoorCharAnimationState::Walk) => &[18, 19],
            (IndoorCharFacing::Side, IndoorCharAnimationState::Sit) => &[20],
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
        if self.state == IndoorCharAnimationState::Sit {
            return;
        }
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
}

/// Selects animation state and facing from the owning character's velocity.
fn select_animation_state(
    roots: Query<(&LinearVelocity, &Children), With<CharacterController>>,
    mut animations: Query<&mut IndoorCharAnimation>,
) {
    for (velocity, children) in &roots {
        for child in children {
            let Ok(mut animation) = animations.get_mut(*child) else {
                continue;
            };
            animation.update_state(**velocity);
        }
    }
}

/// The visual components shared by every indoor character
pub(super) fn indoor_visual_bundle(
    assets: &NPCAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> (IndoorCharAnimation, Sprite, Sprite3d) {
    let animation = IndoorCharAnimation::default();
    let sprite = Sprite {
        image: assets.fox.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(32),
                7,
                3,
                None,
                None,
            )),
            index: animation.frame_index(),
        }),
        ..default()
    };
    let sprite3d = Sprite3d {
        pixels_per_metre: pixel_art::TEXELS_PER_METER,
        double_sided: true,
        ..default()
    };
    (animation, sprite, sprite3d)
}

fn animate(
    time: Res<Time>,
    mut query: Query<(&mut IndoorCharAnimation, &mut Sprite, &mut Transform)>,
) {
    for (mut animation, mut sprite, mut transform) in &mut query {
        animation.tick(time.delta());
        let index = animation.frame_index();
        if sprite
            .texture_atlas
            .as_ref()
            .is_some_and(|atlas| atlas.index != index)
            && let Some(atlas) = sprite.texture_atlas.as_mut()
        {
            atlas.index = index;
        }
        let rotation = Quat::from_rotation_y(if animation.flipped() { PI } else { 0.0 });
        if transform.rotation != rotation {
            transform.rotation = rotation;
        }
    }
}
