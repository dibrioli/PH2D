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

/// Default side length for new atlases (square, RGBA8). 8192 ×
/// 8192 = 256 MiB of GPU memory — enough for several 4K sprites
/// (3840 × 2160 ≈ 8 MiB each) plus the usual HD-2D mix, and exactly
/// at the wgpu Default tier's `max_texture_dimension_2d = 8192`
/// limit on every backend we target. Smaller atlases force regrow
/// on the first 4K import.
pub const ATLAS_DEFAULT_SIZE_PX: u32 = 8192;

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
    ///
    /// **Half-texel inset**: each boundary moves 0.5 px toward the
    /// region center before normalization. With `FilterMode::Linear`
    /// the sampler reads a 2×2 texel neighborhood at the UV corner;
    /// without the inset that neighborhood straddles the region
    /// edge and pulls neighboring atlas content into the bilinear
    /// blend. The visible artifact (user-reported "linha na margem
    /// esquerda") was an opaque black halo from the empty-pixel
    /// rows that border every packed region. Inset by half a texel
    /// shifts the sample square strictly inside, killing the bleed
    /// without measurable cropping (the half-texel offset is well
    /// below the visible threshold at any reasonable zoom level).
    pub fn uv(self, atlas_size: u32) -> [f32; 4] {
        let s = atlas_size.max(1) as f32;
        let inset = 0.5;
        [
            (self.x as f32 + inset) / s,
            (self.y as f32 + inset) / s,
            (self.x as f32 + self.w as f32 - inset) / s,
            (self.y as f32 + self.h as f32 - inset) / s,
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
    /// Atlas V2 (M14.4f): free-list of regions made available by
    /// [`Self::remove`] (and by replace-with-different-size, which
    /// would otherwise leak the slot). Indexed by `(width, height)`
    /// so an `insert` of matching dimensions can reuse a slot without
    /// touching the Skyline packer. BTreeMap (not HashMap) per HR-5.
    free_slots: BTreeMap<(u32, u32), Vec<AtlasRegion>>,
    /// Hard cap for [`Self::regrow_inplace`]. Read from
    /// `gpu.device.limits().max_texture_dimension_2d` at construction
    /// so weaker adapters (mobile/WebGL) don't try to allocate
    /// textures their hardware refuses. Capped at 8192 even when the
    /// adapter exposes 16384 — packer + bind-group cost grows
    /// quadratically with side length.
    max_size_px: u32,
}

impl TextureAtlas {
    /// Build an empty atlas of `size_px × size_px` RGBA8 (sRGB).
    /// Texture is allocated up-front; bytes start as garbage and
    /// only the regions touched by [`Self::insert`] /
    /// [`Self::update_region`] are deterministic.
    pub fn new(gpu: &GpuContext, size_px: u32) -> Self {
        let adapter_cap = gpu.device.limits().max_texture_dimension_2d;
        let max_size_px = adapter_cap.min(8192);
        let size_px = size_px.max(1).min(max_size_px);
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
            free_slots: BTreeMap::new(),
            max_size_px,
        }
    }

    /// Maximum side length [`Self::regrow_inplace`] will allow.
    /// Caller-visible so the shell can pre-emptively refuse imports
    /// larger than this without waiting for the atlas to fail.
    pub fn max_size_px(&self) -> u32 {
        self.max_size_px
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
    /// pixels are rewritten. If the replace dimensions DIFFER from
    /// the original, M14.4f releases the old slot into `free_slots`
    /// and falls through to a fresh pack attempt — the slot will be
    /// reclaimed by the next matching-size insert.
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
        // the existing region. M14.4f: when sizes DIFFER we now
        // release the old slot into `free_slots` (rather than
        // leaking it) and fall through to the fresh-insert flow
        // below, which may consume that same slot if dimensions
        // match a later sprite.
        if let Some(existing) = self.regions.remove(&key) {
            if existing.w == width && existing.h == height {
                upload_region(gpu, &self.texture, existing, rgba);
                self.regions.insert(key, existing);
                return Ok(existing);
            }
            self.free_slots
                .entry((existing.w, existing.h))
                .or_default()
                .push(existing);
        }

        // Fast path: a previously-freed slot of identical dimensions
        // can host this sprite without consulting the Skyline packer.
        // This is what makes `despawn → re-import` cycles survive
        // long sessions without saturating the packer.
        if let Some(bucket) = self.free_slots.get_mut(&(width, height))
            && let Some(region) = bucket.pop()
        {
            if bucket.is_empty() {
                self.free_slots.remove(&(width, height));
            }
            upload_region(gpu, &self.texture, region, rgba);
            self.regions.insert(key, region);
            return Ok(region);
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

    /// Release `key`'s region back into the free-list. The next
    /// [`Self::insert`] of matching dimensions will reuse the slot
    /// (skipping the Skyline packer). Returns the freed region for
    /// the caller's inspection, or `None` if `key` wasn't inserted.
    ///
    /// **Refcount contract**: each `key` is currently owned by
    /// exactly one sprite (the M14.4d `next_import_cell` allocator
    /// guarantees uniqueness). Calling `remove` while another sprite
    /// still references the key is undefined behavior at the user
    /// level (the still-referenced sprite will sample garbage / a
    /// future unrelated sprite's pixels). Refcount tracking lands
    /// when M14.5 introduces prefab/clone semantics that can share
    /// keys.
    pub fn remove(&mut self, key: u32) -> Option<AtlasRegion> {
        let region = self.regions.remove(&key)?;
        self.free_slots
            .entry((region.w, region.h))
            .or_default()
            .push(region);
        Some(region)
    }

    /// Re-pack into a fresh texture of `new_size_px × new_size_px`,
    /// preserving every currently-reserved region. `new_size_px` is
    /// clamped to `[self.size_px, self.max_size_px]`.
    ///
    /// `fetch_pixels(key) -> Option<Vec<u8>>` is called for each
    /// surviving region to re-upload its pixel data. The closure is
    /// the caller's responsibility because the atlas itself does not
    /// cache the source bytes (they live in [`ph2d_asset::AssetDb`]
    /// or — for demo HSV tiles — are regeneratable from the key
    /// index). Returning `None` drops the region from the new atlas.
    ///
    /// Free-list is cleared (old freed slots can't be remapped onto
    /// the new packer's skyline; they'd produce phantom overlaps).
    /// New regions inherit the keys they had pre-regrow so callers
    /// (sprites, render instances) don't need to update.
    ///
    /// Returns the new atlas-internal `size_px`. Errors only when a
    /// surviving region exceeds the new size (which can happen if
    /// the caller passes a smaller size).
    pub fn regrow_inplace<F>(
        &mut self,
        gpu: &GpuContext,
        new_size_px: u32,
        fetch_pixels: F,
    ) -> Result<u32, AtlasInsertError>
    where
        F: Fn(u32) -> Option<Vec<u8>>,
    {
        let new_size_px = new_size_px.clamp(self.size_px, self.max_size_px);
        let old_regions = std::mem::take(&mut self.regions);
        self.free_slots.clear();
        let (texture, view) = create_texture(&gpu.device, new_size_px);
        self.texture = texture;
        self.view = view;
        self.size_px = new_size_px;
        self.packer = rect_packer::DensePacker::new(new_size_px as i32, new_size_px as i32);
        // Re-insert in BTreeMap key order so re-pack is deterministic
        // even when the underlying packer is sensitive to insert
        // sequence (HR-5: identical input → identical layout across
        // runs of the same session, and across machines via blake3
        // hash invariants).
        for (key, region) in old_regions {
            let Some(rgba) = fetch_pixels(key) else {
                continue;
            };
            self.insert(gpu, key, region.w, region.h, &rgba)?;
        }
        Ok(new_size_px)
    }

    /// Free-list size (for tests + diagnostics). Number of regions
    /// currently available for matching-size reuse.
    pub fn free_slot_count(&self) -> usize {
        self.free_slots.values().map(Vec::len).sum()
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
        // Half-texel inset: each boundary shifts 0.5 px inward
        // → (100.5, 200.5, 149.5, 239.5) / 1000.
        assert!((uv[0] - 100.5 / 1000.0).abs() < 1e-6, "u0 = {}", uv[0]);
        assert!((uv[1] - 200.5 / 1000.0).abs() < 1e-6, "v0 = {}", uv[1]);
        assert!((uv[2] - 149.5 / 1000.0).abs() < 1e-6, "u1 = {}", uv[2]);
        assert!((uv[3] - 239.5 / 1000.0).abs() < 1e-6, "v1 = {}", uv[3]);
    }

    #[test]
    fn region_uv_half_texel_inset_prevents_edge_bleed() {
        // A 1-pixel region: both u0 and u1 collapse to the same
        // half-texel center (no width left after the inset). This
        // is OK — a 1-px sprite has no edge to bleed.
        let region = AtlasRegion {
            x: 10,
            y: 10,
            w: 1,
            h: 1,
        };
        let uv = region.uv(100);
        assert!((uv[0] - 0.105).abs() < 1e-6);
        assert!((uv[2] - 0.105).abs() < 1e-6); // collapsed
        // A 4-pixel region: inset shaves 1 texel total across both
        // boundaries, so the effective sampled rect is 3 texels wide.
        let region = AtlasRegion {
            x: 0,
            y: 0,
            w: 4,
            h: 4,
        };
        let uv = region.uv(8);
        // (0+0.5)/8 .. (4-0.5)/8 = 0.0625 .. 0.4375
        assert!((uv[0] - 0.0625).abs() < 1e-6);
        assert!((uv[2] - 0.4375).abs() < 1e-6);
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
        // Half-texel inset on a zero region: u0 = 0.5/1 = 0.5,
        // u1 = -0.5/1 = -0.5. Defensive: zero atlas_size clamps
        // to 1 in `uv()` so we don't div by zero, but the result
        // is degenerate (the renderer ignores zero-area UVs).
        assert!(uv[0] >= 0.0 && uv[0] <= 1.0);
        assert!(uv[1] >= 0.0 && uv[1] <= 1.0);
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

    // ── M14.4f Atlas V2: free-list + remove + regrow ────────────────

    #[test]
    fn remove_returns_region_and_pushes_into_free_list() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        let rgba = vec![0u8; (64 * 64 * 4) as usize];
        let region = atlas.insert(&gpu, 7, 64, 64, &rgba).unwrap();
        assert_eq!(atlas.free_slot_count(), 0);
        let removed = atlas.remove(7).expect("region present");
        assert_eq!(removed, region);
        assert_eq!(atlas.region_count(), 0);
        assert_eq!(atlas.free_slot_count(), 1);
    }

    #[test]
    fn remove_of_missing_key_returns_none() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        assert!(atlas.remove(123).is_none());
        assert_eq!(atlas.free_slot_count(), 0);
    }

    #[test]
    fn insert_reuses_freed_slot_for_matching_size() {
        // After remove, an insert of the SAME dimensions must reuse
        // the freed slot — no new packer.pack() call. We verify by
        // confirming the new region equals the old region.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        let rgba = vec![0u8; (64 * 64 * 4) as usize];
        let region_a = atlas.insert(&gpu, 1, 64, 64, &rgba).unwrap();
        atlas.remove(1).unwrap();
        let region_b = atlas.insert(&gpu, 2, 64, 64, &rgba).unwrap();
        assert_eq!(region_a, region_b, "free-list slot reused");
        assert_eq!(atlas.free_slot_count(), 0);
    }

    #[test]
    fn insert_skips_free_list_when_size_mismatches() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        let rgba_64 = vec![0u8; (64 * 64 * 4) as usize];
        let region_64 = atlas.insert(&gpu, 1, 64, 64, &rgba_64).unwrap();
        atlas.remove(1).unwrap();
        // Insert a 32×32 — the free-list has a 64×64 entry, which is
        // ignored (no fit, no over-allocation: the slot stays in the
        // free-list waiting for another 64×64 caller).
        let rgba_32 = vec![0u8; (32 * 32 * 4) as usize];
        let region_32 = atlas.insert(&gpu, 2, 32, 32, &rgba_32).unwrap();
        assert_ne!(region_32, region_64);
        assert_eq!(atlas.free_slot_count(), 1, "64×64 slot still free");
    }

    #[test]
    fn replace_with_different_size_releases_old_slot() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        let rgba_a = vec![0u8; (64 * 64 * 4) as usize];
        atlas.insert(&gpu, 7, 64, 64, &rgba_a).unwrap();
        // Same key, DIFFERENT size — the old 64×64 region must end
        // up in the free-list (M14.4f bug fix: previously this
        // returned AtlasFull and leaked the slot).
        let rgba_b = vec![0u8; (32 * 32 * 4) as usize];
        atlas.insert(&gpu, 7, 32, 32, &rgba_b).unwrap();
        assert_eq!(atlas.free_slot_count(), 1);
    }

    #[test]
    fn regrow_inplace_doubles_size_and_preserves_regions() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        let rgba = vec![0u8; (64 * 64 * 4) as usize];
        atlas.insert(&gpu, 0, 64, 64, &rgba).unwrap();
        atlas.insert(&gpu, 1, 64, 64, &rgba).unwrap();
        // Fake fetch_pixels — just return zero buffers of the right
        // size. Real callers route through AssetDb.
        let new = atlas
            .regrow_inplace(&gpu, 512, |_key| Some(vec![0u8; 64 * 64 * 4]))
            .unwrap();
        assert_eq!(new, 512);
        assert_eq!(atlas.size_px, 512);
        assert_eq!(atlas.region_count(), 2);
        assert!(atlas.region(0).is_some());
        assert!(atlas.region(1).is_some());
        // Free-list cleared on regrow — old freed slots can't be
        // mapped onto the new skyline without overlap risk.
        assert_eq!(atlas.free_slot_count(), 0);
    }

    #[test]
    fn regrow_inplace_caps_at_max_size_px() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        let cap = atlas.max_size_px();
        let actual = atlas
            .regrow_inplace(&gpu, cap * 4, |_key| Some(Vec::new()))
            .unwrap();
        assert_eq!(actual, cap, "regrow capped at max_size_px");
    }

    #[test]
    fn regrow_inplace_drops_regions_whose_fetcher_returns_none() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        let rgba = vec![0u8; (32 * 32 * 4) as usize];
        atlas.insert(&gpu, 1, 32, 32, &rgba).unwrap();
        atlas.insert(&gpu, 2, 32, 32, &rgba).unwrap();
        // fetcher returns pixels only for key 1; key 2 should drop.
        atlas
            .regrow_inplace(&gpu, 512, |key| {
                if key == 1 {
                    Some(vec![0u8; 32 * 32 * 4])
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(atlas.region_count(), 1);
        assert!(atlas.region(1).is_some());
        assert!(atlas.region(2).is_none());
    }

    #[test]
    fn region_uv_recomputes_against_new_size_after_regrow() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut atlas = TextureAtlas::new(&gpu, 256);
        let rgba = vec![0u8; (128 * 128 * 4) as usize];
        atlas.insert(&gpu, 0, 128, 128, &rgba).unwrap();
        let uv_before = atlas.region_uv(0);
        atlas
            .regrow_inplace(&gpu, 512, |_key| Some(vec![0u8; 128 * 128 * 4]))
            .unwrap();
        let uv_after = atlas.region_uv(0);
        // Same region placement (Skyline picks (0,0) deterministically
        // for the first insert) so x/y stay zero; but the UV math
        // divides by the new `size_px = 512`, halving the
        // normalized width compared to the old `size_px = 256`.
        assert!(uv_before[2] > uv_after[2], "UV must shrink post-regrow");
        // Half-texel inset → (128 - 0.5) / 512 ≈ 0.2490.
        let expected = (128.0_f32 - 0.5) / 512.0;
        assert!(
            (uv_after[2] - expected).abs() < 1e-4,
            "uv1 = {}",
            uv_after[2]
        );
    }
}
