use crate::game::game_view::GameViewPlugin;
use bevy::prelude::*;

pub mod main_view;

mod about_view;
mod load_view;
mod lose_view;
mod settings_view;
mod start_view;
mod story_view;
mod win_view;
mod world_map_view;

pub struct ViewsPlugin;

impl Plugin for ViewsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            main_view::MainViewPlugin,
            start_view::StartViewPlugin,
            GameViewPlugin,
            load_view::LoadViewPlugin,
            lose_view::LoseViewPlugin,
            settings_view::SettingsViewPlugin,
            about_view::AboutViewPlugin,
            story_view::StoryViewPlugin,
            world_map_view::WorldMapViewPlugin,
            win_view::WinViewPlugin,
        ));
    }
}
