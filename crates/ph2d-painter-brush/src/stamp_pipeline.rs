//! `StampPipeline` — GPU compute pipeline that carves stamps onto layer
//! textures. W1 unified shader rampa per [ADR-0044 §2.9](
//! ../../../docs/architecture/decisions/0044-brush-engine-gpu.md) and
//! [spec §1.8.2](../../../docs/Painter_projeto/01_brush_engine.md).
//!
//! ## Scope (T1.5)
//!
//! Build the compute pipeline + bind group layout, encode a dispatch
//! that processes an arbitrary slice of [`Stamp`]s reading from
//! `canvas_in` and writing into `canvas_out`. Workgroup is 8×8;
//! dispatch is 3D — Z carries the stamp index, X·Y cover the largest
//! stamp's bounding-box footprint in 8-pixel blocks. Threads outside
//! smaller stamps' boxes early-out.
//!
//! ## Two-texture ping-pong (T1.5)
//!
//! `canvas_in` is `texture_2d<f32>` (read, @binding(3)); `canvas_out`
//! is `texture_storage_2d<rgba8unorm, write>` (write, @binding(2)).
//! The caller is expected to bind DIFFERENT texture views to the two
//! bindings — same view in both is a wgpu validation error when one
//! is storage-write.
//!
//! For strict temporal ordering across overlapping stamps, callers
//! (e.g. [`StampScheduler`]) dispatch one stamp at a time and swap
//! the textures (A↔B) between calls. Multi-stamp single-dispatch is
//! valid only when the caller has proved the stamps in the slice
//! don't overlap pixelwise — otherwise the per-pixel order is
//! undefined (two workgroups writing the same texel race).
//!
//! Single-texture `read_write` storage on `rgba8unorm` requires
//! adapter-specific features and is gated to W5+ via ADR-0044 §2.9.
//!
//! ## Naga reflection self-test
//!
//! [`StampPipeline::stamp_struct_size_wgsl`] parses the WGSL via naga
//! and returns the byte size of the `Stamp` struct declaration in the
//! shader. Asserted in the test suite to equal `96` — the
//! compile-time Rust ABI cap (ADR-0044 §2.3 + `Stamp`'s 17
//! `offset_of!` const guards). Catches shader-side drift that would
//! otherwise only surface as silent pipeline validation errors at GPU
//! init time.

use std::num::NonZeroU64;

use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;

use crate::{MAX_STAMP_SIZE_PX, MAX_STAMPS_PER_DISPATCH, Stamp};

/// WGSL source for the brush stamp compute kernel.
///
/// Embedded at compile time. The `include_str!` path is relative to
/// this Rust source file; the shader lives in
/// `crates/ph2d-painter-brush/src/shader/stamp.wgsl`.
///
/// **Crate-private:** the shader is an implementation detail of
/// [`StampPipeline::new`]; future callers should consume the pipeline,
/// not the source. (Audit 2026-05-26 D-2.L4 — shrunk public surface.)
pub(crate) const STAMP_WGSL: &str = include_str!("shader/stamp.wgsl");

/// Per-dispatch uniform: canvas geometry + how many stamps in the buffer.
///
/// 16 bytes (`align(16)`), `Pod + Zeroable` so a `&StampGlobals` is a
/// zero-copy view into a uniform buffer write.
///
/// **Invariant:** the trailing 4-byte padding is private; the only legal
/// construction path is [`StampGlobals::new`], which guarantees zeros in
/// that slot. Matches `Stamp._pad` discipline (audit 2026-05-26 D-2.M4).
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct StampGlobals {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub stamp_count: u32,
    _pad: u32,
}

impl StampGlobals {
    #[must_use]
    pub const fn new(canvas_width: u32, canvas_height: u32, stamp_count: u32) -> Self {
        Self {
            canvas_width,
            canvas_height,
            stamp_count,
            _pad: 0,
        }
    }
}

/// Compute pipeline + bind group layout for `cs_stamp`. Build once at
/// app startup, reuse across frames.
///
/// ## Allocation profile
///
/// Construction creates one [`wgpu::ShaderModule`], one
/// [`wgpu::BindGroupLayout`], one [`wgpu::PipelineLayout`], one
/// [`wgpu::ComputePipeline`] — all expected at init time, not in the
/// HR-3 hot path. The per-frame [`Self::encode`] allocates **up to
/// ~384 KB of GPU memory** per call (4096 stamps × 96 B for the
/// storage buffer + 16 B for the globals uniform + bind group object).
/// Acceptable W1 trade-off per ADR-0044 §2.9 (W1 unified rampa →
/// W5+ specialized scheduler with recycled buffer pool); the gate
/// `painter_stamp_specialize_when_budget_pressure` (ADR-0044 §2.10)
/// trips this transition when the painter sub-budget headroom drops
/// below 15 %. T-perf W5+ promotes these to a transient ring.
///
/// ## Dispatch model
///
/// Spec §1.8.2 sketches `dispatch(stamp_count, 1, 1) + stamps[wid.x]`
/// in its pseudo-code, and `ceil(size/8)²` workgroups *per stamp* in
/// prose — those are mutually incompatible. T1.4 picks a 3D dispatch
/// `(wg_x, wg_y, N)` with `stamps[wg.z]` so smaller stamps early-out
/// via the local-id bounds check inside the shader, at the cost of
/// dispatching `max_footprint² × N` threads even when the stroke is
/// heterogeneous (a 256-px stamp next to 999 8-px stamps spawns
/// `32² × 1000` workgroups; 99.9% early-out). Acceptable W1 because
/// Day-5 strokes are uniform-sized; W5+ pre-grouping by `RenderingMode`
/// collapses heterogeneous strokes into per-mode dispatches via a
/// separate scheduler — see ADR-0044 §2.9. Spec §1.8.2 amendment is a
/// follow-up doc task.
pub struct StampPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl StampPipeline {
    /// Build the pipeline. Compiles [`STAMP_WGSL`], constructs the
    /// bind group layout with four entries:
    ///
    /// | Binding | Type                                          | Visibility |
    /// |---------|-----------------------------------------------|------------|
    /// | 0       | storage buffer (read) — `array<Stamp>`         | compute    |
    /// | 1       | uniform buffer — `StampGlobals`               | compute    |
    /// | 2       | storage texture (write) — `rgba8unorm`         | compute    |
    /// | 3       | sampled texture (read) — `texture_2d<f32>`     | compute    |
    ///
    /// `min_binding_size` is set on bindings 0 and 1 so wgpu validation
    /// catches any mismatch at dispatch time.
    ///
    /// Binding 3 (`canvas_in`) is `sample_type: Float { filterable: false }`
    /// — `textureLoad` doesn't sample, but the binding still has to
    /// declare its data shape. `filterable: false` works on every wgpu
    /// 28 backend without feature requests; `rgba8unorm` is always
    /// loadable as `Float`.
    pub fn new(gpu: &GpuContext) -> Self {
        let device = gpu.device.as_ref();

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-painter-brush::stamp"),
            source: wgpu::ShaderSource::Wgsl(STAMP_WGSL.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-painter-brush::stamp::bgl"),
            entries: &[
                // @binding(0) — stamps array, storage read.
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        // Stamp size is 96 bytes (ADR-0044 §2.3); enforce
                        // that the bound slice has at least one element.
                        min_binding_size: NonZeroU64::new(96),
                    },
                    count: None,
                },
                // @binding(1) — globals uniform.
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(16),
                    },
                    count: None,
                },
                // @binding(2) — canvas storage texture, write-only.
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // @binding(3) — canvas read texture (ping-pong source).
                // `Float { filterable: false }` matches `textureLoad` which
                // doesn't sample. `rgba8unorm` is universally loadable as
                // Float on every wgpu 28 backend without feature requests.
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-painter-brush::stamp::layout"),
            bind_group_layouts: &[&bind_group_layout],
            // wgpu 28: push constants were folded into "immediate data" with
            // a single byte budget; 0 = none (we use uniform buffer instead).
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-painter-brush::stamp::pipeline"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("cs_stamp"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    /// Access the bind group layout. **Pre-empt API surface for the
    /// W5+ ping-pong consumer** (`crates/ph2d-tool-painter` will reuse
    /// the layout when building its own bind groups for read/write
    /// canvas swapping). Currently has no in-tree caller; gated below
    /// to suppress dead-code lint without losing the surface.
    #[must_use]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Encode a single compute dispatch that applies `stamps` reading
    /// from `canvas_in` and writing into `canvas_out`. Returns
    /// immediately if `stamps` is empty or if every stamp's `size_px`
    /// is non-finite / non-positive.
    ///
    /// `canvas_dims` is the canvas size in pixels — used for the
    /// shader's bounds check (stamps that fall off the edge are
    /// clipped per-thread).
    ///
    /// ## Ping-pong contract
    ///
    /// **`canvas_in` and `canvas_out` MUST refer to DIFFERENT
    /// textures** (different `wgpu::Texture` resources, not just
    /// different views). Same texture in both bindings violates wgpu
    /// validation when one of the views is storage-write. Callers
    /// alternate A↔B between dispatches so each stamp's output is the
    /// next stamp's input — preserving Porter-Duff alpha-over ordering
    /// across overlapping stamps in a stroke.
    ///
    /// For batches of stamps known not to overlap pixelwise, a single
    /// dispatch with `stamps.len() > 1` is valid — same-dispatch
    /// per-pixel ordering is undefined only when two workgroups
    /// (different stamps) target the same texel.
    ///
    /// ## Input contracts (HR-3 + audit 2026-05-26 D-2.H1+H2)
    ///
    /// - **`stamps.len() ≤ [`MAX_STAMPS_PER_DISPATCH`] (4096)`** —
    ///   debug-asserted; in release the slice is clamped to the cap
    ///   so an out-of-spec batch never produces a workgroup count
    ///   that violates `wgpu::Limits::max_compute_workgroups_per_dimension`.
    ///   Callers (`StampScheduler`) must flush at the cap.
    /// - **`Stamp.size_px` must be finite and `> 0`** — degenerate
    ///   sizes (NaN, ±Inf, ≤ 0) are filtered out of the footprint
    ///   calculation here AND defensively early-out inside the shader.
    /// - **`Stamp.size_px ≤ [`MAX_STAMP_SIZE_PX`] (2048)`** — values
    ///   above are clamped to the cap (mirrors `PropertiesParams::
    ///   max_size_px` upper bound from spec §1.3.11).
    ///
    /// ## Dispatch shape
    ///
    /// The dispatch covers the largest stamp's `ceil(size_px / 8)²`
    /// workgroups; smaller stamps early-out in the shader via the
    /// local-id bounds check. For W1 this is acceptable (Day 5 stamps
    /// are uniform-sized); W5+ pre-grouping by `RenderingMode`
    /// collapses heterogeneous strokes into per-mode dispatches via a
    /// separate scheduler — see ADR-0044 §2.9.
    pub fn encode(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        stamps: &[Stamp],
        canvas_in: &wgpu::TextureView,
        canvas_out: &wgpu::TextureView,
        canvas_dims: (u32, u32),
    ) {
        if stamps.is_empty() {
            return;
        }
        // **R4-LH-7 fix:** debug-time same-texture guard. Caller MUST
        // bind different physical textures to `canvas_in` and
        // `canvas_out` (wgpu validation MAY catch it, but only in
        // adapter-dependent debug paths). Pointer equality on the
        // TextureView refs catches the common shell-side aliasing bug
        // — same TextureView passed twice. Release builds skip the
        // check; wgpu validation remains the last line of defense.
        debug_assert!(
            !std::ptr::eq(canvas_in, canvas_out),
            "StampPipeline::encode: canvas_in and canvas_out must be \
             DIFFERENT TextureViews — aliasing a storage-write view as a \
             sampled-read view is undefined behavior on most backends \
             (audit T1.5 round 4 R4-LH-7)"
        );

        // HR-3 pool cap: callers must flush at MAX_STAMPS_PER_DISPATCH.
        // Debug build catches the violation loudly; release clamps so
        // a bypass cannot poison the GPU pipeline with an oversized
        // dispatch (audit 2026-05-26 D-2.H2).
        debug_assert!(
            stamps.len() <= MAX_STAMPS_PER_DISPATCH,
            "StampPipeline::encode got {} stamps; HR-3 cap is {} (StampScheduler must flush)",
            stamps.len(),
            MAX_STAMPS_PER_DISPATCH,
        );
        let stamps = &stamps[..stamps.len().min(MAX_STAMPS_PER_DISPATCH)];

        // Footprint = largest finite, positive, in-spec size_px (clamped
        // to MAX_STAMP_SIZE_PX). Filters NaN/Inf/zero defensively in
        // BOTH `size_px` and `position_world` (audit 2026-05-26
        // D-2.H1 + Auditor1 follow-up): if either coordinate is
        // non-finite, the shader-side `i32(floor(world_f + 0.5))`
        // conversion is implementation-defined on out-of-range
        // (SPIR-V `OpConvertFToS` is UB; Metal saturates; D3D12 varies).
        // The shader's `world < 0 || >= canvas_w/h` bounds check is the
        // last line of defense, but catching this CPU-side keeps the
        // GPU dispatch deterministic. If every stamp is degenerate we
        // emit no GPU work.
        fn is_stamp_finite(s: &Stamp) -> bool {
            s.size_px.is_finite()
                && s.size_px > 0.0
                && s.position_world[0].is_finite()
                && s.position_world[1].is_finite()
        }
        let max_footprint = stamps
            .iter()
            .filter(|s| is_stamp_finite(s))
            .map(|s| (s.size_px.ceil() as u32).min(MAX_STAMP_SIZE_PX))
            .max()
            .unwrap_or(0);
        if max_footprint == 0 {
            return;
        }

        let device = gpu.device.as_ref();
        let queue = gpu.queue.as_ref();

        // Per-dispatch uploads. The buffer lifetimes end with `encoder`
        // submission; HR-3 hot-path budget is acceptable for T1.4 (a few
        // KB / frame). T-perf in W5+ pools these via a transient ring.
        let stamps_bytes: &[u8] = bytemuck::cast_slice(stamps);
        let stamp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-painter-brush::stamps"),
            size: stamps_bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&stamp_buffer, 0, stamps_bytes);

        let globals = StampGlobals::new(canvas_dims.0, canvas_dims.1, stamps.len() as u32);
        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-painter-brush::globals"),
            size: core::mem::size_of::<StampGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&globals_buffer, 0, bytemuck::bytes_of(&globals));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-painter-brush::stamp::bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: stamp_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(canvas_out),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(canvas_in),
                },
            ],
        });

        let wg_x = max_footprint.div_ceil(8);
        let wg_y = max_footprint.div_ceil(8);
        let wg_z = stamps.len() as u32;

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ph2d-painter-brush::stamp::pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(wg_x, wg_y, wg_z);
    }

    /// Parse [`STAMP_WGSL`] via naga and return the byte size of the
    /// `Stamp` struct declared in the shader. Used by the test suite
    /// to assert shader-side layout matches the Rust ABI (96 bytes —
    /// ADR-0044 §2.3 + `Stamp`'s 17 `offset_of!` const guards).
    ///
    /// **CPU-only.** Does not require a GPU device — safe to call in
    /// headless CI.
    #[cfg(test)]
    fn stamp_struct_size_wgsl() -> usize {
        use naga::TypeInner;
        let module =
            naga::front::wgsl::parse_str(STAMP_WGSL).expect("stamp.wgsl must parse with naga");
        for (_, ty) in module.types.iter() {
            if let TypeInner::Struct { members: _, span } = ty.inner
                && ty.name.as_deref() == Some("Stamp")
            {
                return span as usize;
            }
        }
        panic!("`Stamp` struct not found in stamp.wgsl");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamp_globals_size_is_16_bytes() {
        // Matches the WGSL `Globals` declaration (canvas_w/h, stamp_count,
        // _pad — 4× u32 = 16 bytes) and the bind group layout's
        // min_binding_size on @binding(1).
        assert_eq!(core::mem::size_of::<StampGlobals>(), 16);
        assert_eq!(core::mem::align_of::<StampGlobals>(), 16);
    }

    #[test]
    fn stamp_globals_pod_zero_copy() {
        let g = StampGlobals::new(1024, 768, 42);
        let bytes = bytemuck::bytes_of(&g);
        assert_eq!(bytes.len(), 16);
        // `_pad` is private (audit D-2.M4); read it from the byte slice
        // instead. Offset 12..16 must always be zero.
        assert_eq!(&bytes[12..16], &[0u8; 4]);
        let g2: StampGlobals = bytemuck::pod_read_unaligned(bytes);
        assert_eq!(g2.canvas_width, 1024);
        assert_eq!(g2.canvas_height, 768);
        assert_eq!(g2.stamp_count, 42);
    }

    #[test]
    fn stamp_wgsl_parses_via_naga() {
        // Must compile without errors — naga front-end catches WGSL
        // syntax issues, type mismatches, undeclared identifiers.
        let result = naga::front::wgsl::parse_str(STAMP_WGSL);
        assert!(
            result.is_ok(),
            "stamp.wgsl failed naga parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn stamp_wgsl_validates_via_naga() {
        // Validate also runs the analyzer (data flow + capability checks).
        // This is where a `vec2<f32>` at offset 68 would surface as
        // alignment violation, per the WGSL spec align(8) rule.
        //
        // `Capabilities::empty()` is stricter than the wgpu runtime
        // baseline (`Capabilities::default()` enables MULTISAMPLED_SHADING
        // + CUBE_ARRAY_TEXTURES). Running validation with `empty()`
        // catches any future shader edit that pulls in a capability
        // outside the bare-minimum WebGPU compute surface — even one
        // that wgpu would silently accept (audit 2026-05-26 D-2.F6+D-3
        // refinement).
        let module = naga::front::wgsl::parse_str(STAMP_WGSL).expect("stamp.wgsl must parse");
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        );
        let result = validator.validate(&module);
        assert!(
            result.is_ok(),
            "stamp.wgsl failed naga validation: {:?}",
            result.err()
        );
    }

    #[test]
    fn shader_stamp_struct_size_matches_rust_abi() {
        // Audit 2026-05-26 D-F9 follow-up — guards against drift between
        // the Rust `Stamp` 96-byte ABI (ADR-0044 §2.3, 17 `offset_of!`
        // const asserts in stamp.rs) and the WGSL struct declaration.
        assert_eq!(
            StampPipeline::stamp_struct_size_wgsl(),
            96,
            "WGSL Stamp struct size drifted from 96-byte ABI",
        );
    }

    #[test]
    fn shader_has_cs_stamp_entry_point() {
        let module = naga::front::wgsl::parse_str(STAMP_WGSL).expect("stamp.wgsl must parse");
        let has_entry = module
            .entry_points
            .iter()
            .any(|ep| ep.name == "cs_stamp" && ep.stage == naga::ShaderStage::Compute);
        assert!(has_entry, "stamp.wgsl missing `cs_stamp` compute entry");
    }

    #[test]
    fn shader_workgroup_size_is_8x8x1() {
        let module = naga::front::wgsl::parse_str(STAMP_WGSL).expect("stamp.wgsl must parse");
        let ep = module
            .entry_points
            .iter()
            .find(|ep| ep.name == "cs_stamp")
            .expect("cs_stamp entry");
        assert_eq!(
            ep.workgroup_size,
            [8, 8, 1],
            "workgroup_size drift — spec §1.8.2 demands 8×8×1",
        );
    }

    #[test]
    fn max_stamps_per_dispatch_matches_spec() {
        // Spec §1.4 pins the per-frame pool at 4096 stamps × 96B = 384 KB.
        // If the const drifts, downstream HR-3 reasoning breaks.
        assert_eq!(MAX_STAMPS_PER_DISPATCH, 4096);
    }

    #[test]
    fn max_stamp_size_px_matches_spec() {
        // Spec §1.3.11: PropertiesParams::max_size_px range 1..=2048.
        assert_eq!(MAX_STAMP_SIZE_PX, 2048);
    }

    #[test]
    fn shader_oklab_coefficients_bit_identical_with_rust() {
        // Audit 2026-05-26 D-3.H7 — prove zero-ULP drift between the OKLab
        // matrix coefficients embedded as literals in stamp.wgsl and the
        // Rust path in `ph2d_color::OklabColor::to_linear`. HR-5
        // (determinism where promised) requires CPU/GPU paths to agree
        // bit-exactly; without this gate, a future shader edit that
        // trims a digit (e.g. `0.3963377774` → `0.39633778`) would slip
        // through silently.
        //
        // Method:
        // 1. Assert each WGSL literal string appears verbatim in STAMP_WGSL.
        // 2. Parse each WGSL literal via Rust `f32::FromStr` (IEEE 754
        //    round-to-nearest-even per Rust std::str docs, matching naga's
        //    own parse path in `naga::front::wgsl`).
        // 3. Assert parsed `to_bits()` equals the Rust literal's `to_bits()`.
        //
        // 15 coefficients = 9 forward (CPU `OklabColor::from_linear`) + 6
        // inverse (`OklabColor::to_linear` AND `oklab_to_linear_srgb`
        // shader-side). The shader only uses inverse; CPU forward is for
        // tests + future T-color full encode paths.
        let inverse_pairs: &[(&str, f32)] = &[
            // OKLab → LMS (3 + 3 + 3 = 9 used in shader inverse + CPU inverse)
            ("0.3963377774", 0.396_337_78),
            ("0.2158037573", 0.215_803_76),
            ("0.1055613458", 0.105_561_346),
            ("0.0638541728", 0.063_854_17),
            ("0.0894841775", 0.089_484_18),
            ("1.2914855480", 1.291_485_5),
            // LMS³ → linear sRGB (9 coefficients, with signs baked into matrix)
            ("4.0767416621", 4.076_741_7),
            ("3.3077115913", 3.307_711_6),
            ("0.2309699292", 0.230_969_94),
            ("1.2684380046", 1.268_438),
            ("2.6097574011", 2.609_757_4),
            ("0.3413193965", 0.341_319_38),
            ("0.0041960863", 0.004_196_086_3),
            ("0.7034186147", 0.703_418_6),
            ("1.7076147010", 1.707_614_7),
        ];

        for (wgsl_lit, rust_lit) in inverse_pairs {
            assert!(
                STAMP_WGSL.contains(wgsl_lit),
                "WGSL literal `{wgsl_lit}` not found verbatim in stamp.wgsl — \
                 shader-side OKLab coefficient drift",
            );
            let parsed: f32 = wgsl_lit
                .parse()
                .unwrap_or_else(|e| panic!("WGSL literal `{wgsl_lit}` failed f32 parse: {e:?}"));
            assert_eq!(
                parsed.to_bits(),
                rust_lit.to_bits(),
                "OKLab matrix coefficient drift: WGSL `{wgsl_lit}` parses to bits \
                 0x{:08x}, Rust literal `{rust_lit}` is bits 0x{:08x}",
                parsed.to_bits(),
                rust_lit.to_bits(),
            );
        }
    }
}
