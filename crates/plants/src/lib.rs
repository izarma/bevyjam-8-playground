// we are defining the plant types in code
// and it's shared between jackdaw and teh game so the editor knows
// what "kinds" of plants there are to place and their properties
// i tried to push more of this to the editor but i failed

use bevy::prelude::*;
use jackdaw_scene_types::EditorHidden;
use pixel_art::PixelSprite;
use rand::seq::IndexedRandom;

#[derive(Reflect, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Plant {
    #[default]
    Plant01,
    Plant06,
    Plant13,
}

impl Plant {
    /// What this plant looks like.
    pub fn sprite(self, assets: &AssetServer) -> PixelSprite {
        match self {
            Plant::Plant01 => PixelSprite {
                image: assets.load("sprites/plants.png"),
                size: Some(UVec2::splat(16)),
                cell: 0,
                ..default()
            },
            Plant::Plant06 => PixelSprite {
                image: assets.load("sprites/plants.png"),
                size: Some(UVec2::splat(16)),
                cell: 5,
                ..default()
            },
            Plant::Plant13 => PixelSprite {
                image: assets.load("sprites/plants.png"),
                size: Some(UVec2::splat(16)),
                cell: 12,
                ..default()
            },
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Gatherable {
    pub plants: Vec<Plant>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Growing(pub Plant);

#[derive(Resource, Clone, Copy)]
pub enum Pick {
    First,
    Random,
}

pub struct PlantsPlugin {
    pub pick: Pick,
}

impl Plugin for PlantsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.pick)
            .add_systems(Update, grow_plants);
    }
}

// a spot regrows whenever its list changes, which in the editor is every edit
fn grow_plants(
    mut commands: Commands,
    pick: Res<Pick>,
    spots: Query<(Entity, &Gatherable, Option<&Children>), Changed<Gatherable>>,
    growing: Query<(), With<Growing>>,
    asset_server: Res<AssetServer>,
) {
    for (spot, gatherable, children) in &spots {
        for child in children.into_iter().flatten() {
            if growing.contains(*child) {
                commands.entity(*child).despawn();
            }
        }
        let plant = match *pick {
            Pick::First => gatherable.plants.first(),
            Pick::Random => gatherable.plants.choose(&mut rand::rng()),
        };
        let Some(&plant) = plant else {
            continue;
        };
        commands.spawn((
            Name::new(format!("{plant:?}")),
            Growing(plant),
            plant.sprite(&asset_server),
            Transform::default(),
            EditorHidden,
            ChildOf(spot),
        ));
    }
}
