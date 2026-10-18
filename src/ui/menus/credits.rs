use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::ui::{menus::Menu, widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu.spawn());
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Credits).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_credits_menu() -> impl Scene {
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
        ]
    }
}

fn created_by() -> impl Scene {
    grid(vec![
        ["Cabycat", "Art/Writing"],
        ["Junemotif", "Music/SFX"],
        ["izarma", "Programming"],
    ])
}

fn assets() -> impl Scene {
    grid(vec![
        ["Ducky sprite", "CC0 by Caz Creates Games"],
        ["Button SFX", "CC0 by Jaszunio15"],
        ["Music", "CC BY 3.0 by Kevin MacLeod"],
    ])
}

fn grid(content: Vec<[&'static str; 2]>) -> impl Scene {
    let cols: Vec<RepeatedGridTrack> = RepeatedGridTrack::px(2, 400.0);

    let ch = content
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, text)| {
            let j = if i.is_multiple_of(2) {
                JustifySelf::End
            } else {
                JustifySelf::Start
            };

            Box::new(bsn! {
                @widget::label(text)
                Node {
                    justify_self: j,
                }
            })
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

fn go_back_on_click(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}
