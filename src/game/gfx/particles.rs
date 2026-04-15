use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub(crate) fn create_round_particle_image(size: u32) -> Image {
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
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub(crate) fn create_plasma_particle_image(size: u32) -> Image {
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = (size as f32 - 1.0) * 0.5;
    let radius = center.max(1.0);

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt() / radius;
            let softness = (1.0 - distance).clamp(0.0, 1.0);

            // Brighter and broader profile: a hot white center with a vivid outer glow.
            let core = ((softness - 0.38) / 0.62).clamp(0.0, 1.0).powf(1.35);
            let halo = softness.powf(1.85);

            // Keep a bright center while allowing a wider luminous fringe.
            let alpha_f = (core * 0.98 + halo * 0.55).clamp(0.0, 0.98);
            let alpha = (alpha_f * 255.0) as u8;

            // White-hot center that shifts into a stronger cyan glow toward the edge.
            let r = (165.0 + core * 90.0 + halo * 30.0).clamp(0.0, 255.0) as u8;
            let g = (190.0 + core * 65.0 + halo * 65.0).clamp(0.0, 255.0) as u8;
            let b = (225.0 + core * 30.0 + halo * 30.0).clamp(0.0, 255.0) as u8;

            let index = ((y * size + x) * 4) as usize;
            data[index] = r;
            data[index + 1] = g;
            data[index + 2] = b;
            data[index + 3] = alpha;
        }
    }

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub(crate) fn create_fire_particle_image(size: u32) -> Image {
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = (size as f32 - 1.0) * 0.5;
    let radius = center.max(1.0);

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt() / radius;
            let softness = (1.0 - dist).clamp(0.0, 1.0);

            // Sharp hot core, broad orange glow
            let core = ((softness - 0.28) / 0.72).clamp(0.0, 1.0).powf(1.4);
            let halo = softness.powf(1.7);

            let alpha_f = (core * 1.0 + halo * 0.55).clamp(0.0, 1.0);
            let alpha = (alpha_f * 255.0) as u8;

            // White-hot center → vivid orange glow toward the edge
            let r = 255_u8;
            let g = (255.0 * (core * 0.82 + halo * 0.38)).clamp(0.0, 255.0) as u8;
            let b = (255.0 * (core * 0.20 + halo * 0.02)).clamp(0.0, 255.0) as u8;

            let index = ((y * size + x) * 4) as usize;
            data[index] = r;
            data[index + 1] = g;
            data[index + 2] = b;
            data[index + 3] = alpha;
        }
    }

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
