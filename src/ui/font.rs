use bevy::{prelude::*, text::LineHeight};

use crate::ui::scale::texels;

const EM_TEXELS: u16 = 16;

pub(super) fn plugin(app: &mut App) {
    let font = Font::from_bytes(include_bytes!("../../assets/fonts/BoldPixels.ttf").to_vec());
    app.world_mut()
        .resource_mut::<Assets<Font>>()
        .insert(AssetId::default(), font)
        .expect("the default font handle is always insertable");
}

pub fn text(scale: u16) -> impl Scene {
    bsn! {
        TextFont {
            font_size: texels(EM_TEXELS * scale),
            font_smoothing: FontSmoothing::None,
        }
        LineHeight::RelativeToFont(1.0)
    }
}
