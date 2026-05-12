//! Texture atlas — single GPU texture holding many sprites at native
//! resolution, with a Skyline rect packer choosing each sprite's
//! (x, y) placement at insert time.
//!
//! ## Why dynamic packing
//!
//! M5/M6 shipped a placeholder atlas (a 4×4 grid of 64×64 cells) so
//! the demo could render the HSV dummy tiles. M14.4c reused that
//! grid for image import, which forced every imported PNG into a
//! 64×64 cell — sources lost > 99 % of their pixels and any non-
//! square aspect ratio got stretched to a square. The plan all
//! along (see the comment that lived at the top of this file
//! before M14.4d) was for M6 to swap the grid for real PNG loading;
//! the grid-shaped placeholder leaked into M14.4c and never got
//! removed. This module is the swap.
//!
//! Each insert reserves a region of the atlas large enough for the
//! source's native dimensions. Sprites are looked up by `u32` key
//! (the demo seeds keys 0..15 for its HSV tiles; the importer
//! allocates `next_atlas_key()` keys 16, 17, …). Imported sprites
//! keep their full source resolution up to the atlas size limit
//! ([`ATLAS_DEFAULT_SIZE_PX`]).
//!
//! ## Algorithm
//!
//! [`rect_packer`] under the hood (Skyline heuristic). Cheap (O(n)
//! per insert against n existing skyline nodes) and packs well for
//! sprites of similar height, which is the common case for pixel-
//! art and small UI glyphs. We don't bother with rotation — Skyline
//! handles axis-aligned packs and rotated atlases complicate the
//! UV math downstream for negligible packing gain.
//!
//! ## Growth
//!
//! v1 returns [`AtlasFull`] on overflow and surfaces it as a toast
//! at the import site. Atlas regrow (re-pack everything into a
//! 2×-larger texture) is a follow-up — see the plan §Backlog.

use ph2d_gpu::GpuContext;
// BTreeMap (not HashMap): HR-5 / ADR-0022 forbid unordered maps in
// any path that contributes to a deterministic snapshot. Atlas
// lookups aren't snapshot-relevant (per-frame UV resolution only),
// but the project's lint blocks `std::collections::HashMap`
// project-wide. BTreeMap is fine here — region count stays small
// (≤ a few hundred sprites in practice) so the log-N cost vs a hash
// map is dwarfed by the GPU work that follows the lookup.
use std::collections::BTreeMap;

/// Default side length for new atlases (square, RGBA8). 4096 ×
/// 4096 = 64 MiB of GPU memory — large enough for a few hundred
/// 256-px sprites, comfortably under the wgpu Default tier's
/// `max_texture_dimension_2d = 8192` limit on every backend we
/// target.
pub const ATLAS_DEFAULT_SIZE_PX: u32 = 4096;

/// Pixel side length of each tile in the demo HSV atlas. Used by
/// [`TextureAtlas::dummy`] only; arbitrary-sized sprites can live
/// in the atlas alongside the demo tiles via [`TextureAtlas::insert`].
pub const DEMO_TILE_PX: u32 = 64;

/// Number of HSV demo tiles seeded by [`TextureAtlas::dummy`]. Keys
/// `0..DEMO_TILE_COUNT` are reserved for demo content; user imports
/// should start their key allocator at [`FIRST_IMPORT_KEY`].
pub const DEMO_TILE_COUNT: u32 = 16;

/// First key the importer is free to allocate for user sprites.
/// Sits past the demo-tile range so the demo's HSV grid stays
/// intact while imports are spawned and despawned.
pub const FIRST_IMPORT_KEY: u32 = 16;

/// A reserved rectangle inside the atlas, in atlas-pixel coordinates.
/// Returned by [`TextureAtlas::insert`] / [`TextureAtlas::region`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl AtlasRegion {
    /// Convert pixel coordinates into normalized texture UV
    /// `(u_min, v_min, u_max, v_max)` for the given atlas side
    /// length. Hot path: called per sprite per frame in extract.
    pub fn uv(self, atlas_size: u32) -> [f32; 4] {
        let s = atlas_size.max(1) as f32;
        [
            self.x as f32 / s,
            self.y as f32 / s,
            (self.x + self.w) as f32 / s,
            (self.y + self.h) as f32 / s,
        ]
    }
}

/// Returned by [`TextureAtlas::insert`] when the source can't be
/// packed into the remaining atlas space (or is larger than the
/// atlas itself). Surface as a toast at the call site; the v2
/// regrow strategy will catch this internally and double the
/// texture before retrying.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AtlasInsertError {
    /// No skyline slot fits the requested `width × height`.
    AtlasFull { width: u32, height: u32 },
    /// Source dimensions exceed the atlas's hard size (i.e. would
    /// never fit even into an empty atlas).
    SourceTooLarge { width: u32, height: u32, atlas: u32 },
}

impl std::fmt::Display for AtlasInsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AtlasFull { width, height } => {
                write!(f, "atlas full: no room for {width}×{height} sprite")
            }
            Self::SourceTooLarge {
                width,
                height,
                atlas,
            } => write!(f, "source {width}×{height} exceeds atlas {atlas}×{atlas}"),
        }
    }
}

impl std::error::Error for AtlasInsertError {}

pub struct TextureAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size_px: u32,
    packer: rect_packer::DensePacker,
    regions: BTreeMap<u32, AtlasRegion>,
}

impl TextureAtlas {
    /// Build an empty atlas of `size_px × size_px` RGBA8 (sRGB).
    /// Texture is allocated up-front; bytes start as garbage and
    /// only the regions touched by [`Self::insert`] /
    /// [`Self::update_region`] are deterministic.
    pub fn new(gpu: &GpuContext, size_px: u32) -> Self {
        let size_px = size_px.max(1);
        let (texture, view) = create_texture(&gpu.device, size_px);
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render atlas sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // Linear filtering is more forgiving than Nearest for
            // sprites resampled at non-integer scale (which is
            // common once the camera zooms). Nearest stays
            // available behind a feature flag in M15+ for pixel-
            // art mode.
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            size_px,
            packer: rect_packer::DensePacker::new(size_px as i32, size_px as i32),
            regions: BTreeMap::new(),
        }
    }

    /// Build the demo atlas — empty atlas + 16 HSV-tinted 64×64
    /// tiles inserted at keys `0..16`. Used by the 1000-sprite
    /// demo and by tests that need a populated atlas.
    pub fn dummy(gpu: &GpuContext) -> Self {
        let mut atlas = Self::new(gpu, ATLAS_DEFAULT_SIZE_PX);
        for i in 0..DEMO_TILE_COUNT {
            let pixels = make_hsv_tile(i, DEMO_TILE_PX);
            atlas
                .insert(gpu, i, DEMO_TILE_PX, DEMO_TILE_PX, &pixels)
                .expect("demo HSV tiles always fit into a fresh 4096² atlas");
        }
        atlas
    }

    /// Pack an `(width × height)` source at native resolution into
    /// the atlas under `key` and upload the bytes. Returns the
    /// resulting region's pixel rect on success.
    ///
    /// `rgba` must be tightly-packed `width * height * 4` bytes
    /// (no row padding — wgpu requires `bytes_per_row` aligned to
    /// 256, which we handle internally if needed).
    ///
    /// If `key` was already inserted this is treated as a *replace*
    /// — the existing region is reused (no re-pack) and only the
    /// pixels are rewritten. Source dimensions in the replace path
    /// MUST equal the original `(w, h)` or `AtlasInsertError`
    /// surfaces. Callers wanting to swap a sprite for a different-
    /// sized source should `remove` first (TODO when regrow lands).
    pub fn insert(
        &mut self,
        gpu: &GpuContext,
        key: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<AtlasRegion, AtlasInsertError> {
        if width == 0 || height == 0 {
            return Err(AtlasInsertError::AtlasFull { width, height });
        }
        if width > self.size_px || height > self.size_px {
            return Err(AtlasInsertError::SourceTooLarge {
                width,
                height,
                atlas: self.size_px,
            });
        }
        assert_eq!(
            rgba.len() as u32,
            width * height * 4,
            "insert: rgba buffer must be width*height*4 = {} bytes (got {})",
            width * height * 4,
            rgba.len()
        );

        // Replace path: same key + same size → rewrite pixels in
        // the existing region. Different size invalidates the
        // packer's reservation, so we surface an error rather than
        // silently leaking the slot. Callers can `remove(key)` once
        // that API exists (post-regrow).
        if let Some(existing) = self.regions.get(&key).copied() {
            if existing.w == width && existing.h == height {
                upload_region(gpu, &self.texture, existing, rgba);
                return Ok(existing);
            }
            return Err(AtlasInsertError::AtlasFull { width, height });
        }

        // Fresh insert — ask the packer for a free slot. `false`
        // disables rotation (we don't want rotated UVs upstream).
        let rect = self
            .packer
            .pack(width as i32, height as i32, false)
            .ok_or(AtlasInsertError::AtlasFull { width, height })?;
        let region = AtlasRegion {
            x: rect.x as u32,
            y: rect.y as u32,
            w: width,
            h: height,
        };
        upload_region(gpu, &self.texture, region, rgba);
        self.regions.insert(key, region);
        Ok(region)
    }

    /// Lookup the reserved region for `key` (returned by an earlier
    /// [`Self::insert`]). `None` when the key was never inserted.
    pub fn region(&self, key: u32) -> Option<AtlasRegion> {
        self.regions.get(&key).copied()
    }

    /// Lookup the UV rectangle for `key` (sugar over
    /// `region(key).map(|r| r.uv(size_px))`). Returns a
    /// zero-sized UV at the origin when `key` is missing, so the
    /// extract path can emit a draw without an early-return branch
    /// — the sprite renders as a 1-texel sample that is visually
    /// inert and easy to spot in screenshots if it ever happens.
    pub fn region_uv(&self, key: u32) -> [f32; 4] {
        match self.regions.get(&key) {
            Some(r) => r.uv(self.size_px),
            None => [0.0, 0.0, 0.0, 0.0],
        }
    }

    /// Number of reserved regions. Used by tests; not perf-critical.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }
}

/// Upload `rgba` into `texture` at `region`'s pixel offset.
/// `bytes_per_row` matches the region's stride exactly — wgpu's
/// 256-byte alignment requirement only applies to
/// `copy_buffer_to_texture`, NOT `write_texture` (which takes the
/// stride verbatim and handles internal staging).
fn upload_region(gpu: &GpuContext, texture: &wgpu::Texture, region: AtlasRegion, rgba: &[u8]) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: region.x,
                y: region.y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(region.w * 4),
            rows_per_image: Some(region.h),
        },
        wgpu::Extent3d {
            width: region.w,
            height: region.h,
            depth_or_array_layers: 1,
        },
    );
}

fn create_texture(device: &wgpu::Device, size_px: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render atlas"),
        size: wgpu::Extent3d {
            width: size_px,
            height: size_px,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// Build a `DEMO_TILE_PX × DEMO_TILE_PX` RGBA tile filled with the
/// `index`-th golden-angle HSV color. Used by [`TextureAtlas::dummy`].
fn make_hsv_tile(index: u32, side: u32) -> Vec<u8> {
    let hue = (index as f32 * 0.618_034) % 1.0;
    let (r, g, b) = hsv_to_rgb(hue, 0.85, 0.95);
    let mut out = vec![0u8; (side * side * 4) as usize];
    for chunk in out.chunks_exact_mut(4) {
        chunk[0] = r;
        chunk[1] = g;
        chunk[2] = b;
        chunk[3] = 255;
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

    fn try_headless_gpu() -> Option<GpuContext> {
        let instance = GpuContext::default_instance();
        GpuContext::new(instance, None).ok()
    }

    #[test]
    fn region_uv_normalizes_to_unit_square() {
        let region = AtlasRegion {
            x: 100,
            y: 200,
            w: 50,
            h: 40,
        };
        let uv = region.uv(1000);
        assert!((uv[0] - 0.1).abs() < 1e-6);
        assert!((uv[1] - 0.2).abs() < 1e-6);
        assert!((uv[2] - 0.15).abs() < 1e-6);
        assert!((uv[3] - 0.24).abs() < 1e-6);
    }

    #[test]
    fn region_uv_handles_zero_atlas_size() {
        // Defensive: a zero-size atlas would NaN under naïve `/`.
        // `uv` clamps `atlas_size.max(1)` so the math stays sane.
        let region = AtlasRegion {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
        let uv = region.uv(0);
        assert_eq!(uv, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn hsv_to_rgb_yields_distinct_colors_per_index() {
        use std::collections::BTreeSet;
        let mut seen = BTreeSet::new();
        for i in 0..DEMO_TILE_COUNT {
            let hue = (i as f32 * 0.618_034) % 1.0;
            seen.insert(hsv_to_rgb(hue, 0.85, 0.95));
        }
        assert_eq!(
            seen.len(),
            DEMO_TILE_COUNT as usize,
            "expected {DEMO_TILE_COUNT} distinct golden-angle hues, got {}",
            seen.len()
        );
    }

    #[test]
    fn make_hsv_tile_has_correct_size() {
        let tile = make_hsv_tile(0, DEMO_TILE_PX);
        assert_eq!(tile.len(), (DEMO_TILE_PX * DEMO_TILE_PX * 4) as usize);
        // Every pixel uniform — solid color.
        for chunk in tile.chunks_exact(4) {
            assert_eq!(chunk[0], tile[0]);
            assert_eq!(chunk[1], tile[1]);
            assert_eq!(chunk[2], tile[2]);
            assert_eq!(chunk[3], 255);
        }
    }

    #[test]
    fn insert_returns_region_with_requested_dimensions() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 1024);
        let rgba = vec![0u8; (128 * 64 * 4) as usize];
        let region = atlas.insert(&gpu, 0, 128, 64, &rgba).expect("insert ok");
        assert_eq!(region.w, 128);
        assert_eq!(region.h, 64);
        assert_eq!(atlas.region(0), Some(region));
    }

    #[test]
    fn insert_packs_multiple_arbitrary_sizes() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 1024);
        let sizes = [(200, 100), (50, 50), (300, 80), (10, 200)];
        for (i, &(w, h)) in sizes.iter().enumerate() {
            let rgba = vec![0u8; (w * h * 4) as usize];
            atlas
                .insert(&gpu, i as u32, w, h, &rgba)
                .expect("insert ok");
        }
        assert_eq!(atlas.region_count(), sizes.len());
        // No two regions overlap — Skyline guarantee but pin it here
        // so a future packer swap stays honest.
        let collected: Vec<AtlasRegion> = (0..sizes.len() as u32)
            .map(|k| atlas.region(k).unwrap())
            .collect();
        for (i, a) in collected.iter().enumerate() {
            for b in &collected[i + 1..] {
                let overlap =
                    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
                assert!(!overlap, "regions overlap: {a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn insert_rejects_source_too_large_for_atlas() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        let rgba = vec![0u8; (512 * 16 * 4) as usize];
        let err = atlas
            .insert(&gpu, 0, 512, 16, &rgba)
            .expect_err("expected SourceTooLarge");
        assert!(matches!(err, AtlasInsertError::SourceTooLarge { .. }));
    }

    #[test]
    fn insert_returns_full_when_packer_exhausted() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 128);
        // First 128×128 fills the atlas; second fails.
        let rgba = vec![0u8; (128 * 128 * 4) as usize];
        atlas.insert(&gpu, 0, 128, 128, &rgba).unwrap();
        let err = atlas.insert(&gpu, 1, 64, 64, &vec![0u8; (64 * 64 * 4) as usize]);
        assert!(matches!(err, Err(AtlasInsertError::AtlasFull { .. })));
    }

    #[test]
    fn replace_same_size_succeeds_without_repack() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        let rgba_a = vec![0xAAu8; (64 * 64 * 4) as usize];
        let region_a = atlas.insert(&gpu, 7, 64, 64, &rgba_a).unwrap();
        // Same key + same size → same region (the packer is NOT
        // asked again, important for the hot-reload path).
        let rgba_b = vec![0xBBu8; (64 * 64 * 4) as usize];
        let region_b = atlas.insert(&gpu, 7, 64, 64, &rgba_b).unwrap();
        assert_eq!(region_a, region_b);
        assert_eq!(atlas.region_count(), 1);
    }

    #[test]
    fn dummy_atlas_seeds_16_demo_keys() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let atlas = TextureAtlas::dummy(&gpu);
        assert_eq!(atlas.region_count(), DEMO_TILE_COUNT as usize);
        for i in 0..DEMO_TILE_COUNT {
            let region = atlas.region(i).unwrap_or_else(|| panic!("missing key {i}"));
            assert_eq!(region.w, DEMO_TILE_PX);
            assert_eq!(region.h, DEMO_TILE_PX);
        }
    }
}
