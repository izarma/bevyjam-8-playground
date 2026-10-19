use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::ui::{menus::Menu, widget};

use super::{open_settings_menu, quit_to_title};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu.spawn());
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Pause).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_pause_menu() -> impl Scene {
    bsn! {
        @widget::ui_root("Pause Menu")
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Pause)
        Children [
            @widget::header("Game paused")
            --
            @widget::button("Continue", close_menu)
            --
            @widget::button("Settings", open_settings_menu)
            --
            @widget::button("Quit to title", quit_to_title)
        ]
    }
}

fn close_menu(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
