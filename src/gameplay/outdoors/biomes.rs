use std::collections::HashMap;

use bevy::prelude::*;
use jackdaw_scene_types::node_id::SceneNodeId;
use plants::{Biome, Growing, HerbSpot, Plant};
use rand::seq::IndexedRandom;

use crate::{
    GameState, GameplayScene,
    gameplay::clock::{Clock, START_HOUR},
};

const SPROUT_HOURS: f32 = 12.0;
const MOST_GROWING: usize = 2;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_fields)
        .add_systems(
            Update,
            (sprout, sow)
                .chain()
                .run_if(in_state(GameplayScene::Outdoors)),
        );
}

#[derive(Default)]
struct Field {
    growing: Vec<(SceneNodeId, Plant)>,
    sprouted: u32,
}

#[derive(Component, Default)]
pub(super) struct Fields(HashMap<SceneNodeId, Field>);

impl Fields {
    pub(super) fn picked(&mut self, spot: SceneNodeId) {
        for field in self.0.values_mut() {
            field.growing.retain(|&(growing, _)| growing != spot);
        }
    }
}

#[derive(Component)]
struct Sown;

fn spawn_fields(mut commands: Commands) {
    commands.spawn((
        Name::new("Fields"),
        Fields::default(),
        DespawnOnExit(GameState::Gameplay),
    ));
}

fn sprout(
    mut fields: Single<&mut Fields>,
    clock: Single<&Clock>,
    biomes: Query<(&SceneNodeId, &Biome, &Children)>,
    spots: Query<&SceneNodeId, With<HerbSpot>>,
) {
    let due = ((clock.hours - START_HOUR) / SPROUT_HOURS).floor() as u32 + 1;
    let mut rng = rand::rng();
    for (&biome, Biome { herbs }, children) in &biomes {
        let field = fields.0.entry(biome).or_default();
        while field.sprouted < due {
            field.sprouted += 1;
            if field.growing.len() >= MOST_GROWING {
                continue;
            }
            let free: Vec<SceneNodeId> = spots
                .iter_many(children)
                .flatten()
                .copied()
                .filter(|&spot| field.growing.iter().all(|&(growing, _)| growing != spot))
                .collect();
            if let (Some(&spot), Some(&herb)) = (free.choose(&mut rng), herbs.choose(&mut rng)) {
                field.growing.push((spot, herb));
            }
        }
    }
}

fn sow(
    mut commands: Commands,
    fields: Single<&Fields>,
    spots: Query<(Entity, &SceneNodeId), (With<HerbSpot>, Without<Sown>)>,
    asset_server: Res<AssetServer>,
) {
    for (spot, id) in &spots {
        let Some(&(_, herb)) = fields
            .0
            .values()
            .flat_map(|field| &field.growing)
            .find(|&&(growing, _)| growing == *id)
        else {
            continue;
        };
        commands.spawn((
            Name::new(format!("{herb:?}")),
            Growing(herb),
            herb.sprite(&asset_server),
            Transform::default(),
            ChildOf(spot),
        ));
        commands.entity(spot).insert(Sown);
    }
}

pub(super) fn unsow(commands: &mut Commands, spot: Entity) {
    commands.entity(spot).remove::<Sown>();
}
