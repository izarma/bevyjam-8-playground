use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    GameState,
    engine::camera::{
        CAMERA_SENSITIVITY_STEP, CameraSensitivity, DEFAULT_CAMERA_SENSITIVITY,
        MAX_CAMERA_SENSITIVITY, MIN_CAMERA_SENSITIVITY,
    },
    ui::{menus::Menu, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn_settings_menu.spawn());
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Settings).and_then(input_just_pressed(KeyCode::Escape))),
    );

    app.add_systems(
        Update,
        (
            update_horizontal_sensitivity_label,
            update_vertical_sensitivity_label,
        )
            .run_if(in_state(Menu::Settings)),
    );
}

fn spawn_settings_menu() -> impl Scene {
    bsn! {
        @widget::ui_root("Settings Menu")
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Settings)
        Children [
            @widget::header("Settings")
            --
            @settings_grid()
            --
            @widget::button("Back", go_back_on_click)
        ]
    }
}

fn settings_grid() -> impl Scene {
    let cols: Vec<RepeatedGridTrack> = RepeatedGridTrack::px(2, 400.0);

    bsn! {
        #SettingsGrid
        Node {
            display: Display::Grid,
            row_gap: px(10),
            column_gap: px(30),
            grid_template_columns: cols,
        }
        Children [
            @widget::label("Horizontal Sensitivity")
            Node {
                justify_self: JustifySelf::End,
            }
            --
            @horizontal_sensitivity_widget()
            --
            @widget::label("Vertical Sensitivity")
            Node {
                justify_self: JustifySelf::End,
            }
            --
            @vertical_sensitivity_widget()
        ]
    }
}

fn horizontal_sensitivity_widget() -> impl Scene {
    bsn! {
        #HorizontalSensitivityWidget
        Node {
            justify_self: JustifySelf::Start,
        }
        Children [
            @widget::button_small("-", lower_horizontal_sensitivity)
            --
            #HorizontalSensitivityValue
            Node {
                padding: UiRect::horizontal(px(10)),
                justify_content: JustifyContent::Center,
            }
            Children [
                @widget::label("")
                HorizontalSensitivityLabel
            ]
            --
            @widget::button_small("+", raise_horizontal_sensitivity)
        ]
    }
}

fn vertical_sensitivity_widget() -> impl Scene {
    bsn! {
        #VerticalSensitivityWidget
        Node {
            justify_self: JustifySelf::Start,
        }
        Children [
            @widget::button_small("-", lower_vertical_sensitivity)
            --
            #VerticalSensitivityValue
            Node {
                padding: UiRect::horizontal(px(10)),
                justify_content: JustifyContent::Center,
            }
            Children [
                @widget::label("")
                VerticalSensitivityLabel
            ]
            --
            @widget::button_small("+", raise_vertical_sensitivity)
        ]
    }
}

#[derive(Component, Clone, Default)]
struct HorizontalSensitivityLabel;

#[derive(Component, Clone, Default)]
struct VerticalSensitivityLabel;

fn lower_horizontal_sensitivity(
    _: On<PointerClick>,
    mut cameras: Query<&mut CameraSensitivity, With<Camera>>,
) {
    let Ok(mut sensitivity) = cameras.single_mut() else {
        return;
    };
    sensitivity.x = (sensitivity.x - CAMERA_SENSITIVITY_STEP.x).max(MIN_CAMERA_SENSITIVITY.x);
}

fn raise_horizontal_sensitivity(
    _: On<PointerClick>,
    mut cameras: Query<&mut CameraSensitivity, With<Camera>>,
) {
    let Ok(mut sensitivity) = cameras.single_mut() else {
        return;
    };
    sensitivity.x = (sensitivity.x + CAMERA_SENSITIVITY_STEP.x).min(MAX_CAMERA_SENSITIVITY.x);
}

fn lower_vertical_sensitivity(
    _: On<PointerClick>,
    mut cameras: Query<&mut CameraSensitivity, With<Camera>>,
) {
    let Ok(mut sensitivity) = cameras.single_mut() else {
        return;
    };
    sensitivity.y = (sensitivity.y - CAMERA_SENSITIVITY_STEP.y).max(MIN_CAMERA_SENSITIVITY.y);
}

fn raise_vertical_sensitivity(
    _: On<PointerClick>,
    mut cameras: Query<&mut CameraSensitivity, With<Camera>>,
) {
    let Ok(mut sensitivity) = cameras.single_mut() else {
        return;
    };
    sensitivity.y = (sensitivity.y + CAMERA_SENSITIVITY_STEP.y).min(MAX_CAMERA_SENSITIVITY.y);
}

fn update_horizontal_sensitivity_label(
    cameras: Single<&CameraSensitivity, With<Camera>>,
    mut label: Single<&mut Text, With<HorizontalSensitivityLabel>>,
) {
    label.0 = format!(
        "{:3.0}%",
        sensitivity_percent(cameras.x, DEFAULT_CAMERA_SENSITIVITY.x)
    );
}

fn update_vertical_sensitivity_label(
    cameras: Single<&CameraSensitivity, With<Camera>>,
    mut label: Single<&mut Text, With<VerticalSensitivityLabel>>,
) {
    label.0 = format!(
        "{:3.0}%",
        sensitivity_percent(cameras.y, DEFAULT_CAMERA_SENSITIVITY.y)
    );
}

/// Express a sensitivity value as a percentage of its default.
fn sensitivity_percent(value: f32, default: f32) -> f32 {
    100.0 * value / default
}

fn go_back_on_click(
    _: On<PointerClick>,
    screen: Res<State<GameState>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    next_menu.set(if screen.get() == &GameState::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}

fn go_back(screen: Res<State<GameState>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(if screen.get() == &GameState::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}
