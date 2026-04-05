use bevy::prelude::*;

// Keys into the i18n JSON files for the main menu.
pub(crate) const MENU_ITEMS: [(&str, MenuAction); 4] = [
    ("menu.start", MenuAction::Start),
    ("menu.settings", MenuAction::Settings),
    ("menu.about", MenuAction::About),
    ("menu.exit", MenuAction::Exit),
];

pub(crate) const EXIT_CONFIRM_ITEMS: [(&str, ExitConfirmAction); 2] = [
    ("modal.exit.yes", ExitConfirmAction::Confirm),
    ("modal.exit.no", ExitConfirmAction::Cancel),
];

#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub(crate) enum AppState {
    #[default]
    MainMenu,
    StartView,
    StoryView,
    WorldMapView,
    GameView,
    LoseView,
    WinView,
    LoadView,
    SettingsView,
    AboutView,
}

#[derive(Clone, Copy)]
pub(crate) enum MenuAction {
    Start,
    Settings,
    About,
    Exit,
}

#[derive(Clone, Copy)]
pub(crate) enum ExitConfirmAction {
    Confirm,
    Cancel,
}

#[derive(Resource, Default)]
pub(crate) struct MenuSelection {
    pub(crate) index: usize,
}

#[derive(Resource, Default)]
pub(crate) struct ExitConfirmModalState {
    pub(crate) is_open: bool,
    pub(crate) selection: usize,
    pub(crate) suppress_enter_until_release: bool,
}

#[derive(Component)]
pub(crate) struct MainMenuEntity;

#[derive(Component)]
pub(crate) struct MenuMusicEntity;

#[derive(Component)]
pub(crate) struct MenuButton {
    pub(crate) index: usize,
    pub(crate) action: MenuAction,
}

#[derive(Component)]
pub(crate) struct ExitConfirmModalRoot;

#[derive(Component)]
pub(crate) struct ExitConfirmButton {
    pub(crate) index: usize,
    pub(crate) action: ExitConfirmAction,
}

#[derive(Component)]
pub(crate) struct StartScreenBackground;

