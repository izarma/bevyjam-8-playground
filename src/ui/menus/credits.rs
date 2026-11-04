use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    loading::UIAssets,
    ui::{menus::Menu, scale::texels, widget},
};

use super::{open_main_menu, open_main_menu_on_click};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
    app.add_systems(
        Update,
        open_main_menu
            .run_if(in_state(Menu::Credits).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_credits_menu(mut commands: Commands, textures: Res<UIAssets>) {
    commands.spawn_scene(credits_menu(textures.bevy.clone()));
}

fn credits_menu(bevy_logo: Handle<Image>) -> impl Scene {
    bsn! {
        @widget::ui_root("Credits Menu")
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Credits)
        Children [
            @widget::header("Created by")
            --
            @created_by()
            --
            @widget::header("Assets")
            --
            @assets()
            --
            @widget::button("Back", open_main_menu_on_click)
            --
            @made_with_bevy(bevy_logo)
        ]
    }
}

fn created_by() -> impl Scene {
    grid(vec![
        ("Cabycat", Some("https://cabycat.itch.io/"), "Art/Writing"),
        (
            "Junemotif",
            Some("https://soundcloud.com/junemotif"),
            "Sound Design/Programming",
        ),
        ("izarma", Some("https://izarma.itch.io/"), "Programming"),
    ])
}

fn assets() -> impl Scene {
    grid(vec![
        ("Button SFX", None, "CC0 by Jaszunio15"),
        (
            "Bevy logo",
            None,
            "All rights reserved by the Bevy Foundation",
        ),
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

fn made_with_bevy(logo: Handle<Image>) -> impl Scene {
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
            @widget::label("Made with")
            --
            @widget::link_label("Bevy", "https://bevyengine.org")
        ]
    }
}
