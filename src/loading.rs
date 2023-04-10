use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Menu),
        )
        .add_collection_to_loading_state::<_, FontAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, TextureAssets>(GameState::Loading);
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    pub background: Handle<AudioSource>,
    #[asset(path = "audio/glug.ogg")]
    pub drinking: Handle<AudioSource>,
    #[asset(path = "audio/chatter.ogg")]
    pub chatter: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 32., columns = 4, rows = 1))]
    #[asset(path = "textures/luna-16x32.png")]
    pub luna: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 32., columns = 26, rows = 1))]
    #[asset(path = "textures/npc1-16x32.png")]
    pub npc1: Handle<TextureAtlas>,
    #[asset(path = "textures/floor1-16x16.png")]
    pub floor1: Handle<Image>,
    #[asset(path = "textures/table-16x16.png")]
    pub table: Handle<Image>,
    #[asset(path = "textures/bar-16x24.png")]
    pub bar: Handle<Image>,
    #[asset(path = "textures/barback-16x24.png")]
    pub barback: Handle<Image>,
    #[asset(path = "textures/orange-16x8.png")]
    pub orange: Handle<Image>,
    #[asset(path = "textures/banana-16x8.png")]
    pub banana: Handle<Image>,
    #[asset(path = "textures/cherry-16x8.png")]
    pub cherry: Handle<Image>,
    #[asset(path = "textures/bowl-empty-16x8.png")]
    pub bowl_empty: Handle<Image>,
    #[asset(path = "textures/bowl-filled-16x8.png")]
    pub bowl_filled: Handle<Image>,
    #[asset(path = "textures/bowl-filled-orange-16x8.png")]
    pub bowl_filled_orange: Handle<Image>,
    #[asset(path = "textures/bowl-filled-cherry-16x8.png")]
    pub bowl_filled_cherry: Handle<Image>,
    #[asset(path = "textures/beverage-16x8.png")]
    pub beverage: Handle<Image>,
    #[asset(path = "textures/trash-16x8.png")]
    pub trash: Handle<Image>,
    #[asset(path = "textures/mixer-16x8.png")]
    pub mixer: Handle<Image>,
}
