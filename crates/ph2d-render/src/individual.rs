//! M14.5 C — individual-texture sprite source.
//!
//! Companion to [`crate::atlas::TextureAtlas`]: while the atlas packs
//! many sprites into one shared 4096² texture and renders them in a
//! single draw call, this store gives each sprite its **own**
//! `wgpu::Texture` at the source's native resolution. The renderer
//! groups consecutive same-texture instances into one draw call each
//! (Godot 4 `RenderingServer` pattern) — a pure-CPU sort step that
//! amortizes well when sprite count stays under a few thousand.
//!
//! ## When to pick this over the atlas
//!
//! - **HD 2D content** (Cuphead-tier) where each sprite is large enough
//!   that packing-and-batching wins nothing over per-sprite textures.
//! - **Procedural / hot-reloaded textures** that change dimensions
//!   between reloads — atlas regrow is more expensive than swapping a
//!   single texture handle.
//! - **Mixed-resolution sprites** where the atlas's Skyline packer
//!   would either waste space or evict on every regrow.
//!
//! For tile-sets, UI icons, and same-shape sprite sheets, prefer the
//! shared atlas — it's still one draw call per frame.
//!
//! ## Lifecycle contract
//!
//! Callers (typically the image-import path in `shells/desktop`) must
//! pair every [`IndividualTextureStore::acquire`] with a
//! [`IndividualTextureStore::release`] when the owning sprite
//! despawns. Refcounting catches the common case where the same
//! `AssetId` is referenced by multiple sprites — the texture is held
//! until the last sprite releases it.
//!
//! HR-5 / ADR-0022: uses `BTreeMap`, not `HashMap`, so the iteration
//! order over textures stays deterministic for tests that count
//! distinct runs in [`crate::renderer::SpriteRenderer`].

use ph2d_gpu::GpuContext;
use std::collections::BTreeMap;

/// One individually-owned texture, with a pre-built bind group sized
/// against the renderer's `material_bgl` so the per-frame batcher can
/// `set_bind_group(1, ...)` without re-creating it.
pub struct IndividualTextureEntry {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
    /// Sprites currently referencing this texture. Drops to 0 →
    /// [`IndividualTextureStore::release`] removes the entry and the
    /// `wgpu::Texture` handle drops.
    pub refcount: u32,
}

/// Renderer-side cache of individual sprite textures, keyed by a
/// monotonically-allocated `u32` id (the same value stored in
/// `Sprite::source = SpriteSource::Individual { texture_id }`).
///
/// Id `0` is reserved as the "atlas" sentinel — see
/// [`crate::RenderInstance::ATLAS_TEXTURE_ID`]. The store starts
/// allocation at `1`.
pub struct IndividualTextureStore {
    entries: BTreeMap<u32, IndividualTextureEntry>,
    next_id: u32,
    sampler: wgpu::Sampler,
}

/// Errors returned by [`IndividualTextureStore::acquire`].
#[derive(Debug)]
pub enum IndividualTextureError {
    PixelLengthMismatch { got: usize, expected: usize },
}

impl std::fmt::Display for IndividualTextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PixelLengthMismatch { got, expected } => write!(
                f,
                "rgba buffer length {got} doesn't match width*height*4 = {expected}"
            ),
        }
    }
}

impl std::error::Error for IndividualTextureError {}

impl IndividualTextureStore {
    pub fn new(gpu: &GpuContext) -> Self {
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render individual texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        Self {
            entries: BTreeMap::new(),
            // 1 because 0 is reserved for "shared atlas".
            next_id: 1,
            sampler,
        }
    }

    /// Total number of individually-owned textures currently held.
    /// Used by tests and the future Inspector telemetry. Excludes the
    /// shared atlas.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Upload `rgba` (tightly-packed width*height*4 bytes) to a new
    /// individually-owned texture and return its renderer-side
    /// `texture_id`. Refcount starts at 1.
    ///
    /// Errors only on pixel-length mismatch — the GPU side is
    /// fire-and-forget (`queue.write_texture`). Validation issues
    /// (size too large for the device limit) surface at first render.
    pub fn acquire(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<u32, IndividualTextureError> {
        let expected = (width as usize) * (height as usize) * 4;
        if rgba.len() != expected {
            return Err(IndividualTextureError::PixelLengthMismatch {
                got: rgba.len(),
                expected,
            });
        }
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("IndividualTextureStore: u32 id space exhausted");
        let entry = create_entry(gpu, material_bgl, &self.sampler, width, height, rgba);
        self.entries.insert(id, entry);
        Ok(id)
    }

    /// Increment the refcount for an existing entry. The renderer
    /// uses this when a sprite is duplicated via the M14.6 F context
    /// menu so the source texture survives even if the original
    /// sprite is later deleted.
    pub fn retain(&mut self, id: u32) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.refcount = entry.refcount.saturating_add(1);
        }
    }

    /// Decrement the refcount; drop the entry once it reaches 0.
    /// Returns the post-decrement count, or `None` if the id was
    /// already absent (idempotent for safety).
    pub fn release(&mut self, id: u32) -> Option<u32> {
        let stop = if let Some(entry) = self.entries.get_mut(&id) {
            entry.refcount = entry.refcount.saturating_sub(1);
            entry.refcount
        } else {
            return None;
        };
        if stop == 0 {
            self.entries.remove(&id);
        }
        Some(stop)
    }

    /// Read access to the pre-built bind group for a texture id.
    /// Returns `None` for ids that were never acquired or have been
    /// fully released — the renderer falls back to "skip this batch"
    /// in either case.
    pub fn bind_group(&self, id: u32) -> Option<&wgpu::BindGroup> {
        self.entries.get(&id).map(|e| &e.bind_group)
    }

    /// Replace the pixel contents of an existing entry in place.
    /// Used by the M6 hot-reload bridge when an `AssetId` underlying
    /// an individual sprite changes on disk.
    ///
    /// When `width × height` matches the cached dims, the existing
    /// `wgpu::Texture` is reused (queue.write_texture only); the bind
    /// group survives and the `texture_id` stays stable for SimWorld
    /// references.
    ///
    /// When dims change, the texture/view/bind_group are recreated
    /// against the same id. Sprites referencing the id remain valid;
    /// the next render frame samples the new texture.
    pub fn replace_pixels(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        id: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<(), IndividualTextureError> {
        let expected = (width as usize) * (height as usize) * 4;
        if rgba.len() != expected {
            return Err(IndividualTextureError::PixelLengthMismatch {
                got: rgba.len(),
                expected,
            });
        }
        let Some(entry) = self.entries.get_mut(&id) else {
            return Ok(());
        };
        if entry.width == width && entry.height == height {
            write_pixels(gpu, &entry.texture, width, height, rgba);
        } else {
            let refcount = entry.refcount;
            let new_entry = create_entry(gpu, material_bgl, &self.sampler, width, height, rgba);
            let mut new_entry = new_entry;
            new_entry.refcount = refcount;
            *entry = new_entry;
        }
        Ok(())
    }
}

fn create_entry(
    gpu: &GpuContext,
    material_bgl: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> IndividualTextureEntry {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render individual texture"),
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
    write_pixels(gpu, &texture, width, height, rgba);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ph2d-render individual bg"),
        layout: material_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    IndividualTextureEntry {
        texture,
        view,
        bind_group,
        width,
        height,
        refcount: 1,
    }
}

fn write_pixels(gpu: &GpuContext, texture: &wgpu::Texture, width: u32, height: u32, rgba: &[u8]) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
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
}

// Tests that exercise the GPU paths live alongside `SpriteRenderer`
// in `renderer.rs`'s integration suite (they require a `GpuContext`).
// Pure-Rust tests for refcounting are covered indirectly there as
// well — keeping this module test-light avoids the
// `unsafe { mem::zeroed() }` trick on wgpu handles that wgpu's Drop
// impl would crash on.
