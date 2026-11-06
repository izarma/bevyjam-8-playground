use bevy::prelude::*;

use crate::{
    gameplay::{herbs, inventory::Inventory},
    ui::{font, menus::Menu, palette::HEADER_TEXT, scale::texels, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Inventory), spawn_inventory_menu);
}

fn spawn_inventory_menu(mut commands: Commands, inventory: Single<&Inventory>) {
    let held = inventory
        .plants
        .iter()
        .map(|(&plant, &count)| (plant, count));
    commands.spawn_scene(bsn! {
        @widget::ui_root("Inventory")
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Inventory)
        Children [
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(texels(5)),
                row_gap: texels(4),
            }
            @widget::panel()
            Children [
                Text("Inventory")
                @font::text(1)
                TextColor(HEADER_TEXT)
                --
                @herbs::grid(held, |plant, count| Box::new(herbs::slot(plant, count)))
            ]
        ]
    });
}
