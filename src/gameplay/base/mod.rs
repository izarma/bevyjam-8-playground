use avian3d::collision::collider::Collider;
use bevy::prelude::*;
use bevy_ahoy::CharacterController;
use jackdaw_runtime::prelude::*;

use crate::GameplayScene;

mod anim;
mod npc;
mod player_indoor;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((anim::plugin, npc::plugin, player_indoor::plugin));
    app.add_systems(OnEnter(GameplayScene::Base), spawn_lab_scene);
}

fn spawn_lab_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        JackdawSceneRoot(asset_server.load("lab_scene.bsn")),
        DespawnOnExit(GameplayScene::Base),
    ));
}
const CAPSULE_RADIUS: f32 = 0.25;
pub(super) const CAPSULE_HEIGHT: f32 = 2.0;
const STEP_SIZE: f32 = 0.15;

/// The collider and character controller shared by every indoor character.
pub(super) fn indoor_body(speed: f32) -> (Collider, CharacterController) {
    (
        Collider::capsule(CAPSULE_RADIUS, CAPSULE_HEIGHT - 2.0 * CAPSULE_RADIUS),
        CharacterController {
            speed,
            step_size: STEP_SIZE,
            ..default()
        },
    )
}
