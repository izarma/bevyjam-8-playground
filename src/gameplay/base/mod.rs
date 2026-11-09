use avian3d::collision::collider::Collider;
use bevy::prelude::*;
use bevy_ahoy::CharacterController;
use jackdaw_runtime::prelude::*;

use crate::GameplayScene;

mod anim;
mod interact;
mod npc;
mod player_indoor;

pub(crate) use npc::{BodyOf, NpcDoor, NpcSlot};

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
    // the fake-ish outdoors that we show through the windows
    // there are cheater/cheap ways to do this but this is fast
    commands.spawn((
        JackdawSceneRoot(asset_server.load("lab_outside.bsn")),
        DespawnOnExit(GameplayScene::Base),
    ));
}

#[derive(Component)]
struct LabScene;

pub(crate) fn npc_idle_sprite(image: &Handle<Image>, frame: UVec2) -> pixel_art::PixelSprite {
    anim::indoor_visual_bundle(image, frame).1
}

const BODY_WIDTH_X: f32 = 0.8;
pub(super) const BODY_HEIGHT: f32 = 2.0;
const BODY_DEPTH_Z: f32 = 0.15;
const STEP_SIZE: f32 = 0.15;

const BODY_ROUNDING: f32 = BODY_DEPTH_Z / 2.0;

// indoor body is thinner to help with npc collision
pub(super) fn indoor_body(speed: f32) -> (Collider, CharacterController) {
    (
        Collider::round_cuboid(
            BODY_WIDTH_X - 2.0 * BODY_ROUNDING,
            BODY_HEIGHT - 2.0 * BODY_ROUNDING,
            BODY_DEPTH_Z - 2.0 * BODY_ROUNDING,
            BODY_ROUNDING,
        ),
        CharacterController {
            speed,
            step_size: STEP_SIZE,
            ..default()
        },
    )
}

pub(super) fn ground_below(controller: &CharacterController) -> f32 {
    -BODY_HEIGHT / 2.0 - controller.move_and_slide.skin_width
}
