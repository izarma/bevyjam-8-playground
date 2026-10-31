mod audio;
mod collision;
mod finish_areas;
mod game_object;
pub(crate) mod input;
mod map;
mod player;

use bevy::prelude::*;

use audio::InternalAudioPlugin;
use finish_areas::FinishAreaPlugin;
use game_object::GameObjectPlugin;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            input::plugin,
            InternalAudioPlugin,
            FinishAreaPlugin,
            GameObjectPlugin,
            map::plugin,
            player::plugin,
        ));
    }
}
