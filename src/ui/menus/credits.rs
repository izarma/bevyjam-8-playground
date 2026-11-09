use bevy::prelude::*;

use crate::ui::{font, palette::HEADER_TEXT, scale::texels, widget};

pub(super) fn credits(bevy_logo: Handle<Image>) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: texels(6),
        }
        Pickable::IGNORE
        Children [
            Text("Fever")
            @font::text(4)
            TextColor(HEADER_TEXT)
            Pickable::IGNORE
            --
            @widget::label("A game for Bevy Jam 8")
            --
            @gap()
            --
            @widget::header("Created by")
            --
            @created_by()
            --
            @gap()
            --
            @widget::header("Art and assets")
            --
            @assets()
            --
            @gap()
            --
            @widget::header("Built with")
            --
            @built_with(bevy_logo)
            --
            @gap()
            --
            @gap()
            --
            @widget::header("Thank you for playing")
        ]
    }
}

fn gap() -> impl Scene {
    bsn! {
        Node {
            height: texels(24),
        }
    }
}

fn created_by() -> impl Scene {
    grid(vec![
        (
            "Cabycat",
            Some("https://cabycat.itch.io/"),
            "Art and writing",
        ),
        (
            "Junemotif",
            Some("https://soundcloud.com/junemotif"),
            "Sound design and programming",
        ),
        ("izarma", Some("https://izarma.itch.io/"), "Programming"),
        (
            "Piefayth",
            Some("https://piefayth.itch.io/"),
            "Programming and tech art",
        ),
    ])
}

fn assets() -> impl Scene {
    grid(vec![
        (
            "ErisEsra",
            Some("https://erisesra.itch.io/character-templates-pack"),
            "Character templates",
        ),
        (
            "Calciumtrice",
            Some("https://opengameart.org/content/dungeon-tileset-1"),
            "Dungeon tileset, CC BY 3.0",
        ),
        (
            "Kenney",
            Some("https://kenney.nl/assets/input-prompts-pixel-16"),
            "Input prompts, CC0",
        ),
        (
            "SpringySpringo",
            Some("https://opengameart.org/content/16x16-grass-textures"),
            "Grass and dirt textures, CC BY 4.0",
        ),
        (
            "piiixl",
            Some("https://piiixl.itch.io/textures"),
            "Terrain textures, CC BY 4.0",
        ),
        (
            "YukiPixels",
            Some("https://yukipixels.itch.io/boldpixels"),
            "BoldPixels font, CC BY-SA 4.0",
        ),
        (
            "Danaida",
            Some("https://danaida.itch.io/free-pixel-plants-16x16"),
            "Pixel plants",
        ),
        (
            "AmyVC",
            Some("https://amyvc.itch.io/rpg-plant-sprites-and-icons"),
            "Plant sprites, CC BY 4.0",
        ),
        (
            "alexander",
            Some("https://blockbenchworkshop.com/model/alexander/stick-bundle"),
            "Stick bundle model, CC BY",
        ),
        (
            "conczin",
            Some("https://blockbenchworkshop.com/model/conczin/tiny-box-with-apples"),
            "Tiny box with apples model, CC BY",
        ),
        ("Jaszunio15", None, "Button sounds, CC0"),
        ("Kevin MacLeod", None, "Music, CC BY 3.0"),
        ("Bevy Foundation", None, "Bevy logo, all rights reserved"),
    ])
}

fn grid(content: Vec<(&'static str, Option<&'static str>, &'static str)>) -> impl Scene {
    let cols: Vec<RepeatedGridTrack> = RepeatedGridTrack::auto(2);
    let ch = content
        .into_iter()
        .flat_map(|(name, url, credit)| {
            let name: Box<dyn Scene> = match url {
                Some(url) => Box::new(bsn! {
                    @widget::link_label(name, url)
                    Node {
                        justify_self: JustifySelf::End,
                    }
                }),
                None => Box::new(bsn! {
                    @widget::label(name)
                    Node {
                        justify_self: JustifySelf::End,
                    }
                }),
            };

            [
                name,
                Box::new(bsn! {
                    @widget::label(credit)
                    Node {
                        justify_self: JustifySelf::Start,
                    }
                }),
            ]
        })
        .collect::<Vec<_>>();

    bsn! {
        #Grid
        Node {
            display: Display::Grid,
            row_gap: texels(4),
            column_gap: texels(12),
            grid_template_columns: cols,
        }

        Children [ { ch } ]
    }
}

fn built_with(logo: Handle<Image>) -> impl Scene {
    bsn! {
        #MadeWithBevy
        Node {
            align_items: AlignItems::Center,
            column_gap: texels(6),
        }
        Children [
            // Logo
            ImageNode { image: logo }
            Node {
                width: texels(16),
                height: texels(16),
            }
            --
            @widget::link_label("Bevy", "https://bevyengine.org")
            --
            @widget::label("and")
            --
            @widget::link_label("Jackdaw", "https://github.com/jbuehler23/jackdaw")
        ]
    }
}
