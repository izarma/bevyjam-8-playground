use std::borrow::Cow;

use bevy::{ecs::system::IntoObserverSystem, prelude::*};

use crate::ui::{
    font,
    interaction::InteractionPalette,
    palette::*,
    scale::{TEXEL, texels},
};

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Scene {
    let name = name.into();
    bsn! {
        Name::new(name)
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: texels(6),
        }
        // Don't block picking events for other UI roots.
        Pickable::IGNORE
    }
}

/// A simple header label. Bigger than [`label`].
pub fn header(text: impl Into<String>) -> impl Scene {
    bsn! {
        #Header
        Text({ text.into() })
        @font::text(2)
        TextColor(HEADER_TEXT)
    }
}

/// A simple text label.
pub fn label(text: impl Into<String>) -> impl Scene {
    bsn! {
        #Label
        Text({ text.into() })
        @font::text(1)
        TextColor(LABEL_TEXT)
    }
}

/// A label that opens `url` in the browser when clicked. Rendered as an
/// underlined link in [`LINK_TEXT`] to distinguish it from a plain [`label`].
pub fn link_label(text: impl Into<String>, url: &'static str) -> impl Scene {
    let text = text.into();
    bsn! {
        #Label
        Text(text)
        @font::text(1)
        TextColor(LINK_TEXT)
        Underline
        OpenLink(url)
        @on(open_link)
        @on(hover_link)
        @on(unhover_link)
    }
}

/// The URL opened by a [`link_label`](self::link_label) when clicked.
#[derive(Component, Clone, Default)]
pub struct OpenLink(pub &'static str);

fn open_link(click: On<PointerClick>, links: Query<&OpenLink>) {
    let Ok(link) = links.get(click.entity) else {
        return;
    };
    if let Err(error) = webbrowser::open(link.0) {
        warn!("Failed to open link {error:?}");
    }
}

fn hover_link(over: On<PointerOver>, mut links: Query<&mut TextColor, With<OpenLink>>) {
    let Ok(mut color) = links.get_mut(over.entity) else {
        return;
    };
    *color = LINK_TEXT_HOVERED.into();
}

fn unhover_link(out: On<PointerOut>, mut links: Query<&mut TextColor, With<OpenLink>>) {
    let Ok(mut color) = links.get_mut(out.entity) else {
        return;
    };
    *color = LINK_TEXT.into();
}

/// A large button with text and an action defined as an [`Observer`].
pub fn button<E, M, I>(text: impl Into<String>, action: I) -> impl Scene
where
    E: EntityEvent,
    I: IntoObserverSystem<E, M, ()> + Clone + Sync,
    M: 'static,
{
    button_base(
        text,
        action,
        bsn! {
            Node {
                width: texels(112),
                height: texels(20),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
        },
        UiRect::ZERO,
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, M, I>(text: impl Into<String>, action: I) -> impl Scene
where
    E: EntityEvent,
    I: IntoObserverSystem<E, M, ()> + Clone + Sync,
    M: 'static,
{
    button_base(
        text,
        action,
        bsn! {
            Node {
                width: texels(16),
                height: texels(16),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
        },
        // center the + and - in their buttons, probably font-specific
        UiRect {
            left: texels(1),
            bottom: texels(2),
            ..default()
        },
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
fn button_base<E, M, I>(
    text: impl Into<String>,
    action: I,
    button_bundle: impl Scene,
    text_padding: UiRect,
) -> impl Scene
where
    E: EventPattern<Event: EntityEvent>,
    I: IntoObserverSystem<E, M, ()> + Clone + Sync,
    M: 'static,
{
    let text = text.into();

    bsn! {
        #Button
        Node
        Children [
            #ButtonInner
            // Button
            @panel()
            InteractionPalette {
                none: Color::WHITE,
                hovered: BUTTON_HOVERED_TINT,
                pressed: BUTTON_PRESSED_TINT,
            }
            Children [
                #ButtonText
                Text(text)
                Node {
                    padding: text_padding,
                }
                @font::text(1)
                TextColor(BUTTON_TEXT)
                // Don't bubble picking events from the text up to the button.
                Pickable::IGNORE
            ]
            @button_bundle
            @on(action)
        ]
    }
}

pub fn panel() -> impl Scene {
    bsn! {
        ImageNode {
            image: "ui/9slice.png",
            image_mode: { 
                // these settings are specific to how the 9slice png is authored
                NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(7.0),
                    max_corner_scale: TEXEL,
                    ..default()
                })
            },
        }
    }
}
