use bevy::prelude::*;
use jackdaw_extension::prelude::*;

// i figure we cram whatever we need this one editor extension
// today it only handles allowing drawing billboard sprites in the editor viewport
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
        "Draws the game's pixel sprites in the viewport.".to_string()
    }

    fn register(&self, _registrar: &mut ExtensionRegistrar<'_>) {}

    fn app_plugin(&self) -> Option<Box<dyn Plugin>> {
        Some(Box::new(pixel_sprite::PixelSpritePlugin))
    }
}
