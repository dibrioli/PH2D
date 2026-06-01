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

pub(crate) const LAYER_COMPOSITE_WGSL: &str = include_str!("shaders/layer_composite.wgsl");

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
}

impl LayerCompositor {
    /// Build the compute pipeline. Cheap — no GPU textures until the first
    /// [`Self::composite`].
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render layer_composite shader"),
                source: wgpu::ShaderSource::Wgsl(LAYER_COMPOSITE_WGSL.into()),
            });

        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render layer_composite bgl"),
                entries: &[
                    // 0: ops storage (read)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                core::mem::size_of::<GpuOp>() as u64
                            ),
                        },
                        count: None,
                    },
                    // 1: globals uniform
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                core::mem::size_of::<GpuGlobals>() as u64,
                            ),
                        },
                        count: None,
                    },
                    // 2: layer texture array (sampled via textureLoad — non-filterable)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // 3: output storage texture (write)
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    // 4: sRGB→linear decode LUT (256 × f32 storage, read)
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(SRGB_LUT_LEN as u64 * 4),
                        },
                        count: None,
                    },
                ],
            });

        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-render layer_composite layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });

        let make_pipeline = |entry: &str, label: &str| {
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(label),
                    layout: Some(&layout),
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                })
        };
        let pipeline_flat = make_pipeline("cs_flat", "ph2d-render layer_composite flat");
        let pipeline_grouped = make_pipeline("cs_grouped", "ph2d-render layer_composite grouped");

        let globals_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render layer_composite globals"),
            size: core::mem::size_of::<GpuGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let lut = build_srgb_lut();
        let srgb_lut_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render layer_composite srgb lut"),
            size: (SRGB_LUT_LEN * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue
            .write_buffer(&srgb_lut_buffer, 0, bytemuck::cast_slice(&lut));

        Self {
            pipeline_flat,
            pipeline_grouped,
            bgl,
            device_max_layers: gpu.device.limits().max_texture_array_layers,
            array: None,
            out: None,
            cache: BTreeMap::new(),
            clock: 0,
            scratch_ops: GpuOpScratch::new(),
            op_buffer: None,
            globals_buffer,
            srgb_lut_buffer,
        }
    }

    /// Cache cap for `width × height` on this device: the smaller of the
    /// per-budget cap and the device's `max_texture_array_layers`.
    #[must_use]
    pub fn cache_cap(&self, width: u32, height: u32) -> u32 {
        max_layers_for_budget(width, height, LAYER_CACHE_BUDGET_BYTES).min(self.device_max_layers)
    }

    /// Number of cached layer slices currently resident.
    #[must_use]
    pub fn cached_len(&self) -> usize {
        self.cache.len()
    }

    /// The output texture of the last [`Self::composite`] (region-sized,
    /// straight sRGB8 `rgba8unorm`). The shell blits this onto the sprite;
    /// `None` before the first composite.
    #[must_use]
    pub fn output_texture(&self) -> Option<&wgpu::Texture> {
        self.out.as_ref().map(|o| &o.texture)
    }

    /// Composite `ops` into the output texture, covering `region` of the
    /// `canvas_w × canvas_h` canvas. Uploads only layers whose version changed
    /// since the last call. Encodes + submits one compute dispatch.
    pub fn composite(
        &mut self,
        gpu: &GpuContext,
        ops: &[LayerOp],
        src: &impl LayerPixelProvider,
        canvas_w: u32,
        canvas_h: u32,
        region: Region,
    ) -> Result<(), LayerCompositeError> {
        if canvas_w == 0
            || canvas_h == 0
            || canvas_w > gpu.device.limits().max_texture_dimension_2d
            || canvas_h > gpu.device.limits().max_texture_dimension_2d
        {
            return Err(LayerCompositeError::InvalidCanvas {
                width: canvas_w,
                height: canvas_h,
            });
        }
        validate_op_list(ops)?;

        self.clock += 1;
        let epoch = self.clock;

        // Distinct layer keys referenced this frame, in first-seen order.
        let cap = self.cache_cap(canvas_w, canvas_h).max(1);
        self.ensure_array(gpu, canvas_w, canvas_h, ops, cap)?;

        // Resolve every referenced layer to a slice (uploading dirty pixels),
        // then flatten ops into the reusable GPU scratch.
        for op in ops {
            if let LayerOp::Layer { key, .. } = op {
                self.ensure_slice(gpu, *key, src, canvas_w, canvas_h, epoch, cap)?;
            }
        }
        let cache = &self.cache;
        flatten_layer_ops(
            ops,
            |k| cache.get(&k).map_or(0, |c| c.slice),
            &mut self.scratch_ops,
        );

        let region = region.clamped(canvas_w, canvas_h);
        if region.w == 0 || region.h == 0 {
            return Ok(()); // nothing to recomposite
        }
        self.ensure_out(gpu, region.w, region.h);
        self.upload_op_buffer(gpu);
        self.write_globals(gpu, canvas_w, canvas_h, region);
        let has_groups = ops.iter().any(|o| matches!(o, LayerOp::PushGroup));
        self.dispatch(gpu, region, has_groups);
        Ok(())
    }

    /// Read the output texture back to a region-sized straight-sRGB8 `Vec<u8>`.
    /// Test/verification path only (blocks on `device.poll`); the shell blits
    /// the texture directly. Returns `None` before the first composite.
    #[must_use]
    pub fn read_output(&self, gpu: &GpuContext) -> Option<Vec<u8>> {
        let out = self.out.as_ref()?;
        Some(readback_rgba8(gpu, &out.texture, out.width, out.height))
    }

    // ── internals ────────────────────────────────────────────────────────

    /// Ensure the texture array exists with the right dims + enough capacity
    /// for the distinct layers in `ops`.
    fn ensure_array(
        &mut self,
        gpu: &GpuContext,
        width: u32,
        height: u32,
        ops: &[LayerOp],
        cap: u32,
    ) -> Result<(), LayerCompositeError> {
        let distinct = distinct_layer_count(ops);
        if distinct > cap {
            return Err(LayerCompositeError::TooManyLayers {
                requested: distinct,
                cap,
            });
        }
        let needs_rebuild = match &self.array {
            Some(a) => a.width != width || a.height != height || a.capacity < distinct.max(1),
            None => true,
        };
        if !needs_rebuild {
            return Ok(());
        }
        // Grow capacity to the cap (bounded by distinct need) so subsequent
        // frames rarely rebuild; dims change → cache invalidated.
        let capacity = distinct
            .max(1)
            .max(self.array.as_ref().map_or(0, |a| a.capacity))
            .min(cap);
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render layer_composite array"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: capacity,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("ph2d-render layer_composite array view"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        self.array = Some(LayerArray {
            texture,
            view,
            width,
            height,
            capacity,
        });
        // Dims/capacity changed → existing slice assignments are invalid.
        self.cache.clear();
        Ok(())
    }

    /// Resolve `key` to a slice, uploading its pixels if newly assigned or its
    /// version changed. Evicts the LRU slice if the array is full.
    #[allow(clippy::too_many_arguments)]
    fn ensure_slice(
        &mut self,
        gpu: &GpuContext,
        key: u64,
        src: &impl LayerPixelProvider,
        width: u32,
        height: u32,
        epoch: u64,
        cap: u32,
    ) -> Result<(), LayerCompositeError> {
        let pixels = src
            .layer_pixels(key)
            .filter(|p| p.rgba8.len() == (width as usize) * (height as usize) * 4)
            .ok_or(LayerCompositeError::MissingOrMalformedLayer { key })?;

        if let Some(existing) = self.cache.get_mut(&key) {
            existing.last_used = epoch;
            if existing.version == pixels.version {
                return Ok(()); // clean — already resident
            }
            let slice = existing.slice;
            existing.version = pixels.version;
            self.upload_slice(gpu, slice, width, height, pixels.rgba8);
            return Ok(());
        }

        // New key — find a free slice or evict the LRU.
        let array_cap = self.array.as_ref().map_or(0, |a| a.capacity).min(cap);
        let slice = self.alloc_slice(array_cap, epoch)?;
        self.upload_slice(gpu, slice, width, height, pixels.rgba8);
        self.cache.insert(
            key,
            CachedSlice {
                slice,
                version: pixels.version,
                last_used: epoch,
            },
        );
        Ok(())
    }

    /// Pick a slice index for a new key: the first unused index, else evict the
    /// least-recently-used slice not touched this `epoch`.
    fn alloc_slice(&mut self, array_cap: u32, epoch: u64) -> Result<u32, LayerCompositeError> {
        // Allocation-free (HR-3): scan for the first slice no cached layer
        // holds. O(array_cap × cache_len), array_cap ≤ HARD_CAP_LAYERS, and
        // this runs only on a NEW-key resolution (not the steady-state path)
        // — cheaper than collecting a BTreeSet per new key (audit LOW).
        for s in 0..array_cap {
            if !self.cache.values().any(|c| c.slice == s) {
                return Ok(s);
            }
        }
        // Full — evict the LRU slice whose layer was not used this frame.
        let victim = self
            .cache
            .iter()
            .filter(|(_, c)| c.last_used < epoch)
            .min_by_key(|(_, c)| c.last_used)
            .map(|(k, c)| (*k, c.slice));
        match victim {
            Some((k, slice)) => {
                self.cache.remove(&k);
                Ok(slice)
            }
            // Every slice is live this frame and we still need one more — the
            // caller's distinct count exceeded capacity (should have been
            // caught by ensure_array, but guard defensively).
            None => Err(LayerCompositeError::TooManyLayers {
                requested: array_cap + 1,
                cap: array_cap,
            }),
        }
    }

    /// Upload one canvas-sized straight-sRGB8 layer into array `slice`.
    fn upload_slice(&self, gpu: &GpuContext, slice: u32, width: u32, height: u32, rgba8: &[u8]) {
        let Some(array) = &self.array else { return };
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &array.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: slice,
                },
                aspect: wgpu::TextureAspect::All,
            },
            rgba8,
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

    /// Ensure the output texture is sized `w × h`.
    fn ensure_out(&mut self, gpu: &GpuContext, w: u32, h: u32) {
        if let Some(o) = &self.out
            && o.width == w
            && o.height == h
        {
            return;
        }
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render layer_composite out"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.out = Some(OutTex {
            texture,
            view,
            width: w,
            height: h,
        });
    }

    /// (Re)upload the flattened op-list into the persistent storage buffer,
    /// growing it only when the op count exceeds the current capacity.
    fn upload_op_buffer(&mut self, gpu: &GpuContext) {
        let bytes: &[u8] = bytemuck::cast_slice(&self.scratch_ops.0);
        let needed = bytes.len().max(core::mem::size_of::<GpuOp>()) as u64;
        let grow = match &self.op_buffer {
            Some((_, cap)) => *cap < needed,
            None => true,
        };
        if grow {
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-render layer_composite ops"),
                size: needed,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.op_buffer = Some((buffer, needed));
        }
        if let Some((buffer, _)) = &self.op_buffer {
            gpu.queue.write_buffer(buffer, 0, bytes);
        }
    }

    fn write_globals(&self, gpu: &GpuContext, canvas_w: u32, canvas_h: u32, region: Region) {
        let g = GpuGlobals {
            canvas_width: canvas_w,
            canvas_height: canvas_h,
            region_x: region.x,
            region_y: region.y,
            region_w: region.w,
            region_h: region.h,
            op_count: self.scratch_ops.len() as u32,
            _pad: 0,
        };
        gpu.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&g));
    }

    fn dispatch(&self, gpu: &GpuContext, region: Region, has_groups: bool) {
        let (Some(array), Some(out), Some((op_buffer, _))) =
            (&self.array, &self.out, &self.op_buffer)
        else {
            return;
        };
        let pipeline = if has_groups {
            &self.pipeline_grouped
        } else {
            &self.pipeline_flat
        };
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render layer_composite bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: op_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&array.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&out.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.srgb_lut_buffer.as_entire_binding(),
                },
            ],
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render layer_composite encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-render layer_composite pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                region.w.div_ceil(WORKGROUP_EDGE),
                region.h.div_ceil(WORKGROUP_EDGE),
                1,
            );
        }
        gpu.queue.submit([encoder.finish()]);
    }
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
            LayerOp::Layer { .. } => {}
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
pub struct GpuOpScratch(Vec<GpuOp>);

impl GpuOpScratch {
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Backing capacity — exposed for the no-alloc gate to assert stability.
    #[doc(hidden)]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
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
    let out = &mut scratch.0;
    out.clear();
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
        };
        out.push(g);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_layers_for_budget_4k_and_degenerate() {
        // 4K RGBA8 = 33,177,600 B/slice; 512 MiB / that = 16 slices.
        assert_eq!(
            max_layers_for_budget(3840, 2160, LAYER_CACHE_BUDGET_BYTES),
            16
        );
        // Zero-area canvas → 0 (no divide-by-zero).
        assert_eq!(max_layers_for_budget(0, 0, LAYER_CACHE_BUDGET_BYTES), 0);
        // Tiny canvas is bounded by HARD_CAP_LAYERS, not the byte budget.
        assert_eq!(
            max_layers_for_budget(1, 1, LAYER_CACHE_BUDGET_BYTES),
            HARD_CAP_LAYERS
        );
    }

    #[test]
    fn gpu_pod_sizes_match_wgsl() {
        assert_eq!(core::mem::size_of::<GpuOp>(), 16);
        assert_eq!(core::mem::size_of::<GpuGlobals>(), 32);
    }

    /// The decode LUT the shader binds must equal the canonical
    /// `srgb_to_linear_byte` for every byte — this is what makes the GPU decode
    /// bit-identical to the CPU `compositor::decode` (the table replaced the
    /// in-shader `pow`, so this gate, not the textual one, pins decode parity).
    #[test]
    fn srgb_lut_matches_cpu_transfer() {
        let lut = build_srgb_lut();
        assert_eq!(lut.len(), 256);
        for b in 0..=255u8 {
            assert_eq!(
                lut[b as usize].to_bits(),
                srgb_to_linear_byte(b).to_bits(),
                "LUT[{b}] drifted from srgb_to_linear_byte",
            );
        }
        // The shader recovers the byte index via `round(raw * 255)`; assert the
        // WGSL still indexes the table that way (guards a refactor that breaks
        // the unorm→byte recovery).
        assert!(LAYER_COMPOSITE_WGSL.contains("srgb_lut[u32(raw.r * 255.0 + 0.5)]"));
    }

    #[test]
    fn validate_op_list_balance_and_depth() {
        let layer = LayerOp::Layer {
            key: 1,
            blend_mode: 0,
            opacity: 1.0,
        };
        // Balanced.
        assert!(
            validate_op_list(&[
                layer,
                LayerOp::PushGroup,
                layer,
                LayerOp::PopGroup {
                    blend_mode: 0,
                    opacity: 1.0
                },
            ])
            .is_ok()
        );
        // Unbalanced (push without pop).
        assert_eq!(
            validate_op_list(&[LayerOp::PushGroup]),
            Err(LayerCompositeError::MalformedOpList)
        );
        // Pop without push.
        assert_eq!(
            validate_op_list(&[LayerOp::PopGroup {
                blend_mode: 0,
                opacity: 1.0
            }]),
            Err(LayerCompositeError::MalformedOpList)
        );
        // Too deep (> MAX_GROUP_DEPTH nested pushes).
        let mut deep = Vec::new();
        for _ in 0..MAX_STACK {
            deep.push(LayerOp::PushGroup);
        }
        assert_eq!(
            validate_op_list(&deep),
            Err(LayerCompositeError::MalformedOpList)
        );
    }

    #[test]
    fn flatten_layer_ops_resolves_slots_and_reuses_scratch() {
        let slot_of = |k: u64| match k {
            9 => 1,
            7 => 3,
            _ => 0,
        };
        let ops = vec![
            LayerOp::Layer {
                key: 9,
                blend_mode: 1,
                opacity: 0.5,
            },
            LayerOp::PushGroup,
            LayerOp::Layer {
                key: 7,
                blend_mode: 11,
                opacity: 1.0,
            },
            LayerOp::PopGroup {
                blend_mode: 6,
                opacity: 0.8,
            },
        ];
        let mut scratch = GpuOpScratch::new();
        flatten_layer_ops(&ops, slot_of, &mut scratch);
        assert_eq!(scratch.0.len(), 4);
        assert_eq!(scratch.0[0].kind, OP_LAYER);
        assert_eq!(scratch.0[0].layer_slot, 1); // key 9 → slice 1
        assert_eq!(scratch.0[0].blend_mode, 1);
        assert_eq!(scratch.0[1].kind, OP_PUSH_GROUP);
        assert_eq!(scratch.0[2].layer_slot, 3); // key 7 → slice 3
        assert_eq!(scratch.0[3].kind, OP_POP_GROUP);
        assert_eq!(scratch.0[3].blend_mode, 6);

        // Re-flattening into the same scratch must not grow capacity (HR-3).
        let cap = scratch.capacity();
        flatten_layer_ops(&ops, slot_of, &mut scratch);
        assert_eq!(scratch.capacity(), cap);
    }

    #[test]
    fn distinct_layer_count_dedupes() {
        let ops = vec![
            LayerOp::Layer {
                key: 1,
                blend_mode: 0,
                opacity: 1.0,
            },
            LayerOp::Layer {
                key: 1,
                blend_mode: 0,
                opacity: 1.0,
            },
            LayerOp::Layer {
                key: 2,
                blend_mode: 0,
                opacity: 1.0,
            },
            LayerOp::PushGroup,
        ];
        assert_eq!(distinct_layer_count(&ops), 2);
    }

    // ── WGSL reflection / parity gates (CPU-only, no GPU) ─────────────────

    #[test]
    fn layer_composite_wgsl_parses_via_naga() {
        let r = naga::front::wgsl::parse_str(LAYER_COMPOSITE_WGSL);
        assert!(
            r.is_ok(),
            "layer_composite.wgsl failed naga parse: {:?}",
            r.err()
        );
    }

    #[test]
    fn layer_composite_wgsl_validates_via_naga() {
        let module = naga::front::wgsl::parse_str(LAYER_COMPOSITE_WGSL).expect("must parse");
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        );
        let r = validator.validate(&module);
        assert!(
            r.is_ok(),
            "layer_composite.wgsl failed naga validation: {:?}",
            r.err()
        );
    }

    #[test]
    fn shader_workgroup_size_is_8x8x1() {
        let module = naga::front::wgsl::parse_str(LAYER_COMPOSITE_WGSL).expect("must parse");
        // Both entry points (flat + grouped) must use the 8×8 tiling.
        for name in ["cs_flat", "cs_grouped"] {
            let ep = module
                .entry_points
                .iter()
                .find(|ep| ep.name == name)
                .unwrap_or_else(|| panic!("{name} entry point"));
            assert_eq!(ep.workgroup_size, [WORKGROUP_EDGE, WORKGROUP_EDGE, 1]);
        }
    }

    fn wgsl_struct_span(name: &str) -> usize {
        use naga::TypeInner;
        let module = naga::front::wgsl::parse_str(LAYER_COMPOSITE_WGSL).expect("must parse");
        for (_, ty) in module.types.iter() {
            if let TypeInner::Struct { span, .. } = ty.inner
                && ty.name.as_deref() == Some(name)
            {
                return span as usize;
            }
        }
        panic!("`{name}` struct not found in layer_composite.wgsl");
    }

    #[test]
    fn shader_struct_sizes_match_rust_abi() {
        assert_eq!(wgsl_struct_span("Op"), core::mem::size_of::<GpuOp>());
        assert_eq!(
            wgsl_struct_span("Globals"),
            core::mem::size_of::<GpuGlobals>()
        );
    }

    #[test]
    fn shader_op_kind_discriminants_match_wgsl() {
        // The Rust `OP_*` and WGSL `OP_*` must agree, and the WGSL `switch`
        // arms (`case 0u/1u/2u`) dispatch on them.
        assert!(LAYER_COMPOSITE_WGSL.contains("const OP_LAYER: u32 = 0u;"));
        assert!(LAYER_COMPOSITE_WGSL.contains("const OP_PUSH_GROUP: u32 = 1u;"));
        assert!(LAYER_COMPOSITE_WGSL.contains("const OP_POP_GROUP: u32 = 2u;"));
        assert_eq!(OP_LAYER, 0);
        assert_eq!(OP_PUSH_GROUP, 1);
        assert_eq!(OP_POP_GROUP, 2);
    }

    /// Pin every numeric literal the GPU blend math shares with the Rust
    /// source-of-truth (`ph2d_painter_brush::blend` + `ph2d_color::srgb`) to
    /// zero-ULP agreement — the same discipline as
    /// `shader_oklab_coefficients_bit_identical_with_rust`. Each literal must
    /// (a) appear verbatim in the WGSL and (b) parse to the exact f32 bits of
    /// the Rust constant it mirrors. Catches a drift like `0.59 → 0.587`
    /// (textual) or a transfer-function typo (bit-level).
    #[test]
    fn shader_blend_modes_bit_identical_with_rust() {
        // (wgsl literal that must appear verbatim, expected f32 value)
        let pairs: &[(&str, f32)] = &[
            // sRGB ENCODE transfer (ph2d_color::srgb::linear_to_srgb_byte) —
            // the decode direction is the bit-exact LUT (gate below), so only
            // the encode literals live in the WGSL now.
            ("12.92", 12.92),
            ("0.055", 0.055),
            ("1.055", 1.055),
            ("2.4", 2.4),
            ("0.0031308", 0.003_130_8),
            // W3C HSL luminosity coefficients (blend::lum).
            ("0.3 * c.r", 0.3),
            ("0.59 * c.g", 0.59),
            ("0.11 * c.b", 0.11),
            // W3C soft-light cubic approximation (blend::soft_light).
            ("16.0 * cb", 16.0),
            ("12.0)", 12.0),
            // Exclusion / linear-burn / linear-light shared constants.
            ("2.0 * cb * cs", 2.0),
            // f32::EPSILON guard threshold (blend::apply ao <= EPSILON).
            ("1.1920929e-7", f32::EPSILON),
        ];
        for (lit, expected) in pairs {
            // Verbatim presence.
            assert!(
                LAYER_COMPOSITE_WGSL.contains(lit),
                "literal `{lit}` not found verbatim in layer_composite.wgsl"
            );
            // Bit-exact parse of the leading numeric token of the literal.
            let token: String = lit
                .chars()
                .take_while(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '-' | '+'))
                .collect();
            let parsed: f32 = token
                .parse()
                .unwrap_or_else(|e| panic!("WGSL literal token `{token}` failed f32 parse: {e:?}"));
            assert_eq!(
                parsed.to_bits(),
                expected.to_bits(),
                "blend literal drift: WGSL `{token}` is 0x{:08x}, Rust `{expected}` is 0x{:08x}",
                parsed.to_bits(),
                expected.to_bits(),
            );
        }
    }

    /// The four HSL (non-separable) modes + the two compositing specials must
    /// be dispatched by the right discriminants — guards a copy-paste that
    /// silently routes a mode to the wrong arm.
    #[test]
    fn shader_dispatches_hsl_and_specials_by_canonical_discriminant() {
        // HSL group = 16..=19 (Hue/Saturation/Color/Luminosity).
        for code in ["case 16u", "case 17u", "case 18u", "case 19u"] {
            assert!(
                LAYER_COMPOSITE_WGSL.contains(code),
                "missing HSL arm {code}"
            );
        }
        // Specials: Behind=20, Clear=21 (handled before the blend switch).
        assert!(
            LAYER_COMPOSITE_WGSL.contains("if mode == 20u"),
            "missing Behind special"
        );
        assert!(
            LAYER_COMPOSITE_WGSL.contains("if mode == 21u"),
            "missing Clear special"
        );
        // is_hsl must match exactly the 16..=19 set.
        assert!(
            LAYER_COMPOSITE_WGSL
                .contains("mode == 16u || mode == 17u || mode == 18u || mode == 19u")
        );
    }
}
