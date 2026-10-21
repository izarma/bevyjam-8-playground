use bevy::prelude::*;

use crate::GameState;
use crate::engine::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        animate_sprites
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(GameState::Gameplay)),
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
