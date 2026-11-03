use std::time::Duration;

use bevy::prelude::*;
use bevy_sprite3d::Sprite3d;

use crate::{GameplayScene, PausableSystems, gameplay::player::FRAME_INTERVAL, loading::NPCAssets};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        animate
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
}

pub(super) fn spawn_base_sprite(
    commands: &mut Commands,
    assets: &NPCAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    marker: impl Bundle,
    spawn: Vec3,
) {
    let animation = IndoorCharAnimation::default();
    commands.spawn((
        marker,
        animation.clone(),
        Transform::from_translation(spawn),
        Sprite {
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
        },
        Sprite3d {
            pixels_per_metre: pixel_art::TEXELS_PER_METER,
            double_sided: true,
            ..default()
        },
        DespawnOnExit(GameplayScene::Base),
    ));
}

fn animate(time: Res<Time>, mut query: Query<(&mut IndoorCharAnimation, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut query {
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
    }
}
