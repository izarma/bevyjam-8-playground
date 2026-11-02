// usually you set Environment in jackdaw to change ambient / global camera settings
// for ones it doesnt support like dof and contact shadows we are doing it here

use bevy::{
    pbr::ContactShadows,
    post_process::dof::{DepthOfField, DepthOfFieldMode},
    prelude::*,
};
use bevy_ahoy::prelude::CharacterController;

use crate::{GameplayScene, camera::SceneCamera};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (add_camera_effects, focus_on_player)
            .chain()
            .run_if(in_state(GameplayScene::Outdoors)),
    );
}

fn add_camera_effects(
    mut commands: Commands,
    cameras: Query<(Entity, Has<DepthOfField>, Has<ContactShadows>), SceneCamera>,
) {
    for (camera, has_depth_of_field, has_contact_shadows) in &cameras {
        if !has_depth_of_field {
            commands.entity(camera).insert(DepthOfField {
                mode: DepthOfFieldMode::Bokeh,
                // this setting is what gives it the "tilt shift" effect
                aperture_f_stops: 0.6,
                ..default()
            });
        }
        if !has_contact_shadows {
            commands.entity(camera).insert(ContactShadows::default());
        }
    }
}

fn focus_on_player(
    mut cameras: Query<(&GlobalTransform, &mut DepthOfField), SceneCamera>,
    player: Single<&GlobalTransform, With<CharacterController>>,
) {
    for (camera, mut depth_of_field) in &mut cameras {
        depth_of_field.focal_distance = camera.translation().distance(player.translation());
    }
}
