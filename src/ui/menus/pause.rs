use bevy::prelude::*;

use crate::{
    GameState, Pause,
    gameplay::upgrades::Rewards,
    ui::{
        input_prompt::{KEY_I, KEY_J, KEY_R, prompt},
        menus::Menu,
        scale::texels,
        widget,
    },
};

use super::{close_menu, open_settings_menu, quit_to_title};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(OnExit(GameState::Gameplay), close_menu);
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(GameState::Gameplay)),
    );
}

fn unpause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(false));
}

pub(super) fn open_pause_menu(commands: &mut Commands) {
    commands.set_state(Pause(true));
    commands.set_state(Menu::Pause);
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

fn spawn_pause_menu(mut commands: Commands, rewards: Option<Res<Rewards>>) {
    let radar = rewards.is_some_and(|rewards| rewards.radar());
    commands.spawn_scene(pause_menu(radar));
}

fn pause_menu(radar: bool) -> impl Scene {
    let mut hints: Vec<Box<dyn Scene>> = vec![
        Box::new(control_hint(KEY_I, "Inventory")),
        Box::new(control_hint(KEY_J, "Journal")),
    ];
    if radar {
        hints.push(Box::new(control_hint(KEY_R, "Radar")));
    }
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
            --
            #Controls
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: texels(3),
                margin: UiRect::top(texels(2)),
            }
            Children [
                @widget::label("Controls")
                --
                Node {
                    align_items: AlignItems::Center,
                    column_gap: texels(6),
                }
                Children [ { hints } ]
            ]
        ]
    }
}

fn control_hint(key: u32, action: &'static str) -> impl Scene {
    bsn! {
        Node {
            align_items: AlignItems::Center,
            column_gap: texels(2),
        }
        Children [
            @prompt(key)
            --
            @widget::label(action)
        ]
    }
}

fn close_pause_menu(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
