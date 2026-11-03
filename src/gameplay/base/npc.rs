use bevy::prelude::*;
use bevy_ahoy::{input::AccumulatedInput, prelude::AhoySystems};

use crate::{GameplayScene, PausableSystems, gameplay::base::indoor_body, loading::NPCAssets};

use super::anim::indoor_visual_bundle;

const NPC_SPEED: f32 = 1.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameplayScene::Base), spawn_npcs)
        .add_systems(
            FixedPostUpdate,
            steer_npcs
                .before(AhoySystems::MoveCharacters)
                .in_set(PausableSystems)
                .run_if(in_state(GameplayScene::Base)),
        );
}

/// Identity marker for non-player characters.
#[derive(Component)]
struct Npc;

/// Waypoints an [`Npc`] walks through in order, looping back to the start once
/// the last one is reached.
#[derive(Component)]
struct NpcWaypoints {
    points: Vec<Vec2>,
    next: usize,
    /// Stop requesting movement within this distance (metres) of the current
    /// waypoint. Smaller values make the NPC settle closer to the marker.
    arrival_threshold: f32,
}

fn spawn_npcs(
    assets: Res<NPCAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let points = vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(1.0, 0.0),
    ];

    spawn_npc(
        &assets,
        &mut texture_atlas_layouts,
        &mut commands,
        Vec3::new(1.0, 1.0, 0.0),
        points,
    );
}

fn spawn_npc(
    assets: &NPCAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    commands: &mut Commands,
    start: Vec3,
    points: Vec<Vec2>,
) {
    let (animation, sprite, sprite3d) = indoor_visual_bundle(assets, texture_atlas_layouts);
    let (collider, controller) = indoor_body(NPC_SPEED);

    commands
        .spawn((
            Npc,
            Transform::from_translation(start),
            collider,
            controller,
            NpcWaypoints {
                points,
                next: 0,
                arrival_threshold: 0.15,
            },
            DespawnOnExit(GameplayScene::Base),
        ))
        .insert(children![(
            Transform::IDENTITY,
            animation,
            sprite,
            sprite3d
        )]);
}

/// Steers each NPC toward its current waypoint by writing Ahoy's
/// `AccumulatedInput::last_movement`.
fn steer_npcs(mut npcs: Query<(&Transform, &mut NpcWaypoints, &mut AccumulatedInput)>) {
    for (transform, mut waypoints, mut input) in &mut npcs {
        let Some(target) = waypoints.points.get(waypoints.next).copied() else {
            input.last_movement = None;
            continue;
        };

        // Only horizontal movement is scripted.
        let position = Vec2::new(transform.translation.x, transform.translation.z);
        let to_target = target - position;

        if to_target.length_squared() <= waypoints.arrival_threshold.powi(2) {
            waypoints.next = (waypoints.next + 1) % waypoints.points.len();
            input.last_movement = None;
            continue;
        }

        let direction = to_target.normalize();
        input.last_movement = Some(Vec2::new(direction.x, -direction.y));
    }
}
