use std::time::Duration;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Actions, ActiveInStates};
use bevy_sprite3d::Sprite3d;

use crate::{
    GameplayScene, Pause, camera::FaceCamera, gameplay::game_object::GameObject,
    loading::PlayerAssets,
};

mod animation;
mod input;

pub(crate) use animation::{PlayerAnimation, tick_player_animation};

#[derive(Component, Clone, Default)]
pub struct Player;

/// Interval between animation frames for every sprite animation.
pub(crate) const FRAME_INTERVAL: Duration = Duration::from_millis(125);

/// This plugin handles player related stuff like movement.
/// Player logic is only active in the `GameplayScene::Base` substate of `GameState::Gameplay`.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((input::plugin, animation::plugin))
        .init_resource::<PlayerAtlasLayouts>()
        .add_systems(OnEnter(GameplayScene::Base), spawn_player);
}

/// Cache of texture atlas layouts created at runtime.
#[derive(Resource, Default)]
struct PlayerAtlasLayouts {
    sheet: Option<Handle<TextureAtlasLayout>>,
}

fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut layouts: ResMut<PlayerAtlasLayouts>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = layouts
        .sheet
        .get_or_insert_with(|| {
            texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(32),
                35,
                7,
                None,
                None,
            ))
        })
        .clone();
    let spawn_translation = Vec3::new(0.0, 0.5, 7.0);
    let animation = PlayerAnimation::default();
    let sprite = Sprite {
        image: player_assets.player_sheet.clone(),
        texture_atlas: Some(TextureAtlas {
            layout,
            index: animation.frame_index(),
        }),
        ..default()
    };

    commands
        .spawn_scene(bsn! {
            #Player
            Player
            ~ActiveInStates::<Player, Pause>::single(Pause(false))
            Transform::from_translation(spawn_translation)
            animation
            FaceCamera
            GameObject
        })
        // `Sprite` and `Sprite3d` can't be part of a BSN scene
        // so both are inserted directly.
        .insert((
            sprite,
            Sprite3d {
                pixels_per_metre: 32.0,
                double_sided: true,
                ..default()
            },
        ))
        // Attach the input actions as related entities, declared in BSN (see `player_input_scenes`).
        .queue_spawn_related_scenes::<Actions<Player>>(input::player_input_scenes());
}
