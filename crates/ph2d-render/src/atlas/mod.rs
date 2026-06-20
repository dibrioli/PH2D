//! Texture atlas — single GPU texture holding many sprites at native
//! resolution, with a Skyline rect packer choosing each sprite's
//! (x, y) placement at insert time.
//!
//! ## Module layout
//!
//! This module is split by responsibility:
//! - [`region`] — pure data types ([`AtlasRegion`] + UV math,
//!   [`AtlasInsertError`]).
//! - [`gpu_ops`] — raw `wgpu` texture ops (upload / create / clear /
//!   readback).
//! - [`demo`] — golden-angle HSV demo-tile synthesis.
//! - this file — the [`TextureAtlas`] orchestrator (packing, free-list,
//!   regrow, mip regen) plus the atlas constants and the public
//!   re-exports.
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
//! v1 returns [`AtlasInsertError::AtlasFull`] on overflow and surfaces
//! it as a toast at the import site. Atlas regrow (re-pack everything
//! into a 2×-larger texture) is a follow-up — see the plan §Backlog.
//!
//! ## Mipmaps (2026-06-18 minification fix, Phase 2)
//!
//! The atlas texture carries a full mip chain ([`crate::mipgen`]) so a
//! minified (zoomed-out) sprite samples trilinearly instead of
//! undersampling its antialiased edges into jaggies — the same fix the
//! individual-texture store got in Phase 1, now extended to the SHARED
//! atlas where committed paint strokes and imported images live.
//!
//! Because the atlas packs many regions into ONE texture, a naïve
//! whole-texture downsample would bleed neighbouring regions (and the
//! never-written garbage between them) into each region's edge mips.
//! Two guards bound that bleed to sub-pixel: (1) [`AtlasRegion::uv`]'s
//! half-texel inset keeps level-0 sampling strictly inside each region,
//! and (2) the texture is CLEARED to transparent at creation so edge
//! bleed at higher mips fades toward transparent (correct for a
//! premultiplied sprite edge) instead of pulling in stale garbage. Full
//! gutter padding (needed only for tightly-packed tile-atlases zoomed
//! out past mip ~4) is a documented follow-up; the painter case (large
//! canvas + a few imports with soft/transparent borders) is the benign
//! one.

mod demo;
mod gpu_ops;
mod region;

pub use region::{AtlasInsertError, AtlasRegion};

use demo::make_hsv_tile;
use gpu_ops::{clear_level0_transparent, create_texture, readback_atlas_texture, upload_region};
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
    /// Regenerates the atlas mip chain after each content write so a
    /// minified (zoomed-out) sprite samples trilinearly instead of
    /// aliasing its antialiased edges (2026-06-18 Phase 2). The atlas is
    /// `Rgba8UnormSrgb`, so one generator serves the whole texture; a
    /// full regen costs ~one fullscreen blit per level (≈0.5 ms at
    /// 8192²) and only fires on insert / replace / regrow — never per
    /// frame (the painter live preview uses individual textures).
    mip_gen: crate::mipgen::MipGenerator,
}

impl TextureAtlas {
    /// Build an empty atlas of `size_px × size_px` RGBA8 (sRGB).
    /// Texture is allocated up-front; bytes start as garbage and
    /// only the regions touched by [`Self::insert`] /
    /// [`Self::update_region`] are deterministic.
    pub fn new(gpu: &GpuContext, size_px: u32) -> Self {
        Self::with_filter(gpu, size_px, crate::ImageFilterMode::default())
    }

    /// Build an empty atlas using an explicit [`ImageFilterMode`].
    /// [`Self::new`] defers to this with the project default. The
    /// sampler is the SINGLE canonical sprite sampler
    /// ([`crate::create_sprite_sampler`]) — atlas and individual
    /// textures share the exact same descriptor so they never diverge
    /// (the old hardcoded `Linear` vs `Nearest` bug).
    ///
    /// [`ImageFilterMode`]: crate::ImageFilterMode
    pub fn with_filter(gpu: &GpuContext, size_px: u32, filter: crate::ImageFilterMode) -> Self {
        let adapter_cap = gpu.device.limits().max_texture_dimension_2d;
        let max_size_px = adapter_cap.min(8192);
        let size_px = size_px.max(1).min(max_size_px);
        let (texture, view) = create_texture(&gpu.device, size_px);
        // Clear to transparent so untouched packing space (and the gap
        // between regions) downsamples cleanly instead of bleeding stale
        // garbage into region-edge mips (see module doc §Mipmaps).
        clear_level0_transparent(gpu, &texture);
        let sampler =
            crate::create_sprite_sampler(&gpu.device, filter, "ph2d-render atlas sampler");
        let atlas = Self {
            texture,
            view,
            sampler,
            size_px,
            packer: rect_packer::DensePacker::new(size_px as i32, size_px as i32),
            regions: BTreeMap::new(),
            free_slots: BTreeMap::new(),
            max_size_px,
            mip_gen: crate::mipgen::MipGenerator::new(gpu, wgpu::TextureFormat::Rgba8UnormSrgb),
        };
        // Fill the (transparent) mip chain so the texture is sampleable
        // at any level before the first insert.
        atlas.regen_mips(gpu);
        atlas
    }

    /// Regenerate the atlas mip chain (levels `1..`) from level 0 after a
    /// content write. Cheap enough to run on every insert/replace; never
    /// called per frame.
    fn regen_mips(&self, gpu: &GpuContext) {
        self.mip_gen.run(
            gpu,
            &self.texture,
            crate::mipgen::mip_levels(self.size_px, self.size_px),
        );
    }

    /// Maximum side length [`Self::regrow_inplace`] will allow.
    /// Caller-visible so the shell can pre-emptively refuse imports
    /// larger than this without waiting for the atlas to fail.
    pub fn max_size_px(&self) -> u32 {
        self.max_size_px
    }

    /// Recreate the atlas sampler for a new [`ImageFilterMode`]. The
    /// texture and packed regions are untouched — only how they're
    /// SAMPLED changes — but any bind group referencing the old
    /// `sampler` must be rebuilt afterward (the renderer does this via
    /// `rebuild_material_bind_group`). Cheap: one sampler allocation.
    ///
    /// [`ImageFilterMode`]: crate::ImageFilterMode
    pub fn set_filter_mode(&mut self, gpu: &GpuContext, filter: crate::ImageFilterMode) {
        self.sampler =
            crate::create_sprite_sampler(&gpu.device, filter, "ph2d-render atlas sampler");
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
                self.regen_mips(gpu);
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
            self.regen_mips(gpu);
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
        self.regen_mips(gpu);
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
        clear_level0_transparent(gpu, &texture);
        self.texture = texture;
        self.view = view;
        self.size_px = new_size_px;
        self.packer = rect_packer::DensePacker::new(new_size_px as i32, new_size_px as i32);
        // Define the (transparent) mip chain up front; each re-insert
        // below regenerates it, but an empty regrow still leaves a
        // sampleable chain.
        self.regen_mips(gpu);
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

    /// Source pixel dimensions `(w, h)` of `key`'s packed region — i.e.
    /// the original image size (the atlas packs at native resolution, no
    /// scaling). `None` for an unknown key. Used by the extract to convert
    /// a sprite's pixel-space `region_rect` into an atlas-UV sub-rect.
    pub fn region_px(&self, key: u32) -> Option<(u32, u32)> {
        self.regions.get(&key).map(|r| (r.w, r.h))
    }

    /// Number of reserved regions. Used by tests; not perf-critical.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Read a whole **mip level** of the atlas texture back into a tightly
    /// packed `Vec<u8>` (RGBA8). Level 0 is full res; level `n` is
    /// `(size_px >> n).max(1)` square. One-shot staging copy (same cost
    /// model as the individual store's readback) — for tests and
    /// diagnostics only, never a per-frame path. Returns `(w, h, bytes)`.
    pub fn readback_mip(&self, gpu: &GpuContext, level: u32) -> (u32, u32, Vec<u8>) {
        let side = (self.size_px >> level).max(1);
        let bytes = readback_atlas_texture(gpu, &self.texture, level, side, side);
        (side, side, bytes)
    }
}

#[cfg(test)]
mod tests;
