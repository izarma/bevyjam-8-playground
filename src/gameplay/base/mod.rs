use avian3d::collision::collider::Collider;
use bevy::prelude::*;
use bevy_ahoy::CharacterController;
use jackdaw_runtime::prelude::*;

use crate::GameplayScene;

mod anim;
mod interact;
mod npc;
mod player_indoor;

pub(crate) use npc::{BodyOf, InDialogue};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        anim::plugin,
        interact::plugin,
        npc::plugin,
        player_indoor::plugin,
    ));
    app.add_systems(OnEnter(GameplayScene::Base), spawn_lab_scene);
}

fn spawn_lab_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        LabScene,
        JackdawSceneRoot(asset_server.load("lab_scene.bsn")),
        DespawnOnExit(GameplayScene::Base),
    ));
}

#[derive(Component)]
struct LabScene;

const BODY_WIDTH_X: f32 = 0.8;
pub(super) const BODY_HEIGHT: f32 = 2.0;
const BODY_DEPTH_Z: f32 = 0.3;
const STEP_SIZE: f32 = 0.15;

/// The collider and character controller shared by every indoor character.
pub(super) fn indoor_body(speed: f32) -> (Collider, CharacterController) {
    (
        Collider::round_cuboid(BODY_WIDTH_X, BODY_HEIGHT, BODY_DEPTH_Z, BODY_DEPTH_Z / 2.0),
        CharacterController {
            speed,
            step_size: STEP_SIZE,
            ..default()
        },
    )
}
