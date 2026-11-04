use bevy::prelude::*;

use crate::ui::scale::texels;

// cell indices in the spritesheet
pub(crate) const KEY_F: u32 = 123;
pub(crate) const MOUSE: u32 = 76;

const TILE: u32 = 16;
const COLUMNS: u32 = 34;

pub(crate) fn prompt(tile: u32) -> impl Scene {
    let corner = UVec2::new(tile % COLUMNS, tile / COLUMNS) * TILE;
    let rect = Rect::from_corners(corner.as_vec2(), (corner + TILE).as_vec2());
    bsn! {
        #Prompt
        Node {
            width: texels(16),
            height: texels(16),
        }
        ImageNode {
            image: "ui/input_prompts.png",
            rect: { Some(rect) },
        }
        Pickable::IGNORE
    }
}
