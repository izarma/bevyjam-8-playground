// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::DefaultPlugins;
use bevy::asset::AssetMetaCheck;
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
use std::io::Cursor;
use winit::window::Icon;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.0, 0.0, 0.0)))
        .add_plugins(plugins())
        .add_plugins(bevy_game::plugin) // ToDo: Replace bevy_game with your new crate name.
        .run();
}

fn plugins() -> impl PluginGroup {
    let plugins = DefaultPlugins
        .set(set_window_plug())
        .set(ImagePlugin::default_nearest()) // for pixel art
        .set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        });
    // PIE doesnt work on web
    #[cfg(not(target_arch = "wasm32"))]
    let plugins = jackdaw_runtime::maybe_windowless(plugins);
    plugins
}

fn set_window_plug() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Jam template".to_string(), // ToDo
            // Bind to canvas included in `index.html`
            canvas: Some("#bevy".to_owned()),
            fit_canvas_to_parent: true,
            // Tells wasm not to override default event handling, like F5 and Ctrl+R
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }
}
