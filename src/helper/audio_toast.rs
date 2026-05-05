use bevy::prelude::*;

use crate::helper::i18n::{CurrentLanguage, Translations};

const TOAST_DURATION_SEC: f32 = 1.5;
const TOAST_WIDTH: f32 = 420.0;
const TOAST_HEIGHT: f32 = 52.0;
const TOAST_FONT_SIZE: f32 = 28.0;
const TOAST_TOP_OFFSET: f32 = 84.0;

#[derive(Resource, Default, Debug, Clone)]
pub(crate) struct AudioToastRequest {
    key: Option<&'static str>,
}

impl AudioToastRequest {
    pub(crate) fn set(&mut self, key: &'static str) {
        self.key = Some(key);
    }

    pub(crate) fn take(&mut self) -> Option<&'static str> {
        self.key.take()
    }
}

#[derive(Component)]
struct AudioToastContainerUi;

#[derive(Component)]
struct AudioToastTextUi;

pub(crate) struct AudioToastPlugin;

impl Plugin for AudioToastPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_audio_toast_ui_system)
            .add_systems(Update, update_audio_toast_system);
    }
}

fn spawn_audio_toast_ui_system(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(TOAST_TOP_OFFSET),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-TOAST_WIDTH * 0.5)),
                width: Val::Px(TOAST_WIDTH),
                height: Val::Px(TOAST_HEIGHT),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.75)),
            AudioToastContainerUi,
        ))
        .with_children(|toast| {
            toast.spawn((
                Text::new(""),
                TextFont {
                    font_size: TOAST_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::srgba(0.95, 0.95, 1.0, 0.98)),
                AudioToastTextUi,
            ));
        });
}

fn update_audio_toast_system(
    time: Res<Time>,
    translations: Res<Translations>,
    current_language: Res<CurrentLanguage>,
    mut request: ResMut<AudioToastRequest>,
    mut timer: Local<Option<Timer>>,
    mut container_query: Query<&mut Node, With<AudioToastContainerUi>>,
    mut text_query: Query<&mut Text, With<AudioToastTextUi>>,
) {
    let Ok(mut container_node) = container_query.single_mut() else {
        return;
    };
    let Ok(mut toast_text) = text_query.single_mut() else {
        return;
    };

    if let Some(key) = request.take() {
        let lang = current_language.effective(&translations);
        if let Some(value) = translations.tr(&lang, key) {
            toast_text.0 = value.clone();
        } else {
            toast_text.0 = format!("{{{key}}}");
        }

        container_node.display = Display::Flex;
        *timer = Some(Timer::from_seconds(TOAST_DURATION_SEC, TimerMode::Once));
        return;
    }

    if let Some(active_timer) = timer.as_mut() {
        active_timer.tick(time.delta());
        if active_timer.is_finished() {
            container_node.display = Display::None;
            *timer = None;
        }
    }
}
