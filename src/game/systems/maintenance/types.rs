pub(crate) const PAUSE_MENU_ITEMS: [(&str, PauseMenuAction); 4] = [
    ("pause.restart", PauseMenuAction::Restart),
    ("pause.back_to_worldmap", PauseMenuAction::BackToWorldMap),
    ("pause.back_to_mainmenu", PauseMenuAction::BackToMainMenu),
    ("pause.cancel", PauseMenuAction::Cancel),
];

#[derive(Clone, Copy)]
pub(crate) enum PauseMenuAction {
    Restart,
    BackToWorldMap,
    BackToMainMenu,
    Cancel,
}
