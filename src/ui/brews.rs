use bevy::{
    prelude::*,
    ui::{UiGlobalTransform, UiSystems},
    ui_widgets::{ControlOrientation, ScrollArea, Scrollbar, ScrollbarThumb},
    window::PrimaryWindow,
};
use plants::Plant;

use crate::{
    gameplay::{Brew, BrewHistory, Mark, herbs},
    ui::{
        font,
        palette::{
            BUTTON_HOVERED_TINT, BUTTON_PRESSED_TINT, CAUTION, GAIN_TEXT, HEADER_TEXT, LOSS_TEXT,
        },
        scale::{TEXEL, texels},
        widget,
    },
};

const SLOT_SIZE: u16 = 24;
const SLOT_GAP: u16 = 2;
const ROW_GAP: u16 = 4;
const SCROLLBAR_WIDTH: u16 = 4;
const SCROLLBAR_MIN_THUMB: f32 = 8.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_tooltip)
        .add_systems(Update, follow_histories)
        .add_systems(
            PostUpdate,
            (show_scrollbars, place_tooltip).in_set(UiSystems::PostLayout),
        );
}

#[derive(Component, Clone, Default)]
pub(crate) struct BrewList;

#[derive(Component, Clone)]
pub(crate) struct BrewListOf(pub(crate) Entity);

impl Default for BrewListOf {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

#[derive(Component, Clone, Default)]
struct BrewListScrollbar;

pub(crate) fn panel(history: &BrewHistory) -> impl Scene {
    let entries = entries(history);
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: texels(1),
            padding: UiRect::all(texels(4)),
            overflow: Overflow::clip(),
        }
        @widget::panel()
        Pickable::IGNORE
        Children [
            Text("Brews")
            @font::text(1)
            TextColor(HEADER_TEXT)
            Node {
                margin: UiRect::left(texels(2)),
            }
            Pickable::IGNORE
            --
            Node {
                display: Display::Grid,
                flex_grow: 1.0,
                grid_template_columns: vec![
                    RepeatedGridTrack::flex(1, 1.0),
                    RepeatedGridTrack::auto(1),
                ],
                grid_template_rows: vec![RepeatedGridTrack::flex(1, 1.0)],
                column_gap: texels(1),
                padding: UiRect {
                    right: texels(3),
                    bottom: texels(3),
                },
                align_items: AlignItems::Stretch,
            }
            Pickable::IGNORE
            Children [
                #BrewListScroll
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: texels(ROW_GAP),
                    overflow: Overflow::scroll_y(),
                }
                ScrollArea
                BrewList
                Children [ { entries } ]
                --
                Node {
                    min_width: texels(SCROLLBAR_WIDTH),
                }
                BackgroundColor(BUTTON_PRESSED_TINT)
                BrewListScrollbar
                Scrollbar {
                    target: #BrewListScroll,
                    orientation: ControlOrientation::Vertical,
                    min_thumb_length: SCROLLBAR_MIN_THUMB,
                }
                Children [
                    ScrollbarThumb {
                        border_radius: BorderRadius::all(texels(1)),
                        border: UiRect::ZERO,
                    }
                    BackgroundColor(BUTTON_HOVERED_TINT)
                ]
            ]
        ]
    }
}

pub(crate) fn refill(commands: &mut Commands, list: Entity, history: &BrewHistory) {
    commands.entity(list).despawn_children();
    for entry in entries(history) {
        commands.spawn_scene(entry).insert(ChildOf(list));
    }
}

fn entries(history: &BrewHistory) -> Vec<Box<dyn Scene>> {
    history.brews.iter().map(entry).collect()
}

fn entry(brew: &Brew) -> Box<dyn Scene> {
    let slots: Vec<Box<dyn Scene>> = brew
        .mix
        .iter()
        .zip(brew.marks.iter())
        .map(|(&plant, &mark)| slot(plant, mark))
        .collect();
    Box::new(bsn! {
        Node {
            column_gap: texels(SLOT_GAP),
        }
        Pickable::IGNORE
        Children [ { slots } ]
    })
}

fn slot(plant: Option<Plant>, mark: Mark) -> Box<dyn Scene> {
    let mut children: Vec<Box<dyn Scene>> = vec![Box::new(bsn! {
        @widget::panel_face_color(mark_color(mark))
    })];
    let Some(plant) = plant else {
        return Box::new(bsn! {
            @slot_frame()
            Pickable::IGNORE
            Children [ { children } ]
        });
    };
    children.push(Box::new(herbs::icon(plant)));
    Box::new(bsn! {
        @slot_frame()
        BrewHerb(plant)
        @on(show_tooltip)
        @on(hide_tooltip)
        Children [ { children } ]
    })
}

fn slot_frame() -> impl Scene {
    bsn! {
        Node {
            width: texels(SLOT_SIZE),
            height: texels(SLOT_SIZE),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        @widget::panel()
    }
}

pub(crate) fn mark_color(mark: Mark) -> Color {
    match mark {
        Mark::Absent => LOSS_TEXT,
        Mark::Misplaced => CAUTION,
        Mark::Correct => GAIN_TEXT,
    }
}

fn follow_histories(
    mut commands: Commands,
    panels: Query<(Entity, &BrewListOf)>,
    children: Query<&Children>,
    lists: Query<(), With<BrewList>>,
    histories: Query<Ref<BrewHistory>>,
) {
    for (panel, &BrewListOf(visitor)) in &panels {
        let Ok(history) = histories.get(visitor) else {
            continue;
        };
        if !history.is_changed() {
            continue;
        }
        if let Some(list) = children
            .iter_descendants(panel)
            .find(|&entity| lists.contains(entity))
        {
            refill(&mut commands, list, &history);
        }
    }
}

fn show_scrollbars(
    scrollbars: Query<(Entity, &Scrollbar), With<BrewListScrollbar>>,
    scroll_areas: Query<&ComputedNode>,
    mut visibility: Query<&mut Visibility>,
) {
    for (entity, scrollbar) in &scrollbars {
        let Ok(computed) = scroll_areas.get(scrollbar.target) else {
            continue;
        };
        let wanted = if computed.content_size().y > computed.size().y + 0.5 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if let Ok(mut shown) = visibility.get_mut(entity) {
            shown.set_if_neq(wanted);
        }
    }
}

#[derive(Component, Clone, Default)]
struct BrewTooltip {
    slot: Option<(Entity, Rect)>,
    placed: bool,
}

#[derive(Component, Clone, Default)]
struct BrewHerb(Plant);

fn spawn_tooltip(mut commands: Commands) {
    commands.spawn((
        Name::new("Brew Tooltip"),
        BrewTooltip::default(),
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        Visibility::Hidden,
        GlobalZIndex(10),
        Pickable::IGNORE,
    ));
}

fn show_tooltip(
    over: On<PointerOver>,
    mut commands: Commands,
    herbs: Query<(&BrewHerb, &ComputedNode, &UiGlobalTransform)>,
    tooltip: Single<(Entity, &mut BrewTooltip, &mut Visibility)>,
) {
    let Ok((&BrewHerb(plant), node, at)) = herbs.get(over.entity) else {
        return;
    };
    let (entity, mut tooltip, mut shown) = tooltip.into_inner();
    let scale = node.inverse_scale_factor();
    tooltip.slot = Some((
        over.entity,
        Rect::from_center_size(at.translation * scale, node.size() * scale),
    ));
    tooltip.placed = false;
    *shown = Visibility::Hidden;
    commands.entity(entity).despawn_children();
    commands
        .spawn_scene(herbs::card(plant))
        .insert(ChildOf(entity));
}

fn hide_tooltip(_: On<PointerOut>, tooltip: Single<(&mut BrewTooltip, &mut Visibility)>) {
    let (mut tooltip, mut shown) = tooltip.into_inner();
    tooltip.slot = None;
    *shown = Visibility::Hidden;
}

fn place_tooltip(
    window: Single<&Window, With<PrimaryWindow>>,
    herbs: Query<(), With<BrewHerb>>,
    tooltip: Single<(&mut BrewTooltip, &mut Node, &ComputedNode, &mut Visibility)>,
) {
    let (mut tooltip, mut node, size, mut shown) = tooltip.into_inner();
    let Some((herb, slot)) = tooltip.slot else {
        return;
    };
    if !herbs.contains(herb) {
        tooltip.slot = None;
        *shown = Visibility::Hidden;
        return;
    }
    if tooltip.placed {
        shown.set_if_neq(Visibility::Inherited);
        return;
    }
    if size.size().x <= 0.0 {
        return;
    }
    let scale = size.inverse_scale_factor();
    let width = size.size().x * scale;
    let gap = 2.0 * TEXEL;
    let left = if slot.max.x + gap + width <= window.physical_width() as f32 * scale {
        slot.max.x + gap
    } else {
        slot.min.x - gap - width
    };
    node.left = px(left);
    node.top = px(slot.min.y);
    tooltip.placed = true;
}
