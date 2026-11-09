use bevy::{prelude::*, ui::UiSystems};
use bevy_enhanced_input::prelude::*;
use plants::Growing;

use super::{harvest::InReach, player::Harvest};
use crate::{
    GameplayScene,
    camera::SceneCamera,
    gameplay::{Player, clock::Clock, herbs, upgrades::Rewards},
    ui::{alerts::Announce, scale::TEXEL},
};

const ABOVE_HERB: f32 = 0.8;
const EDGE_MARGIN: f32 = 12.0;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(ping).add_observer(forget).add_systems(
        PostUpdate,
        place_markers
            .before(UiSystems::Layout)
            .run_if(in_state(GameplayScene::Outdoors)),
    );
}

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(super) struct Ping;

#[derive(Component)]
struct RadarMarker(Entity);

fn ping(
    _: On<Start<Ping>>,
    mut commands: Commands,
    mut rewards: ResMut<Rewards>,
    clock: Single<&Clock>,
    player: Option<Single<&GlobalTransform, With<Player>>>,
    plants: Query<(Entity, &Growing, &GlobalTransform)>,
    markers: Query<&RadarMarker>,
) {
    if !rewards.radar() {
        return;
    }
    if rewards.radar_charges == 0 {
        let wait = rewards.hours_to_charge(clock.hours + clock.bank);
        commands.trigger(Announce(format!(
            "{wait} hour{} until next charge",
            if wait == 1 { "" } else { "s" }
        )));
        return;
    }
    let Some(player) = player else {
        return;
    };
    let from = player.translation();
    let nearest = plants
        .iter()
        .filter(|(plant, ..)| markers.iter().all(|marker| marker.0 != *plant))
        .min_by(|a, b| {
            a.2.translation()
                .distance_squared(from)
                .total_cmp(&b.2.translation().distance_squared(from))
        });
    let Some((plant, &Growing(kind), _)) = nearest else {
        commands.trigger(Announce("The radar finds no herbs.".into()));
        return;
    };
    rewards.radar_charges -= 1;
    commands
        .spawn_scene(bsn! {
            Name::new("Radar Marker")
            Node {
                position_type: PositionType::Absolute,
            }
            GlobalZIndex(5)
            Visibility::Hidden
            Pickable::IGNORE
            DespawnOnExit::<GameplayScene>(GameplayScene::Outdoors)
            Children [ @herbs::icon(kind) ]
        })
        .insert(RadarMarker(plant));
    let left = rewards.radar_charges;
    commands.trigger(Announce(format!(
        "Radar: {left} charge{} left.",
        if left == 1 { "" } else { "s" }
    )));
}

fn forget(
    _: On<Start<Harvest>>,
    mut commands: Commands,
    markers: Query<(Entity, &RadarMarker)>,
    in_reach: Query<(), With<InReach>>,
) {
    for (marker, &RadarMarker(plant)) in &markers {
        if in_reach.contains(plant) {
            commands.entity(marker).despawn();
        }
    }
}

fn place_markers(
    mut commands: Commands,
    camera: Single<(&Camera, &Transform), SceneCamera>,
    plants: Query<&GlobalTransform, With<Growing>>,
    mut markers: Query<(Entity, &RadarMarker, &mut Node, &mut Visibility)>,
    ui_scale: Res<UiScale>,
) {
    let (camera, eye) = *camera;
    let eye = GlobalTransform::from(*eye);
    let Some(size) = camera.logical_viewport_size() else {
        return;
    };
    let half = 8.0 * TEXEL * ui_scale.0;
    let margin = EDGE_MARGIN * ui_scale.0 + half;
    for (marker, &RadarMarker(plant), mut node, mut visibility) in &mut markers {
        let Ok(at) = plants.get(plant) else {
            commands.entity(marker).despawn();
            continue;
        };
        let target = at.translation() + Vec3::Y * ABOVE_HERB;
        let toward = (target - eye.translation()).normalize_or_zero();
        let ahead = toward.dot(*eye.forward()) > 0.0;
        let screen = match camera.world_to_viewport(&eye, target) {
            Ok(point) if ahead => point,
            _ => {
                let side = eye.right().dot(toward);
                Vec2::new(if side < 0.0 { 0.0 } else { size.x }, size.y / 2.0)
            }
        };
        let clamped = screen.clamp(Vec2::splat(margin), size - Vec2::splat(margin));
        let left = px((clamped.x - half) / ui_scale.0);
        let top = px((clamped.y - half) / ui_scale.0);
        if node.left != left {
            node.left = left;
        }
        if node.top != top {
            node.top = top;
        }
        visibility.set_if_neq(Visibility::Inherited);
    }
}
