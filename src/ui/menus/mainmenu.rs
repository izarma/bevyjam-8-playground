use bevy::prelude::*;

use crate::{
    GameState,
    ui::{menus::Menu, widget},
};

use super::{close_menu, open_main_menu, open_settings_menu};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Title), open_main_menu);
    app.add_systems(OnExit(GameState::Title), close_menu);
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu.spawn());
}

fn spawn_main_menu() -> impl Scene {
    bsn! {
        @widget::ui_root("Main Menu")
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Main)
        Children [
            @widget::button("Play", enter_gameplay_state)
            --
            @widget::button("Example Scene", enter_example_scene)
            --
            @widget::button("Lab", enter_lab_scene)
            --
            @widget::button("Settings", open_settings_menu)
            --
            @widget::button("Credits", open_credits_menu)
            --
            { #[cfg(not(target_family = "wasm"))] (true).then(|| { bsn! { @widget::button("Exit", exit_app) }})}
        ]
    }
}

fn enter_gameplay_state(_: On<PointerClick>, mut next_screen: ResMut<NextState<GameState>>) {
    next_screen.set(GameState::Gameplay);
}

fn enter_example_scene(_: On<PointerClick>, mut next_screen: ResMut<NextState<GameState>>) {
    next_screen.set(GameState::ExampleScene);
}

fn enter_lab_scene(_: On<PointerClick>, mut next_screen: ResMut<NextState<GameState>>) {
    next_screen.set(GameState::LabScene);
}

fn open_credits_menu(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: On<PointerClick>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
