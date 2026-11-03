// shared rendering between game and jackdaw

use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    shader::Shader,
};

mod model;
mod sprite;
mod water;

pub use model::PixelModel;
pub use sprite::PixelSprite;
pub use water::PixelWater;

// should be shared by everything
pub const TEXELS_PER_METER: f32 = 16.0;

pub struct PixelArtPlugin;

impl Plugin for PixelArtPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "billboard.wesl");
        embedded_asset!(app, "texel.wesl");
        let library = [
            load_embedded_asset!(app, "billboard.wesl"),
            load_embedded_asset!(app, "texel.wesl"),
        ];
        app.insert_resource(ShaderLibrary(library)).add_plugins((
            model::plugin,
            sprite::plugin,
            water::plugin,
        ));
    }
}

#[derive(Resource)]
struct ShaderLibrary(
    #[expect(dead_code, reason = "this keeps the shaders loaded")] [Handle<Shader>; 2],
);
