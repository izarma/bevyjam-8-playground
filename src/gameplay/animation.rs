use std::time::Duration;

use bevy::prelude::*;

use crate::{GameplayScene, PausableSystems};

/// Interval between animation frames for every sprite animation.
pub(crate) const FRAME_INTERVAL: Duration = Duration::from_millis(250);

/// Squared horizontal speed below which the character counts as idle: (0.1 m/s)².
pub(crate) const MOVING_SPEED_SQ: f32 = 0.01;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        animate_sprites
            .in_set(PausableSystems)
            .run_if(in_state(GameplayScene::Base)),
    );
}

#[derive(Component)]
pub(crate) struct Animation {
    pub frames: Vec<usize>, // indices of all the frames in the animation
    pub current: usize,
    pub timer: Timer,
}

fn animate_sprites(time: Res<Time>, mut query: Query<(&mut Animation, &mut Sprite)>) {
    for (mut animation, mut sprite) in query.iter_mut() {
        animation.timer.tick(time.delta());
        if animation.timer.just_finished() {
            let atlas = sprite.texture_atlas.as_mut().unwrap();
            atlas.index = animation.frames[animation.current];
            animation.current += 1;
            animation.current %= animation.frames.len();
        }
    }
}
