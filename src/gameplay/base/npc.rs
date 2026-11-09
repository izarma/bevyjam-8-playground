use avian3d::prelude::{
    Collider, ColliderAabb, CollisionLayers, LayerMask, PhysicsLayer, ShapeCastConfig,
    SpatialQuery, SpatialQueryFilter,
};
use bevy::prelude::*;
use bevy_ahoy::{input::AccumulatedInput, prelude::AhoySystems};
use bevy_yarnspinner::prelude::DialogueRunner;

use crate::{
    GameplayScene, PausableSystems,
    gameplay::{
        CharacterName, DialogueTarget,
        base::{BODY_HEIGHT, LabScene, ground_below, indoor_body},
        upgrades::Trader,
        visitors::{InLineup, Visitor},
    },
    loading::NPCAssets,
};

use super::{
    anim::{IndoorCharAnimation, indoor_visual_bundle},
    interact::InReach,
};

const NPC_SPEED: f32 = 4.0;
// epilon distance for target seeking so npcs dont inch towards their goal forever
const ARRIVAL: f32 = 0.15;
const PROBE: Vec3 = Vec3::new(0.6, 1.6, 0.1);
const PROBE_CLEARANCE: f32 = 0.15;
const DETOUR_PADDING: f32 = 0.6;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, sync_lineup.run_if(in_state(GameplayScene::Base)))
        .add_systems(
            FixedPostUpdate,
            steer_npcs
                .before(AhoySystems::MoveCharacters)
                .in_set(PausableSystems)
                .run_if(in_state(GameplayScene::Base)),
        )
        .add_systems(
            Update,
            end_npc_dialogue
                .in_set(PausableSystems)
                .run_if(in_state(GameplayScene::Base).and_then(any_with_component::<InDialogue>)),
        );
}

/// Marker for the NPC currently talking to the player.
#[derive(Component)]
pub(crate) struct InDialogue;

#[derive(Component)]
#[relationship(relationship_target = Body)]
#[require(Visibility)]
pub(crate) struct BodyOf(pub(crate) Entity);

#[derive(Component)]
#[relationship_target(relationship = BodyOf)]
pub(crate) struct Body(Entity);
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct NpcDoor;

// place where npc stands
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct NpcSlot(pub usize);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct TraderSpot;

// used to decide if npcs need to walk through the door (because we are inside) or not
#[derive(Component)]
pub(super) struct LineupShown;

#[derive(Component)]
struct NpcTarget(Vec2);

fn sync_lineup(
    mut commands: Commands,
    lab: Single<(Entity, Has<LineupShown>), With<LabScene>>,
    door: Single<&GlobalTransform, With<NpcDoor>>,
    slots: Query<(&NpcSlot, &GlobalTransform)>,
    trader_spot: Option<Single<&GlobalTransform, With<TraderSpot>>>,
    visitors: Query<(
        Entity,
        &Visitor,
        Option<&InLineup>,
        Has<Trader>,
        Option<&Body>,
    )>,
    mut bodies: Query<(&Transform, &mut NpcTarget, Has<InDialogue>)>,
    assets: Res<NPCAssets>,
) {
    let (lab, shown) = *lab;
    let door = door.translation();
    let trader_spot = trader_spot.map(|spot| spot.translation());
    debug_assert!(
        trader_spot.is_some(),
        "no TraderSpot in the lab for the trader to stand at"
    );
    for (entity, visitor, lineup, trader, body) in &visitors {
        let place = if trader {
            trader_spot
        } else {
            lineup.map(|lineup| lineup.0).and_then(|slot| {
                let place = slots
                    .iter()
                    .find(|(marker, _)| marker.0 == slot)
                    .map(|(_, at)| at.translation());
                debug_assert!(
                    place.is_some(),
                    "no NpcSlot({slot}) in the lab for {}",
                    visitor.name()
                );
                place
            })
        };
        match (place, body.map(|body| body.0)) {
            (Some(place), None) => {
                let start = if shown { door } else { place };
                let (image, frame) = assets.sprite(visitor.name());
                let (animation, sprite) = indoor_visual_bundle(image, frame);
                let (collider, mut controller) = indoor_body(NPC_SPEED);
                let sprite_at = ground_below(&controller);
                // NPCs pass through each other
                let others = !LayerMask::from(Layer::Npc);
                controller.filter.mask = others;
                commands.spawn((
                    BodyOf(entity),
                    CharacterName(visitor.name()),
                    Transform::from_translation(start + Vec3::Y * BODY_HEIGHT / 2.0),
                    NpcTarget(place.xz()),
                    collider,
                    CollisionLayers::new(Layer::Npc, others),
                    controller,
                    DespawnOnExit(GameplayScene::Base),
                    children![(Transform::from_xyz(0.0, sprite_at, 0.0), animation, sprite,)],
                ));
            }
            (Some(place), Some(body)) => {
                if let Ok((_, mut target, talking)) = bodies.get_mut(body)
                    && !(trader && talking)
                {
                    target.0 = place.xz();
                }
            }
            (None, Some(body)) => {
                let Ok((at, mut target, _)) = bodies.get_mut(body) else {
                    continue;
                };
                if at.translation.xz().distance_squared(door.xz()) <= ARRIVAL.powi(2) {
                    commands.entity(body).despawn();
                } else {
                    target.0 = door.xz();
                }
            }
            (None, None) => {}
        }
    }
    if !shown {
        commands.entity(lab).insert(LineupShown);
    }
}

#[derive(PhysicsLayer, Default)]
enum Layer {
    #[default]
    Default,
    Npc,
}

/// Steers each NPC toward its current waypoint by writing Ahoy's
/// `AccumulatedInput::last_movement`. NPCs stand still while their talk
/// prompt is shown and throughout a dialogue.
fn steer_npcs(
    mut commands: Commands,
    spatial: SpatialQuery,
    bounds: Query<&ColliderAabb>,
    mut npcs: Query<(
        Entity,
        &Transform,
        &NpcTarget,
        Option<&Avoiding>,
        &mut AccumulatedInput,
        &Children,
        Has<InReach>,
        Has<InDialogue>,
    )>,
    mut animations: Query<&mut IndoorCharAnimation>,
) {
    for (npc, transform, target, avoiding, mut input, children, in_reach, in_dialogue) in &mut npcs
    {
        input.last_movement = None;
        if in_reach || in_dialogue {
            continue;
        }
        let from = transform.translation.xz();
        let to_target = target.0 - from;
        if to_target.length_squared() <= ARRIVAL.powi(2) {
            for child in children {
                if let Ok(mut animation) = animations.get_mut(*child) {
                    animation.face_towards(Vec2::Y);
                }
            }
            continue;
        }
        let obstacle = avoiding
            .map(|avoiding| avoiding.0)
            .or_else(|| obstacle(&spatial, &bounds, npc, transform.translation, target.0));
        let toward = match obstacle {
            Some(rect) if crosses(from, target.0, rect) => {
                commands.entity(npc).insert(Avoiding(rect));
                around(
                    &spatial,
                    &bounds,
                    npc,
                    transform.translation,
                    target.0,
                    rect,
                )
            }
            _ => {
                if avoiding.is_some() {
                    commands.entity(npc).remove::<Avoiding>();
                }
                target.0
            }
        };
        let direction = (toward - from).normalize_or(to_target.normalize());
        input.last_movement = Some(Vec2::new(direction.x, -direction.y));
    }
}

fn end_npc_dialogue(
    mut commands: Commands,
    npcs: Query<Entity, With<InDialogue>>,
    runners: Query<&DialogueRunner>,
    mut target: ResMut<DialogueTarget>,
) {
    if runners.iter().any(|runner| runner.is_running()) {
        return;
    }
    target.0 = None;
    for npc in &npcs {
        commands.entity(npc).remove::<InDialogue>();
    }
}

#[derive(Component)]
struct Avoiding(Rect);

fn probe_hit(
    spatial: &SpatialQuery,
    npc: Entity,
    height: f32,
    from: Vec2,
    to: Vec2,
) -> Option<Entity> {
    let probe = Collider::cuboid(PROBE.x, PROBE.y, PROBE.z);
    let filter =
        SpatialQueryFilter::from_mask(!LayerMask::from(Layer::Npc)).with_excluded_entities([npc]);
    let path = to - from;
    let direction = Dir3::new(Vec3::new(path.x, 0.0, path.y)).ok()?;
    spatial
        .cast_shape(
            &probe,
            Vec3::new(from.x, height, from.y),
            Quat::IDENTITY,
            direction,
            &ShapeCastConfig {
                max_distance: path.length(),
                ignore_origin_penetration: true,
                ..default()
            },
            &filter,
        )
        .map(|hit| hit.entity)
}

fn probe_height(at: Vec3) -> f32 {
    at.y - BODY_HEIGHT / 2.0 + PROBE_CLEARANCE + PROBE.y / 2.0
}

fn obstacle(
    spatial: &SpatialQuery,
    bounds: &Query<&ColliderAabb>,
    npc: Entity,
    at: Vec3,
    target: Vec2,
) -> Option<Rect> {
    let hit = probe_hit(spatial, npc, probe_height(at), at.xz(), target)?;
    let aabb = bounds.get(hit).ok()?;
    let rect = Rect::from_corners(aabb.min.xz(), aabb.max.xz()).inflate(DETOUR_PADDING);
    (!rect.contains(target)).then_some(rect)
}

fn crosses(from: Vec2, to: Vec2, rect: Rect) -> bool {
    let inner = rect.inflate(-0.05);
    let path = to - from;
    let mut enter = 0.0f32;
    let mut leave = 1.0f32;
    for axis in 0..2 {
        let (start, step, low, high) = (from[axis], path[axis], inner.min[axis], inner.max[axis]);
        if step.abs() < 1e-6 {
            if start < low || start > high {
                return false;
            }
            continue;
        }
        let (a, b) = ((low - start) / step, (high - start) / step);
        enter = enter.max(a.min(b));
        leave = leave.min(a.max(b));
    }
    enter < leave
}

fn around(
    spatial: &SpatialQuery,
    bounds: &Query<&ColliderAabb>,
    npc: Entity,
    at: Vec3,
    target: Vec2,
    rect: Rect,
) -> Vec2 {
    let from = at.xz();
    let height = probe_height(at);
    let walkable = |a: Vec2, b: Vec2| probe_hit(spatial, npc, height, a, b).is_none();
    let only_this_in_way = |a: Vec2, b: Vec2| {
        probe_hit(spatial, npc, height, a, b).is_none_or(|hit| {
            bounds.get(hit).is_ok_and(|aabb| {
                let other =
                    Rect::from_corners(aabb.min.xz(), aabb.max.xz()).inflate(DETOUR_PADDING);
                other.min.distance(rect.min) < 0.01 && other.max.distance(rect.max) < 0.01
            })
        })
    };
    let corners = [
        rect.min,
        Vec2::new(rect.min.x, rect.max.y),
        Vec2::new(rect.max.x, rect.min.y),
        rect.max,
    ];
    let cost = |corner: &Vec2| from.distance(*corner) + corner.distance(target);
    corners
        .into_iter()
        .filter(|&corner| {
            corner.distance(from) > ARRIVAL
                && !crosses(from, corner, rect)
                && walkable(from, corner)
                && only_this_in_way(corner, target)
        })
        .min_by(|a, b| cost(a).total_cmp(&cost(b)))
        .unwrap_or(target)
}
