use std::collections::HashMap;

use bevy::prelude::*;
use jackdaw_scene_types::node_id::SceneNodeId;
use pixel_art::PixelSprite;
use plants::{Biome, Growing, HerbSpot, Plant};

use crate::{
    GameState, GameplayScene,
    gameplay::clock::{Clock, START_HOUR},
};

pub(super) const REGROW_HOURS: f32 = 12.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_fields)
        .add_systems(
            Update,
            (sow, regrow)
                .chain()
                .run_if(in_state(GameplayScene::Outdoors)),
        );
}

#[derive(Component)]
pub(super) struct Harvested;

#[derive(Component)]
struct Sown;

struct Field {
    herb: Plant,
    ready: f32,
}

#[derive(Component, Default)]
pub(super) struct Fields(HashMap<SceneNodeId, Field>);

impl Fields {
    pub(super) fn ready(&self, spot: SceneNodeId) -> Option<f32> {
        self.0.get(&spot).map(|field| field.ready)
    }

    pub(super) fn picked(&mut self, spot: SceneNodeId, hours: f32) {
        if let Some(field) = self.0.get_mut(&spot) {
            field.ready = hours + REGROW_HOURS;
        }
    }
}

fn spawn_fields(mut commands: Commands) {
    commands.spawn((
        Name::new("Fields"),
        Fields::default(),
        DespawnOnExit(GameState::Gameplay),
    ));
}

fn sow(
    mut commands: Commands,
    mut fields: Single<&mut Fields>,
    clock: Single<&Clock>,
    spots: Query<(Entity, &SceneNodeId, &ChildOf), (With<HerbSpot>, Without<Sown>)>,
    biomes: Query<&Biome>,
    asset_server: Res<AssetServer>,
) {
    let mut unsown = HashMap::<Entity, Vec<SceneNodeId>>::new();
    for (_, &id, parent) in &spots {
        if !fields.0.contains_key(&id) {
            unsown.entry(parent.parent()).or_default().push(id);
        }
    }
    for (biome, ids) in unsown {
        let Ok(biome) = biomes.get(biome) else {
            continue;
        };
        let herbs = biome.deal(ids.len());
        fields.0.extend(ids.into_iter().zip(herbs).map(|(id, herb)| (
            id,
            Field {
                herb,
                ready: START_HOUR,
            },
        )));
    }

    for (spot, id, _) in &spots {
        let Some(field) = fields.0.get(id) else {
            continue;
        };
        let mut sprite = field.herb.sprite(&asset_server);
        let mut plant = commands.spawn((
            Name::new(format!("{:?}", field.herb)),
            Growing(field.herb),
            Transform::default(),
            ChildOf(spot),
        ));
        if field.ready > clock.hours {
            sprite.cell = field.herb.harvested_cell();
            plant.insert(Harvested);
        }
        plant.insert(sprite);
        commands.entity(spot).insert(Sown);
    }
}

fn regrow(
    mut commands: Commands,
    clock: Single<&Clock>,
    fields: Single<&Fields>,
    mut plants: Query<(Entity, &Growing, &ChildOf, &mut PixelSprite), With<Harvested>>,
    spots: Query<&SceneNodeId>,
) {
    for (plant, &Growing(herb), spot, mut sprite) in &mut plants {
        let ready = spots
            .get(spot.parent())
            .ok()
            .and_then(|id| fields.0.get(id))
            .is_some_and(|field| field.ready <= clock.hours);
        if ready {
            sprite.cell = herb.column();
            commands.entity(plant).remove::<Harvested>();
        }
    }
}
