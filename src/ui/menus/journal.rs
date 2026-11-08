//! renders, for every visitor the player has [`Met`]
//!
//! ```text
//! ui_root("Journal")                     // fills window, centers, column
//! └─ panel (column, padded)             // the single visible card
//!    ├─ heading (row)
//!    │  ├─ "Journal" title text          // as wide as the tabs and portrait
//!    │  └─ ailment (row)                 // over the health panel: name, time to its next worsening
//!    └─ content_row (row)
//!       ├─ tab_bar (column)
//!       │  └─ tab × N (one per met visitor)
//!       └─ portrait_view (column)
//!          ├─ top row (row)
//!          │  ├─ portrait_slot (panel → portrait image)
//!          │  ├─ health_panel (panel → 5 element bars, each with what the ailment will do to it)
//!          │  └─ history_panel (panel → "Brews" + brew entries)
//!          └─ description_panel (panel → name + text, spans the top row)
//! ```
//!

use bevy::prelude::*;
use bevy::ui::{UiGlobalTransform, UiSystems};
use bevy::ui_widgets::{ControlOrientation, ScrollArea, Scrollbar, ScrollbarThumb};
use bevy::window::PrimaryWindow;
use plants::Plant;

use crate::{
    gameplay::{
        Ailing, Ailment, Brew, BrewHistory, Died, Health, Mark, Met, Visitor,
        clock::Clock,
        herbs::{self, Element},
    },
    loading::NPCAssets,
    ui::{
        font,
        health::{ailment_countdown, ailment_mark, bar_width, health_bar_color},
        interaction::InteractionPalette,
        menus::Menu,
        palette::{
            BUTTON_HOVERED_TINT, BUTTON_PRESSED_TINT, CAUTION, GAIN_TEXT, HEADER_TEXT, LABEL_TEXT,
            LOSS_TEXT,
        },
        scale::{TEXEL, texels},
        widget,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Journal), spawn_journal_menu);
    app.add_systems(
        PostUpdate,
        (update_history_scrollbar, place_brew_tooltip).in_set(UiSystems::PostLayout),
    );
}

// All sizes are in texels;

// Portrait crop / frame geometry.
const CROP_WIDTH: u16 = 16;
const CROP_HEIGHT: u16 = 24;
const PORTRAIT_WIDTH: u16 = 64;
const PORTRAIT_HEIGHT: u16 = 96;
const PORTRAIT_SLOT_WIDTH: u16 = 72;
const PORTRAIT_SLOT_HEIGHT: u16 = 104;

// Tab bar.
pub(crate) const TAB_SIZE: u16 = 16; // the thumbnail inside a tab
pub(crate) const TAB_PANEL_SIZE: u16 = 24; // the clickable tab square

// Health panel: an element icon, its bar, and what the ailment will do to it.
const HEALTH_BAR_WIDTH: u16 = 72;
const HEALTH_BAR_HEIGHT: u16 = 6;
const AILMENT_MARK_WIDTH: u16 = 16;
const HEALTH_PANEL_WIDTH: u16 = 128;

const CONTENT_GAP: u16 = 4;
const AILMENT_INDENT: u16 = TAB_PANEL_SIZE + CONTENT_GAP + PORTRAIT_SLOT_WIDTH + COLUMN_GAP;

// Profile description panel: spans the whole top row below it.
const COLUMN_GAP: u16 = 4;
const TOP_ROW_WIDTH: u16 =
    PORTRAIT_SLOT_WIDTH + COLUMN_GAP + HEALTH_PANEL_WIDTH + COLUMN_GAP + HISTORY_PANEL_WIDTH;
const DESCRIPTION_PANEL_WIDTH: u16 = TOP_ROW_WIDTH;

// Brew history panel.
const HISTORY_PANEL_WIDTH: u16 = 128;
const BREW_SLOT_SIZE: u16 = 24;
const BREW_SLOT_GAP: u16 = 2;
const SCROLLBAR_WIDTH: u16 = 4;
const SCROLLBAR_MIN_THUMB: f32 = 8.0;

const LINE_HEIGHT: u16 = 16;

const CONTENT_HEIGHT: u16 = 192 - 2 * 5 - LINE_HEIGHT - 4;

const DESCRIPTION_PANEL_HEIGHT: u16 = CONTENT_HEIGHT - PORTRAIT_SLOT_HEIGHT - 2;

fn spawn_journal_menu(
    mut commands: Commands,
    assets: Res<NPCAssets>,
    clock: Single<&Clock>,
    met: Query<(&Visitor, &Health, &BrewHistory, Option<&Ailing>, Has<Died>), With<Met>>,
) {
    let mut met = met.iter().collect::<Vec<_>>();
    met.sort_by_key(|(visitor, ..)| visitor.0);

    // Build one tab per met visitor; the first becomes the initially shown portrait.
    let mut tabs = Vec::new();
    let mut first = None;
    for (index, (visitor, &health, history, ailing, died)) in met.into_iter().enumerate() {
        // the ailment, and the hours until it next worsens; the dead have none
        let ailment = ailing
            .filter(|_| !died)
            .map(|ailing| (ailing.ailment, ailing.next - clock.hours));
        let (image, frame) = assets.sprite(visitor.name());
        let top = portrait_top(visitor.name());
        let icon = centered_crop(frame, TAB_SIZE, TAB_SIZE, top);
        let portrait = centered_crop(frame, CROP_WIDTH, CROP_HEIGHT, top);
        if first.is_none() {
            first = Some((
                image.clone(),
                portrait,
                health,
                ailment,
                visitor.name(),
                visitor.description(),
                history,
            ));
        }
        tabs.push(tab(
            image.clone(),
            icon,
            portrait,
            health,
            ailment,
            visitor.name(),
            visitor.description(),
            history,
            index == 0,
        ));
    }

    // Everything inside the single card below the root.
    let mut body: Vec<Box<dyn Scene>> = Vec::new();
    match first {
        Some((image, portrait, health, ailment, name, description, history)) => {
            body.push(Box::new(heading(Some(ailment))));
            let content: Vec<Box<dyn Scene>> = vec![
                Box::new(tab_bar(tabs)),
                portrait_view(image, portrait, health, ailment, name, description, history),
            ];
            body.push(Box::new(content_row(content)));
        }
        None => body.push(Box::new(heading(None))),
    }

    commands.spawn_scene(journal_root(body));
}

/// "Journal", then, over the health panel, the shown visitor's ailment and the
/// hours until it next worsens. `None` where no visitor is shown at all.
fn heading(ailment: Option<Option<(Ailment, f32)>>) -> impl Scene {
    let ailment_row: Vec<Box<dyn Scene>> = ailment
        .map(|ailment| -> Box<dyn Scene> {
            let (name, hours) = ailment_texts(ailment);
            Box::new(bsn! {
                Node {
                    width: texels(HEALTH_PANEL_WIDTH),
                    column_gap: texels(6),
                    overflow: Overflow::visible(),
                }
                Pickable::IGNORE
                Children [
                    Text({ name })
                    @font::text(1)
                    TextColor(HEADER_TEXT)
                    TextLayout::no_wrap()
                    Node {
                        flex_shrink: 0.0,
                    }
                    JournalAilmentName
                    Pickable::IGNORE
                    --
                    Text({ hours })
                    @font::text(1)
                    TextColor(LABEL_TEXT)
                    TextLayout::no_wrap()
                    Node {
                        flex_shrink: 0.0,
                        margin: UiRect::left(auto()),
                    }
                    JournalAilmentCountdown
                    Pickable::IGNORE
                ]
            })
        })
        .into_iter()
        .collect();
    bsn! {
        Node {
            align_items: AlignItems::Center,
        }
        Pickable::IGNORE
        Children [
            Node {
                width: texels(AILMENT_INDENT),
                flex_shrink: 0.0,
            }
            Pickable::IGNORE
            Children [
                Text("Journal")
                @font::text(1)
                TextColor(HEADER_TEXT)
                Pickable::IGNORE
            ]
            --
            { ailment_row }
        ]
    }
}

/// The ailment's name and the hours until it next worsens, or blanks.
fn ailment_texts(ailment: Option<(Ailment, f32)>) -> (String, String) {
    ailment.map_or_else(Default::default, |(ailment, hours)| {
        (ailment.name(), ailment_countdown(hours))
    })
}

fn journal_root(body: Vec<Box<dyn Scene>>) -> impl Scene {
    bsn! {
        @widget::ui_root("Journal")
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Journal)
        Children [
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(texels(5)),
                row_gap: texels(4),
                height: percent(80),
            }
            @widget::panel()
            Children [ { body } ]
            --
            JournalTooltip
            Node {
                position_type: PositionType::Absolute,
            }
            Visibility::Hidden
            GlobalZIndex(3)
            Pickable::IGNORE
        ]
    }
}

#[derive(Component, Clone, Default)]
struct JournalTooltip {
    slot: Option<Rect>,
    placed: bool,
}

#[derive(Component, Clone, Default)]
struct BrewHerb(Plant);

fn show_brew_tooltip(
    over: On<PointerOver>,
    mut commands: Commands,
    herbs: Query<(&BrewHerb, &ComputedNode, &UiGlobalTransform)>,
    tooltip: Single<(Entity, &mut JournalTooltip, &mut Visibility)>,
) {
    let Ok((&BrewHerb(plant), node, at)) = herbs.get(over.entity) else {
        return;
    };
    let (entity, mut tooltip, mut shown) = tooltip.into_inner();
    let scale = node.inverse_scale_factor();
    tooltip.slot = Some(Rect::from_center_size(
        at.translation * scale,
        node.size() * scale,
    ));
    tooltip.placed = false;
    *shown = Visibility::Hidden;
    commands.entity(entity).despawn_children();
    commands
        .spawn_scene(herbs::card(plant))
        .insert(ChildOf(entity));
}

fn hide_brew_tooltip(_: On<PointerOut>, tooltip: Single<(&mut JournalTooltip, &mut Visibility)>) {
    let (mut tooltip, mut shown) = tooltip.into_inner();
    tooltip.slot = None;
    *shown = Visibility::Hidden;
}

fn place_brew_tooltip(
    window: Single<&Window, With<PrimaryWindow>>,
    tooltip: Single<(
        &mut JournalTooltip,
        &mut Node,
        &ComputedNode,
        &mut Visibility,
    )>,
) {
    let (mut tooltip, mut node, size, mut shown) = tooltip.into_inner();
    let Some(slot) = tooltip.slot else {
        return;
    };
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

/// for portraits with headroom
pub(crate) fn centered_crop(frame: UVec2, width: u16, height: u16, top: u32) -> Rect {
    let crop = UVec2::new(width.into(), height.into());
    let min = UVec2::new(
        frame.x.saturating_sub(crop.x) / 2,
        top.min(frame.y.saturating_sub(crop.y)),
    );
    URect::from_corners(min, min + crop).as_rect()
}

/// Orphan and Old Beauty are drawn with extra headroom in their frames.
pub(crate) fn portrait_top(name: &str) -> u32 {
    match name {
        "Orphan" => 8,
        "Old Beauty" => 6,
        _ => 0,
    }
}

/// Data copied onto a tab so [`select_journal_tab`] can swap the details pane.
#[derive(Component, FromTemplate)]
struct JournalTab {
    image: Handle<Image>,
    portrait: Rect,
    name: &'static str,
    description: &'static str,
}

/// Snapshots
#[derive(Component, Clone, Default)]
struct JournalHealth(Health);

/// The ailment and the hours until it next worsens, as the journal opened.
#[derive(Component, Clone, Default)]
struct JournalAilment(Option<(Ailment, f32)>);

#[derive(Component, Clone, Default)]
struct JournalAilmentName;

#[derive(Component, Clone, Default)]
struct JournalAilmentCountdown;

/// What the ailment will do to one element, beside its bar.
#[derive(Component, Clone, Copy, Default)]
struct JournalAilmentMark(Element);

#[derive(Component, Clone, Default)]
struct JournalHistory(BrewHistory);

#[derive(Component, Clone, Default)]
struct JournalPortrait;

#[derive(Component, Clone, Default)]
struct JournalName;

#[derive(Component, Clone, Default)]
struct JournalDescription;

#[derive(Component, Clone, Default)]
struct JournalHistoryRoot;

#[derive(Component, Clone, Default)]
struct JournalHistoryScrollbar;

#[derive(Component, Clone, Default)]
struct JournalTabSelected;

#[derive(Component, Clone, Copy, Default)]
struct JournalHealthFill(Element);

fn select_journal_tab(
    click: On<PointerClick>,
    mut commands: Commands,
    tabs: Query<&JournalTab>,
    healths: Query<&JournalHealth>,
    ailments: Query<&JournalAilment>,
    histories: Query<&JournalHistory>,
    mut portraits: Query<&mut ImageNode, With<JournalPortrait>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<JournalName>>,
        Query<&mut Text, With<JournalDescription>>,
        Query<&mut Text, With<JournalAilmentName>>,
        Query<&mut Text, With<JournalAilmentCountdown>>,
        Query<(&mut Text, &JournalAilmentMark)>,
    )>,
    roots: Query<Entity, With<JournalHistoryRoot>>,
    mut scrolls: Query<&mut ScrollPosition, With<JournalHistoryRoot>>,
    mut faces: Query<(&JournalTabSelected, &ChildOf, &mut Visibility)>,
    mut fills: Query<(&JournalHealthFill, &mut Node, &mut BackgroundColor)>,
) {
    let Ok(tab) = tabs.get(click.entity) else {
        return;
    };

    // Swap the portrait image.
    let Ok(mut image) = portraits.single_mut() else {
        return;
    };
    image.image = tab.image.clone();
    image.rect = Some(tab.portrait);

    // Swap the visitor name.
    if let Ok(mut name) = texts.p0().single_mut() {
        name.0 = tab.name.to_string();
    }

    // Swap the profile text.
    if let Ok(mut description) = texts.p1().single_mut() {
        description.0 = tab.description.to_string();
    }

    // Swap the ailment, its countdown, and what it will do to each element.
    if let Ok(&JournalAilment(ailment)) = ailments.get(click.entity) {
        let (name, hours) = ailment_texts(ailment);
        if let Ok(mut text) = texts.p2().single_mut() {
            text.0 = name;
        }
        if let Ok(mut text) = texts.p3().single_mut() {
            text.0 = hours;
        }
        for (mut text, &JournalAilmentMark(element)) in &mut texts.p4() {
            text.0 = ailment
                .map(|(ailment, _)| ailment_mark(ailment, element))
                .unwrap_or_default();
        }
    }

    // Refresh the element bars.
    if let Ok(&JournalHealth(health)) = healths.get(click.entity) {
        for (fill, mut node, mut background) in &mut fills {
            let value = health.get(fill.0);
            node.width = percent(bar_width(value));
            *background = BackgroundColor(health_bar_color(value));
        }
    }

    // Rebuild the brew history list.
    if let Ok(JournalHistory(history)) = histories.get(click.entity)
        && let Ok(root) = roots.single()
    {
        commands.entity(root).despawn_children();
        for entry in history_entries(history) {
            commands.spawn_scene(entry).insert(ChildOf(root));
        }
        // New history starts scrolled back to the top.
        if let Ok(mut scroll) = scrolls.single_mut() {
            scroll.0 = Vec2::ZERO;
        }
    }

    // Only the clicked tab keeps its highlight face.
    for (_face, parent, mut visibility) in &mut faces {
        *visibility = if parent.0 == click.entity {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Show the brew history scrollbar only when the list actually overflows.
fn update_history_scrollbar(
    scrollbars: Query<(Entity, &Scrollbar), With<JournalHistoryScrollbar>>,
    scroll_areas: Query<&ComputedNode>,
    mut visibility: Query<&mut Visibility>,
) {
    for (entity, scrollbar) in &scrollbars {
        let Ok(computed) = scroll_areas.get(scrollbar.target) else {
            continue;
        };

        // `content_size` and `size` are both in physical pixels, so we can compare them directly.
        let needed = computed.content_size().y > computed.size().y + 0.5;
        let wanted = if needed {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        if let Ok(mut vis) = visibility.get_mut(entity)
            && *vis != wanted
        {
            *vis = wanted;
        }
    }
}

fn content_row(children: Vec<Box<dyn Scene>>) -> impl Scene {
    bsn! {
        Node {
            column_gap: texels(CONTENT_GAP),
            align_items: AlignItems::FlexStart,
        }
        Pickable::IGNORE
        Children [ { children } ]
    }
}

fn tab_bar(tabs: Vec<Box<dyn Scene>>) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: texels(2),
        }
        Children [ { tabs } ]
    }
}

fn tab(
    image: Handle<Image>,
    icon: Rect,
    portrait: Rect,
    health: Health,
    ailment: Option<(Ailment, f32)>,
    name: &'static str,
    description: &'static str,
    history: &BrewHistory,
    selected: bool,
) -> Box<dyn Scene> {
    let content_image = image.clone();
    let face_visibility = if selected {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    Box::new(bsn! {
        Node {
            width: texels(TAB_PANEL_SIZE),
            height: texels(TAB_PANEL_SIZE),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        @widget::panel()
        InteractionPalette {
            none: Color::WHITE,
            hovered: BUTTON_HOVERED_TINT,
            pressed: BUTTON_PRESSED_TINT,
        }
        JournalTab {
            image: { content_image },
            portrait: { portrait },
            name: { name },
            description: { description },
        }
        JournalHealth(health)
        JournalAilment({ ailment })
        JournalHistory({ history.clone() })
        @on(select_journal_tab)
        Children [
            @widget::panel_face_color(BUTTON_HOVERED_TINT)
            JournalTabSelected
            ~{ face_visibility }
            --
            @tab_icon(image, icon)
        ]
    })
}

pub(crate) fn tab_icon(image: Handle<Image>, rect: Rect) -> impl Scene {
    bsn! {
        Node {
            width: texels(TAB_SIZE),
            height: texels(TAB_SIZE),
        }
        ImageNode {
            image: { image },
            rect: { Some(rect) },
        }
        Pickable::IGNORE
    }
}

/// top row: portrait, health, and brew history
fn portrait_view(
    image: Handle<Image>,
    rect: Rect,
    health: Health,
    ailment: Option<(Ailment, f32)>,
    name: &'static str,
    description: &'static str,
    history: &BrewHistory,
) -> Box<dyn Scene> {
    Box::new(bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: texels(2),
            align_items: AlignItems::FlexStart,
        }
        Pickable::IGNORE
        Children [
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: texels(COLUMN_GAP),
                align_items: AlignItems::FlexStart,
            }
            Pickable::IGNORE
            Children [
                @portrait_slot(image, rect)
                --
                @health_panel(health, ailment.map(|(ailment, _)| ailment))
                --
                @history_panel(history)
            ]
            --
            @description_panel(name, description)
        ]
    })
}

/// The framed portrait image.
fn portrait_slot(image: Handle<Image>, rect: Rect) -> impl Scene {
    bsn! {
        Node {
            width: texels(PORTRAIT_SLOT_WIDTH),
            height: texels(PORTRAIT_SLOT_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        @widget::panel()
        Pickable::IGNORE
        Children [
            Node {
                width: texels(PORTRAIT_WIDTH),
                height: texels(PORTRAIT_HEIGHT),
            }
            JournalPortrait
            ImageNode {
                image: { image },
                rect: { Some(rect) },
            }
            Pickable::IGNORE
        ]
    }
}

fn description_panel(name: &'static str, text: &'static str) -> impl Scene {
    bsn! {
        Node {
            width: texels(DESCRIPTION_PANEL_WIDTH),
            height: texels(DESCRIPTION_PANEL_HEIGHT),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: texels(1),
            padding: UiRect::new(texels(4), texels(4), texels(3), texels(3)),
            overflow: Overflow::clip(),
        }
        @widget::panel()
        Pickable::IGNORE
        Children [
            Text({ name.to_string() })
            @font::text(1)
            TextColor(HEADER_TEXT)
            JournalName
            Pickable::IGNORE
            --
            Text({ text.to_string() })
            @font::text(1)
            TextColor(LABEL_TEXT)
            Node {
                max_width: percent(100),
            }
            JournalDescription
            Pickable::IGNORE
        ]
    }
}

fn history_panel(history: &BrewHistory) -> impl Scene {
    let entries = history_entries(history);
    bsn! {
        Node {
            width: texels(HISTORY_PANEL_WIDTH),
            height: texels(PORTRAIT_SLOT_HEIGHT),
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
            Pickable::IGNORE
            --
            // Scroll area beside its scrollbar.
            Node {
                display: Display::Grid,
                flex_grow: 1.0,
                grid_template_columns: vec![
                    RepeatedGridTrack::flex(1, 1.0),
                    RepeatedGridTrack::auto(1),
                ],
                grid_template_rows: vec![RepeatedGridTrack::flex(1, 1.0)],
                column_gap: texels(1),
                padding: UiRect{
                    right: texels(3),
                    bottom: texels(3),
                },
                align_items: AlignItems::Stretch,
            }
            Pickable::IGNORE
            Children [
                // The scrollable brew list.
                #JournalHistoryScroll
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: texels(BREW_SLOT_GAP),
                    overflow: Overflow::scroll_y(),
                }
                ScrollArea
                JournalHistoryRoot
                Children [ { entries } ]
                --
                // The scrollbar track and draggable thumb.
                Node {
                    min_width: texels(SCROLLBAR_WIDTH),
                }
                BackgroundColor(BUTTON_PRESSED_TINT)
                JournalHistoryScrollbar
                Scrollbar {
                    target: #JournalHistoryScroll,
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

fn history_entries(history: &BrewHistory) -> Vec<Box<dyn Scene>> {
    history.brews.iter().map(brew_entry).collect()
}

fn brew_entry(brew: &Brew) -> Box<dyn Scene> {
    let slots: Vec<Box<dyn Scene>> = brew
        .mix
        .iter()
        .zip(brew.marks.iter())
        .map(|(&plant, &mark)| brew_slot(plant, mark))
        .collect();
    Box::new(bsn! {
        Node {
            column_gap: texels(BREW_SLOT_GAP),
        }
        Pickable::IGNORE
        Children [ { slots } ]
    })
}

fn brew_slot(plant: Option<Plant>, mark: Mark) -> Box<dyn Scene> {
    let mut children: Vec<Box<dyn Scene>> = vec![Box::new(bsn! {
        @widget::panel_face_color(mark_color(mark))
    })];
    let Some(plant) = plant else {
        return Box::new(bsn! {
            @brew_slot_frame()
            Pickable::IGNORE
            Children [ { children } ]
        });
    };
    children.push(Box::new(herbs::icon(plant)));
    Box::new(bsn! {
        @brew_slot_frame()
        BrewHerb(plant)
        @on(show_brew_tooltip)
        @on(hide_brew_tooltip)
        Children [ { children } ]
    })
}

fn brew_slot_frame() -> impl Scene {
    bsn! {
        Node {
            width: texels(BREW_SLOT_SIZE),
            height: texels(BREW_SLOT_SIZE),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        @widget::panel()
    }
}

fn mark_color(mark: Mark) -> Color {
    match mark {
        Mark::Absent => LOSS_TEXT,
        Mark::Misplaced => CAUTION,
        Mark::Correct => GAIN_TEXT,
    }
}

fn health_panel(health: Health, ailment: Option<Ailment>) -> impl Scene {
    bsn! {
        Node {
            width: texels(HEALTH_PANEL_WIDTH),
            height: texels(PORTRAIT_SLOT_HEIGHT),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: texels(3),
            padding: UiRect::new(texels(4), texels(4), texels(8), texels(4)),
        }
        @widget::panel()
        Pickable::IGNORE
        Children [
            { health_rows(health, ailment) }
        ]
    }
}

fn health_rows(health: Health, ailment: Option<Ailment>) -> Vec<Box<dyn Scene>> {
    Element::ALL
        .into_iter()
        .map(|element| {
            let mark = ailment
                .map(|ailment| ailment_mark(ailment, element))
                .unwrap_or_default();
            health_row(element, health.get(element), mark)
        })
        .collect()
}

fn health_row(element: Element, value: u8, mark: String) -> Box<dyn Scene> {
    let width = bar_width(value);
    Box::new(bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            column_gap: texels(3),
        }
        Pickable::IGNORE
        Children [
            { vec![herbs::element_icon(element)] }
            --
            Node {
                width: texels(HEALTH_BAR_WIDTH),
                height: texels(HEALTH_BAR_HEIGHT),
                flex_shrink: 0.0,
            }
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5))
            Pickable::IGNORE
            Children [
                Node {
                    width: percent(width),
                    height: percent(100),
                }
                JournalHealthFill(element)
                BackgroundColor(health_bar_color(value))
                Pickable::IGNORE
            ]
            --
            Text({ mark })
            @font::text(1)
            TextColor(LOSS_TEXT)
            Node {
                width: texels(AILMENT_MARK_WIDTH),
                flex_shrink: 0.0,
            }
            JournalAilmentMark(element)
            Pickable::IGNORE
        ]
    })
}
