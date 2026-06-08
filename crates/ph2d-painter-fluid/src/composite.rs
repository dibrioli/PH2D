//! GPU wet-field composite (ADR-0049 W15.3) — the per-frame K–M glaze on the GPU,
//! removing the pigment readback stall. Band-for-band mirror of
//! [`ph2d_painter_brush::wet_composite::composite_wet_field_cpu`] (the parity
//! ground truth; the gate proves they agree on a real device).
//!
//! The compositor is built once per device. Inputs each composite: the GPU-resident
//! low-res pigment (`vec4` mass), the canvas-res backdrop (RGBA8), and the amortised
//! brush coeffs (computed once on the CPU via
//! [`ph2d_painter_brush::wet_composite::prepare_wet_composite`] +
//! [`ph2d_painter_brush::pigment_mix::spectral_basis`]). Output is canvas-res RGBA8
//! — to a storage buffer (the readback / first-milestone path here) or, in the
//! shell integration, a preview texture (zero readback).

use ph2d_painter_brush::pigment_mix::{SPECTRAL_BANDS, spectral_basis};
use ph2d_painter_brush::wet_composite::{WET_COMPOSITE_SS, WetCompositeBrush, composite_canvas_region};

/// The GPU composite shader source (mirror of the CPU `wet_composite`). Embedded so
/// a dev-test validates it through naga before any GPU init.
pub const COMPOSITE_WGSL: &str = include_str!("shader/composite.wgsl");

/// The WGSL `U` uniform, byte-for-byte (64 B). `#[repr(C)]` Pod for a zero-copy
/// `write_buffer` (HR-3). std140-compatible: `pcol` (vec4) lands at offset 48.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuU {
    cw: u32,
    ch: u32,
    gw: u32,
    gh: u32,
    inv: f32,
    color_sum: f32,
    coverage_k: f32,
    _pad0: f32,
    origin_x: u32,
    origin_y: u32,
    end_x: u32,
    end_y: u32,
    pcol: [f32; 4],
}

/// The WGSL `Coeffs` storage struct, byte-for-byte (1072 B). Flattened spectral
/// basis (`base[7][24]` + `m[3][24]`) + prepared brush side (`ks[24]` + `err`).
/// std430: `err` (vec4) lands 16-aligned at offset 1056.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuCoeffs {
    base: [f32; 168], // 7 * 24
    m: [f32; 72],     // 3 * 24
    ks: [f32; 24],
    err: [f32; 4], // xyz used
}

impl GpuCoeffs {
    /// Pack the constant basis + the amortised brush coeffs for the shader.
    fn build(brush: &WetCompositeBrush) -> Self {
        // Pinned: the shader hard-codes NB=24 (and the flatten strides below) + SS=2.
        assert_eq!(SPECTRAL_BANDS, 24, "composite.wgsl assumes 24 spectral bands");
        assert_eq!(WET_COMPOSITE_SS, 2, "composite.wgsl hard-codes SS=2u (supersampling)");
        let (base, m) = spectral_basis();
        let mut out = Self {
            base: [0.0; 168],
            m: [0.0; 72],
            ks: brush.prepared.ks(),
            err: [0.0; 4],
        };
        for (k, row) in base.iter().enumerate() {
            out.base[k * 24..k * 24 + 24].copy_from_slice(row);
        }
        for (c, row) in m.iter().enumerate() {
            out.m[c * 24..c * 24 + 24].copy_from_slice(row);
        }
        let e = brush.prepared.err();
        out.err = [e[0], e[1], e[2], 0.0];
        out
    }
}

/// Persistent per-stroke GPU state for the fast path — buffers + bind group reused
/// across frames so the hot loop NEVER allocates or re-uploads the canvas. The
/// backdrop + coeffs are constant for a stroke (uploaded once in
/// [`FluidCompositor::begin_stroke`]); only the 64-byte `params` (the region) is
/// written per frame.
struct CompositeState {
    cw: u32,
    ch: u32,
    gw: u32,
    gh: u32,
    scale: u32,
    coverage_k: f32,
    pcol: [f32; 3],
    color_sum: f32,
    params: wgpu::Buffer,
    coeffs: wgpu::Buffer,
    out: wgpu::Buffer,
    backdrop: wgpu::Buffer,
    staging: wgpu::Buffer,
    bind: wgpu::BindGroup,
}

/// The live GPU compositor: one compute pipeline + bind-group layout (built once per
/// device) + persistent per-stroke buffers ([`CompositeState`]) for the hot loop.
/// The one-shot `composite_*` methods (per-call buffers) are the unit-test / CPU
/// convenience; the shell drives [`Self::begin_stroke`] + [`Self::composite_frame`].
pub struct FluidCompositor {
    pipeline: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    state: Option<CompositeState>,
}

impl FluidCompositor {
    /// Build the pipeline + bind-group layout on `device`.
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-painter-fluid composite"),
            source: wgpu::ShaderSource::Wgsl(COMPOSITE_WGSL.into()),
        });
        let storage = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-painter-fluid composite bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage(1, true),  // pig_in (read)
                storage(2, true),  // backdrop (read)
                storage(3, false), // out_buf (read_write)
                storage(4, true),  // coeffs (read)
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-painter-fluid composite layout"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-painter-fluid composite pipeline"),
            layout: Some(&layout),
            module: &module,
            entry_point: Some("cs_composite"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Self { pipeline, bgl, state: None }
    }

    /// **Fast-path stroke setup (call once per stroke / when the backdrop or brush
    /// changes).** (Re)allocates the persistent canvas buffers only on a size change,
    /// uploads the backdrop + the amortised brush coeffs ONCE, and rebuilds the bind
    /// group against the solver's resident `pigment_buf`. After this, [`Self::composite_frame`]
    /// runs each frame writing only the 64-byte region uniform — no per-frame
    /// allocation, no canvas re-upload (the W15.3 hot-loop perf fix).
    #[allow(clippy::too_many_arguments)]
    pub fn begin_stroke(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
        pigment_buf: &wgpu::Buffer,
        backdrop_rgba: &[u8],
        brush: &WetCompositeBrush,
    ) {
        let canvas_bytes = (cw as usize * ch as usize * 4) as u64;
        // (Re)create the canvas-sized buffers only when the size changes.
        let resized = self.state.as_ref().map(|s| (s.cw, s.ch)) != Some((cw, ch));
        if resized {
            let params = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite params (persistent)"),
                size: core::mem::size_of::<GpuU>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let coeffs = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite coeffs (persistent)"),
                size: core::mem::size_of::<GpuCoeffs>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let backdrop = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite backdrop (persistent)"),
                size: canvas_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let out = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite out (persistent)"),
                size: canvas_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let staging = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite readback (persistent)"),
                size: canvas_bytes,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            // Placeholder bind; rebuilt below (needs pigment_buf).
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("composite bg (persistent)"),
                layout: &self.bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: params.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: pigment_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: backdrop.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: out.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: coeffs.as_entire_binding() },
                ],
            });
            self.state = Some(CompositeState {
                cw, ch, gw, gh, scale, coverage_k,
                pcol: brush.pcol, color_sum: brush.color_sum,
                params, coeffs, out, backdrop, staging, bind,
            });
        }
        // SAFETY of unwrap: `state` is Some after the resize branch (or already was).
        let st = self.state.as_mut().expect("composite state set");
        st.gw = gw;
        st.gh = gh;
        st.scale = scale;
        st.coverage_k = coverage_k;
        st.pcol = brush.pcol;
        st.color_sum = brush.color_sum;
        // Backdrop + coeffs are constant for the stroke → upload once here.
        queue.write_buffer(&st.backdrop, 0, backdrop_rgba);
        queue.write_buffer(&st.coeffs, 0, bytemuck::bytes_of(&GpuCoeffs::build(brush)));
        // Rebuild the bind group each stroke so it tracks the current resident pig_a
        // (cheap — just resource references; the canvas buffers are reused).
        if !resized {
            st.bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("composite bg (persistent)"),
                layout: &self.bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: st.params.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: pigment_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: st.backdrop.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: st.out.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: st.coeffs.as_entire_binding() },
                ],
            });
        }
    }

    /// **Fast-path per-frame composite (after [`Self::begin_stroke`]).** Writes only
    /// the 64-byte region uniform, dispatches over the bbox, and reads back the wet
    /// row band. Reuses the persistent buffers/bind — zero per-frame allocation, zero
    /// canvas re-upload. Returns `(band, rect)` like [`Self::composite_buffer_rows`];
    /// `(vec![], (0,0,0,0))` when the region is empty.
    #[must_use]
    pub fn composite_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        grid_region: (u32, u32, u32, u32),
    ) -> (Vec<u8>, (u32, u32, u32, u32)) {
        let Some(st) = self.state.as_ref() else {
            return (Vec::new(), (0, 0, 0, 0));
        };
        let (px_lo, py_lo, px_hi, py_hi) =
            composite_canvas_region(grid_region, st.scale, st.cw, st.ch);
        if py_hi <= py_lo || px_hi <= px_lo {
            return (Vec::new(), (0, 0, 0, 0));
        }
        let uni = GpuU {
            cw: st.cw,
            ch: st.ch,
            gw: st.gw,
            gh: st.gh,
            inv: 1.0 / st.scale as f32,
            color_sum: st.color_sum,
            coverage_k: st.coverage_k,
            _pad0: 0.0,
            origin_x: px_lo,
            origin_y: py_lo,
            end_x: px_hi,
            end_y: py_hi,
            pcol: [st.pcol[0], st.pcol[1], st.pcol[2], 0.0],
        };
        queue.write_buffer(&st.params, 0, bytemuck::bytes_of(&uni));

        let row_bytes = u64::from(st.cw) * 4;
        let band_off = u64::from(py_lo) * row_bytes;
        let band_bytes = u64::from(py_hi - py_lo) * row_bytes;
        let (rw, rh) = (px_hi - px_lo, py_hi - py_lo);
        let mut enc = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("composite frame") });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composite frame pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &st.bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
        }
        enc.copy_buffer_to_buffer(&st.out, band_off, &st.staging, 0, band_bytes);
        queue.submit([enc.finish()]);

        let (tx, rx) = std::sync::mpsc::channel();
        st.staging.slice(0..band_bytes).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let rows = st.staging.slice(0..band_bytes).get_mapped_range().to_vec();
        st.staging.unmap();
        (rows, (px_lo, py_lo, px_hi, py_hi))
    }

    /// Drop the persistent stroke state (frees the canvas buffers) — call when the
    /// field is gone so VRAM isn't held between strokes.
    pub fn end_stroke(&mut self) {
        self.state = None;
    }

    /// One-shot composite to an RGBA8 byte buffer, uploading a CPU pigment field
    /// (the unit-test / CPU-fallback convenience). For the per-frame GPU path use
    /// [`Self::composite_buffer`] with the solver's resident pigment buffer (no
    /// upload, no readback of pigment). `pigment` is the low-res field (`gw*gh`, xyz
    /// mass); `backdrop_rgba` is canvas-res (`cw*ch*4`); `grid_region` scopes work.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn composite_to_rgba(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
        pigment: &[[f32; 4]],
        backdrop_rgba: &[u8],
        brush: &WetCompositeBrush,
        grid_region: (u32, u32, u32, u32),
    ) -> Vec<u8> {
        let pig_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite pig_in (upload)"),
            size: (pigment.len() * 16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&pig_buf, 0, bytemuck::cast_slice(pigment));
        self.composite_buffer(
            device, queue, gw, gh, cw, ch, scale, coverage_k, &pig_buf, backdrop_rgba, brush,
            grid_region,
        )
    }

    /// Composite reading an EXTERNAL pigment buffer (`array<vec4<f32>>`, `gw*gh`) —
    /// bind the solver's [`crate::FluidSolver::pigment_buffer`] here to composite the
    /// GPU-resident bloomed pigment with NO pigment readback (the W15.3 stall fix).
    /// Returns the full canvas RGBA8 (pixels outside the padded region equal the
    /// backdrop, matching the CPU reference's untouched-pixel invariant).
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn composite_buffer(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
        pigment_buf: &wgpu::Buffer,
        backdrop_rgba: &[u8],
        brush: &WetCompositeBrush,
        grid_region: (u32, u32, u32, u32),
    ) -> Vec<u8> {
        let npix = (cw as usize) * (ch as usize);
        let canvas_bytes = (npix * 4) as u64;
        let (px_lo, py_lo, px_hi, py_hi) = composite_canvas_region(grid_region, scale, cw, ch);
        let (params_buf, coeffs_buf, backdrop_buf, out_buf, bind) = self.build_buffers(
            device, queue, gw, gh, cw, ch, scale, coverage_k, pigment_buf, backdrop_rgba, brush,
            grid_region,
        );
        let _keep = (params_buf, coeffs_buf, backdrop_buf);

        let (rw, rh) = (px_hi.saturating_sub(px_lo), py_hi.saturating_sub(py_lo));
        let mut enc = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("composite enc") });
        if rw > 0 && rh > 0 {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composite pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
        }
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite readback"),
            size: canvas_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        enc.copy_buffer_to_buffer(&out_buf, 0, &staging, 0, canvas_bytes);
        queue.submit([enc.finish()]);

        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let mapped = staging.slice(..).get_mapped_range();
        let out = mapped.to_vec();
        drop(mapped);
        staging.unmap();
        out
    }

    /// Like [`Self::composite_buffer`] but reads back ONLY the contiguous wet
    /// **row band** `[py_lo, py_hi)` (full width, a single contiguous copy) — the
    /// per-frame shell path. Returns `(band, (px_lo, py_lo, px_hi, py_hi))`: the
    /// full-width band bytes `(py_hi-py_lo)*cw*4` plus the composited canvas **rect**.
    /// The caller blits ONLY the rect's columns `[px_lo, px_hi)` of each band row
    /// over `canvas_rgba` — outside the rect the pigment is frozen and MUST be left
    /// untouched (a full-width blit would erase parts of the stroke that share rows
    /// with the active wet front). Returns `(vec![], (0,0,0,0))` when empty.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn composite_buffer_rows(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
        pigment_buf: &wgpu::Buffer,
        backdrop_rgba: &[u8],
        brush: &WetCompositeBrush,
        grid_region: (u32, u32, u32, u32),
    ) -> (Vec<u8>, (u32, u32, u32, u32)) {
        let (px_lo, py_lo, px_hi, py_hi) = composite_canvas_region(grid_region, scale, cw, ch);
        if py_hi <= py_lo || px_hi <= px_lo || cw == 0 {
            return (Vec::new(), (0, 0, 0, 0));
        }
        let row_bytes = (cw * 4) as u64;
        let band_off = u64::from(py_lo) * row_bytes;
        let band_bytes = u64::from(py_hi - py_lo) * row_bytes;

        let (params_buf, coeffs_buf, backdrop_buf, out_buf, bind) =
            self.build_buffers(device, queue, gw, gh, cw, ch, scale, coverage_k, pigment_buf, backdrop_rgba, brush, grid_region);
        let (rw, rh) = (px_hi.saturating_sub(px_lo), py_hi.saturating_sub(py_lo));

        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("composite rows enc"),
        });
        if rw > 0 && rh > 0 {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composite rows pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
        }
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite rows readback"),
            size: band_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        enc.copy_buffer_to_buffer(&out_buf, band_off, &staging, 0, band_bytes);
        queue.submit([enc.finish()]);
        // Keep the per-composite buffers alive until the GPU finishes (they back the
        // dispatch + the copy). `_keep` drops after the readback poll below.
        let _keep = (params_buf, coeffs_buf, backdrop_buf, out_buf);

        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let mapped = staging.slice(..).get_mapped_range();
        let rows = mapped.to_vec();
        drop(mapped);
        staging.unmap();
        (rows, (px_lo, py_lo, px_hi, py_hi))
    }

    /// Build the per-composite buffers + bind group (shared by the readback paths).
    #[allow(clippy::too_many_arguments)]
    fn build_buffers(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
        pigment_buf: &wgpu::Buffer,
        backdrop_rgba: &[u8],
        brush: &WetCompositeBrush,
        grid_region: (u32, u32, u32, u32),
    ) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, wgpu::BindGroup) {
        let canvas_bytes = (cw as usize * ch as usize * 4) as u64;
        let (px_lo, py_lo, px_hi, py_hi) = composite_canvas_region(grid_region, scale, cw, ch);
        let uni = GpuU {
            cw,
            ch,
            gw,
            gh,
            inv: 1.0 / scale as f32,
            color_sum: brush.color_sum,
            coverage_k,
            _pad0: 0.0,
            origin_x: px_lo,
            origin_y: py_lo,
            end_x: px_hi,
            end_y: py_hi,
            pcol: [brush.pcol[0], brush.pcol[1], brush.pcol[2], 0.0],
        };
        let coeffs = GpuCoeffs::build(brush);
        let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite params"),
            size: core::mem::size_of::<GpuU>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&params_buf, 0, bytemuck::bytes_of(&uni));
        let coeffs_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite coeffs"),
            size: core::mem::size_of::<GpuCoeffs>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&coeffs_buf, 0, bytemuck::bytes_of(&coeffs));
        let backdrop_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite backdrop"),
            size: canvas_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&backdrop_buf, 0, backdrop_rgba);
        let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite out"),
            size: canvas_bytes,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        queue.write_buffer(&out_buf, 0, backdrop_rgba);
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("composite bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: pigment_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: backdrop_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: out_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: coeffs_buf.as_entire_binding() },
            ],
        });
        (params_buf, coeffs_buf, backdrop_buf, out_buf, bind)
    }
}
