use bevy::prelude::*;

use crate::GameState;

mod credits;
mod gameover;
mod mainmenu;
mod pause;
mod settings;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();
    app.add_plugins((
        mainmenu::plugin,
        pause::plugin,
        credits::plugin,
        gameover::plugin,
        settings::plugin,
    ));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Menu {
    #[default]
    None,
    Main,
    Credits,
    Settings,
    Pause,
    GameOver,
}

pub(super) fn open_main_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

pub(super) fn open_main_menu_on_click(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

pub(super) fn open_game_over_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::GameOver);
}

pub(super) fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

pub(super) fn open_settings_menu(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

pub(super) fn quit_to_title(_: On<PointerClick>, mut next_screen: ResMut<NextState<GameState>>) {
    next_screen.set(GameState::Title);
}
