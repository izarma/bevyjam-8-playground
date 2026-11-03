use bevy::prelude::*;

mod interaction;
pub(crate) mod menus;
pub(crate) mod palette;
pub(crate) mod scale;
mod widget;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((interaction::plugin, menus::plugin, scale::plugin));
}
