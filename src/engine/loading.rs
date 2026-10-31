use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
pub(super) fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Title)
            .load_collection::<UIAssets>()
            .load_collection::<LevelAssets>()
            .load_collection::<PlayerAssets>(),
    );
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct UIAssets {
    #[asset(path = "audio/sfx/button_hover.ogg")]
    pub button_hover: Handle<AudioSource>,
    #[asset(path = "audio/sfx/button_click.ogg")]
    pub button_click: Handle<AudioSource>,
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "audio/music/Monkeys Spinning Monkeys.ogg")]
    pub title_theme: Handle<AudioSource>,
    #[asset(path = "audio/music/Allegories.ogg")]
    pub game_over_bg: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(path = "textures/character/16x32char-sheet-template.png")]
    pub player_sheet: Handle<Image>,
    #[asset(path = "audio/sfx/step1.ogg")]
    pub step1: Handle<AudioSource>,
    #[asset(path = "audio/sfx/step2.ogg")]
    pub step2: Handle<AudioSource>,
    #[asset(path = "audio/sfx/step3.ogg")]
    pub step3: Handle<AudioSource>,
    #[asset(path = "audio/sfx/step4.ogg")]
    pub step4: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct LevelAssets {
    #[asset(path = "audio/music/Fluffing A Duck.ogg")]
    pub game_bg1: Handle<AudioSource>,
    #[asset(path = "textures/tileset/tileset_padded.png")]
    pub level_tilemap: Handle<Image>,
}
