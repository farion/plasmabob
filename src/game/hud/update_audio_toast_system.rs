use bevy::prelude::*;

use crate::game::hud::components::{AudioToastContainerUi, AudioToastTextUi};
use crate::helper::audio_toast::AudioToastRequest;
use crate::helper::i18n::{CurrentLanguage, Translations};

const TOAST_DURATION_SEC: f32 = 1.5;

pub fn update_audio_toast_system(
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

