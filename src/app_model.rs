use bevy::prelude::*;

// Keys into the i18n JSON files for the main menu.
pub(crate) const MENU_ITEMS: [MenuAction; 5] = [
    MenuAction::Start,
    MenuAction::ToggleCharacter,
    MenuAction::Settings,
    MenuAction::About,
    MenuAction::Exit,
];

pub(crate) fn menu_action_label_key(action: MenuAction) -> Option<&'static str> {
    match action {
        MenuAction::Start => Some("menu.start"),
        MenuAction::ToggleCharacter => None,
        MenuAction::Settings => Some("menu.settings"),
        MenuAction::About => Some("menu.about"),
        MenuAction::Exit => Some("menu.exit"),
    }
}

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
    ToggleCharacter,
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
