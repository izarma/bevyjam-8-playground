use bevy::prelude::*;
use jackdaw_extension::prelude::*;

// i figure we cram whatever we need this one editor extension
// today it draws the game's billboard sprites, pixel water and herb spots in the editor viewport
#[derive(Default)]
pub struct BevyjamEditor;

impl JackdawExtension for BevyjamEditor {
    fn id(&self) -> String {
        "bevyjam-editor".to_string()
    }

    fn label(&self) -> String {
        "Bevyjam Editor".to_string()
    }

    fn description(&self) -> String {
        "Draws the game's pixel sprites, water and plants in the viewport.".to_string()
    }

    fn register(&self, _registrar: &mut ExtensionRegistrar<'_>) {}

    fn app_plugin(&self) -> Option<Box<dyn Plugin>> {
        Some(Box::new(|app: &mut App| {
            app.add_plugins((
                pixel_art::PixelArtPlugin,
                plants::SpotPreviewPlugin,
            ));
        }))
    }
}
