use bevy::prelude::*;
use plants::Plant;

use crate::{
    gameplay::{dialogue::view::DialogueRoot, herbs, inventory::Inventory},
    ui::{
        interaction::InteractionPalette,
        palette::{BUTTON_HOVERED_TINT, BUTTON_PRESSED_TINT},
        scale::TEXEL,
    },
};

pub(super) trait HerbPicker: Component {
    fn available(&self, inventory: &Inventory, plant: Plant) -> u32;
}

#[derive(Component, Clone, Default)]
pub(super) struct Ingredient(pub(super) Plant);

pub(super) fn ingredient<T: HerbPicker>(plant: Plant, count: u32) -> Box<dyn Scene> {
    Box::new(bsn! {
        @herbs::slot(plant, count)
        Ingredient(plant)
        InteractionPalette {
            none: Color::WHITE,
            hovered: BUTTON_HOVERED_TINT,
            pressed: BUTTON_PRESSED_TINT,
        }
        @on(start_drag::<T>)
        @on(drag)
        @on(end_drag)
    })
}

#[derive(Component, Clone, Default)]
struct DragGhost;

fn ghost_position(pointer: Vec2, ui_scale: &UiScale) -> (Val, Val) {
    let at = pointer / ui_scale.0 - Vec2::splat(8.0 * TEXEL);
    (px(at.x), px(at.y))
}

fn start_drag<T: HerbPicker>(
    start: On<PointerDragStart>,
    mut commands: Commands,
    ingredients: Query<&Ingredient>,
    picker: Single<&T>,
    inventory: Single<&Inventory>,
    root: Single<Entity, With<DialogueRoot>>,
    ui_scale: Res<UiScale>,
) {
    let Ok(&Ingredient(plant)) = ingredients.get(start.entity) else {
        return;
    };
    if picker.available(&inventory, plant) == 0 {
        return;
    }
    let (left, top) = ghost_position(start.pointer.position, &ui_scale);
    let root = *root;
    commands.spawn_scene(bsn! {
        #DragGhost
        DragGhost
        ChildOf(root)
        Node {
            position_type: PositionType::Absolute,
            left: left,
            top: top,
        }
        GlobalZIndex(4)
        Pickable::IGNORE
        Children [ @herbs::icon(plant) ]
    });
}

fn drag(
    drag: On<PointerDrag>,
    mut ghosts: Query<&mut Node, With<DragGhost>>,
    ui_scale: Res<UiScale>,
) {
    let (left, top) = ghost_position(drag.pointer.position, &ui_scale);
    for mut node in &mut ghosts {
        node.left = left;
        node.top = top;
    }
}

fn end_drag(_: On<PointerDragEnd>, mut commands: Commands, ghosts: Query<Entity, With<DragGhost>>) {
    for ghost in &ghosts {
        commands.entity(ghost).despawn();
    }
}
