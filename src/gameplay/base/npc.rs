use bevy::prelude::*;

use crate::{GameplayScene, loading::NPCAssets};

use super::anim::spawn_base_sprite;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameplayScene::Base), spawn_npc);
}

#[derive(Component)]
struct Npc;

fn spawn_npc(
    assets: Res<NPCAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    spawn_base_sprite(
        &mut commands,
        &assets,
        &mut texture_atlas_layouts,
        Npc,
        Vec3::new(1.0, 0.5, 0.0),
    );
}
