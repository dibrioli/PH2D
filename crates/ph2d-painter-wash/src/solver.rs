//! GPU solver for the minimal watercolor core (ADR-0086). See the crate docs for the
//! model. Three pipelines (`cs_splat`, `cs_step`, `cs_composite`) over five field buffers
//! (`water_a/b`, `pig_a/b`, `paper`). The canonical state always lives in the `*_a` buffers
//! (`step` normalises back to `a` after an odd substep count), so reads are unambiguous.

use bytemuck::{Pod, Zeroable};

/// Per-frame solver coefficients (the `cs_step` UBO). 16 × 4 B = 64 B, 16-aligned.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct WashParams {
    pub width: u32,
    pub height: u32,
    pub region_ox: u32,
    pub region_oy: u32,
    pub region_w: u32,
    pub region_h: u32,
    /// Bloom rate `D` (≤ 0.25 for unconditional stability).
    pub diffusivity: f32,
    /// FlowOutward `λ` — pigment drift toward drier cells (edge-darkening).
    pub flow_outward: f32,
    /// Water lost per step (drying).
    pub evaporation: f32,
    pub w_lo: f32,
    pub w_hi: f32,
    pub perm_valley: f32,
    pub perm_crest: f32,
    pub granulation: f32,
    pub _pad0: f32,
    pub _pad1: f32,
}

impl Default for WashParams {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            region_ox: 0,
            region_oy: 0,
            region_w: 0,
            region_h: 0,
            diffusivity: 0.14,
            flow_outward: 0.0,
            evaporation: 0.0,
            w_lo: 0.05,
            w_hi: 0.4,
            perm_valley: 1.0,
            perm_crest: 1.0,
            granulation: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
        }
    }
}

/// One brush dab (the `cs_splat` storage input). 8 × 4 B = 32 B.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Dab {
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub water_add: f32,
    /// `(absorb.rgb, mass)` deposited at full falloff.
    pub pig: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SplatParams {
    width: u32,
    height: u32,
    n_dabs: u32,
    _pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CParams {
    width: u32,
    height: u32,
    coverage_k: f32,
    _pad: f32,
}

const WG: u32 = 8;
fn groups(n: u32) -> u32 {
    n.div_ceil(WG)
}

/// The minimal watercolor GPU solver.
pub struct WashSolver {
    width: u32,
    height: u32,
    params: std::cell::Cell<WashParams>,
    params_buf: wgpu::Buffer,
    splat_params_buf: wgpu::Buffer,
    cparams_buf: wgpu::Buffer,
    water_a: wgpu::Buffer,
    water_b: wgpu::Buffer,
    paper: wgpu::Buffer,
    pig_a: wgpu::Buffer,
    pig_b: wgpu::Buffer,
    dabs_buf: wgpu::Buffer,
    out_buf: wgpu::Buffer,
    step_pipe: wgpu::ComputePipeline,
    splat_pipe: wgpu::ComputePipeline,
    composite_pipe: wgpu::ComputePipeline,
    bg_step_ab: wgpu::BindGroup,
    bg_step_ba: wgpu::BindGroup,
    bg_splat: wgpu::BindGroup,
    bg_composite: wgpu::BindGroup,
}

const MAX_DABS: u64 = 4096;

impl WashSolver {
    /// Allocate a solver for a `width × height` field. All fields start zeroed.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let n = (width * height) as u64;
        let f32n = n * 4;
        let vec4n = n * 16;

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wash step"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/wash.wgsl").into()),
        });
        let splat_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wash splat"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/splat.wgsl").into()),
        });
        let composite_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wash composite"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/composite.wgsl").into()),
        });

        let uniform = |b: u32| wgpu::BindGroupLayoutEntry {
            binding: b,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let storage = |b: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding: b,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let step_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wash step bgl"),
            entries: &[
                uniform(0),
                storage(1, true),  // water_in
                storage(2, true),  // paper
                storage(3, true),  // pig_in
                storage(4, false), // water_out
                storage(5, false), // pig_out
            ],
        });
        let splat_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wash splat bgl"),
            entries: &[uniform(0), storage(1, false), storage(2, false), storage(3, true)],
        });
        let composite_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wash composite bgl"),
            entries: &[uniform(0), storage(1, true), storage(2, false)],
        });

        let mk_pipe = |bgl: &wgpu::BindGroupLayout, m: &wgpu::ShaderModule, entry: &str| {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("wash layout"),
                bind_group_layouts: &[bgl],
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("wash pipeline"),
                layout: Some(&layout),
                module: m,
                entry_point: Some(entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let step_pipe = mk_pipe(&step_bgl, &module, "cs_step");
        let splat_pipe = mk_pipe(&splat_bgl, &splat_module, "cs_splat");
        let composite_pipe = mk_pipe(&composite_bgl, &composite_module, "cs_composite");

        let storage_buf = |label: &str, size: u64| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            })
        };
        let water_a = storage_buf("wash water_a", f32n);
        let water_b = storage_buf("wash water_b", f32n);
        let paper = storage_buf("wash paper", f32n);
        let pig_a = storage_buf("wash pig_a", vec4n);
        let pig_b = storage_buf("wash pig_b", vec4n);
        let out_buf = storage_buf("wash out", f32n);
        let dabs_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wash dabs"),
            size: MAX_DABS * core::mem::size_of::<Dab>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let ubo = |label: &str, size: u64| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let params_buf = ubo("wash params", core::mem::size_of::<WashParams>() as u64);
        let splat_params_buf = ubo("wash splat params", core::mem::size_of::<SplatParams>() as u64);
        let cparams_buf = ubo("wash cparams", core::mem::size_of::<CParams>() as u64);

        fn e(binding: u32, buf: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
            wgpu::BindGroupEntry { binding, resource: buf.as_entire_binding() }
        }
        let bg_step_ab = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wash bg step a→b"),
            layout: &step_bgl,
            entries: &[
                e(0, &params_buf),
                e(1, &water_a),
                e(2, &paper),
                e(3, &pig_a),
                e(4, &water_b),
                e(5, &pig_b),
            ],
        });
        let bg_step_ba = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wash bg step b→a"),
            layout: &step_bgl,
            entries: &[
                e(0, &params_buf),
                e(1, &water_b),
                e(2, &paper),
                e(3, &pig_b),
                e(4, &water_a),
                e(5, &pig_a),
            ],
        });
        let bg_splat = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wash bg splat"),
            layout: &splat_bgl,
            entries: &[e(0, &splat_params_buf), e(1, &water_a), e(2, &pig_a), e(3, &dabs_buf)],
        });
        let bg_composite = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wash bg composite"),
            layout: &composite_bgl,
            entries: &[e(0, &cparams_buf), e(1, &pig_a), e(2, &out_buf)],
        });

        let mut params = WashParams { width, height, region_w: width, region_h: height, ..Default::default() };
        params.region_ox = 0;
        params.region_oy = 0;

        Self {
            width,
            height,
            params: std::cell::Cell::new(params),
            params_buf,
            splat_params_buf,
            cparams_buf,
            water_a,
            water_b,
            paper,
            pig_a,
            pig_b,
            dabs_buf,
            out_buf,
            step_pipe,
            splat_pipe,
            composite_pipe,
            bg_step_ab,
            bg_step_ba,
            bg_splat,
            bg_composite,
        }
    }

    /// Set the physics coefficients. `width`/`height`/`region_*` are overwritten to the
    /// full grid here (the live integration will scope the region per frame).
    pub fn set_params(&self, queue: &wgpu::Queue, mut p: WashParams) {
        p.width = self.width;
        p.height = self.height;
        p.region_ox = 0;
        p.region_oy = 0;
        p.region_w = self.width;
        p.region_h = self.height;
        self.params.set(p);
        queue.write_buffer(&self.params_buf, 0, bytemuck::bytes_of(&p));
    }

    /// Upload initial fields directly (test/seed path). `pigment` is `(absorb.rgb, mass)`.
    pub fn upload(&self, queue: &wgpu::Queue, water: &[f32], paper: &[f32], pigment: &[[f32; 4]]) {
        queue.write_buffer(&self.water_a, 0, bytemuck::cast_slice(water));
        queue.write_buffer(&self.paper, 0, bytemuck::cast_slice(paper));
        queue.write_buffer(&self.pig_a, 0, bytemuck::cast_slice(pigment));
    }

    /// Splat a dab list onto the canonical (`*_a`) fields.
    pub fn splat(&self, device: &wgpu::Device, queue: &wgpu::Queue, dabs: &[Dab]) {
        assert!(dabs.len() as u64 <= MAX_DABS, "dab count exceeds MAX_DABS");
        queue.write_buffer(&self.dabs_buf, 0, bytemuck::cast_slice(dabs));
        let sp = SplatParams { width: self.width, height: self.height, n_dabs: dabs.len() as u32, _pad: 0 };
        queue.write_buffer(&self.splat_params_buf, 0, bytemuck::bytes_of(&sp));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash splat enc") });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("wash splat"), timestamp_writes: None });
            pass.set_pipeline(&self.splat_pipe);
            pass.set_bind_group(0, &self.bg_splat, &[]);
            pass.dispatch_workgroups(groups(self.width), groups(self.height), 1);
        }
        queue.submit([enc.finish()]);
    }

    /// Run `substeps` of the physics. Normalises the result back into the `*_a` buffers.
    pub fn step(&self, device: &wgpu::Device, queue: &wgpu::Queue, substeps: u32) {
        if substeps == 0 {
            return;
        }
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash step enc") });
        // One compute pass per substep so wgpu inserts the read-after-write barrier between the
        // ping-pong dispatches (the canonical, definitely-correct ordering).
        for k in 0..substeps {
            let bg = if k % 2 == 0 { &self.bg_step_ab } else { &self.bg_step_ba };
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("wash step"), timestamp_writes: None });
            pass.set_pipeline(&self.step_pipe);
            pass.set_bind_group(0, bg, &[]);
            pass.dispatch_workgroups(groups(self.width), groups(self.height), 1);
        }
        // After an odd number of substeps the latest data is in `*_b`; copy it back so the
        // canonical state is always in `*_a`.
        if substeps % 2 == 1 {
            enc.copy_buffer_to_buffer(&self.water_b, 0, &self.water_a, 0, (self.width * self.height * 4) as u64);
            enc.copy_buffer_to_buffer(&self.pig_b, 0, &self.pig_a, 0, (self.width * self.height * 16) as u64);
        }
        queue.submit([enc.finish()]);
    }

    /// Composite the field to packed RGBA8 (white backdrop, v1) and return it.
    pub fn composite(&self, device: &wgpu::Device, queue: &wgpu::Queue, coverage_k: f32) -> Vec<u32> {
        let cp = CParams { width: self.width, height: self.height, coverage_k, _pad: 0.0 };
        queue.write_buffer(&self.cparams_buf, 0, bytemuck::bytes_of(&cp));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash composite enc") });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("wash composite"), timestamp_writes: None });
            pass.set_pipeline(&self.composite_pipe);
            pass.set_bind_group(0, &self.bg_composite, &[]);
            pass.dispatch_workgroups(groups(self.width), groups(self.height), 1);
        }
        queue.submit([enc.finish()]);
        self.read_u32(device, queue, &self.out_buf)
    }

    /// Read the canonical water field.
    pub fn read_water(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<f32> {
        self.read_f32(device, queue, &self.water_a)
    }

    /// Read the canonical pigment field as `(absorb.rgb, mass)` per cell.
    pub fn read_pigment(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<[f32; 4]> {
        let size = u64::from(self.width * self.height) * 16;
        let bytes = self.read_bytes(device, queue, &self.pig_a, size);
        let flat: &[f32] = bytemuck::cast_slice(&bytes);
        flat.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect()
    }

    fn read_f32(&self, device: &wgpu::Device, queue: &wgpu::Queue, buf: &wgpu::Buffer) -> Vec<f32> {
        let size = u64::from(self.width * self.height) * 4;
        bytemuck::cast_slice(&self.read_bytes(device, queue, buf, size)).to_vec()
    }
    fn read_u32(&self, device: &wgpu::Device, queue: &wgpu::Queue, buf: &wgpu::Buffer) -> Vec<u32> {
        let size = u64::from(self.width * self.height) * 4;
        bytemuck::cast_slice(&self.read_bytes(device, queue, buf, size)).to_vec()
    }
    fn read_bytes(&self, device: &wgpu::Device, queue: &wgpu::Queue, buf: &wgpu::Buffer, size: u64) -> Vec<u8> {
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wash readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash readback enc") });
        enc.copy_buffer_to_buffer(buf, 0, &staging, 0, size);
        queue.submit([enc.finish()]);
        staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let data = staging.slice(..).get_mapped_range();
        let out = data.to_vec();
        drop(data);
        staging.unmap();
        out
    }
}
