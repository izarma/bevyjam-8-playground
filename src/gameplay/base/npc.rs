use avian3d::prelude::{CollisionLayers, LayerMask, PhysicsLayer};
use bevy::prelude::*;
use bevy_ahoy::{input::AccumulatedInput, prelude::AhoySystems};
use bevy_yarnspinner::prelude::DialogueRunner;

use crate::{
    GameplayScene, PausableSystems,
    gameplay::{
        CharacterName, DialogueTarget,
        base::{BODY_HEIGHT, LabScene, ground_below, indoor_body},
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
    visitors: Query<(Entity, &Visitor, Option<&InLineup>, Option<&Body>)>,
    mut bodies: Query<(&Transform, &mut NpcTarget)>,
    assets: Res<NPCAssets>,
) {
    let (lab, shown) = *lab;
    let door = door.translation();
    for (entity, visitor, lineup, body) in &visitors {
        let place = lineup.map(|lineup| lineup.0).and_then(|slot| {
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
        });
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
                if let Ok((_, mut target)) = bodies.get_mut(body) {
                    target.0 = place.xz();
                }
            }
            (None, Some(body)) => {
                let Ok((at, mut target)) = bodies.get_mut(body) else {
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
    mut npcs: Query<(
        &Transform,
        &NpcTarget,
        &mut AccumulatedInput,
        &Children,
        Has<InReach>,
        Has<InDialogue>,
    )>,
    mut animations: Query<&mut IndoorCharAnimation>,
) {
    for (transform, target, mut input, children, in_reach, in_dialogue) in &mut npcs {
        input.last_movement = None;
        if in_reach || in_dialogue {
            continue;
        }
        let to_target = target.0 - transform.translation.xz();
        if to_target.length_squared() <= ARRIVAL.powi(2) {
            for child in children {
                if let Ok(mut animation) = animations.get_mut(*child) {
                    animation.face_towards(Vec2::Y);
                }
            }
            continue;
        }
        let direction = to_target.normalize();
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
