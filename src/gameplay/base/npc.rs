use bevy::prelude::*;
use bevy_ahoy::{input::AccumulatedInput, prelude::AhoySystems};
use bevy_yarnspinner::prelude::DialogueRunner;

use crate::{
    GameplayScene, PausableSystems,
    gameplay::{CharacterName, base::indoor_body, herbs::Element},
    loading::NPCAssets,
};

use super::{anim::indoor_visual_bundle, interact::InReach};

const NPC_SPEED: f32 = 4.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameplayScene::Base), spawn_npcs)
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

/// Identity marker for non-player characters.
#[derive(Component)]
pub(super) struct Npc;

/// Marker for the NPC currently talking to the player.
#[derive(Component)]
pub(crate) struct InDialogue;

#[derive(Component, Clone, Copy)]
pub(super) struct TalkTarget(pub(super) &'static str);

const NPC_TALK_TARGETS: [TalkTarget; 5] = [
    TalkTarget("Monk"),
    TalkTarget("Woman"),
    TalkTarget("OldMan"),
    TalkTarget("OldWoman"),
    TalkTarget("Kid"),
];

pub(crate) const BALANCED: u8 = 5;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Health {
    wood: u8,
    fire: u8,
    earth: u8,
    metal: u8,
    water: u8,
}

impl Health {
    pub(crate) fn get(self, element: Element) -> u8 {
        match element {
            Element::Wood => self.wood,
            Element::Fire => self.fire,
            Element::Earth => self.earth,
            Element::Metal => self.metal,
            Element::Water => self.water,
        }
    }
    pub(crate) fn offset(&mut self, delta: i8) {
        let values = [
            &mut self.wood,
            &mut self.fire,
            &mut self.earth,
            &mut self.metal,
            &mut self.water,
        ];
        for value in values {
            *value = (*value as i16 + i16::from(delta)).clamp(1, 10) as u8;
        }
    }

    #[allow(dead_code)]
    pub(crate) fn offset_elements(&mut self, deltas: &[(Element, i8)]) {
        for &(element, delta) in deltas {
            let value = match element {
                Element::Wood => &mut self.wood,
                Element::Fire => &mut self.fire,
                Element::Earth => &mut self.earth,
                Element::Metal => &mut self.metal,
                Element::Water => &mut self.water,
            };
            *value = (*value as i16 + i16::from(delta)).clamp(1, 10) as u8;
        }
    }
}

impl Default for Health {
    fn default() -> Self {
        Self {
            wood: BALANCED,
            fire: BALANCED,
            earth: BALANCED,
            metal: BALANCED,
            water: BALANCED,
        }
    }
}

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
    let base_points = [
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(1.0, 0.0),
    ];

    for (index, x_shift) in [0.0, 2.0, 4.0, 8.0, 16.0].into_iter().enumerate() {
        let points = base_points
            .iter()
            .map(|p| Vec2::new(p.x + x_shift, p.y))
            .collect();

        spawn_npc(
            &assets,
            &mut texture_atlas_layouts,
            &mut commands,
            Vec3::new(1.0 + x_shift, 2.0, 0.0),
            points,
            NPC_TALK_TARGETS[index],
        );
    }
}

fn spawn_npc(
    assets: &NPCAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    commands: &mut Commands,
    start: Vec3,
    points: Vec<Vec2>,
    talk: TalkTarget,
) {
    let (animation, sprite, sprite3d) = indoor_visual_bundle(assets, texture_atlas_layouts);
    let (collider, controller) = indoor_body(NPC_SPEED);

    commands
        .spawn((
            Npc,
            talk,
            Health::default(),
            CharacterName(talk.0),
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
/// `AccumulatedInput::last_movement`. NPCs stand still while their talk
/// prompt is shown and throughout a dialogue.
fn steer_npcs(
    mut npcs: Query<(
        &Transform,
        &mut NpcWaypoints,
        &mut AccumulatedInput,
        Has<InReach>,
        Has<InDialogue>,
    )>,
) {
    for (transform, mut waypoints, mut input, in_reach, in_dialogue) in &mut npcs {
        if in_reach || in_dialogue {
            input.last_movement = None;
            continue;
        }
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

fn end_npc_dialogue(
    mut commands: Commands,
    npcs: Query<Entity, With<InDialogue>>,
    runners: Query<&DialogueRunner>,
) {
    if runners.iter().any(|runner| runner.is_running()) {
        return;
    }
    for npc in &npcs {
        commands.entity(npc).remove::<InDialogue>();
    }
}
