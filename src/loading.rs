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
            .load_collection::<AmbienceAssets>()
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
pub struct AmbienceAssets {
    #[asset(path = "audio/sfx/start.ogg")]
    pub title: Handle<AudioSource>,
    #[asset(path = "audio/ambience/outside_daytime.ogg")]
    pub outdoor_daytime: Handle<AudioSource>,
    #[asset(path = "audio/ambience/outside_night.ogg")]
    pub outdoor_night: Handle<AudioSource>,
    #[asset(path = "audio/music/explore.ogg")]
    pub outdoor_music: Handle<AudioSource>,
    #[asset(path = "audio/music/explore_night.ogg")]
    pub outdoor_night_music: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct UIAssets {
    #[asset(path = "audio/sfx/gong.ogg")]
    pub gong: Handle<AudioSource>,
    #[asset(
        paths(
            "audio/sfx/transition_1.ogg",
            "audio/sfx/transition_2.ogg",
            "audio/sfx/transition_3.ogg",
        ),
        collection(typed)
    )]
    pub transitions: Vec<Handle<AudioSource>>,
    #[asset(path = "audio/sfx/button_hover.ogg")]
    pub button_hover: Handle<AudioSource>,
    #[asset(path = "audio/sfx/button_click.ogg")]
    pub button_click: Handle<AudioSource>,
    #[asset(path = "audio/sfx/button_press.ogg")]
    pub button_press: Handle<AudioSource>,
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "ui/9slice.png")]
    pub panel: Handle<Image>,
    #[asset(path = "ui/9slice_fill.png")]
    pub panel_fill: Handle<Image>,
    #[asset(path = "ui/Element Icons.png")]
    pub elements: Handle<Image>,
    #[asset(path = "ui/Bell Button.png")]
    pub inventory_icon: Handle<Image>,
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
    #[asset(
        paths(
            "audio/sfx/footsteps/concrete/walk_concrete_1.ogg",
            "audio/sfx/footsteps/concrete/walk_concrete_2.ogg",
            "audio/sfx/footsteps/concrete/walk_concrete_3.ogg",
            "audio/sfx/footsteps/concrete/walk_concrete_4.ogg",
            "audio/sfx/footsteps/concrete/walk_concrete_5.ogg",
            "audio/sfx/footsteps/concrete/walk_concrete_6.ogg",
            "audio/sfx/footsteps/concrete/walk_concrete_7.ogg",
            "audio/sfx/footsteps/concrete/walk_concrete_8.ogg",
        ),
        collection(typed)
    )]
    pub concrete: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/footsteps/sand/walk_sand_1.ogg",
            "audio/sfx/footsteps/sand/walk_sand_2.ogg",
            "audio/sfx/footsteps/sand/walk_sand_3.ogg",
            "audio/sfx/footsteps/sand/walk_sand_4.ogg",
            "audio/sfx/footsteps/sand/walk_sand_5.ogg",
            "audio/sfx/footsteps/sand/walk_sand_6.ogg",
            "audio/sfx/footsteps/sand/walk_sand_7.ogg",
            "audio/sfx/footsteps/sand/walk_sand_8.ogg",
        ),
        collection(typed)
    )]
    pub sand: Vec<Handle<AudioSource>>,
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
    #[asset(path = "audio/sfx/pickup.ogg")]
    pub pickup: Handle<AudioSource>,
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
        ),
        collection(typed)
    )]
    pub herb_drops: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/herb_anticipation/herb_anticipation_1.ogg",
            "audio/sfx/herb_anticipation/herb_anticipation_2.ogg",
            "audio/sfx/herb_anticipation/herb_anticipation_3.ogg",
        ),
        collection(typed)
    )]
    pub herb_anticipation: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/herb_good/herb_good_1.ogg",
            "audio/sfx/herb_good/herb_good_2.ogg",
            "audio/sfx/herb_good/herb_good_3.ogg",
        ),
        collection(typed)
    )]
    pub herb_good: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/herb_neutral/herb_neutral_1.ogg",
            "audio/sfx/herb_neutral/herb_neutral_2.ogg",
            "audio/sfx/herb_neutral/herb_neutral_3.ogg",
        ),
        collection(typed)
    )]
    pub herb_neutral: Vec<Handle<AudioSource>>,
    #[asset(
        paths(
            "audio/sfx/herb_bad/herb_bad_1.ogg",
            "audio/sfx/herb_bad/herb_bad_2.ogg",
            "audio/sfx/herb_bad/herb_bad_3.ogg",
        ),
        collection(typed)
    )]
    pub herb_bad: Vec<Handle<AudioSource>>,
}

#[derive(AssetCollection, Resource)]
pub struct NPCAssets {
    #[asset(path = "audio/sfx/dialogue_open.ogg")]
    pub dialogue_open: Handle<AudioSource>,
    #[asset(path = "audio/sfx/dialogue_continue.ogg")]
    pub dialogue_continue: Handle<AudioSource>,
    #[asset(path = "audio/sfx/dialogue_close.ogg")]
    pub dialogue_close: Handle<AudioSource>,
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
            "Guardian" => (&self.guardian, PLAYER_FRAME),
            "Monk" => (&self.monk_man, MONK_MAN_FRAME),
            "Bride" => (&self.lady, NPC_FRAME),
            "Beggar" => (&self.old_beggar, NPC_FRAME),
            "Old Beauty" => (&self.old_lady, NPC_FRAME),
            "Orphan" => (&self.kid, NPC_FRAME),
            _ => (&self.monk_man, MONK_MAN_FRAME),
        }
    }
}
