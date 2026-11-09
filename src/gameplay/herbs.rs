use bevy::{prelude::*, ui::UiGlobalTransform, window::PrimaryWindow};
use plants::Plant;

use crate::ui::{
    font,
    palette::{BUTTON_TEXT, GAIN_TEXT, HEADER_TEXT, LOSS_TEXT},
    scale::{TEXEL, texels},
    widget,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Element {
    #[default]
    Wood,
    Fire,
    Earth,
    Metal,
    Water,
}

impl Element {
    pub(crate) const ALL: [Element; 5] = [
        Element::Wood,
        Element::Fire,
        Element::Earth,
        Element::Metal,
        Element::Water,
    ];

    pub(crate) fn next(self) -> Element {
        match self {
            Element::Wood => Element::Fire,
            Element::Fire => Element::Earth,
            Element::Earth => Element::Metal,
            Element::Metal => Element::Water,
            Element::Water => Element::Wood,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Element::Wood => "Wood",
            Element::Fire => "Fire",
            Element::Earth => "Earth",
            Element::Metal => "Metal",
            Element::Water => "Water",
        }
    }

    pub(crate) fn icon_index(self) -> u32 {
        match self {
            Element::Fire => 0,
            Element::Water => 1,
            Element::Wood => 2,
            Element::Metal => 3,
            Element::Earth => 4,
        }
    }
}

pub(crate) fn name(plant: Plant) -> &'static str {
    match plant {
        Plant::Basil => "Basil",
        Plant::Chili => "Chili",
        Plant::Lemongrass => "Lemongrass",
        Plant::Ginseng => "Ginseng",
        Plant::Chrysanthemum => "Chrysanthemum",
        Plant::Wolfberry => "Wolfberry",
        Plant::Lotus => "Lotus",
        Plant::Licorice => "Licorice",
        Plant::RedMushroom => "Red Mushroom",
        Plant::Grapes => "Grapes",
    }
}

// +2 an element and -1 the next (a tonic), or -2 and +1 (a cooling herb)
#[derive(Clone, Copy)]
pub(crate) struct Effect {
    pub(crate) element: Element,
    pub(crate) by: i8,
}

impl Effect {
    pub(crate) fn shifts(self) -> [(Element, i8); 2] {
        [(self.element, self.by), (self.element.next(), -self.by / 2)]
    }
}

pub(crate) fn effect(plant: Plant) -> Effect {
    let (element, by) = match plant {
        Plant::Basil => (Element::Wood, 2),
        Plant::Chili => (Element::Fire, 2),
        Plant::Ginseng => (Element::Earth, 2),
        Plant::Lotus => (Element::Metal, 2),
        Plant::Wolfberry => (Element::Water, 2),
        Plant::Lemongrass => (Element::Wood, -2),
        Plant::Chrysanthemum => (Element::Fire, -2),
        Plant::Licorice => (Element::Earth, -2),
        Plant::RedMushroom => (Element::Metal, -2),
        Plant::Grapes => (Element::Water, -2),
    };
    Effect { element, by }
}

pub(crate) fn icon_rect(plant: Plant) -> Rect {
    pixel_art::grid_rect(
        plant.picked_cell(),
        plants::COLUMNS,
        UVec2::splat(plants::CELL),
    )
    .as_rect()
}

pub(crate) fn icon(plant: Plant) -> impl Scene {
    let rect = icon_rect(plant);
    bsn! {
        Node {
            width: texels(16),
            height: texels(16),
        }
        ImageNode {
            image: plants::SHEET,
            rect: { Some(rect) },
        }
        Pickable::IGNORE
    }
}

// cell indices in the sheet, in the order: Fire, Water, Wood, Metal, Earth
const ELEMENT_SHEET: &str = "ui/Element Icons.png";
const ELEMENT_TILE: u32 = 16;
const ELEMENT_COLUMNS: u32 = 3;

pub(crate) fn element_icon(element: Element) -> impl Scene {
    let rect = pixel_art::grid_rect(
        element.icon_index(),
        ELEMENT_COLUMNS,
        UVec2::splat(ELEMENT_TILE),
    )
    .as_rect();
    bsn! {
        Node {
            width: texels(16),
            height: texels(16),
        }
        ImageNode {
            image: ELEMENT_SHEET,
            rect: { Some(rect) },
        }
        Pickable::IGNORE
    }
}

const SLOT: u16 = 24;
const GRID_COLUMN_GAP: u16 = 2;
const GRID_ROW_GAP: u16 = 4;

pub(crate) fn grid_height(rows: u16) -> Val {
    texels(rows * SLOT + rows.saturating_sub(1) * GRID_ROW_GAP)
}

#[derive(Component, Clone, Default)]
pub(crate) struct HeldCount(pub(crate) Plant);

pub(crate) fn slot(plant: Plant, count: u32) -> impl Scene {
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
            @icon(plant)
            --
            Text({ count.to_string() })
            HeldCount(plant)
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
            Children [ @card(plant) ]
        ]
    }
}

// the actual inventory grid reused between inv screen and concoct screen
pub(crate) fn grid(
    held: impl IntoIterator<Item = (Plant, u32)>,
    slot: fn(Plant, u32) -> Box<dyn Scene>,
) -> impl Scene {
    let mut held = held
        .into_iter()
        .filter(|&(_, count)| count > 0)
        .collect::<Vec<_>>();
    held.sort();
    let slots = held
        .into_iter()
        .map(|(plant, count)| slot(plant, count))
        .collect::<Vec<_>>();
    let columns: Vec<RepeatedGridTrack> =
        RepeatedGridTrack::px(GRID_COLUMNS, f32::from(SLOT) * TEXEL);
    bsn! {
        Node {
            display: Display::Grid,
            grid_template_columns: columns,
            row_gap: texels(GRID_ROW_GAP),
            column_gap: texels(GRID_COLUMN_GAP),
        }
        Children [ { slots } ]
    }
}

const GRID_COLUMNS: u16 = 4;

#[derive(Component, Clone, Default)]
struct Tooltip;

fn show_tooltip(
    over: On<PointerOver>,
    window: Single<&Window, With<PrimaryWindow>>,
    slots: Query<(&Children, &ComputedNode, &UiGlobalTransform)>,
    mut tooltips: Query<(&mut Visibility, &mut Node, &ComputedNode), With<Tooltip>>,
) {
    let Ok((children, slot, at)) = slots.get(over.entity) else {
        return;
    };
    let slot_right = at.translation.x + slot.size().x / 2.0;
    for child in children {
        let Ok((mut visibility, mut node, tooltip)) = tooltips.get_mut(*child) else {
            continue;
        };
        let gap = 2.0 * TEXEL / tooltip.inverse_scale_factor();
        let fits = slot_right + gap + tooltip.size().x <= window.physical_width() as f32;
        (node.left, node.right) = if fits {
            (percent(100), Val::Auto)
        } else {
            (Val::Auto, percent(100))
        };
        node.margin = if fits {
            UiRect::left(texels(2))
        } else {
            UiRect::right(texels(2))
        };
        *visibility = Visibility::Inherited;
    }
}

fn hide_tooltip(
    out: On<PointerOut>,
    children: Query<&Children>,
    mut tooltips: Query<&mut Visibility, With<Tooltip>>,
) {
    for child in children.get(out.entity).into_iter().flatten() {
        if let Ok(mut tooltip) = tooltips.get_mut(*child) {
            *tooltip = Visibility::Hidden;
        }
    }
}

// never pickable, so it can't block what's under it
pub(crate) fn card(plant: Plant) -> impl Scene {
    let rows: Vec<Box<dyn Scene>> = effect(plant)
        .shifts()
        .into_iter()
        .map(|(element, amount)| -> Box<dyn Scene> {
            let color = if amount > 0 { GAIN_TEXT } else { LOSS_TEXT };
            Box::new(bsn! {
                Node {
                    column_gap: texels(4),
                }
                Pickable::IGNORE
                Children [
                    { vec![element_icon(element)] }
                    --
                    Text({ format!("{amount:+}") })
                    @font::text(1)
                    TextColor(color)
                    Pickable::IGNORE
                ]
            })
        })
        .collect();

    bsn! {
        #HerbCard
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(texels(5)),
        }
        @widget::panel()
        Pickable::IGNORE
        Children [
            Text({ name(plant).to_string() })
            @font::text(1)
            TextColor(HEADER_TEXT)
            Node {
                margin: UiRect::bottom(texels(2)),
            }
            Pickable::IGNORE
            --
            { rows }
        ]
    }
}
