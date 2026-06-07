//! The solver: CPU reference (REUSE of the shipped `diffusion`) + the GPU
//! compute shader source. The live wgpu pipeline lands in W15.3 phase 2.
//!
//! ## CPU reference = the parity truth + det fallback
//! [`step_cpu_reference`] runs the exact gated diffusion-advection that shipped in
//! W15.2 ([`ph2d_painter_brush::diffusion::DiffusionGrid`]). It is (a) the HR-5
//! det-mode path (ADR-0049 §2.11 — GPU is non-deterministic), and (b) the
//! bit-near reference the GPU pass is validated against (phase-2 parity gate).
//!
//! ## GPU pass (phase 2)
//! [`FLUID_WGSL`] is the compute shader mirroring the CPU passes. The
//! diffusion pass is a pure GATHER (each cell sums conductance·(neighbour−self)),
//! which maps directly to a compute kernel. The advection pass is reformulated
//! from the CPU's SCATTER (push to the downstream neighbour) into a GATHER (each
//! cell pulls the net flux from its 4 neighbours) so it is atomics-free and
//! order-independent — the adaptation that makes it correct on the GPU. The wgpu
//! pipeline (storage textures, bind groups, dispatch, bbox upload) is wired in
//! phase 2 against a headless `GpuContext`, with a CPU↔GPU parity test.

use crate::params::FluidParams;
use ph2d_painter_brush::diffusion::DiffusionGrid;

/// The GPU compute shader (mirror of the CPU diffusion-advection). Embedded so a
/// dev-test can validate it through naga before any GPU init, and phase 2 can
/// `create_shader_module` it directly.
pub const FLUID_WGSL: &str = include_str!("shader/fluid.wgsl");

/// Run the CPU reference solver `steps` times in place. The det-mode path AND the
/// parity reference for the GPU pass — both go through the one shipped solver.
pub fn step_cpu_reference(grid: &mut DiffusionGrid, params: &FluidParams, steps: u32) {
    let dp = params.to_diffusion();
    for _ in 0..steps {
        grid.step(&dp);
    }
}

/// The WGSL `Params` UBO, byte-for-byte (12 × 4 = 48 B). `#[repr(C)]` Pod so a
/// per-frame update is a zero-copy `write_buffer` of `bytemuck::bytes_of` (HR-3).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuParams {
    width: u32,
    height: u32,
    diffusivity: f32,
    evaporation: f32,
    downhill: f32,
    flow_outward: f32,
    w_lo: f32,
    w_hi: f32,
    perm_valley: f32,
    perm_crest: f32,
    _pad0: f32,
    _pad1: f32,
}

/// The live GPU solver: the three compute pipelines + the ping-pong storage
/// buffers for one fluid field. Built once per field size; `step` dispatches the
/// passes, `read_pigment` maps the result back (for the parity test + the det/
/// composite read). Storage BUFFERS (not textures) so the layout is 1:1 with the
/// CPU `DiffusionGrid` arrays — the parity reference is exact-shaped.
///
/// Per step the encoder runs three passes (each its own compute pass, so wgpu
/// inserts the RAW barriers): `cs_diffuse` A→B, `cs_advect` B→A, `cs_evaporate`
/// (water). Pigment ends back in A every step, so the bind groups are fixed.
pub struct FluidSolver {
    width: u32,
    height: u32,
    diffuse: wgpu::ComputePipeline,
    advect: wgpu::ComputePipeline,
    evaporate: wgpu::ComputePipeline,
    params_buf: wgpu::Buffer,
    // All field buffers are owned so they outlive the bind groups that reference
    // them (the readback reads `pig_a`; `upload` writes water/paper/pig_a).
    water: wgpu::Buffer,
    paper: wgpu::Buffer,
    pig_a: wgpu::Buffer,
    /// Owned only to keep the GPU buffer alive for the ping-pong bind groups
    /// (`bg_diffuse` writes it, `bg_advect` reads it); never touched directly.
    #[allow(dead_code)]
    pig_b: wgpu::Buffer,
    bg_diffuse: wgpu::BindGroup,
    bg_advect: wgpu::BindGroup,
    bg_evaporate: wgpu::BindGroup,
}

impl FluidSolver {
    /// Build the pipelines + buffers for a `width × height` field on `device`.
    #[must_use]
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let n = (width as usize) * (height as usize);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-painter-fluid solver"),
            source: wgpu::ShaderSource::Wgsl(FLUID_WGSL.into()),
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
            label: Some("ph2d-painter-fluid bgl"),
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
                storage(1, false), // water (read_write — evaporate writes it)
                storage(2, true),  // paper (read)
                storage(3, true),  // pig_in (read)
                storage(4, false), // pig_out (read_write)
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-painter-fluid layout"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });
        let pipe = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ph2d-painter-fluid pipeline"),
                layout: Some(&layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let buf = |label: &str, size: u64, extra: wgpu::BufferUsages| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | extra,
                mapped_at_creation: false,
            })
        };
        let f32n = (n * 4) as u64;
        let vec4n = (n * 16) as u64;
        let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-painter-fluid params"),
            size: core::mem::size_of::<GpuParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let water = buf("fluid water", f32n, wgpu::BufferUsages::COPY_SRC);
        let paper = buf("fluid paper", f32n, wgpu::BufferUsages::empty());
        let pig_a = buf("fluid pig_a", vec4n, wgpu::BufferUsages::COPY_SRC);
        let pig_b = buf("fluid pig_b", vec4n, wgpu::BufferUsages::empty());
        // bind group: (params, water, paper, pig_in, pig_out).
        let bg = |label: &str, pin: &wgpu::Buffer, pout: &wgpu::Buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: water.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: paper.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: pin.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: pout.as_entire_binding() },
                ],
            })
        };
        let bg_diffuse = bg("fluid bg diffuse", &pig_a, &pig_b); // A→B
        let bg_advect = bg("fluid bg advect", &pig_b, &pig_a); // B→A
        let bg_evaporate = bg("fluid bg evaporate", &pig_a, &pig_b); // pig unused
        Self {
            width,
            height,
            diffuse: pipe("cs_diffuse"),
            advect: pipe("cs_advect"),
            evaporate: pipe("cs_evaporate"),
            params_buf,
            water,
            paper,
            pig_a,
            pig_b,
            bg_diffuse,
            bg_advect,
            bg_evaporate,
        }
    }

    /// Upload the initial field. `pigment` is the linear-RGB mass (xyz; w ignored).
    /// Lengths must equal `width * height`.
    pub fn upload(
        &self,
        queue: &wgpu::Queue,
        water: &[f32],
        paper: &[f32],
        pigment: &[[f32; 4]],
    ) {
        queue.write_buffer(&self.water, 0, bytemuck::cast_slice(water));
        queue.write_buffer(&self.paper, 0, bytemuck::cast_slice(paper));
        queue.write_buffer(&self.pig_a, 0, bytemuck::cast_slice(pigment));
    }

    /// Push the solver coefficients (mirrors the CPU `DiffusionParams`).
    pub fn set_params(&self, queue: &wgpu::Queue, params: &FluidParams) {
        let gp = GpuParams {
            width: self.width,
            height: self.height,
            diffusivity: params.diffusivity,
            evaporation: params.evaporation,
            downhill: params.downhill,
            flow_outward: params.flow_outward,
            w_lo: params.w_lo,
            w_hi: params.w_hi,
            perm_valley: params.perm_valley,
            perm_crest: params.perm_crest,
            _pad0: 0.0,
            _pad1: 0.0,
        };
        queue.write_buffer(&self.params_buf, 0, bytemuck::bytes_of(&gp));
    }

    /// Run `substeps` diffusion-advection-evaporation steps on the GPU (one
    /// submit). After this, the pigment is in `pig_a`; read it with [`Self::read_pigment`].
    pub fn step(&self, device: &wgpu::Device, queue: &wgpu::Queue, substeps: u32) {
        let (gx, gy) = (self.width.div_ceil(8), self.height.div_ceil(8));
        let mut enc =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("fluid step") });
        for _ in 0..substeps {
            for (pipe, bg) in [
                (&self.diffuse, &self.bg_diffuse),
                (&self.advect, &self.bg_advect),
                (&self.evaporate, &self.bg_evaporate),
            ] {
                let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("fluid pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(pipe);
                pass.set_bind_group(0, bg, &[]);
                pass.dispatch_workgroups(gx, gy, 1);
            }
        }
        queue.submit([enc.finish()]);
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
    }

    /// The GPU-resident pigment buffer (`pig_a`, `array<vec4<f32>>`) after [`Self::step`].
    /// Exposed so the GPU compositor binds it DIRECTLY (W15.3) — the per-frame
    /// composite reads the bloomed pigment in place, removing the pigment readback
    /// that was the remaining stall (ADR-0049 §0). Bind-compatible with the
    /// compositor's `pig_in` binding (same `vec4` layout, same device).
    #[must_use]
    pub fn pigment_buffer(&self) -> &wgpu::Buffer {
        &self.pig_a
    }

    /// Field dimensions (`width`, `height`) — the pigment buffer holds `width*height`
    /// `vec4<f32>` cells, the layout the compositor's `gw`/`gh` describe.
    #[must_use]
    pub fn dims(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Map the current pigment field (`pig_a`) back to the CPU.
    #[must_use]
    pub fn read_pigment(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<[f32; 4]> {
        let n = (self.width as usize) * (self.height as usize);
        let size = (n * 16) as u64;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluid readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("fluid rb") });
        enc.copy_buffer_to_buffer(&self.pig_a, 0, &staging, 0, size);
        queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let mapped = staging.slice(..).get_mapped_range();
        let out = bytemuck::cast_slice::<u8, [f32; 4]>(&mapped).to_vec();
        drop(mapped);
        staging.unmap();
        out
    }

    /// Map the current wetness field back (companion to [`Self::read_pigment`] —
    /// needed so the drying that happened on the GPU is reflected in the grid).
    #[must_use]
    pub fn read_water(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<f32> {
        let n = (self.width as usize) * (self.height as usize);
        let size = (n * 4) as u64;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluid water readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("fluid water rb") });
        enc.copy_buffer_to_buffer(&self.water, 0, &staging, 0, size);
        queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let mapped = staging.slice(..).get_mapped_range();
        let out = bytemuck::cast_slice::<u8, f32>(&mapped).to_vec();
        drop(mapped);
        staging.unmap();
        out
    }

    /// **Drop-in GPU accelerator for the CPU `DiffusionGrid::step` loop.** Uploads
    /// the grid, runs `substeps` on the GPU, and writes the evolved pigment + water
    /// back into the grid — so the grid stays the CPU source of truth (the composite
    /// reads it, the det fallback uses it) while the heavy diffuse/advect/evaporate
    /// run on the GPU. Equivalent to `step_cpu_reference(grid, params, substeps)`
    /// but on the GPU (proven by the parity gate). Paper is re-uploaded each call
    /// (static + cheap); a persistent-state fast path is a later optimization.
    pub fn step_grid(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        grid: &mut DiffusionGrid,
        params: &FluidParams,
        substeps: u32,
    ) {
        let pig4: Vec<[f32; 4]> = grid
            .pigment()
            .iter()
            .map(|p| [p[0], p[1], p[2], 0.0])
            .collect();
        self.set_params(queue, params);
        self.upload(queue, grid.water(), grid.paper(), &pig4);
        self.step(device, queue, substeps);
        let pig = self.read_pigment(device, queue);
        let water = self.read_water(device, queue);
        let pig3: Vec<[f32; 3]> = pig.iter().map(|p| [p[0], p[1], p[2]]).collect();
        grid.set_pigment_from(&pig3);
        grid.set_water_from(&water);
    }
}
