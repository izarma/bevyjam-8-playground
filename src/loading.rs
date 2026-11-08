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
    #[asset(path = "ui/9slice.png")]
    pub panel: Handle<Image>,
    #[asset(path = "ui/9slice_fill.png")]
    pub panel_fill: Handle<Image>,
    #[asset(path = "ui/Bell Button.png")]
    pub inventory_icon: Handle<Image>,
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
pub struct NPCAssets {
    #[asset(path = "textures/character/MonkMan.png")]
    pub monk_man: Handle<Image>,
    #[asset(path = "textures/character/RichLady.png")]
    pub lady: Handle<Image>,
    #[asset(path = "textures/character/Old Beggar.png")]
    pub old_beggar: Handle<Image>,
    #[asset(path = "textures/character/OldLady.png")]
    pub old_lady: Handle<Image>,
    #[asset(path = "textures/character/kid.png")]
    pub kid: Handle<Image>,
    // a recolor of the player
    #[asset(path = "textures/character/Guardian.png")]
    pub guardian: Handle<Image>,
}

// size of one cell of the monk placeholder sheet, in texels
pub const MONK_MAN_FRAME: UVec2 = UVec2::new(16, 37);
// npc frames
pub const NPC_FRAME: UVec2 = UVec2::new(16, 32);

impl NPCAssets {
    pub fn sprite(&self, name: &str) -> (&Handle<Image>, UVec2) {
        match name {
            "Monk" => (&self.old_beggar, MONK_MAN_FRAME),
            "Guardian" => (&self.guardian, PLAYER_FRAME),
            _ => (&self.monk_man, NPC_FRAME),
        }
    }
}
