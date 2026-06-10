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

/// Errors returned by [`IndividualTextureStore::acquire`] and
/// [`IndividualTextureStore::readback`].
#[derive(Debug)]
pub enum IndividualTextureError {
    PixelLengthMismatch {
        got: usize,
        expected: usize,
    },
    /// `readback`'s requested texture id has no entry in the store.
    NotFound(u32),
    /// A [`IndividualTextureStore::replace_pixels_region`] sub-rect lies
    /// outside the entry's current texture dimensions (a partial write
    /// past the edge would corrupt neighbouring rows or panic in wgpu).
    RegionOutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        tex_width: u32,
        tex_height: u32,
    },
    /// The GPU command queue accepted the copy but the buffer never
    /// finished mapping. Worth surfacing distinctly from a generic
    /// I/O error so the caller can decide whether to retry (device
    /// likely lost — see ADR-0020) or fail loudly.
    ReadbackFailed(String),
    /// A [`IndividualTextureStore::copy_from_texture`] source texture's
    /// dimensions did not match the destination entry's. A `copy_texture_to_
    /// texture` past the edge would be a wgpu validation error, so reject it
    /// at the boundary with a precise diagnostic.
    CopySizeMismatch {
        width: u32,
        height: u32,
        tex_width: u32,
        tex_height: u32,
    },
}

impl std::fmt::Display for IndividualTextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PixelLengthMismatch { got, expected } => write!(
                f,
                "rgba buffer length {got} doesn't match width*height*4 = {expected}"
            ),
            Self::NotFound(id) => write!(f, "no individual texture with id {id}"),
            Self::RegionOutOfBounds {
                x,
                y,
                width,
                height,
                tex_width,
                tex_height,
            } => write!(
                f,
                "region {width}×{height} at ({x},{y}) exceeds texture {tex_width}×{tex_height}"
            ),
            Self::ReadbackFailed(detail) => write!(f, "GPU readback failed: {detail}"),
            Self::CopySizeMismatch {
                width,
                height,
                tex_width,
                tex_height,
            } => write!(
                f,
                "copy source {width}×{height} doesn't match texture {tex_width}×{tex_height}"
            ),
        }
    }
}

impl std::error::Error for IndividualTextureError {}

impl IndividualTextureStore {
    pub fn new(gpu: &GpuContext) -> Self {
        Self::with_filter(gpu, crate::ImageFilterMode::default())
    }

    /// Build the store with an explicit [`ImageFilterMode`]. The
    /// sampler is the SINGLE canonical sprite sampler
    /// ([`crate::create_sprite_sampler`]) shared with the atlas — so a
    /// sprite baked into an Individual texture (e.g. BG-Removal Apply)
    /// samples identically to an atlas sprite. This fixes the
    /// smooth-preview/pixelated-bake divergence: before, this sampler
    /// hardcoded `Nearest` while the atlas hardcoded `Linear`.
    pub fn with_filter(gpu: &GpuContext, filter: crate::ImageFilterMode) -> Self {
        let sampler = crate::create_sprite_sampler(
            &gpu.device,
            filter,
            "ph2d-render individual texture sampler",
        );
        Self {
            entries: BTreeMap::new(),
            // 1 because 0 is reserved for "shared atlas".
            next_id: 1,
            sampler,
        }
    }

    /// Switch the sampling mode for every individually-owned texture.
    /// Recreates the store sampler AND rebuilds each entry's bind group
    /// (the bind group bakes the old sampler in, so it must be
    /// re-created against the new one). The textures and `texture_id`s
    /// are untouched — only how they're sampled — so SimWorld sprite
    /// references stay valid and no pixel data is re-uploaded.
    pub fn set_filter_mode(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        filter: crate::ImageFilterMode,
    ) {
        self.sampler = crate::create_sprite_sampler(
            &gpu.device,
            filter,
            "ph2d-render individual texture sampler",
        );
        for entry in self.entries.values_mut() {
            entry.bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render individual bg (refiltered)"),
                layout: material_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&entry.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
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

    /// Allocate an EMPTY individually-owned texture (`width × height`,
    /// `Rgba8UnormSrgb`, `COPY_DST`) and return its `texture_id`. Refcount
    /// starts at 1. Unlike [`Self::acquire`], no pixels are uploaded — the
    /// texture contents are undefined until the caller fills the slot (e.g.
    /// [`Self::copy_from_texture`] from a GPU compositor output the same
    /// frame, before it is ever sampled). The Painter GPU live preview uses
    /// this to avoid a wasted full-canvas zero upload on every resize.
    pub fn acquire_empty(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> u32 {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("IndividualTextureStore: u32 id space exhausted");
        let entry = create_entry_empty(gpu, material_bgl, &self.sampler, width, height);
        self.entries.insert(id, entry);
        id
    }

    /// Copy a `width × height` source texture into an existing entry's texture
    /// via a GPU texture-to-texture copy — no CPU readback. Used by the Painter
    /// GPU live preview: the layer compositor's straight-sRGB8 `rgba8unorm`
    /// output (after the premultiply blit) is copied byte-for-byte into the
    /// `Rgba8UnormSrgb` preview slot. The two formats are COPY-COMPATIBLE
    /// (`remove_srgb_suffix()` agrees — same texel block; sRGB-ness differs
    /// only at sample time, which is exactly the premultiplied → linear decode
    /// the sprite shader expects). `src` must carry `COPY_SRC`; the entry's
    /// texture already carries `COPY_DST`.
    ///
    /// Errors on an unknown id or a size mismatch. A zero-area copy is a no-op.
    pub fn copy_from_texture(
        &self,
        gpu: &GpuContext,
        id: u32,
        src: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<(), IndividualTextureError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(IndividualTextureError::NotFound(id))?;
        if entry.width != width || entry.height != height {
            return Err(IndividualTextureError::CopySizeMismatch {
                width,
                height,
                tex_width: entry.width,
                tex_height: entry.height,
            });
        }
        if width == 0 || height == 0 {
            return Ok(());
        }
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render individual copy_from_texture encoder"),
            });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &entry.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit([encoder.finish()]);
        Ok(())
    }

    /// GPU→GPU copy of a SUB-RECT of `src` into the same sub-rect of slot `id`,
    /// leaving the rest of the slot untouched. `src_origin` is where the rect
    /// starts in `src`; `dst_x`/`dst_y` where it lands in the slot; `w`/`h` its
    /// size. The dirty-rect sibling of [`Self::copy_from_texture`] — the Painter
    /// E5 live stroke refreshes only the wet envelope of the preview slot instead
    /// of re-copying the whole canvas every frame. The rect must lie within both
    /// textures (caller clamps); an empty rect is a no-op.
    #[allow(clippy::too_many_arguments)]
    pub fn copy_region_from_texture(
        &self,
        gpu: &GpuContext,
        id: u32,
        src: &wgpu::Texture,
        src_x: u32,
        src_y: u32,
        dst_x: u32,
        dst_y: u32,
        w: u32,
        h: u32,
    ) -> Result<(), IndividualTextureError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(IndividualTextureError::NotFound(id))?;
        if dst_x + w > entry.width || dst_y + h > entry.height {
            return Err(IndividualTextureError::CopySizeMismatch {
                width: dst_x + w,
                height: dst_y + h,
                tex_width: entry.width,
                tex_height: entry.height,
            });
        }
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render individual copy_region_from_texture encoder"),
            });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: src_x,
                    y: src_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &entry.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: dst_x,
                    y: dst_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit([encoder.finish()]);
        Ok(())
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

    /// Pixel dimensions `(width, height)` of an entry, or `None` for an
    /// unknown id. Used by the extract to convert a sprite's pixel-space
    /// `region_rect` into a UV sub-rect of the (full-unit) texture.
    pub fn dims(&self, id: u32) -> Option<(u32, u32)> {
        self.entries.get(&id).map(|e| (e.width, e.height))
    }

    /// Copy the GPU pixel contents of an entry back into a fresh
    /// `Vec<u8>` (RGBA8, tightly packed `width * height * 4`).
    ///
    /// Submits a one-shot copy-texture-to-buffer + map, blocks on
    /// `device.poll(Wait)`, then strips row padding (`copy_texture_to_buffer`
    /// requires rows aligned to
    /// [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`]). Allocates one staging
    /// `wgpu::Buffer` and one output `Vec<u8>` — fine for one-shot
    /// editor actions (Trim / BG Removal); **not** acceptable in any
    /// per-frame path (HR-3).
    ///
    /// HR-1: stays in `ph2d-render`, never crosses to `ph2d-core`.
    /// HR-13: peak transient memory is `~ width * (padded_row + 4)`
    /// — for a 2k² sprite, ≈ 16 MB staging + 16 MB output.
    pub fn readback(
        &self,
        gpu: &GpuContext,
        id: u32,
    ) -> Result<(u32, u32, Vec<u8>), IndividualTextureError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(IndividualTextureError::NotFound(id))?;
        readback_texture(gpu, &entry.texture, entry.width, entry.height)
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

    /// Upload a sub-rectangle of pixels into an existing entry's texture,
    /// leaving the rest untouched (`queue.write_texture` with a non-zero
    /// origin + partial extent).
    ///
    /// The Painter dirty-rect composite path uploads only the bounding
    /// box a stroke touched instead of the whole canvas — O(bbox) per
    /// frame instead of O(W×H) (Painter W3 audit item 1a — the GPU end of
    /// the dirty-rect path). The texture id, bind group and dims stay
    /// stable, so SimWorld references and the cached bind group remain
    /// valid; no reallocation occurs.
    ///
    /// `region_rgba` must be tightly packed `width * height * 4` bytes for
    /// the sub-rect ALONE (not the full texture). The region must lie
    /// fully within the entry's current dims. A zero-area region is a
    /// no-op (an empty dirty-rect); an unknown id is a silent no-op
    /// (mirror of [`Self::replace_pixels`]).
    // x/y/w/h is the idiomatic graphics sub-rect form (mirrors
    // `queue.write_texture`'s origin + extent); packing into a struct would
    // break the bridge consumer for no clarity gain.
    #[allow(clippy::too_many_arguments)]
    pub fn replace_pixels_region(
        &mut self,
        gpu: &GpuContext,
        id: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        region_rgba: &[u8],
    ) -> Result<(), IndividualTextureError> {
        let Some(entry) = self.entries.get(&id) else {
            return Ok(());
        };
        // The sub-rect must lie fully within the current texture; a write
        // past the edge would corrupt neighbouring rows or panic in wgpu.
        let in_bounds = x.checked_add(width).is_some_and(|r| r <= entry.width)
            && y.checked_add(height).is_some_and(|b| b <= entry.height);
        if !in_bounds {
            return Err(IndividualTextureError::RegionOutOfBounds {
                x,
                y,
                width,
                height,
                tex_width: entry.width,
                tex_height: entry.height,
            });
        }
        if width == 0 || height == 0 {
            return Ok(()); // empty dirty-rect — nothing to upload
        }
        let expected = (width as usize) * (height as usize) * 4;
        if region_rgba.len() != expected {
            return Err(IndividualTextureError::PixelLengthMismatch {
                got: region_rgba.len(),
                expected,
            });
        }
        write_pixels_region(gpu, &entry.texture, x, y, width, height, region_rgba);
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
    let entry = create_entry_empty(gpu, material_bgl, sampler, width, height);
    write_pixels(gpu, &entry.texture, width, height, rgba);
    entry
}

/// Build an [`IndividualTextureEntry`] with an UNINITIALISED texture (no pixel
/// upload). Shared by [`create_entry`] (which then writes pixels) and
/// [`IndividualTextureStore::acquire_empty`] (which leaves the fill to a later
/// GPU copy). The texture/view/bind-group are otherwise identical, so a slot
/// created either way is sampled the same way.
fn create_entry_empty(
    gpu: &GpuContext,
    material_bgl: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
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
        // COPY_SRC required for `readback()` to copy this texture's
        // contents back into a staging buffer (F2 — Image Tools edit
        // path on Individual-source sprites). COPY_DST also feeds
        // `copy_from_texture` (the Painter GPU preview blit).
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
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

/// Upload a tightly-packed `width × height` RGBA8 sub-rect to `(x, y)` of
/// `texture`, leaving the rest of the texture untouched. Caller guarantees
/// the rect is in-bounds and `region_rgba.len() == width * height * 4`.
#[allow(clippy::too_many_arguments)]
fn write_pixels_region(
    gpu: &GpuContext,
    texture: &wgpu::Texture,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    region_rgba: &[u8],
) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        region_rgba,
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

/// Copy a 2D RGBA8 texture out of GPU memory into a tightly-packed
/// `Vec<u8>`. `copy_texture_to_buffer` requires the destination row
/// pitch to be a multiple of [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`]
/// (256 on every backend), so the staging buffer is padded and the
/// output is unpadded row-by-row.
fn readback_texture(
    gpu: &GpuContext,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Result<(u32, u32, Vec<u8>), IndividualTextureError> {
    if width == 0 || height == 0 {
        return Ok((width, height, Vec::new()));
    }
    let bytes_per_pixel: u32 = 4;
    let unpadded_bpr = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let buffer_size = (padded_bpr as u64) * (height as u64);

    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-render individual readback staging"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-render individual readback encoder"),
        });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);

    // Map + block-on-wait. `device.poll(PollType::Wait)` drives the
    // queue forward until the buffer's map operation completes. We
    // own the channel both ends so the closure's `Send` bound is
    // satisfied without a runtime.
    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|e| IndividualTextureError::ReadbackFailed(format!("poll: {e}")))?;
    rx.recv()
        .map_err(|e| IndividualTextureError::ReadbackFailed(format!("channel: {e}")))?
        .map_err(|e| IndividualTextureError::ReadbackFailed(format!("map_async: {e}")))?;

    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded_bpr as usize) * (height as usize));
    for row in 0..height as usize {
        let start = row * padded_bpr as usize;
        let end = start + unpadded_bpr as usize;
        out.extend_from_slice(&mapped[start..end]);
    }
    drop(mapped);
    staging.unmap();
    Ok((width, height, out))
}

// Tests that exercise the GPU paths live alongside `SpriteRenderer`
// in `renderer.rs`'s integration suite (they require a `GpuContext`).
// Pure-Rust tests for refcounting are covered indirectly there as
// well — keeping this module test-light avoids the
// `unsafe { mem::zeroed() }` trick on wgpu handles that wgpu's Drop
// impl would crash on.
