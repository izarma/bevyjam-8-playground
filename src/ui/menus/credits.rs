use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    loading::TextureAssets,
    ui::{menus::Menu, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Credits).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_credits_menu(mut commands: Commands, textures: Res<TextureAssets>) {
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
            @widget::button("Back", go_back_on_click)
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
    let cols: Vec<RepeatedGridTrack> = RepeatedGridTrack::px(2, 400.0);
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
            row_gap: px(10),
            column_gap: px(30),
            grid_template_columns: cols,
        }

        Children [ { ch } ]
    }
}

/// Footer pinned to the bottom of the menu.
fn made_with_bevy(logo: Handle<Image>) -> impl Scene {
    bsn! {
        #MadeWithBevy
        Node {
            position_type: PositionType::Absolute,
            bottom: px(10),
            width: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(12),
        }
        Children [
            // Logo
            ImageNode { image: logo }
            Node {
                width: px(32),
                height: px(32),
            }
            --
            @widget::label("Made with")
            --
            @widget::link_label("Bevy", "https://bevyengine.org")
        ]
    }
}

fn go_back_on_click(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}
