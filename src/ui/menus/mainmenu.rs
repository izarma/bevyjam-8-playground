use bevy::prelude::*;

use crate::{
    GameState,
    loading_screen::Travel,
    ui::{menus::Menu, scale::texels, widget},
};

use super::{close_menu, open_main_menu, open_settings_menu};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Title), open_main_menu);
    app.add_systems(OnExit(GameState::Title), close_menu);
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        @widget::ui_root("Main Menu")
        Node {
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexEnd,
            padding: UiRect::all(texels(12)),
        }
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Main)
        Children [
            @widget::button("Play", enter_gameplay_state)
            --
            @widget::button("Settings", open_settings_menu)
            --
            @widget::button("Credits", open_credits_menu)
            --
            { #[cfg(debug_assertions)] (true).then(|| { bsn! { @widget::button("Outdoor Scene", enter_example_scene) }}) }
            --
            { #[cfg(not(target_family = "wasm"))] (true).then(|| { bsn! { @widget::button("Exit", exit_app) }})}
        ]
    });
}

fn enter_gameplay_state(_: On<PointerClick>, mut commands: Commands) {
    commands.trigger(Travel::Base);
}

#[cfg(debug_assertions)]
fn enter_example_scene(_: On<PointerClick>, mut commands: Commands) {
    commands.trigger(Travel::Outdoors);
}

fn open_credits_menu(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: On<PointerClick>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
