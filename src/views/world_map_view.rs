use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::window::PrimaryWindow;
use std::f32::consts::TAU;

use crate::game::world::WorldCatalog;
use crate::game::world::find_directional_neighbor;
use crate::{
    AppState, CampaignProgress, LevelSelection, MainCamera, PendingStoryScreen, StoryScreenRequest,
    WorldMapSelection,
};
use crate::i18n::{Translations, CurrentLanguage};

pub struct WorldMapViewPlugin;

#[derive(Component)]
struct WorldMapEntity;

#[derive(Component)]
struct WorldMapBackground;

#[derive(Component)]
struct WorldMapPlanetLabel;

#[derive(Component)]
struct WorldMapSelectionParticle {
    base_offset: Vec2,
    orbit_speed: f32,
    size: f32,
    phase_x: f32,
    phase_y: f32,
    speed_x: f32,
    speed_y: f32,
}

#[derive(Resource, Default, Clone, Copy)]
struct WorldMapRenderConfig {
    virtual_size: Vec2,
    scale: f32,
}

impl Plugin for WorldMapViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldMapRenderConfig>()
            .add_systems(OnEnter(AppState::WorldMapView), setup_world_map_view)
            .add_systems(
                Update,
                (
                    update_world_map_layout,
                    draw_planet_overlays,
                    select_planet_by_keyboard,
                    select_planet_by_click,
                    start_selected_planet,
                    update_planet_label,
                    return_to_world_select,
                )
                    .run_if(in_state(AppState::WorldMapView)),
            )
            .add_systems(OnExit(AppState::WorldMapView), cleanup_world_map_view);
    }
}

fn setup_world_map_view(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    world_catalog: Res<WorldCatalog>,
    mut selection: ResMut<WorldMapSelection>,
    mut progress: ResMut<CampaignProgress>,
    mut pending_story: ResMut<PendingStoryScreen>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(world_index) = progress.world_index else {
        return;
    };

    let Some(world_entry) = world_catalog.world(world_index) else {
        return;
    };

    if !progress.world_start_story_seen {
        if let Some(story_start) = world_entry
            .definition
            .story
            .as_ref()
            .and_then(|story| story.start.as_ref())
        {
            progress.world_start_story_seen = true;
            pending_story.set(StoryScreenRequest {
                text_asset_path: story_start.text.clone(),
                background_asset_path: story_start.background.clone(),
                continue_to: AppState::WorldMapView,
            });
            next_state.set(AppState::StoryView);
            return;
        }

        progress.world_start_story_seen = true;
    }

    if world_entry.definition.planets.is_empty() {
        return;
    }

    selection.index = selection
        .index
        .min(world_entry.definition.planets.len().saturating_sub(1));

    commands.spawn((
        Sprite::from_image(asset_server.load(&world_entry.definition.background)),
        Transform::from_xyz(0.0, 0.0, -1.0),
        WorldMapBackground,
        WorldMapEntity,
    ));

    let particle_image = ensure_world_map_particle_image(&mut images);
    for index in 0..30 {
        let seed = index as f32;
        let base_offset = random_point_in_unit_disk(seed * 11.7 + 0.9)
            * hash_range(seed * 5.3 + 3.4, 0.35, 1.9);
        let orbit_speed = hash_range(seed * 19.4 + 2.2, -0.45, 0.45);
        let size = 20.0 + hash01(seed * 17.1 + 4.2) * 14.0;
        let speed_x = 0.8 + hash01(seed * 2.9 + 1.7) * 1.2;
        let speed_y = 0.9 + hash01(seed * 3.7 + 9.1) * 1.1;

        commands.spawn((
            Sprite {
                image: particle_image.clone(),
                color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 2.0),
            WorldMapSelectionParticle {
                base_offset,
                orbit_speed,
                size,
                phase_x: hash01(seed * 13.9 + 5.5) * TAU,
                phase_y: hash01(seed * 7.1 + 8.8) * TAU,
                speed_x,
                speed_y,
            },
            WorldMapEntity,
        ));
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                top: Val::Px(24.0),
                ..default()
            },
            WorldMapEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                WorldMapPlanetLabel,
                WorldMapEntity,
            ));
        });
}

fn update_world_map_layout(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<&Camera, With<MainCamera>>,
    world_catalog: Res<WorldCatalog>,
    progress: Res<CampaignProgress>,
    mut render_config: ResMut<WorldMapRenderConfig>,
    mut backgrounds: Query<(&mut Sprite, &mut Transform), With<WorldMapBackground>>,
) {
    let Some(world) = selected_world(&world_catalog, &progress) else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok(camera) = camera_query.single() else {
        return;
    };

    let Ok((mut sprite, mut transform)) = backgrounds.single_mut() else {
        return;
    };

    let virtual_size = world.virtual_size();
    if virtual_size.x <= 0.0 || virtual_size.y <= 0.0 {
        return;
    }

    let viewport_size = camera
        .logical_viewport_size()
        .unwrap_or(Vec2::new(window.width(), window.height()));
    let scale = (viewport_size.x / virtual_size.x).min(viewport_size.y / virtual_size.y);

    sprite.custom_size = Some(virtual_size);
    transform.scale = Vec3::splat(scale);
    render_config.virtual_size = virtual_size;
    render_config.scale = scale;
}

fn draw_planet_overlays(
    world_catalog: Res<WorldCatalog>,
    progress: Res<CampaignProgress>,
    selection: Res<WorldMapSelection>,
    render_config: Res<WorldMapRenderConfig>,
    time: Res<Time>,
    mut particles: Query<(&WorldMapSelectionParticle, &mut Transform, &mut Sprite)>,
) {
    const PARTICLE_SIZE_MULTIPLIER: f32 = 1.4;
    const PARTICLE_BRIGHTNESS_MULTIPLIER: f32 = 4.0;

    let Some(world) = selected_world(&world_catalog, &progress) else {
        return;
    };

    let Some(planet) = world.planets.get(selection.index) else {
        return;
    };

    let center = map_to_world(planet.position_vec2(), *render_config);
    let radius = (planet.radius * render_config.scale).max(4.0);
    let base_color = planet.color_vec3();
    let color = (base_color * PARTICLE_BRIGHTNESS_MULTIPLIER).min(Vec3::ONE);
    let elapsed = time.elapsed_secs();

    for (particle, mut transform, mut sprite) in &mut particles {
        let orbit_angle = elapsed * particle.orbit_speed;
        let orbit_offset = rotate_vec2(particle.base_offset, orbit_angle) * radius;
        let base_pos = center + orbit_offset;

        let wobble = Vec2::new(
            (elapsed * particle.speed_x + particle.phase_x).sin() * 3.8,
            (elapsed * particle.speed_y + particle.phase_y).cos() * 3.8,
        );
        let position = base_pos + wobble;
        transform.translation.x = position.x;
        transform.translation.y = position.y;

        let shimmer = (elapsed * 2.2 + particle.phase_x).sin() * 0.5 + 0.5;
        let size = particle.size * (1.05 + shimmer * 0.55) * PARTICLE_SIZE_MULTIPLIER;
        let alpha = 0.92 + shimmer * 0.08;

        sprite.custom_size = Some(Vec2::splat(size));
        sprite.color = Color::srgba(color.x, color.y, color.z, alpha);
    }
}

fn hash01(value: f32) -> f32 {
    (value.sin() * 43_758.547).fract().abs()
}

fn hash_range(seed: f32, min: f32, max: f32) -> f32 {
    min + hash01(seed) * (max - min)
}

fn random_point_in_unit_disk(seed: f32) -> Vec2 {
    let angle = hash01(seed * 1.31 + 0.17) * TAU;
    let radius = hash01(seed * 2.71 + 0.63).sqrt();
    Vec2::new(angle.cos(), angle.sin()) * radius
}

fn rotate_vec2(value: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(value.x * cos - value.y * sin, value.x * sin + value.y * cos)
}

fn ensure_world_map_particle_image(images: &mut Assets<Image>) -> Handle<Image> {
    images.add(create_round_particle_image(32))
}

fn create_round_particle_image(size: u32) -> Image {
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = (size as f32 - 1.0) * 0.5;
    let radius = center.max(1.0);

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt() / radius;
            let softness = (1.0 - distance).clamp(0.0, 1.0);
            let alpha = (softness * softness * 255.0) as u8;

            let index = ((y * size + x) * 4) as usize;
            data[index] = 255;
            data[index + 1] = 255;
            data[index + 2] = 255;
            data[index + 3] = alpha;
        }
    }

    Image::new(
        bevy::render::render_resource::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn select_planet_by_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    world_catalog: Res<WorldCatalog>,
    progress: Res<CampaignProgress>,
    mut selection: ResMut<WorldMapSelection>,
) {
    let Some(world) = selected_world(&world_catalog, &progress) else {
        return;
    };

    if world.planets.is_empty() {
        return;
    }

    let direction = if keys.just_pressed(KeyCode::ArrowLeft) {
        Some(Vec2::new(-1.0, 0.0))
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        Some(Vec2::new(1.0, 0.0))
    } else if keys.just_pressed(KeyCode::ArrowUp) {
        Some(Vec2::new(0.0, 1.0))
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        Some(Vec2::new(0.0, -1.0))
    } else {
        None
    };

    let Some(direction) = direction else {
        return;
    };

    if let Some(next_index) = find_directional_neighbor(&world.planets, selection.index, direction) {
        selection.index = next_index;
    }
}

fn select_planet_by_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    world_catalog: Res<WorldCatalog>,
    progress: Res<CampaignProgress>,
    render_config: Res<WorldMapRenderConfig>,
    mut selection: ResMut<WorldMapSelection>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world) = selected_world(&world_catalog, &progress) else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(world_cursor) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for (index, planet) in world.planets.iter().enumerate() {
        let center = map_to_world(planet.position_vec2(), *render_config);
        let radius = (planet.radius * render_config.scale).max(4.0);

        if world_cursor.distance(center) <= radius {
            selection.index = index;
            break;
        }
    }
}

fn start_selected_planet(
    keys: Res<ButtonInput<KeyCode>>,
    world_catalog: Res<WorldCatalog>,
    mut progress: ResMut<CampaignProgress>,
    selection: Res<WorldMapSelection>,
    mut level_selection: ResMut<LevelSelection>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !keys.just_pressed(KeyCode::Enter) && !keys.just_pressed(KeyCode::NumpadEnter) {
        return;
    }

    let Some(world) = selected_world(&world_catalog, &progress) else {
        return;
    };

    let Some(planet) = world.planets.get(selection.index) else {
        return;
    };

    let Some(first_level) = planet.levels.first() else {
        return;
    };

    progress.planet_index = Some(selection.index);
    progress.level_index = 0;
    level_selection.set_asset_path(&first_level.json);
    next_state.set(AppState::LoadView);
}

fn update_planet_label(
    world_catalog: Res<WorldCatalog>,
    progress: Res<CampaignProgress>,
    selection: Res<WorldMapSelection>,
    mut labels: Query<&mut Text, With<WorldMapPlanetLabel>>,
    translations: Res<Translations>,
    current: Res<CurrentLanguage>,
) {
    let Ok(mut label) = labels.single_mut() else {
        return;
    };

    let Some(world) = selected_world(&world_catalog, &progress) else {
        return;
    };

    let Some(planet) = world.planets.get(selection.index) else {
        label.0 = translations
            .tr(&current.0, "worldmap.no_selection_hint")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Arrow/click: choose planet | Enter: start | Esc: worlds".to_string());
        return;
    };

    let level_info = if planet.levels.is_empty() {
        translations
            .tr(&current.0, "worldmap.no_level")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "(no levels)".to_string())
    } else {
        let first_level = &planet.levels[0];
        format!("{} Level (Start: {})", planet.levels.len(), first_level.name)
    };

    if let Some(fmt) = translations.tr(&current.0, "worldmap.label_format") {
        label.0 = fmt.replace("{world}", &world.name).replace("{planet}", &planet.name).replace("{level_info}", &level_info);
    } else {
        label.0 = format!(
            "World: {} | Planet: {} {} | Enter: Start | Esc: Worlds",
            world.name, planet.name, level_info
        );
    }
}

fn return_to_world_select(
    keys: Res<ButtonInput<KeyCode>>,
    world_catalog: Res<WorldCatalog>,
    mut progress: ResMut<CampaignProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        progress.clear_planet_progress();
        // If there's only one world available, skip the world selection and
        // return to the main menu instead of StartView.
        if world_catalog.worlds().len() == 1 {
            next_state.set(AppState::MainMenu);
        } else {
            next_state.set(AppState::StartView);
        }
    }
}

fn cleanup_world_map_view(mut commands: Commands, entities: Query<Entity, (With<WorldMapEntity>, Without<ChildOf>)>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

fn selected_world<'a>(world_catalog: &'a WorldCatalog, progress: &CampaignProgress) -> Option<&'a crate::game::world::WorldDefinition> {
    let world_index = progress.world_index?;
    world_catalog.world(world_index).map(|entry| &entry.definition)
}

fn map_to_world(position: Vec2, config: WorldMapRenderConfig) -> Vec2 {
    let local = position - (config.virtual_size * 0.5);
    local * config.scale
}


