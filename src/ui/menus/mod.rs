use bevy::prelude::*;

use crate::GameState;

mod credits;
mod mainmenu;
mod pause;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();
    app.add_plugins((mainmenu::plugin, pause::plugin, credits::plugin));
    app.add_systems(OnEnter(GameState::Title), open_main_menu);
    app.add_systems(OnExit(GameState::Title), close_menu);
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Menu {
    #[default]
    None,
    Main,
    Credits,
    Settings,
    Pause,
}

fn open_main_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
