use crate::GameState;
use crate::engine::camera::{CameraTarget, FaceCamera};
use crate::engine::char_controller::{
    AccumulatedInput, PhysicalTranslation, PreviousPhysicalTranslation, Velocity,
};
use crate::engine::loading::PlayerAssets;
use crate::gameplay::game_object::GameObject;
use bevy::prelude::*;
use bevy_sprite3d::Sprite3d;

#[derive(Component)]
pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Gameplay`

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_player);
}

fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        35,
        7,
        None,
        None,
    ));
    let spawn_translation = Vec3::new(0.0, 0.5, 0.0);

    commands.spawn((
        Name::new("Player"),
        Player,
        Sprite3d {
            pixels_per_metre: 32.0,
            double_sided: true,
            ..default()
        },
        Sprite {
            image: player_assets.player_sheet.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: layout.clone(),
                index: 0, //animation.frame_index(),
            }),
            ..default()
        },
        Transform::from_translation(spawn_translation),
        Velocity::default(),
        // The physical translation must match the spawn translation, since
        // `interpolate_rendered_transform` overwrites the `Transform` from these every frame.
        // Half the sprite height (32px @ 32px/m = 1m tall, pivot at center) keeps feet on the ground.
        PhysicalTranslation(spawn_translation),
        PreviousPhysicalTranslation(spawn_translation),
        AccumulatedInput::default(),
        CameraTarget,
        FaceCamera,
        GameObject,
    ));
}
