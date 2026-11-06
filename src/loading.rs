use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

use crate::GameState;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin.
/// Alternatively you can write the logic to load assets yourself.
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
pub(super) fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Title)
            .load_collection::<UIAssets>()
            .load_collection::<MusicAssets>()
            .load_collection::<PlayerAssets>()
            .load_collection::<FootstepAssets>()
            .load_collection::<HarvestingAssets>()
            .load_collection::<ConcoctionAssets>()
            .load_collection::<NPCAssets>(),
    );
}

// The following asset collections will be loaded during the State `GameState::Loading`.
// When done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>).

#[derive(AssetCollection, Resource)]
pub struct UIAssets {
    #[asset(path = "audio/sfx/button_hover.ogg")]
    pub button_hover: Handle<AudioSource>,
    #[asset(path = "audio/sfx/button_click.ogg")]
    pub button_click: Handle<AudioSource>,
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct MusicAssets {
    #[asset(path = "audio/music/Monkeys Spinning Monkeys.ogg")]
    pub title_theme: Handle<AudioSource>,
    #[asset(path = "audio/music/Allegories.ogg")]
    pub game_over_bg: Handle<AudioSource>,
    #[asset(path = "audio/music/Fluffing A Duck.ogg")]
    pub game_bg1: Handle<AudioSource>,
}

// size of one cell of the player sheet, in texels
pub const PLAYER_FRAME: UVec2 = UVec2::new(32, 38);
// cells per row of the sheet
pub const PLAYER_SHEET_COLUMNS: u32 = 13;

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(path = "textures/character/monk-sheet.png")]
    pub player_sheet: Handle<Image>,
}

#[derive(AssetCollection, Resource, Default)]
pub struct FootstepAssets {
    #[asset(path = "audio/sfx/jump.ogg")]
    pub jump: Handle<AudioSource>,
    #[asset(
        paths(
            "audio/sfx/footsteps/dirt/walk_dirt_1.ogg",
            "audio/sfx/footsteps/dirt/walk_dirt_2.ogg",
            "audio/sfx/footsteps/dirt/walk_dirt_3.ogg",
            "audio/sfx/footsteps/dirt/walk_dirt_4.ogg",
            "audio/sfx/footsteps/dirt/walk_dirt_5.ogg",
            "audio/sfx/footsteps/dirt/walk_dirt_6.ogg",
            "audio/sfx/footsteps/dirt/walk_dirt_7.ogg",
            "audio/sfx/footsteps/dirt/walk_dirt_8.ogg",
        ),
        collection(typed)
    )]
    pub dirt: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/footsteps/wood/walk_wood_1.ogg",
            "audio/sfx/footsteps/wood/walk_wood_2.ogg",
            "audio/sfx/footsteps/wood/walk_wood_3.ogg",
            "audio/sfx/footsteps/wood/walk_wood_4.ogg",
            "audio/sfx/footsteps/wood/walk_wood_5.ogg",
            "audio/sfx/footsteps/wood/walk_wood_6.ogg",
            "audio/sfx/footsteps/wood/walk_wood_7.ogg",
            "audio/sfx/footsteps/wood/walk_wood_8.ogg",
        ),
        collection(typed)
    )]
    pub wood: Vec<Handle<AudioSource>>,
    pub wet_dirt: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/footsteps/grass/walk_grass_1.ogg",
            "audio/sfx/footsteps/grass/walk_grass_2.ogg",
            "audio/sfx/footsteps/grass/walk_grass_3.ogg",
            "audio/sfx/footsteps/grass/walk_grass_4.ogg",
            "audio/sfx/footsteps/grass/walk_grass_5.ogg",
            "audio/sfx/footsteps/grass/walk_grass_6.ogg",
            "audio/sfx/footsteps/grass/walk_grass_7.ogg",
            "audio/sfx/footsteps/grass/walk_grass_8.ogg",
        ),
        collection(typed)
    )]
    pub grass: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/footsteps/water/walk_water_1.ogg",
            "audio/sfx/footsteps/water/walk_water_2.ogg",
            "audio/sfx/footsteps/water/walk_water_3.ogg",
            "audio/sfx/footsteps/water/walk_water_4.ogg",
            "audio/sfx/footsteps/water/walk_water_5.ogg",
            "audio/sfx/footsteps/water/walk_water_6.ogg",
            "audio/sfx/footsteps/water/walk_water_7.ogg",
            "audio/sfx/footsteps/water/walk_water_8.ogg",
        ),
        collection(typed)
    )]
    pub water: Vec<Handle<AudioSource>>,
}

#[derive(AssetCollection, Resource)]
pub struct HarvestingAssets {
    #[asset(
        paths(
            "audio/sfx/harvesting/harvesting_1.ogg",
            "audio/sfx/harvesting/harvesting_2.ogg",
            "audio/sfx/harvesting/harvesting_3.ogg",
            "audio/sfx/harvesting/harvesting_4.ogg",
            "audio/sfx/harvesting/harvesting_5.ogg",
            "audio/sfx/harvesting/harvesting_6.ogg",
            "audio/sfx/harvesting/harvesting_7.ogg",
            "audio/sfx/harvesting/harvesting_8.ogg",
        ),
        collection(typed)
    )]
    pub scratches: Vec<Handle<AudioSource>>,
}

#[derive(AssetCollection, Resource)]
pub struct ConcoctionAssets {
    #[asset(
        paths(
            "audio/sfx/herb_drop/herb_drop_1.ogg",
            "audio/sfx/herb_drop/herb_drop_2.ogg",
            "audio/sfx/herb_drop/herb_drop_3.ogg",
            "audio/sfx/herb_drop/herb_drop_4.ogg",
            "audio/sfx/herb_drop/herb_drop_5.ogg",
            "audio/sfx/herb_drop/herb_drop_6.ogg",
            "audio/sfx/herb_drop/herb_drop_7.ogg",
            "audio/sfx/herb_drop/herb_drop_8.ogg",
        ),
        collection(typed)
    )]
    pub herb_drops: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/herb_anticipation/herb_anticipation_1.ogg",
            "audio/sfx/herb_anticipation/herb_anticipation_2.ogg",
            "audio/sfx/herb_anticipation/herb_anticipation_3.ogg",
            "audio/sfx/herb_anticipation/herb_anticipation_4.ogg",
        ),
        collection(typed)
    )]
    pub herb_anticipation: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/herb_heal/herb_heal_1.ogg",
            "audio/sfx/herb_heal/herb_heal_2.ogg",
            "audio/sfx/herb_heal/herb_heal_3.ogg",
            "audio/sfx/herb_heal/herb_heal_4.ogg",
        ),
        collection(typed)
    )]
    pub herb_good: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/herb_neutral/herb_neutral_1.ogg",
            "audio/sfx/herb_neutral/herb_neutral_2.ogg",
            "audio/sfx/herb_neutral/herb_neutral_3.ogg",
            "audio/sfx/herb_neutral/herb_neutral_4.ogg",
        ),
        collection(typed)
    )]
    pub herb_neutral: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/herb_bad/herb_bad_1.ogg",
            "audio/sfx/herb_bad/herb_bad_2.ogg",
            "audio/sfx/herb_bad/herb_bad_3.ogg",
            "audio/sfx/herb_bad/herb_bad_4.ogg",
        ),
        collection(typed)
    )]
    pub herb_bad: Vec<Handle<AudioSource>>,
}

#[derive(AssetCollection, Resource)]
pub struct NPCAssets {
    #[asset(path = "audio/sfx/dialogue_open.ogg")]
    pub dialogue_open: Handle<AudioSource>,
    #[asset(path = "audio/sfx/dialogue_close.ogg")]
    pub dialogue_close: Handle<AudioSource>,
    #[asset(path = "textures/character/fox.png")]
    pub fox: Handle<Image>,
}
