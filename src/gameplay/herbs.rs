use bevy::prelude::*;
use plants::Plant;

use crate::ui::{
    font,
    palette::{BUTTON_TEXT, GAIN_TEXT, HEADER_TEXT, LOSS_TEXT},
    scale::texels,
    widget,
};

#[derive(Clone, Copy, Default)]
pub enum Element {
    #[default]
    Wood,
    Fire,
    Earth,
    Metal,
    Water,
}

impl Element {
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

// placeholder
pub(crate) fn name(plant: Plant) -> &'static str {
    match plant {
        Plant::Plant01 => "Plant 01",
        Plant::Plant06 => "Plant 06",
        Plant::Plant13 => "Plant 13",
    }
}

fn yields(plant: Plant) -> &'static [(Element, i32)] {
    match plant {
        Plant::Plant01 => &[(Element::Wood, 2), (Element::Metal, -1)],
        Plant::Plant06 => &[(Element::Water, 3)],
        Plant::Plant13 => &[(Element::Earth, 1), (Element::Fire, -1)],
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
