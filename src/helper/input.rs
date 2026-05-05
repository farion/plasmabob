use bevy::prelude::*;
use std::collections::HashSet;

use crate::helper::key_bindings::{KeyAction, KeyBindings};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum Action {
    MoveLeft,
    MoveRight,
    Jump,
    Crouch,
    Shoot,
    ToggleFullscreen,
    ToggleMusicMute,
    ToggleSoundMute,
    ToggleDebugFps,
    ToggleDebugCounters,
    ToggleEnemyAiDebug,
    ToggleHitboxDebug,
}

#[derive(Message, Clone, Copy, Debug)]
pub(crate) struct ActionPressed(pub(crate) Action);

#[derive(Message, Clone, Copy, Debug)]
#[allow(dead_code)]
pub(crate) struct ActionReleased(pub(crate) Action);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Binding {
    Key(KeyCode),
    KeyWithAlt(KeyCode),
}

#[derive(Resource, Debug, Clone)]
pub(crate) struct InputBindings {
    mapping: Vec<(Action, Vec<Binding>)>,
}

impl InputBindings {
    pub(crate) fn from_key_bindings(key_bindings: &KeyBindings) -> Self {
        Self {
            mapping: vec![
                (Action::MoveLeft, vec![Binding::Key(key_bindings.get(KeyAction::MoveLeft))]),
                (Action::MoveRight, vec![Binding::Key(key_bindings.get(KeyAction::MoveRight))]),
                (Action::Jump, vec![Binding::Key(key_bindings.get(KeyAction::Jump))]),
                (Action::Crouch, vec![Binding::Key(key_bindings.get(KeyAction::Crouch))]),
                (Action::Shoot, vec![Binding::Key(key_bindings.get(KeyAction::Shoot))]),
                (
                    Action::ToggleFullscreen,
                    vec![Binding::Key(key_bindings.get(KeyAction::Fullscreen))],
                ),
                (
                    Action::ToggleMusicMute,
                    vec![Binding::Key(key_bindings.get(KeyAction::ToggleMute))],
                ),
                (
                    Action::ToggleSoundMute,
                    vec![Binding::Key(key_bindings.get(KeyAction::ToggleSound))],
                ),
                (Action::ToggleDebugFps, vec![Binding::KeyWithAlt(KeyCode::F2)]),
                (
                    Action::ToggleDebugCounters,
                    vec![Binding::KeyWithAlt(KeyCode::F3)],
                ),
                (
                    Action::ToggleEnemyAiDebug,
                    vec![Binding::KeyWithAlt(KeyCode::F4)],
                ),
                (
                    Action::ToggleHitboxDebug,
                    vec![Binding::KeyWithAlt(KeyCode::F5)],
                ),
            ],
        }
    }

    fn actions<'a>(&'a self) -> impl Iterator<Item = (Action, &'a [Binding])> + 'a {
        self.mapping
            .iter()
            .map(|(action, bindings)| (*action, bindings.as_slice()))
    }
}

#[derive(Resource, Default, Debug)]
pub(crate) struct InputActionState {
    pressed: HashSet<Action>,
    just_pressed: HashSet<Action>,
    just_released: HashSet<Action>,
}

impl InputActionState {
    pub(crate) fn pressed(&self, action: Action) -> bool {
        self.pressed.contains(&action)
    }

    pub(crate) fn just_pressed(&self, action: Action) -> bool {
        self.just_pressed.contains(&action)
    }

    #[allow(dead_code)]
    pub(crate) fn just_released(&self, action: Action) -> bool {
        self.just_released.contains(&action)
    }
}

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        let key_bindings = app
            .world()
            .get_resource::<KeyBindings>()
            .cloned()
            .unwrap_or_else(KeyBindings::default);

        app.insert_resource(InputBindings::from_key_bindings(&key_bindings))
            .init_resource::<InputActionState>()
            .add_message::<ActionPressed>()
            .add_message::<ActionReleased>()
            .add_systems(PreUpdate, update_input_bindings)
            .add_systems(PreUpdate, poll_actions.after(update_input_bindings));
    }
}

fn update_input_bindings(
    key_bindings: Res<KeyBindings>,
    mut input_bindings: ResMut<InputBindings>,
    mut action_state: ResMut<InputActionState>,
) {
    if !key_bindings.is_changed() {
        return;
    }

    *input_bindings = InputBindings::from_key_bindings(&key_bindings);
    action_state.pressed.clear();
    action_state.just_pressed.clear();
    action_state.just_released.clear();
}

fn poll_actions(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<InputActionState>,
    mut pressed_writer: MessageWriter<ActionPressed>,
    mut released_writer: MessageWriter<ActionReleased>,
) {
    state.just_pressed.clear();
    state.just_released.clear();

    for (action, binding_list) in bindings.actions() {
        let currently_pressed = binding_list
            .iter()
            .any(|binding| binding_pressed(*binding, &keys));
        let was_pressed = state.pressed.contains(&action);

        if currently_pressed && !was_pressed {
            state.pressed.insert(action);
            state.just_pressed.insert(action);
            pressed_writer.write(ActionPressed(action));
        } else if !currently_pressed && was_pressed {
            state.pressed.remove(&action);
            state.just_released.insert(action);
            released_writer.write(ActionReleased(action));
        }
    }
}

fn binding_pressed(binding: Binding, keys: &ButtonInput<KeyCode>) -> bool {
    match binding {
        Binding::Key(key) => keys.pressed(key),
        Binding::KeyWithAlt(key) => {
            let alt = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
            alt && keys.pressed(key)
        }
    }
}
