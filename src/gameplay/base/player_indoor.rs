use bevy::prelude::*;
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    GameplayScene, Pause,
    gameplay::{
        base::{CAPSULE_HEIGHT, indoor_body},
        player::Player,
    },
    loading::NPCAssets,
};

use super::anim::indoor_visual_bundle;

const WALK_SPEED: f32 = 2.5;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct IndoorPlayerSpawn;

#[derive(Component, Clone, Default)]
struct IndoorPlayer;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_player_base)
        .add_input_context::<IndoorPlayer>()
        .sync_context_to_state::<IndoorPlayer, Pause>();
}

fn spawn_player_base(
    spawn: On<Insert<IndoorPlayerSpawn>>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    assets: Res<NPCAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let Ok(feet) = transforms.get(spawn.entity) else {
        return;
    };
    let spawn = feet.translation() + Vec3::Y * CAPSULE_HEIGHT / 2.0;

    let (animation, sprite, sprite3d) = indoor_visual_bundle(&assets, &mut texture_atlas_layouts);
    let (collider, controller) = indoor_body(WALK_SPEED);

    commands
        .spawn_scene(bsn! {
            IndoorPlayer
            Player
            Transform::from_translation(spawn)
            collider
            controller
            ~ActiveInStates::<IndoorPlayer, Pause>::single(Pause(false))
            DespawnOnExit::<GameplayScene>(GameplayScene::Base)
        })
        .insert(children![(
            Transform::IDENTITY,
            animation,
            sprite,
            sprite3d,
        )])
        .queue_spawn_related_scenes::<Actions<IndoorPlayer>>(bsn_list! {
            Action::<Movement>::new()
            DeadZone
            Bindings [
                {preset::cardinal::wasd_keys()}
                --
                {preset::axial::left_stick()}
            ]
        });
}
