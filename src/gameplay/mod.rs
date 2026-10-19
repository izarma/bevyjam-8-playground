mod actions;
mod audio;
mod collision;
mod finish_areas;
mod game_object;
mod player;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use actions::ActionsPlugin;
use audio::InternalAudioPlugin;
use finish_areas::FinishAreaPlugin;
use game_object::GameObjectPlugin;
use player::PlayerPlugin;

use crate::{GameState, Pause, ui::menus::Menu};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ActionsPlugin,
            InternalAudioPlugin,
            FinishAreaPlugin,
            GameObjectPlugin,
            PlayerPlugin,
        ));
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
            ),
        );
        app.add_systems(OnExit(GameState::Gameplay), (close_menu, unpause));
        app.add_systems(
            OnEnter(Menu::None),
            unpause.run_if(in_state(GameState::Gameplay)),
        );
    }
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

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
