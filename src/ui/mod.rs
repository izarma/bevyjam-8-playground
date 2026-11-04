use bevy::prelude::*;

pub(crate) mod font;
pub(crate) mod input_prompt;

pub(crate) mod interaction;
pub(crate) mod menus;
pub(crate) mod palette;
pub(crate) mod scale;
pub(crate) mod widget;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        font::plugin,
        interaction::plugin,
        menus::plugin,
        scale::plugin,
    ));
}
