//! GPU solver for the minimal watercolor core (ADR-0086). Two pipelines — `cs_splat`
//! (brush input) and `cs_step` (the whole physics) — over five field buffers
//! (`water_a/b`, `pig_a/b`, `paper`). Compositing lives in [`crate::WashCompositor`]
//! (it reads the solver's pigment buffer via [`WashSolver::pig_buffer`]).
//!
//! The canonical state always lives in the `*_a` buffers (`step`/`encode_step` normalise
//! back to `a` after an odd substep count), so reads + composite are unambiguous.

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

impl Dab {
    /// Build a dab from a linear-sRGB pigment colour + a deposit mass. The pigment is stored
    /// as Beer–Lambert optical density: a colour (reflectance) `c` has absorbance `−ln(c)`, so
    /// `exp(−absorb)` reproduces the colour over white; stacking adds absorbance (subtractive).
    /// `absorb` is baked × `mass` (more mass ⇒ darker/denser), and `mass` rides the `.w` channel.
    #[must_use]
    pub fn from_color_mass(cx: f32, cy: f32, r: f32, water_add: f32, color: [f32; 3], mass: f32) -> Self {
        let a = |c: f32| -> f32 { -c.clamp(0.02, 1.0).ln() * mass };
        Self { cx, cy, r, water_add, pig: [a(color[0]), a(color[1]), a(color[2]), mass] }
    }

    /// Build a dab for the spectral **Kubelka–Munk** colour mode: `pig` carries the 4 base-pigment
    /// concentrations (see [`crate::km`]) instead of RGB absorbance. The composite reads them via the
    /// K–M branch; the transport (a linear gather) is identical to the RGB path.
    #[must_use]
    pub fn from_concentrations(cx: f32, cy: f32, r: f32, water_add: f32, conc: [f32; 4]) -> Self {
        Self { cx, cy, r, water_add, pig: conc }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SplatParams {
    width: u32,
    height: u32,
    n_dabs: u32,
    region_ox: u32,
    region_oy: u32,
    region_w: u32,
    region_h: u32,
    _pad: u32,
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
    water_a: wgpu::Buffer,
    water_b: wgpu::Buffer,
    paper: wgpu::Buffer,
    pig_a: wgpu::Buffer,
    pig_b: wgpu::Buffer,
    dabs_buf: wgpu::Buffer,
    step_pipe: wgpu::ComputePipeline,
    splat_pipe: wgpu::ComputePipeline,
    bg_step_ab: wgpu::BindGroup,
    bg_step_ba: wgpu::BindGroup,
    bg_splat: wgpu::BindGroup,
}

const MAX_DABS: u64 = 4096;

impl WashSolver {
    /// Allocate a solver for a `width × height` field. All fields start zeroed.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let n = u64::from(width) * u64::from(height);
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

        fn e(binding: u32, buf: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
            wgpu::BindGroupEntry { binding, resource: buf.as_entire_binding() }
        }
        let bg_step_ab = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wash bg step a→b"),
            layout: &step_bgl,
            entries: &[e(0, &params_buf), e(1, &water_a), e(2, &paper), e(3, &pig_a), e(4, &water_b), e(5, &pig_b)],
        });
        let bg_step_ba = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wash bg step b→a"),
            layout: &step_bgl,
            entries: &[e(0, &params_buf), e(1, &water_b), e(2, &paper), e(3, &pig_b), e(4, &water_a), e(5, &pig_a)],
        });
        let bg_splat = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wash bg splat"),
            layout: &splat_bgl,
            entries: &[e(0, &splat_params_buf), e(1, &water_a), e(2, &pig_a), e(3, &dabs_buf)],
        });

        let params = WashParams { width, height, region_w: width, region_h: height, ..Default::default() };

        Self {
            width,
            height,
            params: std::cell::Cell::new(params),
            params_buf,
            splat_params_buf,
            water_a,
            water_b,
            paper,
            pig_a,
            pig_b,
            dabs_buf,
            step_pipe,
            splat_pipe,
            bg_step_ab,
            bg_step_ba,
            bg_splat,
        }
    }

    /// The canonical pigment buffer (`(absorb.rgb, mass)` per cell) — the compositor binds this.
    #[must_use]
    pub fn pig_buffer(&self) -> &wgpu::Buffer {
        &self.pig_a
    }
    /// The canonical water field (for an optional wet-sheen view).
    #[must_use]
    pub fn water_buffer(&self) -> &wgpu::Buffer {
        &self.water_a
    }

    /// Set the physics coefficients. The dispatch region defaults to the full grid; pass a
    /// scoped region to [`WashSolver::encode_step`] to override per frame.
    pub fn set_params(&self, queue: &wgpu::Queue, mut p: WashParams) {
        p.width = self.width;
        p.height = self.height;
        let cur = self.params.get();
        p.region_ox = cur.region_ox;
        p.region_oy = cur.region_oy;
        p.region_w = cur.region_w.max(1).min(self.width);
        p.region_h = cur.region_h.max(1).min(self.height);
        if p.region_w == 0 {
            p.region_w = self.width;
        }
        if p.region_h == 0 {
            p.region_h = self.height;
        }
        self.params.set(p);
        queue.write_buffer(&self.params_buf, 0, bytemuck::bytes_of(&p));
    }

    /// Zero all dynamic fields (per-stroke reset). Paper is static and kept.
    pub fn clear(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash clear enc") });
        for b in [&self.water_a, &self.water_b, &self.pig_a, &self.pig_b] {
            enc.clear_buffer(b, 0, None);
        }
        queue.submit([enc.finish()]);
    }

    /// Upload initial fields directly (test/seed path). `pigment` is `(absorb.rgb, mass)`.
    pub fn upload(&self, queue: &wgpu::Queue, water: &[f32], paper: &[f32], pigment: &[[f32; 4]]) {
        queue.write_buffer(&self.water_a, 0, bytemuck::cast_slice(water));
        queue.write_buffer(&self.paper, 0, bytemuck::cast_slice(paper));
        queue.write_buffer(&self.pig_a, 0, bytemuck::cast_slice(pigment));
    }

    /// Splat a dab list onto the canonical (`*_a`) fields (own submit; test/standalone path).
    pub fn splat(&self, device: &wgpu::Device, queue: &wgpu::Queue, dabs: &[Dab]) {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash splat enc") });
        self.encode_splat(queue, &mut enc, dabs, (0, 0, self.width, self.height));
        queue.submit([enc.finish()]);
    }

    /// Run `substeps` of the physics over the FULL grid (own submit; test/standalone path).
    pub fn step(&self, device: &wgpu::Device, queue: &wgpu::Queue, substeps: u32) {
        if substeps == 0 {
            return;
        }
        let region = (0, 0, self.width, self.height);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash step enc") });
        self.encode_substeps(queue, &mut enc, substeps, region);
        queue.submit([enc.finish()]);
    }

    /// Single-submit frame for the live path: splat the dabs then run `substeps` over `region`,
    /// appending to `enc` and returning it (the caller adds the composite + one `submit`).
    #[must_use]
    pub fn encode_step(
        &self,
        queue: &wgpu::Queue,
        mut enc: wgpu::CommandEncoder,
        dabs: &[Dab],
        substeps: u32,
        region: (u32, u32, u32, u32),
    ) -> wgpu::CommandEncoder {
        if !dabs.is_empty() {
            self.encode_splat(queue, &mut enc, dabs, region);
        }
        self.encode_substeps(queue, &mut enc, substeps, region);
        enc
    }

    fn encode_splat(&self, queue: &wgpu::Queue, enc: &mut wgpu::CommandEncoder, dabs: &[Dab], region: (u32, u32, u32, u32)) {
        assert!(dabs.len() as u64 <= MAX_DABS, "dab count exceeds MAX_DABS");
        if dabs.is_empty() {
            return;
        }
        let rw = region.2.clamp(1, self.width);
        let rh = region.3.clamp(1, self.height);
        queue.write_buffer(&self.dabs_buf, 0, bytemuck::cast_slice(dabs));
        let sp = SplatParams {
            width: self.width,
            height: self.height,
            n_dabs: dabs.len() as u32,
            region_ox: region.0,
            region_oy: region.1,
            region_w: rw,
            region_h: rh,
            _pad: 0,
        };
        queue.write_buffer(&self.splat_params_buf, 0, bytemuck::bytes_of(&sp));
        let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("wash splat"), timestamp_writes: None });
        pass.set_pipeline(&self.splat_pipe);
        pass.set_bind_group(0, &self.bg_splat, &[]);
        pass.dispatch_workgroups(groups(rw), groups(rh), 1);
    }

    fn encode_substeps(&self, queue: &wgpu::Queue, enc: &mut wgpu::CommandEncoder, substeps: u32, region: (u32, u32, u32, u32)) {
        if substeps == 0 {
            return;
        }
        // Write the region into the params UBO (physics coefficients are already cached).
        let mut p = self.params.get();
        p.region_ox = region.0;
        p.region_oy = region.1;
        p.region_w = region.2.clamp(1, self.width);
        p.region_h = region.3.clamp(1, self.height);
        self.params.set(p);
        queue.write_buffer(&self.params_buf, 0, bytemuck::bytes_of(&p));
        // One compute pass per substep ⇒ wgpu inserts the ping-pong read-after-write barrier.
        for k in 0..substeps {
            let bg = if k % 2 == 0 { &self.bg_step_ab } else { &self.bg_step_ba };
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("wash step"), timestamp_writes: None });
            pass.set_pipeline(&self.step_pipe);
            pass.set_bind_group(0, bg, &[]);
            pass.dispatch_workgroups(groups(p.region_w), groups(p.region_h), 1);
        }
        // After an odd substep count the latest data is in `*_b`; copy it back to `*_a`.
        if substeps % 2 == 1 {
            let bytes = u64::from(self.width) * u64::from(self.height);
            enc.copy_buffer_to_buffer(&self.water_b, 0, &self.water_a, 0, bytes * 4);
            enc.copy_buffer_to_buffer(&self.pig_b, 0, &self.pig_a, 0, bytes * 16);
        }
    }

    /// Read the canonical water field.
    pub fn read_water(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<f32> {
        let size = u64::from(self.width) * u64::from(self.height) * 4;
        bytemuck::cast_slice(&read_bytes(device, queue, &self.water_a, size)).to_vec()
    }

    /// Read the canonical pigment field as `(absorb.rgb, mass)` per cell.
    pub fn read_pigment(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<[f32; 4]> {
        let size = u64::from(self.width) * u64::from(self.height) * 16;
        let bytes = read_bytes(device, queue, &self.pig_a, size);
        let flat: &[f32] = bytemuck::cast_slice(&bytes);
        flat.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect()
    }
}

/// Blocking readback of a storage buffer (test / parity path).
pub(crate) fn read_bytes(device: &wgpu::Device, queue: &wgpu::Queue, buf: &wgpu::Buffer, size: u64) -> Vec<u8> {
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
