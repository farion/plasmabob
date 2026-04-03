use bevy::asset::io::AssetSourceId;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use futures_lite::AsyncReadExt;
use pulldown_cmark::{Event, Options, Parser, Tag};

use crate::key_bindings::KeyBindings;
use crate::{AppState, PendingStoryScreen};

pub struct StoryViewPlugin;

#[derive(Component)]
struct StoryViewEntity;

#[derive(Component)]
struct StoryContinueTarget(AppState);

#[derive(Component)]
struct StoryBackground;

#[derive(Component)]
struct StoryTextViewport;

#[derive(Component)]
struct StoryTextContent;

#[derive(Debug, Clone)]
enum MarkdownBlockKind {
    Heading(u8),
    Paragraph,
    ListItem,
}

#[derive(Debug, Clone)]
struct MarkdownBlock {
    kind: MarkdownBlockKind,
    text: String,
}

#[derive(Resource, Debug, Clone, Copy)]
struct StoryScrollState {
    offset_px: f32,
    target_offset_px: f32,
    velocity_px: f32,
    max_offset_px: f32,
}

impl Default for StoryScrollState {
    fn default() -> Self {
        Self {
            offset_px: 0.0,
            target_offset_px: 0.0,
            velocity_px: 0.0,
            max_offset_px: 0.0,
        }
    }
}

impl Plugin for StoryViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StoryView), setup_story_view)
            .init_resource::<StoryScrollState>()
            .add_systems(
                Update,
                (
                    fit_story_background_to_viewport,
                    read_story_scroll_input,
                    apply_story_scroll,
                    smooth_scroll,
                    continue_story,
                )
                    .run_if(in_state(AppState::StoryView)),
            )
            .add_systems(OnExit(AppState::StoryView), cleanup_story_view);
    }
}

fn setup_story_view(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pending_story: ResMut<PendingStoryScreen>,
    mut scroll_state: ResMut<StoryScrollState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    scroll_state.offset_px = 0.0;
    scroll_state.target_offset_px = 0.0;
    scroll_state.velocity_px = 0.0;
    scroll_state.max_offset_px = 0.0;

    let Some(story) = pending_story.take() else {
        next_state.set(AppState::MainMenu);
        return;
    };

    let story_text = read_asset_text_from_server(&asset_server, &story.text_asset_path)
        .unwrap_or_else(|error| format!("Story-Text konnte nicht geladen werden: {error}"));

    commands.spawn((
        Sprite::from_image(asset_server.load(&story.background_asset_path)),
        Transform::from_xyz(0.0, 0.0, -1.0),
        StoryBackground,
        StoryViewEntity,
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            StoryViewEntity,
            StoryContinueTarget(story.continue_to),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(50.0),
                        min_width: Val::Px(420.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(18.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
                    StoryViewEntity,
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        StoryTextViewport,
                        StoryViewEntity,
                    ))
                    .with_children(|viewport| {
                        viewport.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            StoryTextContent,
                            StoryViewEntity,
                        ))
                        .with_children(|content| {
                            let blocks = render_markdown_blocks(&story_text);
                            for block in blocks {
                                let font_size = markdown_font_size(&block.kind);
                                let text_color = markdown_text_color(&block.kind);
                                content.spawn((
                                    Text::new(block.text),
                                    TextFont {
                                        font_size,
                                        ..default()
                                    },
                                    TextColor(text_color),
                                    Node {
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                    StoryViewEntity,
                                ));
                            }
                        });
                    });

                    panel.spawn((
                        Text::new("Enter / Schiessen / Springen: Weiter"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.75, 0.75, 0.75)),
                        StoryViewEntity,
                    ));
                });
        });
}

fn fit_story_background_to_viewport(
    windows: Query<&Window, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    mut backgrounds: Query<(&Sprite, &mut Transform), With<StoryBackground>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let viewport = Vec2::new(window.width(), window.height());

    for (sprite, mut transform) in &mut backgrounds {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };

        let image_size = Vec2::new(
            image.texture_descriptor.size.width as f32,
            image.texture_descriptor.size.height as f32,
        );

        if image_size.x <= 0.0 || image_size.y <= 0.0 {
            continue;
        }

        // Contain scaling: keep full image visible without stretching.
        let scale = (viewport.x / image_size.x).min(viewport.y / image_size.y);
        transform.scale = Vec3::splat(scale);
    }
}

fn read_story_scroll_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut scroll: ResMut<StoryScrollState>,
) {
    const KEY_STEP_PX: f32 = 44.0;
    const WHEEL_STEP_PX: f32 = 42.0;

    let mut delta = 0.0;
    if keys.just_pressed(KeyCode::ArrowDown) {
        delta += KEY_STEP_PX;
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        delta -= KEY_STEP_PX;
    }

    for event in mouse_wheel.read() {
        delta -= event.y * WHEEL_STEP_PX;
    }

    if delta.abs() <= f32::EPSILON {
        return;
    }

    scroll.target_offset_px = (scroll.target_offset_px + delta).clamp(0.0, scroll.max_offset_px);
}

fn apply_story_scroll(
    viewport_query: Query<&ComputedNode, With<StoryTextViewport>>,
    mut content_query: Query<(&mut Node, &ComputedNode), With<StoryTextContent>>,
    mut scroll: ResMut<StoryScrollState>,
) {
    let Ok(viewport) = viewport_query.get_single() else {
        return;
    };
    let Ok((_content_style, content_node)) = content_query.get_single_mut() else {
        return;
    };

    let viewport_height = viewport.size().y;
    let content_height = content_node.size().y;

    scroll.max_offset_px = (content_height - viewport_height).max(0.0);
    scroll.target_offset_px = scroll.target_offset_px.clamp(0.0, scroll.max_offset_px);
    scroll.offset_px = scroll.offset_px.clamp(0.0, scroll.max_offset_px);
}

fn smooth_scroll(
    time: Res<Time>,
    mut scroll: ResMut<StoryScrollState>,
    mut content_query: Query<&mut Node, With<StoryTextContent>>,
) {
    let Ok(mut content_style) = content_query.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    // Simple critically-damped spring approximation for smooth following.
    const STIFFNESS: f32 = 200.0;
    const DAMPING: f32 = 30.0;

    let delta = scroll.target_offset_px - scroll.offset_px;
    // Integrate velocity
    scroll.velocity_px += delta * STIFFNESS * dt;
    // Apply damping
    scroll.velocity_px *= (1.0 / (1.0 + DAMPING * dt)).clamp(0.0, 1.0);
    // Integrate position
    scroll.offset_px += scroll.velocity_px * dt;

    // Clamp and zero velocity if hitting bounds
    if scroll.offset_px < 0.0 {
        scroll.offset_px = 0.0;
        scroll.velocity_px = 0.0;
    }
    if scroll.offset_px > scroll.max_offset_px {
        scroll.offset_px = scroll.max_offset_px;
        scroll.velocity_px = 0.0;
    }

    content_style.top = Val::Px(-scroll.offset_px);
}

fn continue_story(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    target_query: Query<&StoryContinueTarget>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(target) = target_query.get_single() else {
        return;
    };

    // Reserve arrow keys for scrolling while StoryView is active.
    let jump_pressed = keys.just_pressed(key_bindings.jump)
        && key_bindings.jump != KeyCode::ArrowUp
        && key_bindings.jump != KeyCode::ArrowDown;

    if keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::NumpadEnter)
        || keys.just_pressed(key_bindings.shoot)
        || jump_pressed
    {
        next_state.set(target.0);
    }
}

fn cleanup_story_view(mut commands: Commands, entities: Query<Entity, (With<StoryViewEntity>, Without<Parent>)>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}

fn read_asset_text_from_server(asset_server: &AssetServer, asset_path: &str) -> Result<String, String> {
    let source = asset_server
        .get_source(AssetSourceId::Default)
        .map_err(|error| format!("Asset source error: {error}"))?;

    let mut bytes = Vec::new();
    pollster::block_on(async {
        let mut reader = source
            .reader()
            .read(asset_path.as_ref())
            .await
            .map_err(|error| format!("Asset '{asset_path}' konnte nicht gelesen werden: {error}"))?;

        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|error| format!("Asset-Bytes fuer '{asset_path}' konnten nicht gelesen werden: {error}"))?;

        Ok::<(), String>(())
    })?;

    let text = String::from_utf8(bytes)
        .map_err(|error| format!("Asset '{asset_path}' ist kein gueltiges UTF-8: {error}"))?;

    Ok(text.trim_start_matches('\u{feff}').to_string())
}

fn render_markdown_blocks(md: &str) -> Vec<MarkdownBlock> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(md, options);

    let mut blocks: Vec<MarkdownBlock> = Vec::new();
    let mut list_stack: Vec<Option<u64>> = Vec::new();
    let mut current_kind: Option<MarkdownBlockKind> = None;
    let mut current_text = String::new();

    let flush_current = |blocks: &mut Vec<MarkdownBlock>,
                         current_kind: &mut Option<MarkdownBlockKind>,
                         current_text: &mut String| {
        if let Some(kind) = current_kind.take() {
            let text = current_text.trim().to_string();
            if !text.is_empty() {
                blocks.push(MarkdownBlock { kind, text });
            }
            current_text.clear();
        }
    };

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading(level, _, _) => {
                    flush_current(&mut blocks, &mut current_kind, &mut current_text);
                    let heading_level = match level {
                        pulldown_cmark::HeadingLevel::H1 => 1,
                        pulldown_cmark::HeadingLevel::H2 => 2,
                        pulldown_cmark::HeadingLevel::H3 => 3,
                        pulldown_cmark::HeadingLevel::H4 => 4,
                        pulldown_cmark::HeadingLevel::H5 => 5,
                        pulldown_cmark::HeadingLevel::H6 => 6,
                    };
                    current_kind = Some(MarkdownBlockKind::Heading(heading_level));
                }
                Tag::List(start) => list_stack.push(start),
                Tag::Item => {
                    flush_current(&mut blocks, &mut current_kind, &mut current_text);
                    current_kind = Some(MarkdownBlockKind::ListItem);
                    if let Some(Some(index)) = list_stack.last() {
                        current_text.push_str(&format!("{}. ", index));
                    } else {
                        current_text.push_str("- ");
                    }
                }
                Tag::Paragraph => {
                    if current_kind.is_none() {
                        current_kind = Some(MarkdownBlockKind::Paragraph);
                    }
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                Tag::Heading(_, _, _) => {
                    flush_current(&mut blocks, &mut current_kind, &mut current_text);
                }
                Tag::List(_) => {
                    list_stack.pop();
                }
                Tag::Item => {
                    flush_current(&mut blocks, &mut current_kind, &mut current_text);
                    if let Some(Some(index)) = list_stack.last_mut() {
                        *index += 1;
                    }
                }
                Tag::Paragraph => {
                    flush_current(&mut blocks, &mut current_kind, &mut current_text);
                }
                _ => {}
            },
            Event::Text(text) | Event::Code(text) => {
                if current_kind.is_none() {
                    current_kind = Some(MarkdownBlockKind::Paragraph);
                }
                current_text.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak => current_text.push('\n'),
            _ => {}
        }
    }

    flush_current(&mut blocks, &mut current_kind, &mut current_text);

    if blocks.is_empty() {
        let fallback = md.trim();
        if !fallback.is_empty() {
            blocks.push(MarkdownBlock {
                kind: MarkdownBlockKind::Paragraph,
                text: fallback.to_string(),
            });
        }
    }

    blocks
}

fn markdown_font_size(kind: &MarkdownBlockKind) -> f32 {
    match kind {
        MarkdownBlockKind::Heading(1) => 44.0,
        MarkdownBlockKind::Heading(2) => 38.0,
        MarkdownBlockKind::Heading(3) => 34.0,
        MarkdownBlockKind::Heading(_) => 30.0,
        MarkdownBlockKind::Paragraph => 28.0,
        MarkdownBlockKind::ListItem => 27.0,
    }
}

fn markdown_text_color(kind: &MarkdownBlockKind) -> Color {
    match kind {
        MarkdownBlockKind::Heading(_) => Color::srgb(1.0, 0.95, 0.85),
        _ => Color::WHITE,
    }
}

