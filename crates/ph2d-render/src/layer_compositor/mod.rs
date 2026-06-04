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

#[cfg(test)]
mod tests;

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

/// VRAM budget for the layer texture-array cache, in bytes. A 4K RGBA8 slice
/// is ~33.2 MB, so this 512 MB budget holds ~15 layers at 4K and far more at
/// typical canvas sizes. Gated by [`max_layers_for_budget`] /
/// `layers_max_count_per_budget`. The hard ceiling is also bounded by the
/// device's `max_texture_array_layers`.
pub const LAYER_CACHE_BUDGET_BYTES: u64 = 512 * 1024 * 1024;

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

/// One flattened compositor op, emitted by the caller from its `LayerStack`
/// (top-down, bottom-to-top within each sibling list — the same order the CPU
/// `composite_into` recurses). `key` is the caller's stable layer identifier
/// (`LayerId.0`); `blend_mode` is the `BlendMode` wire `u8`; `opacity` folds
/// into the source alpha. Groups bracket their children with
/// [`LayerOp::PushGroup`] … [`LayerOp::PopGroup`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LayerOp {
    /// Blend a raster layer's pixels over the current accumulator.
    Layer {
        key: u64,
        blend_mode: u8,
        opacity: f32,
    },
    /// Begin a group: push a fresh sub-accumulator.
    PushGroup,
    /// End a group: blend the sub-accumulator over the parent as one layer.
    PopGroup { blend_mode: u8, opacity: f32 },
    /// Apply a non-destructive adjustment to the current accumulator (everything
    /// below it). `kind` is an `ADJ_*` code — the caller maps its
    /// `AdjustmentKind` to a code (the render crate stays decoupled from the
    /// painter tool); an unknown code is an identity no-op in the shader.
    /// `params` are the kind's ≤3 scalar params (see the WGSL `apply_adjustment`);
    /// `blend_mode`/`opacity` are the adjustment's own — the effect blends back
    /// over the base by these. W4 (ADR-0045).
    Adjustment {
        kind: u8,
        params: [f32; 3],
        blend_mode: u8,
        opacity: f32,
    },
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

/// One op as the shader sees it (16 bytes; mirrors WGSL `Op`).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuOp {
    kind: u32,
    layer_slot: u32,
    blend_mode: u32,
    opacity: f32,
}

/// Op kind discriminants — mirror the WGSL `OP_*` consts.
const OP_LAYER: u32 = 0;
const OP_PUSH_GROUP: u32 = 1;
const OP_POP_GROUP: u32 = 2;
const OP_ADJUSTMENT: u32 = 3;

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
    _pad: u32,
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
}

/// Validate group push/pop balance + depth without touching the GPU.
fn validate_op_list(ops: &[LayerOp]) -> Result<(), LayerCompositeError> {
    let mut depth: u32 = 0;
    for op in ops {
        match op {
            LayerOp::PushGroup => {
                depth += 1;
                if depth + 1 > MAX_STACK {
                    return Err(LayerCompositeError::MalformedOpList);
                }
            }
            LayerOp::PopGroup { .. } => {
                depth = depth
                    .checked_sub(1)
                    .ok_or(LayerCompositeError::MalformedOpList)?;
            }
            // Layers + adjustments are depth-neutral (an adjustment transforms
            // the current accumulator in place, like a layer blends over it).
            LayerOp::Layer { .. } | LayerOp::Adjustment { .. } => {}
        }
    }
    if depth != 0 {
        return Err(LayerCompositeError::MalformedOpList);
    }
    Ok(())
}

/// Distinct `Layer` keys referenced by `ops`.
fn distinct_layer_count(ops: &[LayerOp]) -> u32 {
    // Allocation-free (HR-3): count each `Layer` key only at its FIRST
    // occurrence. O(n²) in op count, but n is a few hundred at most and this is
    // off the GPU-bound cost — cheaper than a per-`composite()` BTreeSet alloc
    // on the documented real-time path (audit 2026-06-01 LOW).
    let mut count = 0u32;
    for (i, op) in ops.iter().enumerate() {
        if let LayerOp::Layer { key, .. } = op {
            let is_first = !ops[..i]
                .iter()
                .any(|o| matches!(o, LayerOp::Layer { key: k, .. } if k == key));
            if is_first {
                count += 1;
            }
        }
    }
    count
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
pub fn flatten_layer_ops(
    ops: &[LayerOp],
    slot_of: impl Fn(u64) -> u32,
    scratch: &mut GpuOpScratch,
) {
    scratch.ops.clear();
    scratch.adj.clear();
    for op in ops {
        let g = match op {
            LayerOp::Layer {
                key,
                blend_mode,
                opacity,
            } => GpuOp {
                kind: OP_LAYER,
                layer_slot: slot_of(*key),
                blend_mode: *blend_mode as u32,
                // Clamp to [0,1] to match the CPU reference (compositor.rs
                // clamps layer.opacity before folding into source alpha); an
                // out-of-range opacity would otherwise diverge (audit LOW).
                opacity: opacity.clamp(0.0, 1.0),
            },
            LayerOp::PushGroup => GpuOp {
                kind: OP_PUSH_GROUP,
                layer_slot: 0,
                blend_mode: 0,
                opacity: 1.0,
            },
            LayerOp::PopGroup {
                blend_mode,
                opacity,
            } => GpuOp {
                kind: OP_POP_GROUP,
                layer_slot: 0,
                blend_mode: *blend_mode as u32,
                opacity: opacity.clamp(0.0, 1.0),
            },
            LayerOp::Adjustment {
                kind,
                params,
                blend_mode,
                opacity,
            } => {
                // The op's `layer_slot` indexes the params we stash in parallel.
                let params_index = scratch.adj.len() as u32;
                scratch.adj.push(AdjParamsGpu {
                    kind: *kind as u32,
                    p0: params[0],
                    p1: params[1],
                    p2: params[2],
                });
                GpuOp {
                    kind: OP_ADJUSTMENT,
                    layer_slot: params_index,
                    blend_mode: *blend_mode as u32,
                    opacity: opacity.clamp(0.0, 1.0),
                }
            }
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
