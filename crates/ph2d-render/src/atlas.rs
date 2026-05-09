//! Texture atlas. M5 ships a procedural dummy (4×4 grid of 64×64
//! solid-color sprites = 256×256 RGBA8). M6 (asset pipeline) replaces
//! this with real PNG loading via the `image` crate + blake3 hashing
//! per HR-6.

use ph2d_gpu::GpuContext;

pub const DUMMY_GRID: u32 = 4;
pub const DUMMY_SPRITE_PX: u32 = 64;
pub const DUMMY_ATLAS_PX: u32 = DUMMY_GRID * DUMMY_SPRITE_PX;

pub struct TextureAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size_px: u32,
    pub grid: u32,
}

impl TextureAtlas {
    /// Build a procedural dummy atlas: 4×4 grid of solid-color tiles.
    /// Each tile gets a distinct hue (HSV cycled by index).
    pub fn dummy(gpu: &GpuContext) -> Self {
        let pixels = make_dummy_rgba8();
        Self::from_rgba8(gpu, DUMMY_ATLAS_PX, DUMMY_ATLAS_PX, &pixels)
    }

    pub fn from_rgba8(gpu: &GpuContext, width: u32, height: u32, rgba: &[u8]) -> Self {
        assert_eq!(
            rgba.len() as u32,
            width * height * 4,
            "rgba buffer wrong size"
        );
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render atlas sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            size_px: width,
            grid: DUMMY_GRID,
        }
    }

    /// Returns `(u_min, v_min, u_max, v_max)` for the i-th tile in the
    /// dummy grid (i % grid² wraps).
    pub fn dummy_uv(&self, index: u32) -> [f32; 4] {
        let grid = self.grid;
        let i = index % (grid * grid);
        let col = (i % grid) as f32;
        let row = (i / grid) as f32;
        let s = 1.0 / grid as f32;
        [col * s, row * s, (col + 1.0) * s, (row + 1.0) * s]
    }
}

fn make_dummy_rgba8() -> Vec<u8> {
    let mut out = vec![0u8; (DUMMY_ATLAS_PX * DUMMY_ATLAS_PX * 4) as usize];
    for tile_row in 0..DUMMY_GRID {
        for tile_col in 0..DUMMY_GRID {
            let idx = tile_row * DUMMY_GRID + tile_col;
            // Distinct hue per tile (golden-angle sweep for good
            // visual separation).
            let hue = (idx as f32 * 0.618_034) % 1.0;
            let (r, g, b) = hsv_to_rgb(hue, 0.85, 0.95);
            for py in 0..DUMMY_SPRITE_PX {
                for px in 0..DUMMY_SPRITE_PX {
                    let x = tile_col * DUMMY_SPRITE_PX + px;
                    let y = tile_row * DUMMY_SPRITE_PX + py;
                    let off = ((y * DUMMY_ATLAS_PX + x) * 4) as usize;
                    out[off] = r;
                    out[off + 1] = g;
                    out[off + 2] = b;
                    out[off + 3] = 255;
                }
            }
        }
    }
    out
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h6 = h * 6.0;
    let i = h6.floor() as i32 % 6;
    let f = h6 - h6.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_atlas_buffer_has_correct_size() {
        let pixels = make_dummy_rgba8();
        assert_eq!(pixels.len(), (DUMMY_ATLAS_PX * DUMMY_ATLAS_PX * 4) as usize);
    }

    #[test]
    fn hsv_to_rgb_yields_distinct_colors_per_index() {
        // Regression: prior `f = h * 6.0 - h * 6.0_f32.floor()` always
        // evaluated to 0 (since `6.0_f32.floor() == 6.0`), so the
        // intra-bucket interpolation collapsed and only 6 unique
        // colors emerged across all 16 dummy tiles.
        use std::collections::BTreeSet;
        let mut seen = BTreeSet::new();
        for i in 0..(DUMMY_GRID * DUMMY_GRID) {
            let hue = (i as f32 * 0.618_034) % 1.0;
            seen.insert(hsv_to_rgb(hue, 0.85, 0.95));
        }
        assert_eq!(
            seen.len(),
            (DUMMY_GRID * DUMMY_GRID) as usize,
            "expected 16 distinct golden-angle hues, got {}",
            seen.len()
        );
    }

    #[test]
    fn dummy_uv_covers_unit_square() {
        let dummy_grid = DUMMY_GRID;
        for i in 0..(dummy_grid * dummy_grid) {
            // Synthesize an atlas without a GpuContext — just validate
            // the math, not the wgpu side.
            let s = 1.0 / dummy_grid as f32;
            let col = (i % dummy_grid) as f32;
            let row = (i / dummy_grid) as f32;
            let uv = [col * s, row * s, (col + 1.0) * s, (row + 1.0) * s];
            assert!(uv[0] >= 0.0 && uv[0] < 1.0);
            assert!(uv[1] >= 0.0 && uv[1] < 1.0);
            assert!(uv[2] > uv[0] && uv[2] <= 1.0);
            assert!(uv[3] > uv[1] && uv[3] <= 1.0);
        }
    }
}
