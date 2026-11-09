use std::collections::HashMap;

use bevy::prelude::*;
use jackdaw_scene_types::node_id::SceneNodeId;
use plants::{Biome, Growing, HerbSpot, Plant};
use rand::{RngExt, seq::IndexedRandom};

use crate::{
    GameState, GameplayScene,
    gameplay::clock::{Clock, START_HOUR},
};

const SPROUT_HOURS: f32 = 12.0;
const WITHER_HOURS: f32 = 48.0;
const MOST_GROWING: usize = 2;
const WILD_CHANCE: f64 = 1.0 / 20.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_fields)
        .add_systems(
            Update,
            (sprout, wither, sow)
                .chain()
                .run_if(in_state(GameplayScene::Outdoors)),
        );
}

struct Growth {
    spot: SceneNodeId,
    herb: Plant,
    since: f32,
}

#[derive(Default)]
struct Field {
    growing: Vec<Growth>,
    sprouted: u32,
}

impl Field {
    fn wither(&mut self, hours: f32) {
        self.growing
            .retain(|growth| growth.since + WITHER_HOURS > hours);
    }

    fn growing(&self, spot: SceneNodeId) -> Option<Plant> {
        self.growing
            .iter()
            .find(|growth| growth.spot == spot)
            .map(|growth| growth.herb)
    }
}

#[derive(Component, Default)]
pub(super) struct Fields(HashMap<SceneNodeId, Field>);

impl Fields {
    pub(super) fn picked(&mut self, spot: SceneNodeId) {
        for field in self.0.values_mut() {
            field.growing.retain(|growth| growth.spot != spot);
        }
    }

    fn growing(&self, spot: SceneNodeId) -> Option<Plant> {
        self.0.values().find_map(|field| field.growing(spot))
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

fn sprout_hour(sprout: u32) -> f32 {
    START_HOUR + sprout.saturating_sub(MOST_GROWING as u32 - 1) as f32 * SPROUT_HOURS
}

fn sprout(
    mut fields: Single<&mut Fields>,
    clock: Single<&Clock>,
    biomes: Query<(&SceneNodeId, &Biome, &Children)>,
    spots: Query<&SceneNodeId, With<HerbSpot>>,
) {
    let mut rng = rand::rng();
    for (&biome, Biome { herbs }, children) in &biomes {
        let field = fields.0.entry(biome).or_default();
        while sprout_hour(field.sprouted) <= clock.hours {
            let hour = sprout_hour(field.sprouted);
            field.sprouted += 1;
            field.wither(hour);
            if field.growing.len() >= MOST_GROWING {
                continue;
            }
            let free: Vec<SceneNodeId> = spots
                .iter_many(children)
                .flatten()
                .copied()
                .filter(|&spot| field.growing(spot).is_none())
                .collect();
            let fresh: Vec<Plant> = herbs
                .iter()
                .copied()
                .filter(|&herb| field.growing.iter().all(|growth| growth.herb != herb))
                .collect();
            let pool = if fresh.is_empty() { herbs } else { &fresh };
            let herb = if rng.random_bool(WILD_CHANCE) {
                Some(Plant::BlueHibiscus)
            } else {
                pool.choose(&mut rng).copied()
            };
            if let (Some(&spot), Some(herb)) = (free.choose(&mut rng), herb) {
                field.growing.push(Growth {
                    spot,
                    herb,
                    since: hour,
                });
            }
        }
        field.wither(clock.hours);
    }
}

fn wither(
    mut commands: Commands,
    fields: Single<&Fields>,
    spots: Query<(Entity, &SceneNodeId, &Children), (With<HerbSpot>, With<Sown>)>,
    plants: Query<(), With<Growing>>,
) {
    for (spot, id, children) in &spots {
        if fields.growing(*id).is_some() {
            continue;
        }
        for &plant in children {
            if plants.contains(plant) {
                commands.entity(plant).despawn();
            }
        }
        commands.entity(spot).remove::<Sown>();
    }
}

fn sow(
    mut commands: Commands,
    fields: Single<&Fields>,
    spots: Query<(Entity, &SceneNodeId), (With<HerbSpot>, Without<Sown>)>,
    asset_server: Res<AssetServer>,
) {
    for (spot, id) in &spots {
        let Some(herb) = fields.growing(*id) else {
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
