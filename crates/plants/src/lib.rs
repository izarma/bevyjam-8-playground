// we are defining the plant types in code
// and it's shared between jackdaw and teh game so the editor knows
// what "kinds" of plants there are to place and their properties
// i tried to push more of this to the editor but i failed

use bevy::{platform::collections::HashMap, prelude::*};
use jackdaw_scene_types::EditorHidden;
use pixel_art::PixelSprite;
use rand::seq::SliceRandom;

#[derive(Reflect, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[reflect(Default)]
pub enum Plant {
    #[default]
    Basil,
    Chili,
    Lemongrass,
    Ginseng,
    Chrysanthemum,
    Wolfberry,
    Lotus,
    Licorice,
    RedMushroom,
    Grapes,
}

// `Plant::ALL` is in the same order that these are left to right
pub const SHEET: &str = "sprites/Herbs-Sheet.png";
pub const CELL: u32 = 16;
pub const COLUMNS: u32 = Plant::ALL.len() as u32;
const HARVESTED: u32 = 1;
const PICKED: u32 = 2;

impl Plant {
    pub const ALL: [Plant; 10] = [
        Plant::Basil,
        Plant::Chili,
        Plant::Lemongrass,
        Plant::Ginseng,
        Plant::Chrysanthemum,
        Plant::Wolfberry,
        Plant::Lotus,
        Plant::Licorice,
        Plant::RedMushroom,
        Plant::Grapes,
    ];

    pub fn column(self) -> u32 {
        self as u32
    }

    /// The picked herb's cell in [`SHEET`].
    pub fn harvested_cell(self) -> u32 {
        HARVESTED * COLUMNS + self.column()
    }

    pub fn picked_cell(self) -> u32 {
        PICKED * COLUMNS + self.column()
    }

    pub fn sprite(self, assets: &AssetServer) -> PixelSprite {
        PixelSprite {
            image: assets.load(SHEET),
            size: Some(UVec2::splat(CELL)),
            cell: self.column(),
            ..default()
        }
    }
}

// where a herb grows: one of its biome's herbs, dealt when a game starts
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct HerbSpot;

// parent of herb spots, and the herbs they can grow
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Biome {
    pub herbs: Vec<Plant>,
}

impl Biome {
    /// a herb for each of `spots` spots, dealt so no herb repeats until every
    /// herb has a spot
    pub fn deal(&self, spots: usize) -> Vec<Plant> {
        let mut rng = rand::rng();
        let mut dealt = Vec::with_capacity(spots);
        while dealt.len() < spots && !self.herbs.is_empty() {
            let mut round = self.herbs.clone();
            round.shuffle(&mut rng);
            dealt.extend(round.into_iter().take(spots - dealt.len()));
        }
        dealt
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Growing(pub Plant);

// deals each biome's herbs onto its spots, like a game would
pub struct SpotPreviewPlugin;

impl Plugin for SpotPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, preview_spots);
    }
}

#[derive(Component)]
struct Preview;

// what a biome was last dealt from: its spots and its herbs
type Dealt = (Vec<Entity>, Vec<Plant>);

fn preview_spots(
    mut commands: Commands,
    mut dealt: Local<HashMap<Entity, Dealt>>,
    biomes: Query<(Entity, &Biome, Option<&Children>)>,
    spots: Query<Option<&ChildOf>, With<HerbSpot>>,
    previews: Query<(Entity, &ChildOf), With<Preview>>,
    asset_server: Res<AssetServer>,
) {
    let mut redealt = Vec::new();
    for (biome, Biome { herbs }, children) in &biomes {
        let now = (
            children
                .into_iter()
                .flatten()
                .copied()
                .filter(|&child| spots.contains(child))
                .collect::<Vec<_>>(),
            herbs.clone(),
        );
        if dealt.get(&biome) == Some(&now) {
            continue;
        }
        let deal = Biome { herbs: now.1.clone() }.deal(now.0.len());
        for (&spot, herb) in now.0.iter().zip(deal) {
            commands.spawn((
                Name::new("Herb Preview"),
                Preview,
                herb.sprite(&asset_server),
                Transform::default(),
                EditorHidden,
                ChildOf(spot),
            ));
        }
        redealt.extend(now.0.iter().copied());
        dealt.insert(biome, now);
    }
    dealt.retain(|&biome, _| biomes.contains(biome));

    // clear what was redealt, and any spot no longer in a biome
    for (preview, spot) in &previews {
        let in_biome = spots
            .get(spot.parent())
            .ok()
            .flatten()
            .is_some_and(|parent| biomes.contains(parent.parent()));
        if redealt.contains(&spot.parent()) || !in_biome {
            commands.entity(preview).despawn();
        }
    }
}
