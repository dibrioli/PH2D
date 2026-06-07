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
use ph2d_painter_brush::wet_composite::{WetCompositeBrush, composite_canvas_region};

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
        // Pinned: the shader hard-codes NB=24 (and the flatten strides below).
        assert_eq!(SPECTRAL_BANDS, 24, "composite.wgsl assumes 24 spectral bands");
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

/// The live GPU compositor: one compute pipeline + its bind-group layout, built
/// once per device. Buffers are created per composite call (sized to the canvas);
/// a persistent-buffer fast path is a later optimization.
pub struct FluidCompositor {
    pipeline: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
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
        Self { pipeline, bgl }
    }

    /// One-shot composite to an RGBA8 byte buffer (the readback / first-milestone
    /// path + the parity test). `pigment` is the low-res field (`gw*gh`, xyz mass);
    /// `backdrop_rgba` is canvas-res (`cw*ch*4`); `grid_region` scopes the work.
    /// Returns the full canvas RGBA8 (pixels outside the padded region equal the
    /// backdrop, matching the CPU reference's untouched-pixel invariant).
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
        let npix = (cw as usize) * (ch as usize);
        let canvas_bytes = (npix * 4) as u64;
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

        let pig_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite pig_in"),
            size: (pigment.len() * 16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&pig_buf, 0, bytemuck::cast_slice(pigment));

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
        // Seed the output with the backdrop so pixels outside the dispatched bbox
        // match the CPU (whose canvas keeps the backdrop where the loop never runs).
        queue.write_buffer(&out_buf, 0, backdrop_rgba);

        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("composite bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: pig_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: backdrop_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: out_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: coeffs_buf.as_entire_binding() },
            ],
        });

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
}
