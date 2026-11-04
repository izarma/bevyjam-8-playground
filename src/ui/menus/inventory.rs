use bevy::prelude::*;
use plants::Plant;

use crate::{
    gameplay::{herbs, inventory::Inventory},
    ui::{
        font,
        menus::Menu,
        palette::{BUTTON_TEXT, HEADER_TEXT},
        scale::{TEXEL, texels},
        widget,
    },
};

const COLUMNS: u16 = 4;
const SLOT: u16 = 24;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Inventory), spawn_inventory_menu)
        .add_systems(Update, fit_icons.run_if(in_state(Menu::Inventory)));
}

fn spawn_inventory_menu(mut commands: Commands, inventory: Single<&Inventory>) {
    let mut held = inventory
        .plants
        .iter()
        .map(|(&plant, &count)| (plant, count))
        .collect::<Vec<_>>();
    held.sort();
    let slots = held
        .into_iter()
        .map(|(plant, count)| -> Box<dyn Scene> { Box::new(slot(plant, count)) })
        .collect::<Vec<_>>();

    let columns: Vec<RepeatedGridTrack> = RepeatedGridTrack::px(COLUMNS, f32::from(SLOT) * TEXEL);
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
                Node {
                    display: Display::Grid,
                    grid_template_columns: columns,
                    row_gap: texels(2),
                    column_gap: texels(2),
                }
                Children [ { slots } ]
            ]
        ]
    });
}

fn slot(plant: Plant, count: u32) -> impl Scene {
    bsn! {
        Node {
            width: texels(SLOT),
            height: texels(SLOT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        @widget::panel()
        @on(show_tooltip)
        @on(hide_tooltip)
        Children [
            Node {
                width: texels(16),
                height: texels(16),
            }
            ImageNode
            Icon(plant)
            Pickable::IGNORE
            --
            Text({ count.to_string() })
            @font::text(1)
            TextColor(BUTTON_TEXT)
            Node {
                position_type: PositionType::Absolute,
                right: texels(2),
                bottom: texels(0),
            }
            Pickable::IGNORE
            --
            Tooltip
            Node {
                position_type: PositionType::Absolute,
                left: percent(100),
                top: texels(0),
                margin: UiRect::left(texels(2)),
            }
            Visibility::Hidden
            GlobalZIndex(3)
            Pickable::IGNORE
            Children [ @herbs::card(plant) ]
        ]
    }
}

#[derive(Component, Clone, Default)]
struct Icon(Plant);

fn fit_icons(
    mut icons: Query<(&Icon, &mut ImageNode)>,
    images: Res<Assets<Image>>,
    assets: Res<AssetServer>,
) {
    for (Icon(plant), mut node) in &mut icons {
        if node.rect.is_some() {
            continue;
        }
        let sprite = plant.sprite(&assets);
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };
        node.rect = Some(sprite.rect(image.size()).as_rect());
        node.image = sprite.image;
        node.flip_x = sprite.flip_x;
    }
}

#[derive(Component, Clone, Default)]
struct Tooltip;

fn show_tooltip(
    over: On<PointerOver>,
    children: Query<&Children>,
    mut tooltips: Query<&mut Visibility, With<Tooltip>>,
) {
    set_tooltip(over.entity, Visibility::Inherited, &children, &mut tooltips);
}

fn hide_tooltip(
    out: On<PointerOut>,
    children: Query<&Children>,
    mut tooltips: Query<&mut Visibility, With<Tooltip>>,
) {
    set_tooltip(out.entity, Visibility::Hidden, &children, &mut tooltips);
}

fn set_tooltip(
    slot: Entity,
    visibility: Visibility,
    children: &Query<&Children>,
    tooltips: &mut Query<&mut Visibility, With<Tooltip>>,
) {
    for child in children.get(slot).into_iter().flatten() {
        if let Ok(mut tooltip) = tooltips.get_mut(*child) {
            *tooltip = visibility;
        }
    }
}
