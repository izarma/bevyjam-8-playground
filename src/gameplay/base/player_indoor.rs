use bevy::prelude::*;

use crate::{GameplayScene, gameplay::player::Player, loading::NPCAssets};

use super::anim::spawn_base_sprite;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameplayScene::Base), spawn_player_base);
}

fn spawn_player_base(
    assets: Res<NPCAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    spawn_base_sprite(
        &mut commands,
        &assets,
        &mut texture_atlas_layouts,
        Player,
        Vec3::new(0.0, 0.5, 0.0),
    );
}
