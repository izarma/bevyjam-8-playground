use bevy::{platform::collections::HashMap, prelude::*};
use plants::Plant;

use crate::{
    GameState, Harvesting, Pause,
    loading::UIAssets,
    ui::{
        input_prompt::{KEY_I, prompt},
        menus::Menu,
        scale::texels,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gameplay), spawn_inventory);
}

#[derive(Component, Default, Debug)]
pub(crate) struct Inventory {
    pub(crate) plants: HashMap<Plant, u32>,
}

fn spawn_inventory(mut commands: Commands, ui: Res<UIAssets>) {
    let icon = ui.inventory_icon.clone();
    commands.spawn((
        Name::new("Inventory"),
        Inventory::default(),
        DespawnOnExit(GameState::Gameplay),
    ));
    // Inventory HUD
    commands.spawn_scene(bsn! {
        Name::new("Inventory HUD")
        Node {
            position_type: PositionType::Absolute,
            left: texels(4),
            bottom: texels(4),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: texels(1),
        }
        Pickable::IGNORE
        Children [
            @prompt(KEY_I)
            --
            Name::new("Inventory Icon")
            ImageNode { image: icon }
            Node {
                width: texels(16),
                height: texels(16),
            }
            @on(toggle_inventory_on_click)
        ]
        DespawnOnExit<GameState>(GameState::Gameplay)
    });
}

fn toggle_inventory_on_click(
    _: On<PointerClick>,
    mut commands: Commands,
    menu: Res<State<Menu>>,
    screen: Res<State<GameState>>,
    harvesting: Option<Res<State<Harvesting>>>,
) {
    let harvesting = harvesting.is_some_and(|harvesting| harvesting.0);
    match menu.get() {
        Menu::None if *screen.get() == GameState::Gameplay && !harvesting => {
            commands.set_state(Pause(true));
            commands.set_state(Menu::Inventory);
        }
        Menu::Inventory => commands.set_state(Menu::None),
        _ => {}
    }
}
