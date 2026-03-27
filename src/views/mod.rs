use bevy::prelude::*;
use crate::game::game_view::GameViewPlugin;

mod about_view;
mod load_view;
mod settings_view;
mod start_view;

pub struct ViewsPlugin;

impl Plugin for ViewsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            start_view::StartViewPlugin,
            GameViewPlugin,
            load_view::LoadViewPlugin,
            settings_view::SettingsViewPlugin,
            about_view::AboutViewPlugin,
        ));
    }
}

