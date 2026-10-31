mod finish_areas;
mod map;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((map::plugin, finish_areas::plugin));
}
