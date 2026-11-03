// usually you set Environment in jackdaw to change ambient / global camera settings
// for ones it doesnt support like dof, contact shadows and shadow cascades we are
// doing it here

use bevy::{
    light::CascadeShadowConfigBuilder,
    pbr::ContactShadows,
    post_process::dof::{DepthOfField, DepthOfFieldMode},
    prelude::*,
};
use bevy_ahoy::prelude::CharacterController;

use crate::{
    GameplayScene, camera::SceneCamera, gameplay::player::Player, graphics::GraphicsSettings,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (add_camera_effects, focus_on_player, fit_sun_cascades)
            .chain()
            .run_if(in_state(GameplayScene::Outdoors)),
    );
}

fn add_camera_effects(
    mut commands: Commands,
    settings: Res<GraphicsSettings>,
    cameras: Query<(Entity, Has<DepthOfField>, Has<ContactShadows>), SceneCamera>,
) {
    for (camera, has_depth_of_field, has_contact_shadows) in &cameras {
        if has_depth_of_field && !settings.depth_of_field {
            commands.entity(camera).remove::<DepthOfField>();
        }
        if !has_depth_of_field && settings.depth_of_field {
            commands.entity(camera).insert(DepthOfField {
                mode: DepthOfFieldMode::Gaussian,
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
    player: Single<&GlobalTransform, (With<CharacterController>, With<Player>)>,
) {
    for (camera, mut depth_of_field) in &mut cameras {
        depth_of_field.focal_distance = camera.translation().distance(player.translation());
    }
}

// performance
// the defaults reach 150 m in 4 cascades; the whole map is 64 m across so we reduce it here
// could be a perf setting later?
fn fit_sun_cascades(mut commands: Commands, suns: Query<Entity, Added<DirectionalLight>>) {
    for sun in &suns {
        commands.entity(sun).insert(
            CascadeShadowConfigBuilder {
                num_cascades: 2,
                first_cascade_far_bound: 15.0,
                maximum_distance: 50.0,
                ..default()
            }
            .build(),
        );
    }
}
