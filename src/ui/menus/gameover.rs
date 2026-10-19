use bevy::prelude::*;

use crate::{
    GameResult, GameState,
    ui::{menus::Menu, widget},
};

use super::{close_menu, open_game_over_menu, quit_to_title};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::GameOver), open_game_over_menu);
    app.add_systems(OnExit(GameState::GameOver), close_menu);
    app.add_systems(OnEnter(Menu::GameOver), spawn_gameover_ui);
}

fn spawn_gameover_ui(mut commands: Commands, game_result: Option<Res<GameResult>>) {
    let (title, color) = match game_result.as_deref() {
        Some(GameResult::Won) => ("You Win!", Color::hsl(120.0, 0.95, 0.7)),
        _ => ("You Lose!", Color::hsl(0.0, 0.95, 0.7)),
    };
    commands.spawn_scene(gameover_menu(title, color));
}

fn gameover_menu(title: &'static str, color: Color) -> impl Scene {
    bsn! {
        @widget::ui_root("Game Over")
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::GameOver)
        Children [
            @widget::header(title)
            TextColor(color)
            --
            @widget::button("Play Again", play_again)
            --
            @widget::button("Quit to Title", quit_to_title)
        ]
    }
}

fn play_again(_: On<PointerClick>, mut next_screen: ResMut<NextState<GameState>>) {
    next_screen.set(GameState::Playing);
}
