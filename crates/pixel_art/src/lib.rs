// shared rendering between game and jackdaw

use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    shader::Shader,
};

mod model;
mod sky;
mod sprite;
mod water;

pub use model::{PixelModel, UnswappedModels};
pub use sky::PixelSky;
pub use sprite::{PixelSprite, PixelSpriteMaterial};
pub use water::PixelWater;

// // given a "cell" index, provides its `URect` on a particular sheet configuration
pub fn grid_rect(cell: u32, columns: u32, size: UVec2) -> URect {
    let min = UVec2::new(cell % columns, cell / columns) * size;
    URect::from_corners(min, min + size)
}

// should be shared by everything
pub const TEXELS_PER_METER: f32 = 16.0;

pub struct PixelArtPlugin;

impl Plugin for PixelArtPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "billboard.wesl");
        embedded_asset!(app, "texel.wesl");
        embedded_asset!(app, "lighting.wesl");
        let library = [
            load_embedded_asset!(app, "billboard.wesl"),
            load_embedded_asset!(app, "texel.wesl"),
            load_embedded_asset!(app, "lighting.wesl"),
        ];
        app.insert_resource(ShaderLibrary(library)).add_plugins((
            model::plugin,
            sky::plugin,
            sprite::plugin,
            water::plugin,
        ));
    }
}

#[derive(Resource)]
struct ShaderLibrary(
    #[expect(dead_code, reason = "this keeps the shaders loaded")] [Handle<Shader>; 3],
);
