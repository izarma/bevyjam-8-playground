use bevy::prelude::*;
use jackdaw_runtime::{TerrainTileVariety, TileVariant, TileVariety};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(TerrainTileVariety(
        [
            (
                "material_1.bsn",
                &[
                    ("textures/dirt1.png", 1.0),
                    ("textures/dirt2.png", 1.0),
                    ("textures/dirt3.png", 1.0),
                    ("textures/dirt4.png", 1.0),
                ][..],
            ),
            (
                "material_2.bsn",
                &[
                    ("textures/grass1.png", 1.0),
                    ("textures/grass2.png", 15.0),
                    ("textures/grass3.png", 1.0),
                    ("textures/grass4.png", 1.0),
                ],
            ),
            (
                "material_sand.bsn",
                &[
                    ("textures/sand1.png", 5.0),
                    ("textures/sand2.png", 5.0),
                    ("textures/sand3.png", 1.0),
                    ("textures/sand4.png", 1.0),
                ],
            ),
        ]
        .into_iter()
        .map(|(material, tiles)| {
            let variants = tiles
                .iter()
                .map(|&(albedo, weight)| TileVariant {
                    albedo: albedo.into(),
                    weight,
                })
                .collect();
            (material.into(), TileVariety { variants, turn: true })
        })
        .collect(),
    ));
}
