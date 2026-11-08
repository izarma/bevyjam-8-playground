use bevy::{prelude::*, ui::InteractionDisabled};
use plants::Plant;

use super::{Concoction, DROP, LIFT, SETTLE, SHAKE, STAMP_AT};
use crate::{
    gameplay::{
        dialogue::{YarnSpinnerDialogueViewSystemSet, view::DialogueRoot},
        herbs::{self, HeldCount},
        inventory::Inventory,
        remedy::{Mark, Remedy, SLOTS},
    },
    ui::{
        font,
        interaction::InteractionPalette,
        palette::{BUTTON_DISABLED_TINT, CAUTION, GAIN_TEXT, HEADER_TEXT, LOSS_TEXT},
        scale::{TEXEL, texels},
        widget,
    },
};

// parameters for the "stamping" animation
const LIFT_TEXELS: f32 = 8.0;
const LIFTED_SCALE: f32 = 1.2;
const STAMPED_SCALE: f32 = 0.85;
const SHAKE_HZ: f32 = 9.0;
const SHAKE_RADIANS: f32 = 0.12;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct PanelSystems;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (pose_slots, show_mix, show_held, disable_brew)
            .in_set(PanelSystems)
            .in_set(YarnSpinnerDialogueViewSystemSet)
            .run_if(any_with_component::<Concoction>),
    );
}

pub(super) fn spawn<'a>(
    commands: &'a mut Commands,
    options: Entity,
    inventory: &Inventory,
) -> EntityCommands<'a> {
    let held = inventory
        .plants
        .iter()
        .map(|(&plant, &count)| (plant, count));
    let mix_slots = (0..SLOTS)
        .map(|slot| -> Box<dyn Scene> { Box::new(mix_slot(slot)) })
        .collect::<Vec<_>>();
    commands.spawn_scene(bsn! {
        #Concoction
        ChildOf(options)
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(texels(5)),
            row_gap: texels(4),
        }
        @widget::panel()
        Children [
            Text("Concoction")
            @font::text(1)
            TextColor(HEADER_TEXT)
            Pickable::IGNORE
            --
            Node {
                column_gap: texels(2),
                align_items: AlignItems::Center,
            }
            Children [
                { mix_slots }
                --
                @widget::button("Brew", brew)
                BrewButton
            ]
            --
            @herbs::grid(held, ingredient)
        ]
    })
}

fn mark_color(mark: Mark) -> Color {
    match mark {
        Mark::Absent => LOSS_TEXT,
        Mark::Misplaced => CAUTION,
        Mark::Correct => GAIN_TEXT,
    }
}

#[derive(Component, Clone, Default)]
struct MixSlot(usize);

#[derive(Component, Clone, Default)]
struct MixIcon;

// the "score indicator"
#[derive(Component, Clone, Default)]
struct MixMark;

fn mix_slot(index: usize) -> impl Scene {
    bsn! {
        Node {
            width: texels(24),
            height: texels(24),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        @widget::panel()
        MixSlot(index)
        @on(drop_herb)
        @on(clear_slot)
        Children [
            MixMark
            @widget::panel_face()
            Visibility::Hidden
            Pickable::IGNORE
            --
            @herbs::icon(Plant::default())
            MixIcon
            Visibility::Hidden
        ]
    }
}

#[derive(Component, Clone, Default)]
struct Ingredient(Plant);

fn ingredient(plant: Plant, count: u32) -> Box<dyn Scene> {
    Box::new(bsn! {
        @herbs::slot(plant, count)
        Ingredient(plant)
        @on(start_drag)
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

fn start_drag(
    start: On<PointerDragStart>,
    mut commands: Commands,
    ingredients: Query<&Ingredient>,
    concoction: Single<&Concoction>,
    inventory: Single<&Inventory>,
    root: Single<Entity, With<DialogueRoot>>,
    ui_scale: Res<UiScale>,
) {
    let Ok(&Ingredient(plant)) = ingredients.get(start.entity) else {
        return;
    };
    if !concoction.mixing() || concoction.available(&inventory, plant) == 0 {
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

fn drop_herb(
    drop: On<PointerDragDrop>,
    slots: Query<&MixSlot>,
    ingredients: Query<&Ingredient>,
    mut concoction: Single<&mut Concoction>,
    inventory: Single<&Inventory>,
) {
    if let (Ok(&MixSlot(slot)), Ok(&Ingredient(plant))) =
        (slots.get(drop.entity), ingredients.get(drop.dropped))
    {
        concoction.put(slot, plant, &inventory);
    }
}

fn clear_slot(
    click: On<PointerClick>,
    slots: Query<&MixSlot>,
    mut concoction: Single<&mut Concoction>,
) {
    if let Ok(&MixSlot(slot)) = slots.get(click.entity) {
        concoction.clear(slot);
    }
}

#[derive(Component, Clone, Default)]
struct BrewButton;

fn brew(
    _: On<PointerClick>,
    mut concoction: Single<&mut Concoction>,
    remedies: Query<&Remedy>,
    mut inventory: Single<&mut Inventory>,
) {
    if let Ok(remedy) = remedies.get(concoction.visitor) {
        concoction.start_brewing(remedy, &mut inventory);
    }
}

fn pose_slots(concoction: Single<&Concoction>, mut slots: Query<(&MixSlot, &mut UiTransform)>) {
    for (&MixSlot(index), mut transform) in &mut slots {
        let pose = match concoction.brewing() {
            Some((slot, elapsed)) if slot == index => stamp_pose(elapsed),
            _ => UiTransform::IDENTITY,
        };
        if *transform != pose {
            *transform = pose;
        }
    }
}

fn stamp_pose(t: f32) -> UiTransform {
    let smooth = |k: f32| k * k * (3.0 - 2.0 * k);
    // lift phase
    let (height, scale, angle) = if t < LIFT {
        let k = smooth(t / LIFT);
        (k, 1.0 + (LIFTED_SCALE - 1.0) * k, 0.0)
    } else if t < LIFT + SHAKE {
        // shake phase
        let shaking = t - LIFT;
        let angle = (shaking * SHAKE_HZ * std::f32::consts::TAU).sin() * SHAKE_RADIANS;
        (1.0, LIFTED_SCALE, angle)
    } else if t < STAMP_AT {
        // stamp phase
        let k = (t - LIFT - SHAKE) / DROP;
        (
            1.0 - k * k,
            LIFTED_SCALE + (STAMPED_SCALE - LIFTED_SCALE) * k,
            0.0,
        )
    } else if t < STAMP_AT + SETTLE {
        // settling phase
        let k = smooth((t - STAMP_AT) / SETTLE);
        (0.0, STAMPED_SCALE + (1.0 - STAMPED_SCALE) * k, 0.0)
    } else {
        (0.0, 1.0, 0.0)
    };
    UiTransform {
        translation: Val2::px(0.0, -height * LIFT_TEXELS * TEXEL),
        scale: Vec2::splat(scale),
        rotation: Rot2::radians(angle),
    }
}

fn show_mix(
    concoction: Single<&Concoction, Changed<Concoction>>,
    slots: Query<(&MixSlot, &Children)>,
    mut faces: Query<(&mut ImageNode, &mut Visibility), With<MixMark>>,
    mut icons: Query<(&mut ImageNode, &mut Visibility), (With<MixIcon>, Without<MixMark>)>,
) {
    let marks: [Option<Mark>; SLOTS] = concoction.marks();
    for (&MixSlot(slot), children) in &slots {
        for &child in children {
            if let Ok((mut face, mut visibility)) = faces.get_mut(child) {
                match marks[slot] {
                    Some(mark) => {
                        face.color = mark_color(mark);
                        *visibility = Visibility::Inherited;
                    }
                    None => *visibility = Visibility::Hidden,
                }
            }
            if let Ok((mut icon, mut visibility)) = icons.get_mut(child) {
                match concoction.herb(slot) {
                    Some(plant) => {
                        icon.rect = Some(herbs::icon_rect(plant));
                        *visibility = Visibility::Inherited;
                    }
                    None => *visibility = Visibility::Hidden,
                }
            }
        }
    }
}

fn show_held(
    concoction: Single<&Concoction, Changed<Concoction>>,
    inventory: Single<&Inventory>,
    ingredients: Query<&Children, With<Ingredient>>,
    mut counts: Query<(&HeldCount, &mut Text)>,
) {
    for &child in ingredients.iter().flatten() {
        if let Ok((&HeldCount(plant), mut text)) = counts.get_mut(child) {
            text.0 = concoction.available(&inventory, plant).to_string();
        }
    }
}

fn disable_brew(
    mut commands: Commands,
    concoction: Single<&Concoction, Changed<Concoction>>,
    buttons: Query<&Children, With<BrewButton>>,
    mut faces: Query<&mut ImageNode, (With<InteractionPalette>, Without<InteractionDisabled>)>,
) {
    if concoction.mixing() {
        return;
    }
    for &face in buttons.iter().flatten() {
        if let Ok(mut image) = faces.get_mut(face) {
            image.color = BUTTON_DISABLED_TINT;
            commands.entity(face).insert(InteractionDisabled);
        }
    }
}
