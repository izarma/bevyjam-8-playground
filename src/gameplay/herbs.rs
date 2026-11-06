use bevy::prelude::*;
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

    pub(crate) fn name(self) -> &'static str {
        match self {
            Element::Wood => "Wood",
            Element::Fire => "Fire",
            Element::Earth => "Earth",
            Element::Metal => "Metal",
            Element::Water => "Water",
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
        Plant::BlueHibiscus => "Blue Hibiscus",
    }
}

// placeholder values
pub(crate) fn yields(plant: Plant) -> &'static [(Element, i8)] {
    match plant {
        Plant::Basil => &[(Element::Wood, 2), (Element::Earth, -1)],
        Plant::Chili => &[(Element::Fire, 2), (Element::Metal, -1)],
        Plant::Lemongrass => &[(Element::Metal, 1), (Element::Wood, -1)],
        Plant::Ginseng => &[(Element::Earth, 2), (Element::Water, -1)],
        Plant::Chrysanthemum => &[(Element::Water, 1), (Element::Fire, -2)],
        Plant::Wolfberry => &[(Element::Water, 2), (Element::Fire, -1)],
        Plant::Lotus => &[(Element::Metal, 2), (Element::Earth, -1)],
        Plant::Licorice => &[(Element::Earth, 1), (Element::Wood, -1)],
        Plant::BlueHibiscus => &[(Element::Wood, 1), (Element::Metal, -1)],
    }
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

const SLOT: u16 = 24;

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
            row_gap: texels(2),
            column_gap: texels(2),
        }
        Children [ { slots } ]
    }
}

const GRID_COLUMNS: u16 = 4;

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

// never pickable, so it can't block what's under it
pub(crate) fn card(plant: Plant) -> impl Scene {
    let rows = yields(plant)
        .iter()
        .map(|&(element, amount)| -> Box<dyn Scene> {
            let color = if amount > 0 { GAIN_TEXT } else { LOSS_TEXT };
            Box::new(bsn! {
                Node {
                    column_gap: texels(4),
                }
                Pickable::IGNORE
                Children [
                    Text({ element.name().to_string() })
                    @font::text(1)
                    TextColor(BUTTON_TEXT)
                    Pickable::IGNORE
                    --
                    Text({ format!("{amount:+}") })
                    @font::text(1)
                    TextColor(color)
                    Pickable::IGNORE
                ]
            })
        })
        .collect::<Vec<_>>();

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
