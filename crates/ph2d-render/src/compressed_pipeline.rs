//! Compressed-texture upload path for cooked KTX2 assets (texture-
//! compression Wave 2, W2.T3).
//!
//! ## Design conclusion (PASSO 0): ONE shared pipeline, not pipeline-per-format
//!
//! The plan sketch ("pipeline-per-format selection — BC7/BC6H/ASTC/ETC2/
//! RGBA8 paths; bind group differs per format, ~600 LOC") was a *worst-
//! case* estimate written before the wgpu-28 sample-type table was
//! checked. It does **not** hold:
//!
//! `wgpu::TextureFormat::sample_type()` (wgpu-types 28, `texture/format.rs`)
//! returns `Some(TextureSampleType::Float { filterable: true })` for **every**
//! format this wave uploads — `Rgba8Unorm`/`Srgb`, `Rgba16Float`,
//! `Rgba16Unorm`, **every** BC variant (`Bc1..Bc7`, **including the HDR
//! `Bc6hRgbUfloat`/`Bc6hRgbFloat`**), **every** ETC2 variant, and
//! `Astc { .. }` (all block sizes). They therefore all bind through the
//! *same* bind-group layout — a `Float { filterable: true }` D2 texture plus
//! a `Filtering` sampler — which is byte-for-byte the layout the existing
//! [`crate::pipeline::SpritePipeline`] already declares as its
//! `material_bgl`. A single `@group(1)` material bind group and a single
//! render pipeline sample all of them; only the **`wgpu::TextureFormat` of
//! the created texture** changes (resolved by W2.T1's
//! [`wgpu_format_from_ktx2_format`]).
//!
//! BC6H is HDR (texel values escape `[0, 1]`), but HDR-ness is a *value-
//! range* property handled downstream by tonemapping ([`crate::tonemap`] /
//! the `GameRt` float path), **not** a sample-type or blend-state property:
//! its `sample_type()` is still `Float { filterable: true }`, so it needs no
//! distinct pipeline or bind-group layout at the upload/bind layer. The only
//! mapped format whose sample type differs is `Rgba32Float` (filterability
//! gated on `FLOAT32_FILTERABLE`); full 32-bit float HDR is the `GameRt`
//! float path, not the cooked-sprite path, so [`CompressedTexturePipeline`]
//! **rejects** it at [`CompressedTexturePipeline::resolve_format`]
//! ([`CompressedUploadError::NotFilterableFloat`]) rather than risk a bind-
//! time validation error — keeping the "one shared layout binds everything
//! this pipeline uploads" invariant true by construction, not by assumption.
//!
//! So inventing N redundant per-format pipelines would be pure waste. This
//! module instead exposes ONE [`CompressedTexturePipeline`] (the shared
//! bind-group layout + sampler) and a per-asset
//! [`CompressedTexturePipeline::upload`] that creates the texture in the
//! correct `wgpu::TextureFormat` and writes every mip with **block-aligned**
//! row/layer pitches. The genuinely format-*specific* work — block-aligned
//! `bytes_per_row` / `rows_per_image` for BC/ASTC/ETC2's 4×4..12×12 blocks —
//! lives in [`MipUploadLayout`], the part that *does* differ per format, and
//! is unit-tested without a GPU.
//!
//! ## Feature gating
//!
//! Upload is gated on [`CompressionFeatureSet::supports`]: a format whose
//! family feature (`TEXTURE_COMPRESSION_BC`/`ASTC`/`ETC2` /
//! `TEXTURE_FORMAT_16BIT_NORM`) the device lacks is rejected as
//! [`CompressedUploadError::FormatUnsupportedByDevice`] — the caller (W2.T4
//! loader) is expected to have already picked a tier the device can sample,
//! so reaching here with an unsupported format is a loader bug, surfaced
//! loudly rather than panicking inside wgpu.
//!
//! ## Hookup (deferred to W2.T2 / W2.T4 — NOT wired here)
//!
//! This module deliberately stops at "a ready-to-bind
//! [`UploadedCompressedTexture`] (texture + view + material bind group)".
//! Wiring it into [`crate::sprite::SpriteSource`] (the `CookedTexture`
//! variant — W2.T2, a separate breaking task) and resolving the asset bytes
//! via the loader (W2.T4) are downstream. A consumer builds a
//! [`CompressedTexturePipeline`] once (from the same `material_bgl` the
//! `SpritePipeline` uses, so the bind group is pipeline-compatible), then
//! calls [`CompressedTexturePipeline::upload`] per cooked asset and binds the
//! returned `bind_group` at `@group(1)` exactly like an atlas/individual
//! texture.

use crate::ktx2_format::{CompressionFeatureSet, FormatError, wgpu_format_from_ktx2_format};
use ph2d_asset_ktx2::{Ktx2Format, Ktx2Image, MipLevel};
use ph2d_gpu::GpuContext;

/// Why a cooked KTX2 image could not be uploaded to a GPU texture.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompressedUploadError {
    /// The image's [`ph2d_asset_ktx2::Ktx2Format`] has no renderable wgpu
    /// mapping (out-of-subset VkFormat, or a future unmapped variant). Wraps
    /// the underlying [`FormatError`] from W2.T1.
    UnmappableFormat(FormatError),
    /// The format maps to a valid `wgpu::TextureFormat`, but the device this
    /// pipeline targets does not advertise the family feature required to
    /// *sample* it (e.g. a BC artifact on a WebGPU adapter). The loader
    /// (W2.T4) should have selected a device-supported tier upstream, so this
    /// is a loader contract violation, not a property of the asset. Carries
    /// the resolved format for telemetry.
    FormatUnsupportedByDevice(wgpu::TextureFormat),
    /// The decoded image carried zero mip levels. [`decode_ktx2_bytes`] never
    /// produces this (it rejects empty pyramids), but a hand-built
    /// [`Ktx2Image`] struct literal could — guarded so `upload` never
    /// constructs a zero-mip texture descriptor (which wgpu would reject).
    ///
    /// [`decode_ktx2_bytes`]: ph2d_asset_ktx2::decode_ktx2_bytes
    NoMipLevels,
    /// A mip level's payload is shorter than its block-aligned footprint
    /// requires — a truncated / corrupt artifact. Caught on the CPU before
    /// handing a too-short slice to `write_texture` (which would either
    /// panic or read out of bounds inside the backend). Carries
    /// `(mip_index, expected_bytes, actual_bytes)`.
    MipDataTooShort {
        /// 0-based index into the mip pyramid.
        mip_index: usize,
        /// Bytes the block-aligned layout requires for this mip.
        expected: usize,
        /// Bytes actually present in `MipLevel::data`.
        actual: usize,
    },
    /// The format is renderable and device-supported, but it does **not**
    /// sample as `Float { filterable: true }` on this device, so it cannot
    /// bind through this pipeline's shared filterable-float / `Filtering`-
    /// sampler material layout (a wgpu bind-group validation error would
    /// otherwise fire at bind time, far from the cause).
    ///
    /// The only KTX2-mapped format this can be is `Rgba32Float` on a device
    /// lacking `FLOAT32_FILTERABLE`: full 32-bit float HDR is **not** the
    /// cooked-sprite path — it belongs to the `GameRt` float pipeline
    /// ([`crate::game_rt`]). Every other mapped format (RGBA8/16,
    /// BC1..BC7 incl. HDR BC6H, ASTC, ETC2) is unconditionally filterable-
    /// float and never hits this arm. Carries the resolved format.
    NotFilterableFloat(wgpu::TextureFormat),
}

impl core::fmt::Display for CompressedUploadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnmappableFormat(e) => write!(f, "cannot map KTX2 format to wgpu: {e}"),
            Self::FormatUnsupportedByDevice(fmt) => write!(
                f,
                "device lacks the feature to sample {fmt:?} (loader picked a tier this GPU can't read)"
            ),
            Self::NoMipLevels => f.write_str("KTX2 image has zero mip levels"),
            Self::MipDataTooShort {
                mip_index,
                expected,
                actual,
            } => write!(
                f,
                "mip {mip_index} payload truncated: need {expected} bytes, got {actual}"
            ),
            Self::NotFilterableFloat(fmt) => write!(
                f,
                "{fmt:?} does not sample as filterable-float on this device — not a cooked-sprite \
                 format (32-bit float HDR uses the GameRt pipeline)"
            ),
        }
    }
}

impl core::error::Error for CompressedUploadError {}

impl From<FormatError> for CompressedUploadError {
    fn from(e: FormatError) -> Self {
        Self::UnmappableFormat(e)
    }
}

/// Block-aligned `write_texture` parameters for ONE mip level.
///
/// This is the part that genuinely varies per format: BC/ASTC/ETC2 are
/// uploaded in **whole texel blocks** (4×4 for BC/ETC2; 4×4..12×12 for ASTC),
/// not pixels, so `bytes_per_row`/`rows_per_image` must be expressed in
/// blocks — `bytes_per_row = block_count_x * block_copy_size`, where
/// `block_count_x = ceil(mip_width / block_width)`. The mip's *pixel*
/// dimensions still go into the copy `Extent3d` (wgpu maps pixels→blocks
/// internally and validates the row pitch against them). Uncompressed RGBA*
/// has `block_dimensions == (1, 1)`, so the same math degenerates to the
/// familiar `width * bytes_per_pixel`.
///
/// Computed purely from the `wgpu::TextureFormat` (via its canonical
/// [`wgpu::TextureFormat::block_dimensions`] /
/// [`wgpu::TextureFormat::block_copy_size`] — single source of truth, no
/// hand-maintained block table) and the mip's pixel dimensions, so it is
/// fully unit-testable without a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MipUploadLayout {
    /// Mip width in pixels (goes into the copy `Extent3d`).
    pub width_px: u32,
    /// Mip height in pixels (goes into the copy `Extent3d`).
    pub height_px: u32,
    /// `bytes_per_row` for `write_texture`: full rows of blocks
    /// (`ceil(width / block_w) * block_copy_size`).
    pub bytes_per_row: u32,
    /// `rows_per_image` for `write_texture`: rows of blocks
    /// (`ceil(height / block_h)`).
    pub rows_per_image: u32,
    /// Total block-aligned payload bytes this mip occupies
    /// (`bytes_per_row * rows_per_image`). The mip's `data` slice must be at
    /// least this long.
    pub total_bytes: usize,
}

impl MipUploadLayout {
    /// Compute the block-aligned upload layout for a `width_px × height_px`
    /// mip in `format`.
    ///
    /// Returns `None` only when `format.block_copy_size(None)` is `None` —
    /// which, for the color formats this pipeline handles, never happens
    /// (that case is reserved for depth/stencil/multi-planar formats that
    /// need an explicit aspect, none of which are KTX2-cooked color
    /// textures). Callers treat `None` as "not a uploadable color format".
    #[must_use]
    pub fn for_mip(format: wgpu::TextureFormat, width_px: u32, height_px: u32) -> Option<Self> {
        let (block_w, block_h) = format.block_dimensions();
        let block_bytes = format.block_copy_size(None)?;
        // ceil-div in blocks: a partial trailing block at a non-block-multiple
        // mip edge (common for small mips of a non-power-of-block texture)
        // still occupies one full block of storage.
        let blocks_x = width_px.div_ceil(block_w);
        let blocks_y = height_px.div_ceil(block_h);
        // checked_mul: a pathologically large mip — only reachable via a
        // caller that bypasses the decoder's MAX_DIMENSION cap (e.g. a future
        // loader feeding raw dims) — would otherwise overflow `u32` here
        // (panic in debug, silent wrap in release). On overflow return `None`,
        // which `mip_layouts` surfaces as a clean `CompressedUploadError`,
        // honoring the "corrupt artifact → clean error, no panic" contract.
        let bytes_per_row = blocks_x.checked_mul(block_bytes)?;
        let rows_per_image = blocks_y;
        let total_bytes = (bytes_per_row as usize).checked_mul(rows_per_image as usize)?;
        Some(Self {
            width_px,
            height_px,
            bytes_per_row,
            rows_per_image,
            total_bytes,
        })
    }
}

/// HR-13 budget ceiling (MB) for the cooked-texture cache (W2.T5).
///
/// **Declared, NOT yet enforced.** Nothing compares
/// [`CookedTextureStore::cache_bytes`](crate::cooked_texture::CookedTextureStore::cache_bytes)
/// against this constant today — there is no eviction or upload rejection on
/// overrun. This is the plan's §6 W2.T5 *interim mitigation* ("declarar
/// constante mesmo sem aggregator"): the constant + the
/// [`compressed_size_per_format`] accounting primitive land now; the
/// cross-subsystem aggregator that actually checks it against a platform total
/// (`architecture_render_budget_registered`, summing render + tools + core vs
/// `Platform::max_total_mb`) is a follow-up.
///
/// The `256` is an interim ceiling, not a measured working set: the plan's W2
/// VRAM projection puts *total* render textures+meshes near ~200 MB on the
/// iPad/ASTC tier, so 256 MB for the cooked cache alone is a deliberately
/// loose placeholder to be tightened when the aggregator lands.
pub const COMPRESSED_TEXTURE_CACHE_BUDGET_MB: u32 = 256;

/// Total GPU byte footprint of a `width × height` texture in `format` with
/// `mip_count` mip levels — the W2.T5 HR-13 budget-accounting primitive.
///
/// wgpu exposes no cross-vendor VRAM query (`device.poll` only drives
/// command completion; Metal/D3D12/Vulkan introspection isn't portable), so
/// the plan's §6 strategy is a deterministic block-math estimate: sum
/// [`MipUploadLayout::for_mip`]'s block-aligned `total_bytes` over every mip
/// (each dimension halved per level, floored at 1). It therefore shares the
/// EXACT block math the uploader uses — a compressed format counts its real
/// block footprint; uncompressed RGBA8 degenerates to `w*h*4` plus the
/// ~⅓ mip tail. Returns `0` for a zero-size texture or a non-color format
/// (`for_mip` → `None`). `u64` so an 8192² RGBA8 pyramid (~256 MB) can't
/// overflow; a `mip_count` past the pyramid floor contributes 1×1 mips only
/// (via the saturating shift), never panics.
#[must_use]
pub fn compressed_size_per_format(
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    mip_count: u32,
) -> u64 {
    // A degenerate 0-area base texture is never uploadable → 0 bytes. (The
    // per-level `.max(1)` floor below is only for SUB-1 mip-tail levels of a
    // real pyramid, not for a genuinely empty base.)
    if width == 0 || height == 0 {
        return 0;
    }
    let mut total = 0u64;
    for level in 0..mip_count {
        // `checked_shr` saturates a shift ≥ 32 to None → `0`, floored to 1,
        // so a pathological `mip_count` adds 1×1 mips instead of panicking.
        let w = width.checked_shr(level).unwrap_or(0).max(1);
        let h = height.checked_shr(level).unwrap_or(0).max(1);
        if let Some(layout) = MipUploadLayout::for_mip(format, w, h) {
            total += layout.total_bytes as u64;
        }
    }
    total
}

/// A fully uploaded cooked texture, ready to bind at `@group(1)`.
///
/// The `bind_group` is built against the bind-group layout passed to
/// [`CompressedTexturePipeline::new`] — pass the same `material_bgl` the
/// [`crate::pipeline::SpritePipeline`] uses and this binds interchangeably
/// with atlas / individual textures (the whole point of the shared-pipeline
/// design: no per-format pipeline to switch to).
#[derive(Debug)]
pub struct UploadedCompressedTexture {
    /// The GPU texture in the cooked artifact's native (possibly compressed)
    /// `wgpu::TextureFormat`.
    pub texture: wgpu::Texture,
    /// A default 2D view over all mip levels.
    pub view: wgpu::TextureView,
    /// Material bind group (`@group(1)`): the view at binding 0, the shared
    /// sampler at binding 1.
    pub bind_group: wgpu::BindGroup,
    /// The resolved wgpu format (for telemetry / HDR routing decisions).
    pub format: wgpu::TextureFormat,
    /// Base-level (mip 0) dimensions in pixels.
    pub width: u32,
    /// Base-level (mip 0) height in pixels.
    pub height: u32,
    /// Number of mip levels uploaded.
    pub mip_level_count: u32,
}

/// The shared upload pipeline for cooked KTX2 textures.
///
/// Holds the device's compression-feature capabilities (to gate uploads) and
/// the sampler + bind-group layout every uploaded texture binds through.
/// There is **one** of these for the whole renderer — see the module header
/// for why a per-format fan-out is unnecessary in wgpu 28.
#[derive(Debug)]
pub struct CompressedTexturePipeline {
    /// What this device can actually sample (W2.T1.5). Uploads are rejected
    /// for formats outside this set.
    feature_set: CompressionFeatureSet,
    /// The `@group(1)` material layout — a `Float { filterable: true }`
    /// texture + `Filtering` sampler. Owned here so callers that don't have a
    /// live `SpritePipeline` (e.g. tests, headless tooling) can still build
    /// bind groups; pass the `SpritePipeline`'s own `material_bgl` to
    /// [`Self::with_layout`] when one exists to guarantee pipeline-compat.
    material_bgl: wgpu::BindGroupLayout,
    /// Filtering sampler bound at material binding 1.
    sampler: wgpu::Sampler,
}

impl CompressedTexturePipeline {
    /// Build a pipeline that owns its own material bind-group layout
    /// (identical in shape to [`crate::pipeline::SpritePipeline`]'s).
    ///
    /// Use [`Self::with_layout`] instead when a `SpritePipeline` already
    /// exists, so the produced bind groups are guaranteed layout-compatible
    /// with that pipeline's `@group(1)`.
    #[must_use]
    pub fn new(gpu: &GpuContext, feature_set: CompressionFeatureSet) -> Self {
        let material_bgl = Self::material_bind_group_layout(&gpu.device);
        Self::with_layout(gpu, feature_set, material_bgl)
    }

    /// Build a pipeline that binds through an EXTERNALLY-owned material
    /// bind-group layout (typically `SpritePipeline::material_bgl`). The
    /// produced [`UploadedCompressedTexture::bind_group`] is then usable with
    /// any pipeline built against that same layout.
    #[must_use]
    pub fn with_layout(
        gpu: &GpuContext,
        feature_set: CompressionFeatureSet,
        material_bgl: wgpu::BindGroupLayout,
    ) -> Self {
        // Filtering sampler — clamped, linear min/mag, linear mip. Cooked
        // textures ship a full mip pyramid, so trilinear (`mipmap = Linear`)
        // is the sensible default; the caller can swap in a custom sampler by
        // building bind groups itself if a sprite needs nearest filtering.
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render cooked-texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        Self {
            feature_set,
            material_bgl,
            sampler,
        }
    }

    /// The canonical material `@group(1)` layout: a `Float { filterable:
    /// true }` D2 texture at binding 0 + a `Filtering` sampler at binding 1.
    /// Mirrors [`crate::pipeline::SpritePipeline`]'s `material_bgl` exactly
    /// (every format this module uploads samples as filterable-float — see
    /// the module header), so a texture uploaded here binds against the
    /// sprite pipeline without a second layout.
    #[must_use]
    pub fn material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-render cooked-texture material bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    /// The compression capabilities this pipeline gates uploads against.
    #[must_use]
    pub fn feature_set(&self) -> CompressionFeatureSet {
        self.feature_set
    }

    /// Resolve + feature-gate a cooked image's format **without touching the
    /// GPU**. Returns the `wgpu::TextureFormat` to create the texture in, or
    /// the same error [`Self::upload`] would raise. Split out so the loader /
    /// tests can pre-flight a format decision cheaply.
    ///
    /// # Errors
    /// [`CompressedUploadError::UnmappableFormat`] for an out-of-subset /
    /// unmapped KTX2 format; [`CompressedUploadError::FormatUnsupportedByDevice`]
    /// when this device cannot sample the (otherwise valid) format.
    pub fn resolve_format(
        &self,
        ktx2_format: Ktx2Format,
    ) -> Result<wgpu::TextureFormat, CompressedUploadError> {
        let (format, _required) = wgpu_format_from_ktx2_format(ktx2_format)?;
        // `supports` reuses the SAME format→feature table (W2.T1), so this
        // can't drift from the mapping above.
        if !self.feature_set.supports(ktx2_format) {
            return Err(CompressedUploadError::FormatUnsupportedByDevice(format));
        }
        // Enforce the shared-layout contract: this pipeline binds every
        // texture through ONE filterable-float / `Filtering`-sampler material
        // layout, so a format that doesn't *unconditionally* sample as
        // `Float { filterable: true }` would fail wgpu bind-group validation
        // at bind time (far from the cause). We pass `None` for device
        // features deliberately: a format that is only filterable *with* an
        // extra device feature (the sole such mapped case is `Rgba32Float`
        // gated on `FLOAT32_FILTERABLE`) is still rejected here — full 32-bit
        // float HDR is the `GameRt` path, not the cooked-sprite path.
        if format.sample_type(None, None)
            != Some(wgpu::TextureSampleType::Float { filterable: true })
        {
            return Err(CompressedUploadError::NotFilterableFloat(format));
        }
        Ok(format)
    }

    /// Upload a decoded cooked KTX2 image to a GPU texture and return a
    /// ready-to-bind [`UploadedCompressedTexture`].
    ///
    /// Thin wrapper over [`Self::upload_parts`] — it reads the four fields the
    /// upload actually needs (`format`, `width`, `height`, `mip_levels`) off
    /// the decoded image. The parts entry point exists because [`Ktx2Image`]
    /// is `#[non_exhaustive]` and cannot be constructed outside its crate, so
    /// the loader (W2.T4) — which may resolve mips from an `Arc<Asset>` — and
    /// the GPU smoke tests both drive uploads through the primitives instead.
    ///
    /// # Errors
    /// See [`CompressedUploadError`]: unmappable / device-unsupported format,
    /// zero mip levels, or a mip payload shorter than its block footprint.
    pub fn upload(
        &self,
        gpu: &GpuContext,
        image: &Ktx2Image,
        label: Option<&str>,
    ) -> Result<UploadedCompressedTexture, CompressedUploadError> {
        self.upload_parts(
            gpu,
            image.format,
            image.width,
            image.height,
            &image.mip_levels,
            label,
        )
    }

    /// Upload from the raw KTX2 parts (format + base dimensions + mip
    /// pyramid) — the [`Self::upload`] core, callable without an owned
    /// [`Ktx2Image`].
    ///
    /// Every mip level is written with block-aligned `bytes_per_row` /
    /// `rows_per_image` ([`MipUploadLayout`]) so BC/ASTC/ETC2 block payloads
    /// land correctly. Each mip's `data` slice is length-checked against its
    /// block-aligned footprint **before** the `write_texture` call, so a
    /// truncated artifact is rejected on the CPU instead of faulting in the
    /// backend. `width`/`height` are the base-level (mip 0) pixel dimensions.
    ///
    /// # Errors
    /// See [`CompressedUploadError`]: unmappable / device-unsupported format,
    /// zero mip levels, or a mip payload shorter than its block footprint.
    pub fn upload_parts(
        &self,
        gpu: &GpuContext,
        ktx2_format: Ktx2Format,
        width: u32,
        height: u32,
        mip_levels: &[MipLevel],
        label: Option<&str>,
    ) -> Result<UploadedCompressedTexture, CompressedUploadError> {
        let format = self.resolve_format(ktx2_format)?;

        if mip_levels.is_empty() {
            return Err(CompressedUploadError::NoMipLevels);
        }

        // Pre-flight: every mip's layout + length BEFORE creating the texture,
        // so a corrupt artifact yields a clean error and no orphan GPU texture.
        let layouts = mip_layouts(format, mip_levels)?;

        let mip_level_count = mip_levels.len() as u32;
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            // Cooked textures are read-only GPU inputs: sampled in the
            // fragment shader (TEXTURE_BINDING) and filled via write_texture
            // (COPY_DST). No COPY_SRC — there is no readback path for a
            // compressed source (the Image-Tools edit path is RGBA8 only).
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for (mip_index, (mip, layout)) in mip_levels.iter().zip(&layouts).enumerate() {
            gpu.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: mip_index as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &mip.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(layout.bytes_per_row),
                    rows_per_image: Some(layout.rows_per_image),
                },
                wgpu::Extent3d {
                    width: layout.width_px,
                    height: layout.height_px,
                    depth_or_array_layers: 1,
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render cooked-texture material bg"),
            layout: &self.material_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        Ok(UploadedCompressedTexture {
            texture,
            view,
            bind_group,
            format,
            width,
            height,
            mip_level_count,
        })
    }
}

/// Compute + length-validate the per-mip upload layout for the whole
/// pyramid. Pure (no GPU), so `upload` can pre-flight before allocating a
/// texture and tests can exercise the block math directly.
fn mip_layouts(
    format: wgpu::TextureFormat,
    mips: &[MipLevel],
) -> Result<Vec<MipUploadLayout>, CompressedUploadError> {
    if mips.is_empty() {
        return Err(CompressedUploadError::NoMipLevels);
    }
    let mut out = Vec::with_capacity(mips.len());
    for (mip_index, mip) in mips.iter().enumerate() {
        // `for_mip` only returns None for aspect-required (depth/stencil/
        // multi-planar) formats, which a KTX2 color texture never is; treat a
        // None as an unmappable format rather than panicking.
        let layout = MipUploadLayout::for_mip(format, mip.width, mip.height).ok_or(
            CompressedUploadError::UnmappableFormat(FormatError::UnmappedKtx2Format),
        )?;
        if mip.data.len() < layout.total_bytes {
            return Err(CompressedUploadError::MipDataTooShort {
                mip_index,
                expected: layout.total_bytes,
                actual: mip.data.len(),
            });
        }
        out.push(layout);
    }
    Ok(out)
}

#[cfg(test)]
#[path = "compressed_pipeline_tests.rs"]
mod tests;
