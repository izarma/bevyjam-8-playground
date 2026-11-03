use bevy::prelude::*;
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    GameplayScene, PausableSystems, Pause,
    camera::SceneCamera,
    gameplay::{
        base::{BODY_HEIGHT, indoor_body},
        player::Player,
    },
    loading::NPCAssets,
};

use super::anim::indoor_visual_bundle;

const WALK_SPEED: f32 = 5.0;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct IndoorPlayerSpawn;

#[derive(Component, Clone, Default)]
struct IndoorPlayer;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_player_base)
        .add_input_context::<IndoorPlayer>()
        .sync_context_to_state::<IndoorPlayer, Pause>()
        .add_systems(
            Update,
            (sync_camera_x, exit_to_outdoors)
                .in_set(PausableSystems)
                .run_if(in_state(GameplayScene::Base)),
        );
}

const CAM_X_MIN: f32 = -15.0;
const CAM_X_MAX: f32 = 15.0;

/// Past this X coordinate the player leaves the base through the right exit.
const EXIT_X: f32 = 20.0;

fn sync_camera_x(
    player: Query<&Transform, (With<IndoorPlayer>, Without<Camera3d>)>,
    mut cameras: Query<&mut Transform, SceneCamera>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let x = player.translation.x.clamp(CAM_X_MIN, CAM_X_MAX);
    for mut camera in &mut cameras {
        camera.translation.x = x;
    }
}

fn exit_to_outdoors(
    player: Query<&Transform, With<IndoorPlayer>>,
    mut next_scene: ResMut<NextState<GameplayScene>>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    if player.translation.x >= EXIT_X {
        next_scene.set(GameplayScene::Outdoors);
    }
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
    let spawn = feet.translation() + Vec3::Y * BODY_HEIGHT / 2.0;

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
