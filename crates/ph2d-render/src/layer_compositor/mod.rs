//! GPU layer compositor (Painter W3, Block 2) — the real-time sibling of the
//! CPU reference `ph2d_tool_painter::compositor::composite`.
//! `docs/Painter_projeto/02_layers.md` §2.11 + §2.12.
//!
//! # What this is
//!
//! A single compute pass that composites a flattened layer op-list into a
//! straight-sRGB8 output texture, blending in linear light with the 22 W3C
//! Compositing Level 1 modes. The blend math source-of-truth is
//! `ph2d_painter_brush::blend::apply`; this crate stays decoupled from the
//! painter tool (it speaks raw `u8` blend codes, not the `BlendMode` enum), so
//! there is no dependency cycle — the tool flattens its `LayerStack` into
//! [`LayerOp`]s and hands them here.
//!
//! # Caching / dirty-rect
//!
//! Layer pixels live in a canvas-sized `texture_2d_array` (`rgba8unorm`, one
//! slice per cached layer). A [`BTreeMap`] (HR-5) maps each layer key to its
//! slice + last-uploaded content version; a slice is re-uploaded only when its
//! version changes, and least-recently-used slices are evicted when the array
//! is full. The compute dispatch covers only a [`Region`] (the dirty rect), so
//! a stroke that touches one corner does not recomposite the whole 4K canvas.
//!
//! # Determinism caveat
//!
//! The shader literals (sRGB transfer, luminosity coefficients, W3C constants)
//! are pinned bit-identical to the Rust source by
//! `shader_blend_modes_bit_identical_with_rust`. Runtime outputs are NOT
//! guaranteed bit-identical across GPU backends (`pow`/`sqrt` are ULP-bounded);
//! the GPU↔CPU parity gate asserts agreement within ±1 byte. Same discipline
//! as `stamp.wgsl` and the OKLab coefficient gate.

use ph2d_color::srgb::srgb_to_linear_byte;
use ph2d_gpu::GpuContext;
use std::collections::BTreeMap;

mod compositor;
mod ops; // the op-list model + its pure queries (LOC cap: sibling module)

#[cfg(test)]
mod tests;

use ops::{COMBINE_BLOOM, COMBINE_GAUSSIAN, COMBINE_SHARPEN};
pub use ops::{
    LayerMask, LayerOp, MAX_BLUR_HALF, SPATIAL_BLOOM, SPATIAL_CHROMA, SPATIAL_GAUSSIAN,
    SPATIAL_MOTION, SPATIAL_SHADOWS_HIGHLIGHTS, SPATIAL_SHARPEN, gaussian_weights, has_spatial,
    motion_weights,
};
use ops::{distinct_layer_count, op_mask, validate_op_list};

pub(crate) const LAYER_COMPOSITE_WGSL: &str = include_str!("../shaders/layer_composite.wgsl");

/// Workgroup edge (mirrors the `@workgroup_size(8, 8, 1)` in the shader).
const WORKGROUP_EDGE: u32 = 8;

/// sRGB decode LUT length (one entry per 8-bit byte value).
const SRGB_LUT_LEN: usize = 256;

/// Build the 256-entry sRGB→linear decode table the shader binds. `[b]` is
/// `srgb_to_linear_byte(b)` — the exact f32 the CPU compositor decodes with, so
/// the GPU decode is bit-identical (no `pow` rounding drift, no hardware-sRGB
/// approximation). Pinned by `srgb_lut_matches_cpu_transfer`.
fn build_srgb_lut() -> [f32; SRGB_LUT_LEN] {
    let mut lut = [0.0f32; SRGB_LUT_LEN];
    for (b, slot) in lut.iter_mut().enumerate() {
        *slot = srgb_to_linear_byte(b as u8);
    }
    lut
}

/// VRAM the layer texture-array cache may hold on a **shared-memory** device
/// (integrated GPU, software adapter): 512 MiB. Discrete cards get more — see
/// [`layer_cache_budget`], which is what the compositor actually asks.
///
/// ⚠️ The doc this replaces claimed *"a 4K RGBA8 slice is ~33.2 MB, so this 512 MB
/// budget holds ~15 layers at 4K"*. 33.2 MB is 4096×2048 — a "4K" **video** frame.
/// A square 4096² canvas, which is what an illustrator actually opens, is
/// **64 MiB a slice**, so the real answer was **8**, and on the ninth layer the
/// compositor refused and handed the document to a producer 107× slower. A budget
/// whose own arithmetic is off by 2× is not a budget; it is a number nobody
/// re-derived.
pub const LAYER_CACHE_BUDGET_SHARED_BYTES: u64 = 512 * 1024 * 1024;

/// VRAM the layer cache may hold on a **discrete** GPU: 1 GiB — 16 slices at
/// 4096², 64 at 2048², 256 at 1024².
///
/// **The resource is VRAM held by this cache**, and the number is measured, not
/// chosen (`measure_how_far_the_array_actually_goes`, RTX 5060 Ti 16 GB, full
/// recompose of a 4096² canvas):
///
/// | slices | VRAM | GPU floor | CPU fallback for the same document |
/// |---|---|---|---|
/// | 8 | 512 MiB | 2,06 ms | ~149 ms |
/// | **16** | **1 GiB** | **3,86 ms** | **~254 ms** |
/// | 32 | 2 GiB | 7,66 ms | — |
/// | 48 | 3 GiB | 16,13 ms | — |
/// | 64 | 4 GiB | 14,82 ms | — |
///
/// ⚠️ **The device is not the binding constraint and that is the point.** It held
/// 4 GiB without complaining, and the interactive knee — where a full recompose
/// stops fitting a 60 Hz frame — is around **32 slices / 2 GiB**. The budget is
/// set below both because PH2D is not the card's only tenant (the sprite atlas,
/// Vello's buffers, the preview slots and the impasto planes are all resident
/// too), and because wgpu 28 **exposes no VRAM query at all** — `AdapterInfo`
/// carries `device_type` and nothing about memory — so a bigger number would be
/// an assumption about hardware this machine cannot measure.
///
/// Raising it further is a real option and it has a prerequisite, not a bigger
/// literal: **make the array allocation fallible** (`push_error_scope(OutOfMemory)`)
/// so a card that cannot hold the ask degrades to fewer slices instead of losing
/// the device. Until then, 1 GiB is the largest number that is safe on hardware
/// nobody here has.
pub const LAYER_CACHE_BUDGET_DISCRETE_BYTES: u64 = 1024 * 1024 * 1024;

/// The whole point of the split: a discrete card gets more. Asserted at COMPILE
/// time, because it is a fact about two literals — a runtime test of it would
/// only ever tell you what the compiler already knew.
const _: () = assert!(LAYER_CACHE_BUDGET_DISCRETE_BYTES > LAYER_CACHE_BUDGET_SHARED_BYTES);

/// The cache's VRAM budget on `device_type` — the one property wgpu exposes that
/// says anything about where the memory comes from.
///
/// A discrete card has VRAM of its own; an integrated one spends the SYSTEM's
/// RAM, where a gigabyte of layer cache competes with the document, the undo
/// queue and the OS. Same cache, different resource, different number.
#[must_use]
pub fn layer_cache_budget(device_type: wgpu::DeviceType) -> u64 {
    match device_type {
        wgpu::DeviceType::DiscreteGpu => LAYER_CACHE_BUDGET_DISCRETE_BYTES,
        // Integrated / virtual / CPU / unknown all share memory with the host.
        _ => LAYER_CACHE_BUDGET_SHARED_BYTES,
    }
}

/// Largest layer stack the contract allows — spec §2.2 `HARD_CAP_LAYERS = 999`
/// (mirrors Procreate), matching `ph2d_tool_painter::layers::HARD_CAP_LAYERS`
/// and the savefile `MAX_LAYERS = 1000` (999 + overflow headroom). Independent
/// of the per-budget cap — the absolute ceiling the cache will never exceed.
pub const HARD_CAP_LAYERS: u32 = 999;

/// Maximum cached slices for a `width × height` canvas under `budget` bytes.
/// `0` for a degenerate (zero-area) canvas. The compositor additionally clamps
/// to the device's `max_texture_array_layers`.
#[must_use]
pub fn max_layers_for_budget(width: u32, height: u32, budget: u64) -> u32 {
    let per_slice = (width as u64) * (height as u64) * 4;
    if per_slice == 0 {
        return 0;
    }
    (budget / per_slice).min(HARD_CAP_LAYERS as u64) as u32
}

/// A rectangular sub-region of the canvas (dirty rect). Clamped to canvas
/// bounds by the compositor — mirror of `compositor::Region`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Region {
    /// The whole `width × height` canvas.
    #[must_use]
    pub fn full(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            w: width,
            h: height,
        }
    }

    /// Clamp to `width × height` (same arithmetic as the CPU compositor).
    #[must_use]
    fn clamped(self, width: u32, height: u32) -> Self {
        let x = self.x.min(width);
        let y = self.y.min(height);
        Self {
            x,
            y,
            w: self.w.min(width - x),
            h: self.h.min(height - y),
        }
    }
}

/// Borrowed straight-sRGB8 pixels for one layer plus a cheap content version.
/// The compositor re-uploads a slice only when `version` changes, so the
/// caller must bump it whenever the layer's pixels change (e.g. a committed
/// stroke). `rgba8` must be exactly `canvas_w * canvas_h * 4` bytes.
pub struct LayerPixels<'a> {
    pub version: u64,
    pub rgba8: &'a [u8],
}

/// Resolves a layer key to its current pixels. Implemented by the painter tool
/// over its `BTreeMap<LayerId, LayerImage>`; the render crate never sees the
/// tool's types.
pub trait LayerPixelProvider {
    fn layer_pixels(&self, key: u64) -> Option<LayerPixels<'_>>;
}

/// Why a composite could not be encoded.
#[derive(Debug, PartialEq, Eq)]
pub enum LayerCompositeError {
    /// The op-list references more distinct live layers than fit in the cache
    /// for this canvas size (per-budget / device array-layer cap). Carries the
    /// requested count and the cap.
    TooManyLayers { requested: u32, cap: u32 },
    /// A [`LayerOp::Layer`] referenced a key the provider could not supply, or
    /// the supplied pixel buffer was not `canvas_w * canvas_h * 4` bytes.
    MissingOrMalformedLayer { key: u64 },
    /// Group push/pop ops were unbalanced or exceeded the max group depth.
    MalformedOpList,
    /// The canvas dimensions were zero or exceeded the device limit.
    InvalidCanvas { width: u32, height: u32 },
}

impl core::fmt::Display for LayerCompositeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooManyLayers { requested, cap } => {
                write!(
                    f,
                    "layer compositor: {requested} live layers exceed cache cap {cap}"
                )
            }
            Self::MissingOrMalformedLayer { key } => {
                write!(
                    f,
                    "layer compositor: layer key {key} missing or wrong-sized"
                )
            }
            Self::MalformedOpList => write!(f, "layer compositor: unbalanced or too-deep op-list"),
            Self::InvalidCanvas { width, height } => {
                write!(f, "layer compositor: invalid canvas {width}x{height}")
            }
        }
    }
}

impl core::error::Error for LayerCompositeError {}

/// Root accumulator + `MAX_GROUP_DEPTH` (8) — mirror of the shader's
/// `MAX_STACK` and `ph2d_tool_painter::layers::MAX_GROUP_DEPTH`.
const MAX_STACK: u32 = 9;

// ── GPU-side POD mirrors ─────────────────────────────────────────────────

/// One op as the shader sees it (32 bytes; mirrors WGSL `Op`).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuOp {
    kind: u32,
    layer_slot: u32,
    blend_mode: u32,
    opacity: f32,
    /// Texture-array slice of this op's mask, or [`NO_MASK_SLOT`].
    mask_slot: u32,
    /// [`FLAG_CLIPPING`] | [`FLAG_MASK_INVERTED`].
    flags: u32,
    _pad0: u32,
    _pad1: u32,
}

/// `mask_slot` sentinel: this op carries no mask. Distinct from slice 0, which
/// is a perfectly ordinary layer.
const NO_MASK_SLOT: u32 = u32::MAX;
/// `flags` bit 0 — the op clips to the current clip base.
const FLAG_CLIPPING: u32 = 1;
/// `flags` bit 1 — the mask reads `1 - luma`.
const FLAG_MASK_INVERTED: u32 = 2;

/// Op kind discriminants — mirror the WGSL `OP_*` consts.
const OP_LAYER: u32 = 0;
const OP_PUSH_GROUP: u32 = 1;
const OP_POP_GROUP: u32 = 2;
const OP_ADJUSTMENT: u32 = 3;
/// Placeholder for a [`LayerOp::SpatialAdjustment`] in the flattened GPU op
/// array: the segment compute loop treats it as a no-op (the spatial effect is
/// driven CPU-side as a pass break), but emitting it keeps the GPU op indices
/// 1:1 with the `LayerOp` list so segment ranges are trivial to compute.
const OP_SPATIAL: u32 = 4;

/// Per-adjustment params as the shader sees them (16 bytes; mirrors WGSL
/// `AdjParams`). `kind` is an `ADJ_*` code; `p0/p1/p2` are the kind's scalar
/// params. Lives in a storage buffer indexed by an `OP_ADJUSTMENT` op's
/// `layer_slot`.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct AdjParamsGpu {
    kind: u32,
    p0: f32,
    p1: f32,
    p2: f32,
}

/// Compositor globals (32 bytes; mirrors WGSL `Globals`).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuGlobals {
    canvas_width: u32,
    canvas_height: u32,
    region_x: u32,
    region_y: u32,
    region_w: u32,
    region_h: u32,
    op_count: u32,
    /// `0` ⇒ write the output at REGION-LOCAL coords into a region-sized `out`
    /// (the dirty-rect readback path). `1` ⇒ write at CANVAS coords into a
    /// canvas-sized, persistent `out` so a region dispatch only refreshes the
    /// dirty rect and leaves the rest intact (E5 live-stroke region recomposite,
    /// ADR-0078 S2 perf). Occupies the former `_pad` slot — size stays 32 B.
    out_canvas_coords: u32,
}

// ── Segmented (spatial pass-graph) GPU mirrors ───────────────────────────────
//
// The single-pass `cs_flat`/`cs_grouped` write straight sRGB8 directly and start
// from a zeroed accumulator. The segmented path instead composites *runs* of ops
// into linear `Rgba32Float` intermediates (so a spatial kernel can read a radius
// of neighbours), with these per-pass uniforms. Each pass operates on the
// `work_region` = the requested dirty rect dilated by the total blur halo (so the
// kernel has valid neighbours up to the region edge); intermediate local coords
// map to canvas coords via `region_*`, exactly like `resolve_pixel`.

/// Globals for `cs_segment` (48 bytes; mirrors WGSL `SegGlobals`). Composites
/// `ops[op_start..op_end]` over `work_region` into a linear target, starting the
/// accumulator from `base_in` when `seg_from_base != 0` (else from zero).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SegGlobals {
    canvas_width: u32,
    canvas_height: u32,
    region_x: u32,
    region_y: u32,
    region_w: u32,
    region_h: u32,
    op_start: u32,
    op_end: u32,
    seg_from_base: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

/// Globals for the blur passes (32 bytes; mirrors WGSL `BlurGlobals`). A
/// convolution over a `width × height` linear texture with the symmetric kernel
/// `weights[0..=half]` (clamp-to-edge). `cs_blur_h`/`cs_blur_v` ignore the
/// direction (axis-aligned); `cs_blur_dir` samples taps along `(dir_x, dir_y)`.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurGlobals {
    width: u32,
    height: u32,
    half: u32,
    _pad0: u32,
    dir_x: f32,
    dir_y: f32,
    /// `1.0` = premultiply each tap on read (the first blur pass), `0.0` = the
    /// source is already premultiplied (later passes). Premultiplied convolution
    /// feathers coverage into transparency instead of leaking garbage RGB from
    /// transparent texels (identity for an opaque base).
    premul_read: f32,
    _pad2: f32,
}

/// Globals for `cs_combine` (32 bytes; mirrors WGSL `CombineGlobals`). Blends
/// the blurred result back over the base per the adjustment's `blend_mode` +
/// `opacity` — the spatial mirror of `apply_adjustment_op`.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CombineGlobals {
    width: u32,
    height: u32,
    blend_mode: u32,
    combine_mode: u32, // COMBINE_GAUSSIAN (0) | COMBINE_SHARPEN (1)
    opacity: f32,
    amount: f32, // unsharp amount for COMBINE_SHARPEN (ignored otherwise)
    _pad2: f32,
    _pad3: f32,
}

/// Globals for `cs_encode` (16 bytes; mirrors WGSL `EncodeGlobals`). Reads the
/// final linear `work_region` intermediate at `(src_off_x, src_off_y)` and
/// writes the `out_w × out_h` straight-sRGB8 output (the requested dirty rect).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct EncodeGlobals {
    out_w: u32,
    out_h: u32,
    src_off_x: u32,
    src_off_y: u32,
}

/// Globals for `cs_chroma` (32 bytes; mirrors WGSL `ChromaGlobals`). The radial
/// centre is given in `work_region`-LOCAL coords (`canvas_centre − work_origin`),
/// and `scale_c = shift_c / half_diag` is precomputed CPU-side, so the gather's
/// per-pixel displacement `dir·scale_c` (`dir = local − centre`) uses only exact
/// IEEE ops — no per-pixel `sqrt` → parity-robust nearest sampling.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ChromaGlobals {
    width: u32,
    height: u32,
    center_x: f32,
    center_y: f32,
    scale_r: f32,
    scale_g: f32,
    scale_b: f32,
    _pad: f32,
}

/// Globals for `cs_bloom_bright` (16 bytes; mirrors WGSL `BloomGlobals`). The
/// bright-pass extracts `color·α·smoothstep(threshold, threshold+falloff, luma)`
/// as a premultiplied glow over the `width × height` work region.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BloomGlobals {
    width: u32,
    height: u32,
    threshold: f32,
    falloff: f32,
}

/// Globals for `cs_bloom_down`/`cs_bloom_up` (32 bytes; mirrors WGSL
/// `BloomMipGlobals`). The radius-independent Bloom blur: downsample the glow by
/// `factor`, blur at low res, bilinear-upsample back.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BloomMipGlobals {
    src_w: u32,
    src_h: u32,
    dst_w: u32,
    dst_h: u32,
    factor: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

/// Bloom downsamples until its low-res blur radius is ≤ this — bounding the kernel
/// so Bloom costs ~the same at any radius (the dual-filter idea, one level).
const BLOOM_MAX_LOW_RADIUS: f32 = 16.0;

/// The power-of-two downsample factor for a Bloom `radius`: the smallest so the
/// low-res blur radius (`radius / factor`) is ≤ [`BLOOM_MAX_LOW_RADIUS`]. `1` (no
/// downsample) for small radii, where the direct blur is already cheap.
fn bloom_downsample_factor(radius: f32) -> u32 {
    if radius <= BLOOM_MAX_LOW_RADIUS {
        return 1;
    }
    ((radius / BLOOM_MAX_LOW_RADIUS).ceil() as u32)
        .next_power_of_two()
        .clamp(1, 32)
}

/// Globals for `cs_sh_luma` + `cs_combine_sh` (48 bytes; mirrors WGSL `ShGlobals`).
/// The luma pass reads only `width`/`height`; the combine reads the 6 tonal scalars
/// plus the adjustment's own `blend_mode`/`opacity`. The two radii drive the blurs
/// (CPU-side weights), so they are NOT in this uniform.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ShGlobals {
    width: u32,
    height: u32,
    shadows_amount: f32,
    highlights_amount: f32,
    shadows_tonal_width: f32,
    highlights_tonal_width: f32,
    color_correction: f32,
    midtone_contrast: f32,
    blend_mode: u32,
    opacity: f32,
    _pad0: u32,
    _pad1: u32,
}

/// A cached layer's place in the texture array + dirty tracking.
struct CachedSlice {
    slice: u32,
    version: u64,
    last_used: u64,
}

/// The texture array backing the layer cache.
struct LayerArray {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    capacity: u32,
}

/// The region-sized output (straight sRGB8, `rgba8unorm` storage).
struct OutTex {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

/// GPU layer compositor. One per painter session; holds the compute pipeline,
/// the layer texture-array cache, and reusable per-dispatch buffers.
pub struct LayerCompositor {
    /// Lean entry for group-free op-lists (the common case) — high occupancy.
    pipeline_flat: wgpu::ComputePipeline,
    /// Entry with the per-pixel group stack — used only when ops nest groups.
    pipeline_grouped: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    device_max_layers: u32,
    /// VRAM this cache may hold, resolved from the device at construction
    /// ([`layer_cache_budget`]).
    cache_budget_bytes: u64,
    array: Option<LayerArray>,
    out: Option<OutTex>,
    cache: BTreeMap<u64, CachedSlice>,
    /// Monotonic frame clock for LRU eviction.
    clock: u64,
    /// Reusable scratch for the flattened GPU op-list (HR-3: no per-frame
    /// realloc once warm).
    scratch_ops: GpuOpScratch,
    /// Persistent op storage buffer (grown when `op_count` exceeds capacity).
    op_buffer: Option<(wgpu::Buffer, u64)>,
    /// Persistent globals uniform buffer.
    globals_buffer: wgpu::Buffer,
    /// Immutable sRGB→linear decode LUT (uploaded once at construction).
    srgb_lut_buffer: wgpu::Buffer,
    /// Persistent adjustment-params storage buffer (grown as needed; always
    /// holds ≥1 element so binding 5 is never zero-sized).
    adj_params_buffer: Option<(wgpu::Buffer, u64)>,
    /// Persistent display-space transfer-LUT storage buffer (W4 Curves/Levels;
    /// grown as needed; always holds ≥1 f32 so binding 6 is never zero-sized).
    adj_luts_buffer: Option<(wgpu::Buffer, u64)>,

    // ── Segmented spatial pass-graph (W4) ────────────────────────────────────
    // Only built/used when an op-list contains a `LayerOp::SpatialAdjustment`;
    // the single-pass path above is untouched (bit-identical for the common
    // case). Pipelines are created once at construction (cheap, no textures).
    /// Composites a run of ops into a linear `Rgba32Float` intermediate, optionally
    /// starting from a base texture (segment between pass breaks).
    pipeline_segment: wgpu::ComputePipeline,
    /// Separable-blur horizontal / vertical passes (shared bgl `bgl_blur`).
    pipeline_blur_h: wgpu::ComputePipeline,
    pipeline_blur_v: wgpu::ComputePipeline,
    /// Directional (motion) blur — single 1-D pass along `(dir_x, dir_y)`.
    pipeline_blur_dir: wgpu::ComputePipeline,
    /// Chromatic-aberration gather — per-channel radial shift (single pass).
    pipeline_chroma: wgpu::ComputePipeline,
    /// Bloom bright-pass — extract the premultiplied bright-excess glow.
    pipeline_bloom_bright: wgpu::ComputePipeline,
    /// Bloom radius-independent blur: box-downsample + bilinear-upsample around a
    /// bounded low-res separable blur (O(1) regardless of radius).
    pipeline_bloom_down: wgpu::ComputePipeline,
    pipeline_bloom_up: wgpu::ComputePipeline,
    /// Shadows/Highlights luma extract — display luma → `.r` for the two scalar blurs.
    pipeline_sh_luma: wgpu::ComputePipeline,
    /// Shadows/Highlights tonal combine — local correction from base + 2 luma maps.
    pipeline_sh_combine: wgpu::ComputePipeline,
    /// Blends the blurred result back over the base (spatial `apply_adjustment_op`).
    pipeline_combine: wgpu::ComputePipeline,
    /// Encodes the final linear intermediate → straight-sRGB8 output (cropped to
    /// the requested dirty rect).
    pipeline_encode: wgpu::ComputePipeline,
    bgl_segment: wgpu::BindGroupLayout,
    bgl_blur: wgpu::BindGroupLayout,
    bgl_combine: wgpu::BindGroupLayout,
    bgl_encode: wgpu::BindGroupLayout,
    bgl_chroma: wgpu::BindGroupLayout,
    bgl_bloom: wgpu::BindGroupLayout,
    bgl_bloom_mip: wgpu::BindGroupLayout,
    bgl_sh_luma: wgpu::BindGroupLayout,
    bgl_sh_combine: wgpu::BindGroupLayout,
    /// Per-pass uniform buffers (rewritten + submitted per pass; queue ordering
    /// makes single shared buffers safe). Created once.
    seg_globals_buffer: wgpu::Buffer,
    blur_globals_buffer: wgpu::Buffer,
    combine_globals_buffer: wgpu::Buffer,
    encode_globals_buffer: wgpu::Buffer,
    chroma_globals_buffer: wgpu::Buffer,
    bloom_globals_buffer: wgpu::Buffer,
    bloom_mip_globals_buffer: wgpu::Buffer,
    sh_globals_buffer: wgpu::Buffer,
    /// Separable Gaussian weights (`weights[0..=half]`), grown as needed.
    blur_weights_buffer: Option<(wgpu::Buffer, u64)>,
    /// 1×1 linear dummy bound as `base_in` for the first segment (start-from-zero).
    seg_base_dummy: wgpu::TextureView,
    /// Linear `Rgba32Float` work intermediates, sized to the current `work_region`
    /// (rebuilt when it grows). `base[2]` ping-pong across segments + combines;
    /// `blur[2]` ping-pong across the separable H/V passes. Reused across frames.
    work: Option<WorkTextures>,
}

/// Linear `Rgba32Float` intermediates for the segmented pass-graph, all sized to
/// the active `work_region`. Each texture carries both `STORAGE_BINDING` (written
/// by a pass) and `TEXTURE_BINDING` (sampled by a later pass) usage; the single
/// default view serves either role (never the same texture in one pass).
struct WorkTextures {
    width: u32,
    height: u32,
    base: [WorkTex; 2],
    blur: [WorkTex; 2],
    /// Shadows/Highlights scratch: `sh[0]` holds the extracted luma field, `sh[1]`
    /// the blurred shadows tone map (the highlights map reuses `blur[1]`, the blur
    /// temp `blur[0]`). Only written by the S/H sub-graph; allocated with the rest.
    sh: [WorkTex; 2],
}

struct WorkTex {
    #[allow(dead_code)] // kept alive; the view is what binds
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

/// Reusable scratch for the flattened GPU op-list. Construct once, reuse
/// across frames: [`flatten_layer_ops`] clears and refills it without
/// allocating once it is warm (HR-3 — `layers_no_alloc_hot_compose`).
#[derive(Default)]
pub struct GpuOpScratch {
    ops: Vec<GpuOp>,
    /// Per-adjustment params, parallel to the `OP_ADJUSTMENT` ops (each such op's
    /// `layer_slot` indexes this). Reused across frames like `ops` (HR-3).
    adj: Vec<AdjParamsGpu>,
}

impl GpuOpScratch {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of compositor ops (`len()` historically meant the op count).
    #[must_use]
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Backing op capacity — exposed for the no-alloc gate to assert stability.
    #[doc(hidden)]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.ops.capacity()
    }
}

/// Flatten [`LayerOp`]s into `scratch`, resolving each layer key to its cached
/// slice via `slot_of`. The hot per-frame CPU work; reuses `scratch`'s
/// capacity so it does NOT allocate once warm (HR-3). A key `slot_of` resolves
/// to (defaulting to 0 for an absent key) becomes the texture-array slice the
/// shader samples. `composite` calls this with the live cache as the resolver.
///
/// A **mask** key resolves through `mask_slot_of`, which is deliberately
/// fallible where `slot_of` is not: a mask the provider could not serve at
/// canvas size degrades to *no mask* (the CPU reference's behaviour), rather
/// than poisoning the whole composite. See [`LayerMask`].
pub fn flatten_layer_ops(
    ops: &[LayerOp],
    slot_of: impl Fn(u64) -> u32,
    mask_slot_of: impl Fn(u64) -> Option<u32>,
    scratch: &mut GpuOpScratch,
) {
    /// `(mask_slot, flags)` for an op's optional mask + clipping flag.
    fn coverage(
        mask: Option<LayerMask>,
        clipping: bool,
        mask_slot_of: &impl Fn(u64) -> Option<u32>,
    ) -> (u32, u32) {
        let mut flags = if clipping { FLAG_CLIPPING } else { 0 };
        let slot = match mask.and_then(|m| mask_slot_of(m.key).map(|s| (s, m.inverted))) {
            Some((slot, inverted)) => {
                if inverted {
                    flags |= FLAG_MASK_INVERTED;
                }
                slot
            }
            None => NO_MASK_SLOT,
        };
        (slot, flags)
    }

    scratch.ops.clear();
    scratch.adj.clear();
    for op in ops {
        let g = match op {
            LayerOp::Layer {
                key,
                blend_mode,
                opacity,
                mask,
                clipping,
            } => {
                let (mask_slot, flags) = coverage(*mask, *clipping, &mask_slot_of);
                GpuOp {
                    kind: OP_LAYER,
                    layer_slot: slot_of(*key),
                    blend_mode: *blend_mode as u32,
                    // Clamp to [0,1] to match the CPU reference (compositor.rs
                    // clamps layer.opacity before folding into source alpha); an
                    // out-of-range opacity would otherwise diverge (audit LOW).
                    opacity: opacity.clamp(0.0, 1.0),
                    mask_slot,
                    flags,
                    _pad0: 0,
                    _pad1: 0,
                }
            }
            LayerOp::PushGroup => GpuOp {
                kind: OP_PUSH_GROUP,
                layer_slot: 0,
                blend_mode: 0,
                opacity: 1.0,
                mask_slot: NO_MASK_SLOT,
                flags: 0,
                _pad0: 0,
                _pad1: 0,
            },
            LayerOp::PopGroup {
                blend_mode,
                opacity,
            } => GpuOp {
                kind: OP_POP_GROUP,
                layer_slot: 0,
                blend_mode: *blend_mode as u32,
                opacity: opacity.clamp(0.0, 1.0),
                mask_slot: NO_MASK_SLOT,
                flags: 0,
                _pad0: 0,
                _pad1: 0,
            },
            LayerOp::Adjustment {
                kind,
                params,
                blend_mode,
                opacity,
                mask,
            } => {
                // The op's `layer_slot` indexes the params we stash in parallel.
                let params_index = scratch.adj.len() as u32;
                scratch.adj.push(AdjParamsGpu {
                    kind: *kind as u32,
                    p0: params[0],
                    p1: params[1],
                    p2: params[2],
                });
                let (mask_slot, flags) = coverage(*mask, false, &mask_slot_of);
                GpuOp {
                    kind: OP_ADJUSTMENT,
                    layer_slot: params_index,
                    blend_mode: *blend_mode as u32,
                    opacity: opacity.clamp(0.0, 1.0),
                    mask_slot,
                    flags,
                    _pad0: 0,
                    _pad1: 0,
                }
            }
            // Spatial adjustments are driven CPU-side as pass breaks; emit a
            // no-op placeholder so GPU op indices mirror the `LayerOp` list 1:1
            // (the segment compute loop ignores `OP_SPATIAL`). The kernel/params
            // are read from the original op-list by the segmented orchestrator.
            LayerOp::SpatialAdjustment { .. } => GpuOp {
                kind: OP_SPATIAL,
                layer_slot: 0,
                blend_mode: 0,
                opacity: 1.0,
                mask_slot: NO_MASK_SLOT,
                flags: 0,
                _pad0: 0,
                _pad1: 0,
            },
        };
        scratch.ops.push(g);
    }
}

/// Block-on-GPU readback of an `rgba8unorm` texture to a tight `w*h*4` buffer
/// (strips the 256-byte row padding). Test/verification only.
fn readback_rgba8(gpu: &GpuContext, texture: &wgpu::Texture, width: u32, height: u32) -> Vec<u8> {
    if width == 0 || height == 0 {
        return Vec::new();
    }
    let unpadded_bpr = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let buffer_size = (padded_bpr as u64) * (height as u64);
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-render layer_composite readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-render layer_composite readback encoder"),
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

    let (tx, rx) = std::sync::mpsc::channel();
    staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    // Check the map result: on failure (device lost / validation) return empty
    // rather than letting `get_mapped_range` panic with an opaque "not mapped"
    // message that hides the real cause (audit 2026-06-01 LOW; test-path only).
    match rx.recv() {
        Ok(Ok(())) => {}
        _ => return Vec::new(),
    }

    let mapped = staging.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((unpadded_bpr as usize) * (height as usize));
    for row in 0..height as usize {
        let start = row * padded_bpr as usize;
        out.extend_from_slice(&mapped[start..start + unpadded_bpr as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}
