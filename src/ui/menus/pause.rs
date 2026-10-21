use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    GameState,
    engine::Pause,
    ui::{menus::Menu, widget},
};

use super::{close_menu, open_settings_menu, quit_to_title};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu.spawn());
    // Toggle pause on key press.
    app.add_systems(
        Update,
        (
            (pause, spawn_pause_overlay, open_pause_menu).run_if(
                in_state(GameState::Gameplay)
                    .and_then(in_state(Menu::None))
                    .and_then(
                        input_just_pressed(KeyCode::KeyP)
                            .or_else(input_just_pressed(KeyCode::Escape)),
                    ),
            ),
            close_menu.run_if(
                in_state(GameState::Gameplay)
                    .and_then(not(in_state(Menu::None)))
                    .and_then(input_just_pressed(KeyCode::KeyP)),
            ),
            close_menu.run_if(in_state(Menu::Pause).and_then(input_just_pressed(KeyCode::Escape))),
        ),
    );
    app.add_systems(OnExit(GameState::Gameplay), (close_menu, unpause));
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(GameState::Gameplay)),
    );
}

fn unpause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(false));
}

fn pause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(true));
}

fn spawn_pause_overlay(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #PauseOverlay
        Node {
            width: percent(100),
            height: percent(100),
        }
        GlobalZIndex(1)
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8))
        DespawnOnExit<Pause>(Pause(true))
    });
}

fn open_pause_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Pause);
}

fn spawn_pause_menu() -> impl Scene {
    bsn! {
        @widget::ui_root("Pause Menu")
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Pause)
        Children [
            @widget::header("Game paused")
            --
            @widget::button("Continue", close_pause_menu)
            --
            @widget::button("Settings", open_settings_menu)
            --
            @widget::button("Quit to title", quit_to_title)
        ]
    }
}

fn close_pause_menu(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
