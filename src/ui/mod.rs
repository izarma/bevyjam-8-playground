use bevy::prelude::*;

pub(crate) mod billboard;
pub(crate) mod font;
pub(crate) mod health;
pub(crate) mod input_prompt;

pub(crate) mod interaction;
pub(crate) mod menus;
pub(crate) mod palette;
pub(crate) mod scale;
pub(crate) mod widget;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        billboard::plugin,
        font::plugin,
        interaction::plugin,
        menus::plugin,
        scale::plugin,
    ));
}
