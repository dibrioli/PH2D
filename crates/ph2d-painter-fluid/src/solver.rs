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

/// The GPU dab-splat shader (the resident-sim input pass — 4K real-time arch §4).
/// Splats a tiny `array<Dab>` directly onto the resident water + pigment, replacing
/// the per-frame full-grid `deposit` upload. Mirrors the CPU `DiffusionGrid::splat`
/// (same covered cells + falloff; ~1e-7 FMA-noise — `cs_splat_matches_cpu_splat`).
pub const SPLAT_WGSL: &str = include_str!("shader/splat.wgsl");

/// The GPU field-reduction shader (max-water + wet bbox), so the CPU never scans
/// the O(grid) water mirror (4K real-time arch §4). Read back sporadically.
pub const REDUCE_WGSL: &str = include_str!("shader/reduce.wgsl");

/// The GPU pigment-deposition shader (`cs_transfer`, ADR-0078 S3) — the GPU mirror of
/// the CPU `DiffusionGrid::transfer_pigment`: freezes flowing pigment into the
/// deposited layer (edge-darkening + granulation). Dormant while deposition params 0.
pub const TRANSFER_WGSL: &str = include_str!("shader/transfer.wgsl");

/// The GPU combine shader (`cs_combine`, ADR-0078 S3c) — writes `total = flowing +
/// deposited` so the compositor reads the whole pigment (wash + frozen layer).
pub const COMBINE_WGSL: &str = include_str!("shader/combine.wgsl");

/// Tasteful default watercolor deposition (ADR-0078 S3c) — applied to every live fluid
/// stroke via [`FluidSolver::set_deposition`]. Tuned for a visible-but-natural dark
/// perimeter (`_DRY`) + paper granulation (`_GRAN`) without muddying the wash (`_BASE`
/// kept low). A later FluidParams amendment can make these per-brush.
pub const WATERCOLOR_DEPOSITION_BASE: f32 = 0.012;
pub const WATERCOLOR_DEPOSITION_DRY: f32 = 0.10;
pub const WATERCOLOR_GRANULATION: f32 = 1.4;

/// Result of [`FluidSolver::read_field_stats`]: the max wetness anywhere (the
/// dry-check signal) + the wet bounding box (`None` when the field is bone-dry).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FieldStats {
    /// Largest `water` value in the field — the field is "dry" when this falls below
    /// the tool's dry threshold.
    pub max_water: f32,
    /// `(x0, y0, x1, y1)` inclusive bbox of cells wetter than the query threshold,
    /// or `None` if no cell is wet.
    pub bbox: Option<(u32, u32, u32, u32)>,
}

/// Upper bound on dabs splatted in ONE `cs_splat` dispatch. A frame's dab count is
/// tens; the persistent `dabs` storage buffer is sized to this (`× 32 B` = 128 KiB)
/// so the bind group is fixed. A frame exceeding it is processed in sequential
/// chunks (each its own pass → wgpu inserts the RAW barrier, so the accumulation
/// order — and thus the bit-exact parity — is preserved across chunks).
pub const MAX_DABS_PER_DISPATCH: usize = 4096;

/// A single brush dab for [`FluidSolver::splat_dabs`]: centre, effective radius,
/// per-touch water, and the (pre-weighted) linear-RGB pigment. `#[repr(C)]` Pod so
/// the dab list is a zero-copy `write_buffer` into the `array<Dab>` storage binding
/// (32 B std430 stride — `rgb` is vec3-aligned at offset 16, matching `splat.wgsl`).
///
/// Build with [`DabGpu::new`] so the `radius <= 0` early-return and the `r.max(0.5)`
/// floor exactly mirror the CPU `DiffusionGrid::splat` (a raw `r <= 0` would make the
/// shader's `1.0 / r` non-finite).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DabGpu {
    cx: f32,
    cy: f32,
    r: f32,
    water_add: f32,
    rgb: [f32; 3],
    _pad: f32,
}

impl DabGpu {
    /// A dab at `(cx, cy)` with brush `radius`, raising wetness by `water_add` and
    /// depositing linear-RGB `rgb` (already weighted by opacity/deposit, as the CPU
    /// splat receives it). Returns `None` for `radius <= 0` (the CPU splat's
    /// early-return — such a touch contributes nothing); otherwise the radius is
    /// floored to `0.5` (the CPU `radius.max(0.5)`).
    #[must_use]
    pub fn new(cx: f32, cy: f32, radius: f32, water_add: f32, rgb: [f32; 3]) -> Option<Self> {
        if radius <= 0.0 {
            return None;
        }
        Some(Self {
            cx,
            cy,
            r: radius.max(0.5),
            water_add,
            rgb,
            _pad: 0.0,
        })
    }
}

/// The `cs_splat` uniform: grid size + the dispatch's union-bbox origin + the live
/// dab count. 32 B (`#[repr(C)]` Pod), padded to a 16-B multiple for the UBO.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSplatParams {
    width: u32,
    height: u32,
    origin_x: u32,
    origin_y: u32,
    n_dabs: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}

/// The `cs_reduce` uniform: grid size + the wet-bbox threshold. 16 B Pod.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuReduceParams {
    width: u32,
    height: u32,
    threshold: f32,
    _pad: u32,
}

/// Run the CPU reference solver `steps` times in place. The det-mode path AND the
/// parity reference for the GPU pass — both go through the one shipped solver.
pub fn step_cpu_reference(grid: &mut DiffusionGrid, params: &FluidParams, steps: u32) {
    let dp = params.to_diffusion();
    for _ in 0..steps {
        grid.step(&dp);
    }
}

/// The WGSL `Params` UBO, byte-for-byte (16 × 4 = 64 B). `#[repr(C)]` Pod so a
/// per-frame update is a zero-copy `write_buffer` of `bytemuck::bytes_of` (HR-3).
/// The `region_*` fields scope the diffuse/advect/evaporate dispatch to the wet
/// envelope (ADR-0078 S1) — full-grid `(0, 0, width, height)` is the un-scoped pass.
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
    region_ox: u32,
    region_oy: u32,
    region_w: u32,
    region_h: u32,
    deposition: f32,
    deposition_dry: f32,
    granulation: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

/// Pad (cells) added around the composite envelope for the region-scoped solver
/// dispatch (ADR-0078 S1). Must keep the solver region ⊇ the composite read region
/// so the visible field is always fully stepped (no frozen-pigment clip — the §2
/// lesson). `composite_canvas_region` reads `env − 2 .. env + 3` grid cells (+ ~1
/// for the bicubic tap); 6 covers it with margin. Cells beyond are never composited.
const SOLVER_REGION_PAD: u32 = 6;

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
    /// W15.3 resident path: `pig_a += deposit` (the dabs splatted this frame).
    deposit_pipe: wgpu::ComputePipeline,
    params_buf: wgpu::Buffer,
    /// CPU mirror of the params UBO (the per-stroke coeffs from `set_params`). The
    /// region fields are overwritten per dispatch (`write_params_with_region`) so the
    /// solver can scope diffuse/advect/evaporate to the wet envelope (ADR-0078 S1)
    /// without re-deriving the coeffs each frame. `Cell` keeps the step methods `&self`.
    params_cache: std::cell::Cell<GpuParams>,
    // All field buffers are owned so they outlive the bind groups that reference
    // them (the readback reads `pig_a`; `upload` writes water/paper/pig_a).
    water: wgpu::Buffer,
    paper: wgpu::Buffer,
    pig_a: wgpu::Buffer,
    /// Owned only to keep the GPU buffer alive for the ping-pong bind groups
    /// (`bg_diffuse` writes it, `bg_advect` reads it); never touched directly.
    #[allow(dead_code)]
    pig_b: wgpu::Buffer,
    /// W15.3 resident path: the per-frame dab deposit (uploaded, added to `pig_a`).
    deposit: wgpu::Buffer,
    bg_diffuse: wgpu::BindGroup,
    bg_advect: wgpu::BindGroup,
    bg_evaporate: wgpu::BindGroup,
    /// Deposit add: `pig_in = deposit`, `pig_out = pig_a` (read_write, in-place).
    bg_deposit: wgpu::BindGroup,
    /// Resident-sim input pass: `cs_splat` adds a dab list to `water` + `pig_a`
    /// directly (replaces the full-grid `deposit` upload — 4K real-time arch §4).
    splat: wgpu::ComputePipeline,
    splat_params_buf: wgpu::Buffer,
    /// Persistent `array<Dab>` storage (sized to [`MAX_DABS_PER_DISPATCH`]); the
    /// live dabs are written to its front each call.
    dabs_buf: wgpu::Buffer,
    /// `cs_splat` bind group: (splat_params, water rw, pig_a rw, dabs read).
    bg_splat: wgpu::BindGroup,
    /// GPU field reduction (max-water + wet bbox) so the dry-check + envelope need no
    /// CPU O(grid) scan (4K real-time arch §4). `stats_buf` holds 5 u32 atomics.
    reduce: wgpu::ComputePipeline,
    reduce_params_buf: wgpu::Buffer,
    stats_buf: wgpu::Buffer,
    bg_reduce: wgpu::BindGroup,
    /// Pigment-deposition pass (`cs_transfer`, ADR-0078 S3): freezes flowing pigment
    /// (`pig_a`) into the resident `deposited` layer (edge-darkening + granulation).
    /// Own bind-group layout — both pigment buffers are read_write.
    transfer: wgpu::ComputePipeline,
    deposited: wgpu::Buffer,
    bg_transfer: wgpu::BindGroup,
    /// Combine pass (`cs_combine`, ADR-0078 S3c): `total = flowing + deposited`, the
    /// buffer the compositor reads (so deposited pigment is visible).
    combine: wgpu::ComputePipeline,
    total: wgpu::Buffer,
    bg_combine: wgpu::BindGroup,
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
        let deposit = buf("fluid deposit", vec4n, wgpu::BufferUsages::empty());
        // Resident deposited-pigment layer (ADR-0078 S3); COPY_SRC for the parity readback.
        let deposited = buf("fluid deposited", vec4n, wgpu::BufferUsages::COPY_SRC);
        // total = flowing + deposited (ADR-0078 S3c) — the buffer the compositor reads.
        let total = buf("fluid total", vec4n, wgpu::BufferUsages::COPY_SRC);
        // bind group: (params, water, paper, pig_in, pig_out).
        let bg = |label: &str, pin: &wgpu::Buffer, pout: &wgpu::Buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: water.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: paper.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: pin.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: pout.as_entire_binding(),
                    },
                ],
            })
        };
        let bg_diffuse = bg("fluid bg diffuse", &pig_a, &pig_b); // A→B
        let bg_advect = bg("fluid bg advect", &pig_b, &pig_a); // B→A
        let bg_evaporate = bg("fluid bg evaporate", &pig_a, &pig_b); // pig unused
        // Deposit add: pig_in = deposit (read), pig_out = pig_a (read_write) → in-place.
        let bg_deposit = bg("fluid bg deposit", &deposit, &pig_a);

        // ── Resident-sim input pass (`cs_splat`, its own bind-group layout) ──
        let splat_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-painter-fluid splat"),
            source: wgpu::ShaderSource::Wgsl(SPLAT_WGSL.into()),
        });
        let splat_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-painter-fluid splat bgl"),
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
                storage(1, false), // water (read_write)
                storage(2, false), // pig_a (read_write)
                storage(3, true),  // dabs (read)
            ],
        });
        let splat_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-painter-fluid splat layout"),
            bind_group_layouts: &[&splat_bgl],
            immediate_size: 0,
        });
        let splat = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-painter-fluid splat pipeline"),
            layout: Some(&splat_layout),
            module: &splat_module,
            entry_point: Some("cs_splat"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let splat_params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-painter-fluid splat params"),
            size: core::mem::size_of::<GpuSplatParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let dabs_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-painter-fluid dabs"),
            size: (MAX_DABS_PER_DISPATCH * core::mem::size_of::<DabGpu>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bg_splat = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-painter-fluid bg splat"),
            layout: &splat_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: splat_params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: water.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: pig_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: dabs_buf.as_entire_binding(),
                },
            ],
        });

        // ── GPU field reduction (`cs_reduce`, its own bind-group layout) ──
        let reduce_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-painter-fluid reduce"),
            source: wgpu::ShaderSource::Wgsl(REDUCE_WGSL.into()),
        });
        let reduce_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-painter-fluid reduce bgl"),
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
                storage(1, true),  // water (read)
                storage(2, false), // stats (atomic read_write)
            ],
        });
        let reduce_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-painter-fluid reduce layout"),
            bind_group_layouts: &[&reduce_bgl],
            immediate_size: 0,
        });
        let reduce = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-painter-fluid reduce pipeline"),
            layout: Some(&reduce_layout),
            module: &reduce_module,
            entry_point: Some("cs_reduce"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let reduce_params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-painter-fluid reduce params"),
            size: core::mem::size_of::<GpuReduceParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let stats_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-painter-fluid stats"),
            size: (5 * core::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let bg_reduce = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-painter-fluid bg reduce"),
            layout: &reduce_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: reduce_params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: water.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: stats_buf.as_entire_binding(),
                },
            ],
        });

        // ── Pigment-deposition pass (`cs_transfer`, its own bind-group layout) ──
        let transfer_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-painter-fluid transfer"),
            source: wgpu::ShaderSource::Wgsl(TRANSFER_WGSL.into()),
        });
        let transfer_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-painter-fluid transfer bgl"),
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
                storage(1, true),  // water (read)
                storage(2, true),  // paper (read)
                storage(3, false), // flowing = pig_a (read_write)
                storage(4, false), // deposited (read_write)
            ],
        });
        let transfer_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-painter-fluid transfer layout"),
            bind_group_layouts: &[&transfer_bgl],
            immediate_size: 0,
        });
        let transfer = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-painter-fluid transfer pipeline"),
            layout: Some(&transfer_layout),
            module: &transfer_module,
            entry_point: Some("cs_transfer"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let bg_transfer = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-painter-fluid bg transfer"),
            layout: &transfer_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: water.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: paper.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: pig_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: deposited.as_entire_binding(),
                },
            ],
        });

        // ── Combine pass (`cs_combine`: total = flowing + deposited) ──
        let combine_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-painter-fluid combine"),
            source: wgpu::ShaderSource::Wgsl(COMBINE_WGSL.into()),
        });
        let combine_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-painter-fluid combine bgl"),
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
                storage(1, true),  // flowing = pig_a (read)
                storage(2, true),  // deposited (read)
                storage(3, false), // total (read_write)
            ],
        });
        let combine_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-painter-fluid combine layout"),
            bind_group_layouts: &[&combine_bgl],
            immediate_size: 0,
        });
        let combine = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-painter-fluid combine pipeline"),
            layout: Some(&combine_layout),
            module: &combine_module,
            entry_point: Some("cs_combine"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let bg_combine = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-painter-fluid bg combine"),
            layout: &combine_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pig_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: deposited.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: total.as_entire_binding(),
                },
            ],
        });
        Self {
            width,
            height,
            diffuse: pipe("cs_diffuse"),
            advect: pipe("cs_advect"),
            evaporate: pipe("cs_evaporate"),
            deposit_pipe: pipe("cs_deposit"),
            params_buf,
            // Coeffs filled by `set_params`; region defaults to the full grid.
            params_cache: std::cell::Cell::new(GpuParams {
                width,
                height,
                diffusivity: 0.0,
                evaporation: 0.0,
                downhill: 0.0,
                flow_outward: 0.0,
                w_lo: 0.0,
                w_hi: 0.0,
                perm_valley: 0.0,
                perm_crest: 0.0,
                region_ox: 0,
                region_oy: 0,
                region_w: width,
                region_h: height,
                deposition: 0.0,
                deposition_dry: 0.0,
                granulation: 0.0,
                _pad0: 0.0,
                _pad1: 0.0,
                _pad2: 0.0,
            }),
            water,
            paper,
            pig_a,
            pig_b,
            deposit,
            bg_diffuse,
            bg_advect,
            bg_evaporate,
            bg_deposit,
            splat,
            splat_params_buf,
            dabs_buf,
            bg_splat,
            reduce,
            reduce_params_buf,
            stats_buf,
            bg_reduce,
            transfer,
            deposited,
            bg_transfer,
            combine,
            total,
            bg_combine,
        }
    }

    /// Upload the initial field. `pigment` is the linear-RGB mass (xyz; w ignored).
    /// Lengths must equal `width * height`.
    pub fn upload(&self, queue: &wgpu::Queue, water: &[f32], paper: &[f32], pigment: &[[f32; 4]]) {
        queue.write_buffer(&self.water, 0, bytemuck::cast_slice(water));
        queue.write_buffer(&self.paper, 0, bytemuck::cast_slice(paper));
        queue.write_buffer(&self.pig_a, 0, bytemuck::cast_slice(pigment));
    }

    /// Push the solver coefficients (mirrors the CPU `DiffusionParams`). Caches them
    /// CPU-side (with a full-grid region) so the per-frame region-scoped writer can
    /// reuse them; uploads with the full-grid region (the un-scoped default).
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
            region_ox: 0,
            region_oy: 0,
            region_w: self.width,
            region_h: self.height,
            // Deposition stays off until `set_deposition` (S3b parity tests) or the
            // S3c FluidParams amendment enables it — preserves the GPU parity gates.
            deposition: 0.0,
            deposition_dry: 0.0,
            granulation: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        self.params_cache.set(gp);
        queue.write_buffer(&self.params_buf, 0, bytemuck::bytes_of(&gp));
    }

    /// Enable the pigment-deposition layer (ADR-0078 S3) on the GPU `cs_transfer`
    /// pass: mirrors `DiffusionParams::{deposition, deposition_dry, granulation}`.
    /// Updates the cached params (the per-frame region writer carries them forward);
    /// uploads immediately so a subsequent `step()`/`step_resident_splat` sees them.
    /// `0,0,0` (the default from `set_params`) leaves the pass a no-op.
    pub fn set_deposition(
        &self,
        queue: &wgpu::Queue,
        deposition: f32,
        deposition_dry: f32,
        granulation: f32,
    ) {
        let mut gp = self.params_cache.get();
        gp.deposition = deposition;
        gp.deposition_dry = deposition_dry;
        gp.granulation = granulation;
        self.params_cache.set(gp);
        queue.write_buffer(&self.params_buf, 0, bytemuck::bytes_of(&gp));
    }

    /// Write the cached coeffs with a specific dispatch region (ADR-0078 S1) + return
    /// the workgroup counts to dispatch. `region = (x0, y0, x1, y1)` inclusive, in
    /// grid cells; clamped to the grid. The diffuse/advect/evaporate kernels only
    /// write cells inside it (full grid = the un-scoped pass).
    fn write_params_with_region(
        &self,
        queue: &wgpu::Queue,
        region: (u32, u32, u32, u32),
    ) -> (u32, u32) {
        let (x0, y0, x1, y1) = region;
        let x0 = x0.min(self.width - 1);
        let y0 = y0.min(self.height - 1);
        let x1 = x1.min(self.width - 1).max(x0);
        let y1 = y1.min(self.height - 1).max(y0);
        let (rw, rh) = (x1 - x0 + 1, y1 - y0 + 1);
        let mut gp = self.params_cache.get();
        gp.region_ox = x0;
        gp.region_oy = y0;
        gp.region_w = rw;
        gp.region_h = rh;
        queue.write_buffer(&self.params_buf, 0, bytemuck::bytes_of(&gp));
        (rw.div_ceil(8), rh.div_ceil(8))
    }

    /// Run `substeps` diffusion-advection-evaporation steps on the GPU (one
    /// submit). After this, the pigment is in `pig_a`; read it with [`Self::read_pigment`].
    pub fn step(&self, device: &wgpu::Device, queue: &wgpu::Queue, substeps: u32) {
        // Full-grid region (resets any prior scoped dispatch on a reused solver).
        let (gx, gy) =
            self.write_params_with_region(queue, (0, 0, self.width - 1, self.height - 1));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid step"),
        });
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

    /// The GPU-resident deposited-pigment buffer (ADR-0078 S3, `array<vec4<f32>>`).
    /// Exposed so the compositor can add it to the flowing pigment (S3c) — the
    /// composited colour is `flowing + deposited`.
    #[must_use]
    pub fn deposited_buffer(&self) -> &wgpu::Buffer {
        &self.deposited
    }

    /// The combined-pigment buffer (`total = flowing + deposited`, ADR-0078 S3c) —
    /// **bind THIS to the compositor** (not `pigment_buffer`) so the deposited layer
    /// (edge-darkening + granulation) is visible. Refreshed each `step_resident_splat`
    /// by `cs_combine`; equals `flowing` when nothing is deposited (look unchanged).
    #[must_use]
    pub fn total_buffer(&self) -> &wgpu::Buffer {
        &self.total
    }

    /// Field dimensions (`width`, `height`) — the pigment buffer holds `width*height`
    /// `vec4<f32>` cells, the layout the compositor's `gw`/`gh` describe.
    #[must_use]
    pub fn dims(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Upload the static paper-height field (W15.3 resident path — call ONCE per
    /// stroke; the tooth doesn't change while painting).
    pub fn upload_paper(&self, queue: &wgpu::Queue, paper: &[f32]) {
        queue.write_buffer(&self.paper, 0, bytemuck::cast_slice(paper));
    }

    /// Zero the resident pigment (`pig_a`) — call at stroke begin so a reused solver
    /// (same grid size, new stroke) starts from a bare field (W15.3 resident path).
    pub fn clear_resident_pigment(&self, queue: &wgpu::Queue) {
        let zeros = vec![[0.0f32; 4]; (self.width as usize) * (self.height as usize)];
        queue.write_buffer(&self.pig_a, 0, bytemuck::cast_slice(&zeros));
    }

    /// Zero the resident pigment ON the GPU (`clear_buffer`, no CPU upload) — the
    /// fast stroke-begin reset. Avoids the per-stroke megabyte upload of a zero buffer
    /// that `clear_resident_pigment` does (a hitch on large canvases).
    pub fn clear_resident_pigment_gpu(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid clear pig_a"),
        });
        enc.clear_buffer(&self.pig_a, 0, None);
        queue.submit([enc.finish()]);
    }

    /// Zero the resident water ON the GPU (companion to
    /// [`Self::clear_resident_pigment_gpu`]) — the resident-sim stroke-begin reset so
    /// a reused solver starts from a dry field before the first `cs_splat`.
    pub fn clear_resident_water_gpu(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid clear water"),
        });
        enc.clear_buffer(&self.water, 0, None);
        queue.submit([enc.finish()]);
    }

    /// Zero the resident deposited-pigment layer ON the GPU (ADR-0078 S3) — the
    /// stroke-begin reset for the `cs_transfer` output, alongside the pigment/water clears.
    pub fn clear_resident_deposited_gpu(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid clear deposited"),
        });
        enc.clear_buffer(&self.deposited, 0, None);
        queue.submit([enc.finish()]);
    }

    /// **Splat a dab list onto the resident water + pigment (`cs_splat`).** The
    /// resident-sim input pass (4K real-time arch §4): the tool pushes the dabs it
    /// emitted this frame (O(dabs) ~ tens) and they are added DIRECTLY to the GPU
    /// buffers — no full-grid `deposit` upload, no CPU alloc/scan. Mirrors the CPU
    /// `DiffusionGrid::splat` (same shape; ~1e-7 FMA-noise — `cs_splat_matches_cpu_splat`),
    /// so the live look is unchanged. One submit, no readback (the field stays
    /// GPU-resident); a frame
    /// with more than [`MAX_DABS_PER_DISPATCH`] dabs is split into sequential chunks
    /// (separate passes → ordered, so the accumulation stays bit-exact). No-op for an
    /// empty list.
    pub fn splat_dabs(&self, device: &wgpu::Device, queue: &wgpu::Queue, dabs: &[DabGpu]) {
        if dabs.is_empty() {
            return;
        }
        // One submit per chunk: the chunk's dabs are `write_buffer`d into the shared
        // `dabs_buf` then consumed by that submit, so a later chunk can't clobber the
        // buffer before an earlier chunk's pass runs (single-chunk = one submit).
        for chunk in dabs.chunks(MAX_DABS_PER_DISPATCH) {
            // Union bbox of the chunk (clamped to the grid) bounds the dispatch; the
            // shader's `dist < 1` cutoff then rejects corners exactly as the CPU does.
            let (mut x0, mut y0, mut x1, mut y1) = (self.width - 1, self.height - 1, 0u32, 0u32);
            for d in chunk {
                let lo_x = ((d.cx - d.r).floor().max(0.0) as u32).min(self.width - 1);
                let lo_y = ((d.cy - d.r).floor().max(0.0) as u32).min(self.height - 1);
                let hi_x = ((d.cx + d.r).ceil().max(0.0) as u32).min(self.width - 1);
                let hi_y = ((d.cy + d.r).ceil().max(0.0) as u32).min(self.height - 1);
                x0 = x0.min(lo_x);
                y0 = y0.min(lo_y);
                x1 = x1.max(hi_x);
                y1 = y1.max(hi_y);
            }
            if x1 < x0 || y1 < y0 {
                continue;
            }
            queue.write_buffer(&self.dabs_buf, 0, bytemuck::cast_slice(chunk));
            let sp = GpuSplatParams {
                width: self.width,
                height: self.height,
                origin_x: x0,
                origin_y: y0,
                n_dabs: chunk.len() as u32,
                _p0: 0,
                _p1: 0,
                _p2: 0,
            };
            queue.write_buffer(&self.splat_params_buf, 0, bytemuck::bytes_of(&sp));
            let (gx, gy) = ((x1 - x0 + 1).div_ceil(8), (y1 - y0 + 1).div_ceil(8));
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("fluid splat"),
            });
            {
                let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("fluid splat pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.splat);
                pass.set_bind_group(0, &self.bg_splat, &[]);
                pass.dispatch_workgroups(gx, gy, 1);
            }
            queue.submit([enc.finish()]);
        }
    }

    /// **Fully GPU-resident per-frame step (4K real-time arch §4 — no upload, no
    /// readback).** The hot loop the 4K rewrite targets: this frame's `dabs` are
    /// splatted onto the resident water + pigment by `cs_splat` (no full-grid deposit
    /// upload), then `substeps` of diffuse → advect → **evaporate** run on the GPU
    /// (water is resident now, so evaporation moves OFF the CPU). Nothing comes back
    /// to the CPU — the field stays in `water`/`pig_a` across frames; the compositor
    /// reads `pig_a` directly and the dry-check/bbox come from [`Self::read_field_stats`]
    /// (sporadic). Equivalent to the CPU `splat`-then-`step×substeps` reference (the
    /// `step_resident_splat_matches_cpu` gate), so the live look is unchanged.
    ///
    /// `set_params` + `clear_resident_{water,pigment}_gpu` + `upload_paper` are the
    /// driver's once-per-stroke setup (epoch change); this is the per-frame call.
    ///
    /// **Region-scoped (ADR-0078 S1):** `region = (x0, y0, x1, y1)` is the wet
    /// envelope (inclusive grid cells); the diffuse/advect/evaporate dispatch is
    /// scoped to `region` padded by [`SOLVER_REGION_PAD`] (clamped to the grid) — the
    /// per-frame cost becomes `O(wet frontier)`, not `O(grid)`. The pad keeps the
    /// solver region ⊇ the composite read region, so the visible field is always fully
    /// stepped (no frozen-pigment clip). Pass the full grid `(0,0,w-1,h-1)` for the
    /// un-scoped pass (e.g. parity tests). `cs_splat` is already scoped to its own dab
    /// union bbox; water only ever exists inside the envelope, so scoped evaporate
    /// still dries every wet cell.
    pub fn step_resident_splat(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        dabs: &[DabGpu],
        substeps: u32,
        region: (u32, u32, u32, u32),
    ) {
        // Splat submit (resident water + pig_a updated); ordered before the step submit
        // on the queue timeline, so diffuse reads the just-splatted field.
        self.splat_dabs(device, queue, dabs);
        // Pad the envelope so the solver region ⊇ what the compositor reads.
        let p = SOLVER_REGION_PAD;
        let padded = (
            region.0.saturating_sub(p),
            region.1.saturating_sub(p),
            region.2.saturating_add(p),
            region.3.saturating_add(p),
        );
        let (gx, gy) = self.write_params_with_region(queue, padded);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid resident splat step"),
        });
        for _ in 0..substeps {
            // Per substep: diffuse A→B, advect B→A, TRANSFER (flowing→deposited),
            // evaporate water — the CPU `step` order (pigment ends back in pig_a;
            // transfer freezes from pig_a; evaporate touches only water). `cs_transfer`
            // is a no-op while the deposition params are 0 (the GPU parity default).
            for (pipe, bg) in [
                (&self.diffuse, &self.bg_diffuse),
                (&self.advect, &self.bg_advect),
                (&self.transfer, &self.bg_transfer),
                (&self.evaporate, &self.bg_evaporate),
            ] {
                let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("fluid resident splat pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(pipe);
                pass.set_bind_group(0, bg, &[]);
                pass.dispatch_workgroups(gx, gy, 1);
            }
        }
        // Combine once per frame (region-scoped, after the substeps): total = flowing +
        // deposited — the buffer the compositor reads (ADR-0078 S3c). Params still hold
        // the padded region, so `total` is fresh wherever the composite samples it.
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("fluid combine pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.combine);
            pass.set_bind_group(0, &self.bg_combine, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        queue.submit([enc.finish()]);
        // No poll: the field stays GPU-resident; the compositor's readback (or a
        // sporadic `read_field_stats`) is the only sync point.
    }

    /// **Reduce the resident water field to its max + wet bbox ON the GPU** (4K
    /// real-time arch §4), so the dry-check + composite envelope need no CPU O(grid)
    /// scan. One atomic pass + a tiny 5-u32 readback (this DOES sync the queue, so
    /// call it SPORADICALLY — drying is slow, a few frames of latency is invisible).
    /// `threshold` selects which cells count as wet for the bbox (the tool's
    /// `water_bbox` threshold). The reported `max_water` is over the WHOLE field.
    #[must_use]
    pub fn read_field_stats(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        threshold: f32,
    ) -> FieldStats {
        // Seed the atomics: max=0, min_x/min_y=U32_MAX, max_x/max_y=0.
        let init: [u32; 5] = [0, u32::MAX, u32::MAX, 0, 0];
        queue.write_buffer(&self.stats_buf, 0, bytemuck::cast_slice(&init));
        let rp = GpuReduceParams {
            width: self.width,
            height: self.height,
            threshold,
            _pad: 0,
        };
        queue.write_buffer(&self.reduce_params_buf, 0, bytemuck::bytes_of(&rp));
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluid stats readback"),
            size: (5 * core::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let (gx, gy) = (self.width.div_ceil(8), self.height.div_ceil(8));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid reduce"),
        });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("fluid reduce pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.reduce);
            pass.set_bind_group(0, &self.bg_reduce, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        enc.copy_buffer_to_buffer(&self.stats_buf, 0, &staging, 0, staging.size());
        queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let mapped = staging.slice(..).get_mapped_range();
        let s: [u32; 5] = bytemuck::cast_slice::<u8, u32>(&mapped)[..5]
            .try_into()
            .expect("5 u32 stats");
        drop(mapped);
        staging.unmap();
        let max_water = f32::from_bits(s[0]);
        let bbox = if s[1] == u32::MAX {
            None // no cell crossed the threshold → bone-dry
        } else {
            Some((s[1], s[2], s[3], s[4]))
        };
        FieldStats { max_water, bbox }
    }

    /// **W15.3 resident per-frame step (no readback).** Keeps the bloomed pigment
    /// GPU-resident in `pig_a`: uploads the CPU water mirror (the CPU owns water +
    /// evaporation + the dry-check) and this frame's dab `deposit`, adds the deposit
    /// to `pig_a`, then runs `substeps` of diffuse+advect (NO GPU evaporate — water
    /// came pre-evaporated from the CPU). One submit, no `device.poll` wait, no
    /// pigment readback — the composite reads `pig_a` directly afterwards (its own
    /// readback, or a GPU→GPU copy, syncs the queue). `water`/`deposit` lengths must
    /// equal `width*height`.
    pub fn step_resident(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        water: &[f32],
        deposit: &[[f32; 4]],
        substeps: u32,
    ) {
        queue.write_buffer(&self.water, 0, bytemuck::cast_slice(water));
        queue.write_buffer(&self.deposit, 0, bytemuck::cast_slice(deposit));
        let (gx, gy) = (self.width.div_ceil(8), self.height.div_ceil(8));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid step resident"),
        });
        // pig_a += deposit (the dabs this frame), then diffuse+advect ×substeps.
        let mut pass_seq: Vec<(&wgpu::ComputePipeline, &wgpu::BindGroup)> =
            vec![(&self.deposit_pipe, &self.bg_deposit)];
        for _ in 0..substeps {
            pass_seq.push((&self.diffuse, &self.bg_diffuse)); // A→B
            pass_seq.push((&self.advect, &self.bg_advect)); // B→A
        }
        for (pipe, bg) in pass_seq {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("fluid resident pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, bg, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        queue.submit([enc.finish()]);
        // No poll: pigment stays GPU-resident in `pig_a`; the next queue submit (the
        // compositor) is ordered after this one and its readback (if any) syncs.
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
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid rb"),
        });
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

    /// Map the current deposited-pigment field back (ADR-0078 S3 — companion to
    /// [`Self::read_pigment`], for the `cs_transfer` parity gate + the pen-up bake).
    #[must_use]
    pub fn read_deposited(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<[f32; 4]> {
        let n = (self.width as usize) * (self.height as usize);
        let size = (n * 16) as u64;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluid deposited readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid deposited rb"),
        });
        enc.copy_buffer_to_buffer(&self.deposited, 0, &staging, 0, size);
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

    /// Map the combined-pigment field (`total`, ADR-0078 S3c) back — the buffer the
    /// compositor reads. For the `cs_combine` gate + the pen-up bake.
    #[must_use]
    pub fn read_total(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<[f32; 4]> {
        let n = (self.width as usize) * (self.height as usize);
        let size = (n * 16) as u64;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluid total readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid total rb"),
        });
        enc.copy_buffer_to_buffer(&self.total, 0, &staging, 0, size);
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
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid water rb"),
        });
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
