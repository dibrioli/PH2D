//! ⭐⭐⭐⭐ **A SURFACE NEIGHBOURHOOD on the GPU** (`docs/3D/30` §14, W6).
//!
//! The compositor's neighbourhood kinds (Gaussian, Sharpen, Bloom, Shadows/Highlights) own
//! ONE thing per kind that depends on who a texel's neighbours are: the low-pass. On the
//! image grid it is the separable blur; on a SURFACE — the sculpt piece's samples folded into
//! the canvas rows — it is the heat kernel of the surface's graph, `e^{−tA}`, applied by the
//! Chebyshev polynomial the CALLER's law computes (`ph2d_mesh_colors::difusao`, the CPU
//! reference). This module holds the graph (CSR buffers) and runs that polynomial between two
//! work textures (`surface_heat.wgsl`); the stages, the combine, the mask and the blend stay
//! the compositor's, untouched. Without a surface installed nothing here runs.
//!
//! ⛔ Motion and Chroma read the image PLANE; a surface never receives them (the caller
//! refuses them) — if one arrives anyway, its low-pass is the identity.

use super::*;
use wgpu::util::DeviceExt;

/// The `HeatPass` uniform of `surface_heat.wgsl` (one per pass, at a 256-byte stride).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct HeatPass {
    n: u32,
    total: u32,
    width: u32,
    mode: u32,
    coef: f32,
    escala: f32,
    _pad0: u32,
    _pad1: u32,
}

/// The dynamic-offset stride of the per-pass uniforms (the WebGPU minimum alignment).
const PASS_STRIDE: u64 = 256;
/// `cs_heat_step`'s workgroup (1-D over the samples).
const STEP_GROUP: u32 = 256;
/// The largest workgroup count per dispatch dimension WebGPU guarantees.
const MAX_GROUPS: u32 = 65_535;

/// The surface's graph, as the caller's law laid it out (`Difusao::csr`).
pub struct SurfaceGraph<'a> {
    /// Row starts, `n + 1` entries.
    pub ini: &'a [u32],
    /// Neighbours.
    pub viz: &'a [u32],
    /// The weight of each entry.
    pub peso: &'a [f32],
    /// `1 / m_a` per sample.
    pub inv_massa: &'a [f32],
    /// The spectrum's ceiling (the polynomial's interval is `[0, λ_sup]`).
    pub lambda_sup: f32,
}

/// Why a surface cannot be installed — the caller composes on the CPU.
#[derive(Debug, PartialEq, Eq)]
pub enum SurfaceRefusal {
    /// A buffer is larger than the device binds (`bytes` over `limit`).
    TooLarge { bytes: u64, limit: u64 },
    /// The graph does not describe `n ≤ width · height` samples.
    Inconsistent,
}

/// ⭐⭐⭐ **A surface the compositor low-passes over** — the graph on the device and the
/// polynomial (`sigma → coefficients`) of the caller's law.
pub struct SurfaceNeighbourhood {
    key: u64,
    n: u32,
    width: u32,
    total: u32,
    escala: f32,
    polynomial: Box<dyn Fn(f32) -> Vec<f32> + Send + Sync>,
    ini: wgpu::Buffer,
    viz: wgpu::Buffer,
    peso: wgpu::Buffer,
    inv_massa: wgpu::Buffer,
}

impl SurfaceNeighbourhood {
    /// Uploads `graph` for a canvas of `width × height` texels (sample `i` ↔ texel
    /// `(i % width, i / width)`). `key` names the graph for the caller (a new key = a new
    /// graph); `polynomial(sigma)` returns the coefficients the CPU applies.
    ///
    /// # Errors
    /// [`SurfaceRefusal`] — the graph does not fit the canvas or the device's bindings.
    pub fn new(
        gpu: &GpuContext,
        key: u64,
        (width, height): (u32, u32),
        graph: &SurfaceGraph<'_>,
        polynomial: Box<dyn Fn(f32) -> Vec<f32> + Send + Sync>,
    ) -> Result<Self, SurfaceRefusal> {
        let n = graph.inv_massa.len();
        let total = u64::from(width) * u64::from(height);
        let consistent = graph.ini.len() == n + 1
            && graph.viz.len() == graph.peso.len()
            && graph
                .ini
                .last()
                .is_some_and(|&e| e as usize == graph.viz.len())
            && (n as u64) <= total
            && graph.lambda_sup > 0.0
            && graph.lambda_sup.is_finite();
        if !consistent {
            return Err(SurfaceRefusal::Inconsistent);
        }
        let limits = gpu.device.limits();
        let limit = limits
            .max_storage_buffer_binding_size
            .min(limits.max_buffer_size);
        let largest = (total * 16).max(graph.viz.len() as u64 * 4);
        if largest > limit {
            return Err(SurfaceRefusal::TooLarge {
                bytes: largest,
                limit,
            });
        }
        let upload = |label: &str, bytes: &[u8]| {
            let dummy = [0u8; 4];
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: if bytes.is_empty() { &dummy } else { bytes },
                    usage: wgpu::BufferUsages::STORAGE,
                })
        };
        Ok(Self {
            key,
            n: n as u32,
            width,
            total: total as u32,
            escala: 2.0 / graph.lambda_sup,
            polynomial,
            ini: upload("ph2d-render surface ini", bytemuck::cast_slice(graph.ini)),
            viz: upload("ph2d-render surface viz", bytemuck::cast_slice(graph.viz)),
            peso: upload("ph2d-render surface peso", bytemuck::cast_slice(graph.peso)),
            inv_massa: upload(
                "ph2d-render surface inv_massa",
                bytemuck::cast_slice(graph.inv_massa),
            ),
        })
    }

    /// The caller's name for this graph.
    #[must_use]
    pub fn key(&self) -> u64 {
        self.key
    }
}

/// The heat pipelines + their scratch, built on the first installed surface.
pub(super) struct HeatGpu {
    load: wgpu::ComputePipeline,
    step: wgpu::ComputePipeline,
    store: wgpu::ComputePipeline,
    bgl_load: wgpu::BindGroupLayout,
    bgl_step: wgpu::BindGroupLayout,
    bgl_store: wgpu::BindGroupLayout,
    passes: Option<(wgpu::Buffer, u64)>,
    /// `u`, `T₀..T₂`, `y` — `total` texels each.
    scratch: Option<(u32, [wgpu::Buffer; 5])>,
}

/// Which work texture a heat pass reads or writes.
#[derive(Clone, Copy)]
pub(super) enum WorkSel {
    Base(usize),
    Blur(usize),
    Sh(usize),
}

fn entry_buffer(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn buf(binding: u32, b: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: b.as_entire_binding(),
    }
}

fn entry_pass() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: true,
            min_binding_size: wgpu::BufferSize::new(size_of::<HeatPass>() as u64),
        },
        count: None,
    }
}

impl HeatGpu {
    pub(super) fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render surface_heat shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../shaders/surface_heat.wgsl").into(),
                ),
            });
        let bgl = |label: &str, entries: &[wgpu::BindGroupLayoutEntry]| {
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(label),
                    entries,
                })
        };
        let bgl_load = bgl(
            "ph2d-render surface_heat load bgl",
            &[
                entry_pass(),
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                entry_buffer(7, false),
                entry_buffer(11, false),
            ],
        );
        let bgl_step = bgl(
            "ph2d-render surface_heat step bgl",
            &[
                entry_pass(),
                entry_buffer(1, true),
                entry_buffer(2, true),
                entry_buffer(3, true),
                entry_buffer(4, true),
                entry_buffer(8, true),
                entry_buffer(9, true),
                entry_buffer(10, false),
                entry_buffer(11, false),
            ],
        );
        let bgl_store = bgl(
            "ph2d-render surface_heat store bgl",
            &[
                entry_pass(),
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                entry_buffer(11, false),
            ],
        );
        let pipeline = |layout_bgl: &wgpu::BindGroupLayout, entry: &str| {
            let layout = gpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(entry),
                    bind_group_layouts: &[Some(layout_bgl)],
                    immediate_size: 0,
                });
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(entry),
                    layout: Some(&layout),
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                })
        };
        Self {
            load: pipeline(&bgl_load, "cs_heat_load"),
            step: pipeline(&bgl_step, "cs_heat_step"),
            store: pipeline(&bgl_store, "cs_heat_store"),
            bgl_load,
            bgl_step,
            bgl_store,
            passes: None,
            scratch: None,
        }
    }

    fn ensure(&mut self, gpu: &GpuContext, total: u32, passes: usize) {
        if self.scratch.as_ref().is_none_or(|(t, _)| *t != total) {
            let make = |label: &str| {
                gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size: u64::from(total.max(1)) * 16,
                    usage: wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                })
            };
            self.scratch = Some((
                total,
                [
                    make("ph2d-render surface u"),
                    make("ph2d-render surface t0"),
                    make("ph2d-render surface t1"),
                    make("ph2d-render surface t2"),
                    make("ph2d-render surface y"),
                ],
            ));
        }
        let needed = passes as u64 * PASS_STRIDE;
        if self.passes.as_ref().is_none_or(|(_, cap)| *cap < needed) {
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-render surface passes"),
                size: needed.next_power_of_two().max(PASS_STRIDE),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.passes = Some((buffer, needed.next_power_of_two().max(PASS_STRIDE)));
        }
    }
}

impl LayerCompositor {
    /// ⭐ **Install (or remove) the surface the neighbourhood kinds low-pass over.** With one
    /// installed, every Gaussian / Sharpen / Bloom / Shadows-Highlights blur stage runs the
    /// surface heat instead of the separable grid blur; the canvas must be the surface's
    /// fold, composed whole.
    pub fn set_surface(&mut self, gpu: &GpuContext, surface: Option<SurfaceNeighbourhood>) {
        if surface.is_some() && self.heat.is_none() {
            self.heat = Some(HeatGpu::new(gpu));
        }
        self.surface = surface;
    }

    /// The key of the installed surface (`None` = the image grid).
    #[must_use]
    pub fn surface_key(&self) -> Option<u64> {
        self.surface.as_ref().map(SurfaceNeighbourhood::key)
    }

    /// ⭐⭐⭐ **The surface low-pass `src → dst`**: `e^{−tA}` with `t = σ²/2`, premultiplied
    /// on read when `premul` (the grid's first blur pass does the same). `sigma ≤ 0` copies.
    pub(super) fn run_surface_heat(
        &mut self,
        gpu: &GpuContext,
        src: WorkSel,
        dst: WorkSel,
        sigma: f32,
        premul: bool,
    ) {
        let Self {
            surface,
            heat,
            work,
            ..
        } = self;
        let (Some(s), Some(h), Some(work)) = (surface.as_ref(), heat.as_mut(), work.as_ref())
        else {
            return;
        };
        let view = |w: WorkSel| match w {
            WorkSel::Base(i) => &work.base[i].view,
            WorkSel::Blur(i) => &work.blur[i].view,
            WorkSel::Sh(i) => &work.sh[i].view,
        };
        let coefs = (s.polynomial)(sigma);
        let pass = |mode: u32, coef: f32| HeatPass {
            n: s.n,
            total: s.total,
            width: s.width,
            mode,
            coef,
            escala: s.escala,
            _pad0: 0,
            _pad1: 0,
        };
        // [load, step₀ … step_{K−1}, store], each at a 256-byte stride.
        let mut uniforms = vec![pass(u32::from(premul), 0.0)];
        for (k, &d) in coefs.iter().enumerate() {
            uniforms.push(pass(k.min(2) as u32, d));
        }
        uniforms.push(pass(0, 0.0));
        h.ensure(gpu, s.total, uniforms.len());
        let (Some((pass_buf, _)), Some((_, [u, t0, t1, t2, y]))) = (&h.passes, &h.scratch) else {
            return;
        };
        let mut bytes = vec![0u8; uniforms.len() * PASS_STRIDE as usize];
        for (i, p) in uniforms.iter().enumerate() {
            let o = i * PASS_STRIDE as usize;
            bytes[o..o + size_of::<HeatPass>()].copy_from_slice(bytemuck::bytes_of(p));
        }
        gpu.queue.write_buffer(pass_buf, 0, &bytes);
        let pass_binding = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: pass_buf,
            offset: 0,
            size: wgpu::BufferSize::new(size_of::<HeatPass>() as u64),
        });
        let load_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render surface_heat load bg"),
            layout: &h.bgl_load,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pass_binding.clone(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(view(src)),
                },
                buf(7, u),
                buf(11, y),
            ],
        });
        // `(t_cur, t_prev, t_out)` of each step: T₀ from u; then the three T's rotate.
        let ts = [t0, t1, t2];
        let step_bg = |cur: &wgpu::Buffer, prev: &wgpu::Buffer, out: &wgpu::Buffer| {
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render surface_heat step bg"),
                layout: &h.bgl_step,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: pass_binding.clone(),
                    },
                    buf(1, &s.ini),
                    buf(2, &s.viz),
                    buf(3, &s.peso),
                    buf(4, &s.inv_massa),
                    buf(8, cur),
                    buf(9, prev),
                    buf(10, out),
                    buf(11, y),
                ],
            })
        };
        let step_bgs: Vec<wgpu::BindGroup> = (0..coefs.len().min(5))
            .map(|k| match k {
                0 => step_bg(u, ts[2], ts[0]),
                1 => step_bg(ts[0], ts[2], ts[1]),
                k => step_bg(ts[(k - 1) % 3], ts[(k - 2) % 3], ts[k % 3]),
            })
            .collect();
        let store_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render surface_heat store bg"),
            layout: &h.bgl_store,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pass_binding,
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(view(dst)),
                },
                buf(11, y),
            ],
        });
        let height = s.total.div_ceil(s.width.max(1));
        let texel_groups = (
            s.width.div_ceil(WORKGROUP_EDGE),
            height.div_ceil(WORKGROUP_EDGE),
        );
        let groups = s.n.div_ceil(STEP_GROUP);
        let step_groups = (groups.min(MAX_GROUPS), groups.div_ceil(MAX_GROUPS));
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render surface_heat"),
            });
        {
            let mut cp = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-render surface_heat"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("render.surface_heat"),
            });
            let offset = |i: usize| (i as u64 * PASS_STRIDE) as u32;
            cp.set_pipeline(&h.load);
            cp.set_bind_group(0, &load_bg, &[offset(0)]);
            cp.dispatch_workgroups(texel_groups.0, texel_groups.1, 1);
            if groups > 0 {
                cp.set_pipeline(&h.step);
                for k in 0..coefs.len() {
                    // The rotation repeats every 3 from k = 2: bind groups 2..=4.
                    let bg = if k < 2 { k } else { 2 + (k - 2) % 3 };
                    cp.set_bind_group(0, &step_bgs[bg], &[offset(1 + k)]);
                    cp.dispatch_workgroups(step_groups.0, step_groups.1, 1);
                }
            }
            cp.set_pipeline(&h.store);
            cp.set_bind_group(0, &store_bg, &[offset(uniforms.len() - 1)]);
            cp.dispatch_workgroups(texel_groups.0, texel_groups.1, 1);
        }
        gpu.queue.submit([encoder.finish()]);
    }
}
