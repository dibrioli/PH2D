//! `StampPipeline` — GPU compute pipeline that carves stamps onto layer
//! textures. W1 unified shader rampa per [ADR-0044 §2.9](
//! ../../../docs/architecture/decisions/0044-brush-engine-gpu.md) and
//! [spec §1.8.2](../../../docs/Painter_projeto/01_brush_engine.md).
//!
//! ## Scope (T1.4)
//!
//! Build the compute pipeline + bind group layout, encode a dispatch
//! that processes an arbitrary slice of [`Stamp`]s. Workgroup is 8×8;
//! dispatch is 3D — Z carries the stamp index, X·Y cover the largest
//! stamp's bounding-box footprint in 8-pixel blocks. Threads outside
//! smaller stamps' boxes early-out.
//!
//! ## Limitation registered as W5+ follow-up
//!
//! `canvas_out` is bound write-only (`texture_storage_2d<rgba8unorm,
//! write>`) — portable across every wgpu 28 adapter. Without read-side
//! access on the same texture, stamps that overlap in the SAME dispatch
//! produce undefined ordering. Single-stamp dispatches (Day 5 smoke)
//! are unaffected. Multi-stamp alpha-over correctness lands in T1.5
//! when `StampScheduler` emits stamps with temporal ordering and the
//! pipeline gains a per-stamp ping-pong path (ADR-0044 §2.9
//! W5+-specialized rampa). The public API of [`StampPipeline::encode`]
//! does not change at that transition.
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
/// **Allocations:** construction creates one [`wgpu::ShaderModule`],
/// one [`wgpu::BindGroupLayout`], one [`wgpu::PipelineLayout`], one
/// [`wgpu::ComputePipeline`] — all expected at init time, not in the
/// HR-3 hot path. The per-frame [`Self::encode`] is `&self` and
/// allocates only the per-dispatch upload buffers (stamps + globals),
/// which are throwaway-scoped to the encoder lifetime; T-perf in W5+
/// can elevate them to a recycled pool if `painter_stamp_specialize_
/// when_budget_pressure` flips hard.
pub struct StampPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl StampPipeline {
    /// Build the pipeline. Compiles [`STAMP_WGSL`], constructs the
    /// bind group layout with three entries:
    ///
    /// | Binding | Type                                         | Visibility |
    /// |---------|----------------------------------------------|------------|
    /// | 0       | storage buffer (read) — `array<Stamp>`        | compute    |
    /// | 1       | uniform buffer — `StampGlobals`              | compute    |
    /// | 2       | storage texture (write) — `rgba8unorm`        | compute    |
    ///
    /// `min_binding_size` is set on bindings 0 and 1 so wgpu validation
    /// catches any mismatch at dispatch time.
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

    /// Encode a single compute dispatch that applies `stamps` onto
    /// `canvas_out`. Returns immediately if `stamps` is empty or if
    /// every stamp's `size_px` is non-finite / non-positive.
    ///
    /// `canvas_dims` is the canvas size in pixels — used for the
    /// shader's bounds check (stamps that fall off the edge are
    /// clipped per-thread).
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
        canvas_out: &wgpu::TextureView,
        canvas_dims: (u32, u32),
    ) {
        if stamps.is_empty() {
            return;
        }

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
        // to MAX_STAMP_SIZE_PX). Filters NaN/Inf/zero defensively;
        // if every stamp is degenerate we emit no GPU work (audit
        // 2026-05-26 D-2.H1).
        let max_footprint = stamps
            .iter()
            .filter(|s| s.size_px.is_finite() && s.size_px > 0.0)
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
        // Capabilities is intentionally empty so the test catches a
        // future shader edit that pulls in a capability the wgpu
        // baseline device doesn't request (audit 2026-05-26 D-2.M6).
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
}
