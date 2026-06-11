//! The GPU watercolor solver (ADR-0085: the GPU is the single live source of truth —
//! the CPU twin in `ph2d_painter_brush::diffusion` is no longer a runtime fallback).
//! [`FluidSolver`] owns the resident pigment/water/deposited fields + the compute
//! pipelines; the physics is validated by the GPU-only invariant gates
//! (`tests/physical_invariants.rs`), not a bit-for-bit CPU reference.
//!
//! [`FLUID_WGSL`] is the compute shader for the gated diffusion-advection. The
//! diffusion pass is a pure GATHER (each cell sums conductance·(neighbour−self)),
//! which maps directly to a compute kernel. The advection pass is reformulated
//! from the original SCATTER (push to the downstream neighbour) into a GATHER (each
//! cell pulls the net flux from its 4 neighbours) so it is atomics-free and
//! order-independent — the adaptation that makes it correct on the GPU. The wgpu
//! pipeline (storage buffers, bind groups, dispatch, region upload) runs against a
//! headless `GpuContext` in the invariant gates.

use crate::params::FluidParams;
use ph2d_painter_brush::diffusion::{DiffusionGrid, DiffusionParams, PIG_CH, RELAX_ITERS, WetCell};

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

/// The GPU shallow-water shader (`cs_add_forces` / `cs_divergence` / `cs_clear_pressure` /
/// `cs_jacobi` / `cs_project` / `cs_advect_velocity`, ADR-0078 S3d) — the GPU mirror of the
/// CPU `DiffusionGrid::move_water` (+ velocity-mode advect): a momentum-carrying velocity
/// field `(u,v)` + pressure projection → directional flow + backruns. Dormant while the
/// velocity master scale is 0.
pub const SHALLOW_WGSL: &str = include_str!("shader/shallow.wgsl");

/// The GPU capillary-fringe shader (`cs_capillary` + `cs_copy_water`, ADR-0078 S5) — the GPU
/// mirror of the CPU `DiffusionGrid::capillary_flow`: wicks water outward into the dry paper
/// so the wet gate (then the pigment) creeps into the soft fringe. Two region-scoped passes
/// around the `water_b` ping-pong scratch. Dormant while the capillary rate is 0.
pub const CAPILLARY_WGSL: &str = include_str!("shader/capillary.wgsl");

/// Tasteful default watercolor deposition (ADR-0078 S3c) — applied to every live fluid
/// stroke via [`FluidSolver::set_deposition`]. Tuned for a visible-but-natural dark
/// perimeter (`_DRY`) + paper granulation (`_GRAN`) without muddying the wash (`_BASE`
/// kept low). A later FluidParams amendment can make these per-brush.
pub const WATERCOLOR_DEPOSITION_BASE: f32 = 0.012;
pub const WATERCOLOR_DEPOSITION_DRY: f32 = 0.10;
pub const WATERCOLOR_GRANULATION: f32 = 1.4;

/// Tasteful default shallow-water velocity tuning (ADR-0078 S3d) — applied to every live
/// fluid stroke via [`FluidSolver::set_shallow_water`]. `_VELOCITY` is the master scale on
/// the velocity field's pigment transport (0 ⇒ dormant, the gated-diffusion look);
/// `_VISCOSITY`/`_DRAG` keep the flow coherent + settling; `_PRESSURE` is the projection
/// strength that builds the directional flow + backrun rings. Tuned for a natural wash
/// (2026-06-08 Enio feedback: the first pass read as excess large-scale noise + hard edges):
/// higher `_VISCOSITY` smooths the velocity field (kills any collocated-grid checkerboard +
/// softens), lower `_PRESSURE` keeps the backrun ring present but gentle (natural edges). The
/// paper-tooth mottling was fixed separately — the `−β·∇h` force is gone from `add_forces`
/// (see `diffusion.rs`). A later FluidParams amendment (S4) can make these per-brush.
pub const WATERCOLOR_VELOCITY: f32 = 1.3;
pub const WATERCOLOR_VISCOSITY: f32 = 0.18;
pub const WATERCOLOR_DRAG: f32 = 0.1;
pub const WATERCOLOR_PRESSURE: f32 = 0.3;

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

/// **Watercolor v2 (ADR-0085) — planar (struct-of-arrays) pigment layout.** The GPU field
/// stores channel-vec4 `v` of cell `c` at `pidx(c,v) = v·n + c` (channel-major) so neighbour
/// gathers coalesce. `WetCell` is `PIG_CH` floats **cell-major** (8 vec4 contiguous per cell);
/// these two transpose at the CPU↔GPU boundary (uploads + parity readbacks). The live path is
/// GPU-resident (`cs_splat` writes planar directly), so it never transposes — only the lift
/// upload + the parity read/write paths do. Float `k` of channel-vec4 `v` of cell `c` lives at
/// SoA float index `(v·n + c)·4 + k`, vs cell-major `c·PIG_CH + (v·4 + k)`.
pub(crate) fn pack_soa(cells: &[WetCell]) -> Vec<f32> {
    let n = cells.len();
    let flat: &[f32] = bytemuck::cast_slice(cells); // n·PIG_CH floats, cell-major
    let mut out = vec![0.0f32; n * PIG_CH];
    for c in 0..n {
        for ch in 0..PIG_CH {
            // ch = v·4 + k  ⇒  SoA float index (v·n + c)·4 + k = (ch/4)·n·4 + c·4 + ch%4.
            out[(ch / 4) * n * 4 + c * 4 + (ch % 4)] = flat[c * PIG_CH + ch];
        }
    }
    out
}

/// Inverse of [`pack_soa`]: planar GPU floats → cell-major `WetCell`s (parity readbacks).
fn unpack_soa(soa: &[f32], n: usize) -> Vec<WetCell> {
    let mut flat = vec![0.0f32; n * PIG_CH];
    for c in 0..n {
        for ch in 0..PIG_CH {
            flat[c * PIG_CH + ch] = soa[(ch / 4) * n * 4 + c * 4 + (ch % 4)];
        }
    }
    bytemuck::cast_slice::<f32, WetCell>(&flat).to_vec()
}

/// A single brush dab for [`FluidSolver::splat_dabs`]: centre, effective radius,
/// per-touch water, and the (pre-weighted) linear-RGB pigment. `#[repr(C)]` Pod so
/// the dab list is a zero-copy `write_buffer` into the `array<Dab>` storage binding
/// (144 B std430 stride — the leading 4 f32 fill the first 16 B, then `pig` is 8 vec4
/// (the 32-channel pigment incl. stain, ADR-0080/0081) at offset 16, matching `splat.wgsl`).
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
    /// The dab's per-cell pigment channels (mass-weighted K/S + err + mass + stain), already
    /// scaled by the peak deposit mass — `cs_splat` adds `pig * fall`. (ADR-0080 K/S; ADR-0081
    /// stain in the `PIG_STAIN` channel; was a gray RGB.)
    pig: [f32; PIG_CH],
}

impl DabGpu {
    /// A dab at `(cx, cy)` with brush `radius`, raising wetness by `water_add` and depositing
    /// a pigment of `color` (linear sRGB) at peak coverage `pigment_mass` with `staining` ∈ [0,1]
    /// (ADR-0081) — the GPU mirror of the CPU `DiffusionGrid::splat(cx, cy, r, water_add, color,
    /// pigment_mass, staining)`. The pigment channels are built once via the same
    /// `cell_from_color_mass` (ADR-0080), then the mass-weighted staining is laid into the
    /// `PIG_STAIN` channel (matching the CPU splat's `cell[PIG_STAIN] += staining·mass`), so a
    /// cell adds `pig·fall`. Returns `None` for `radius <= 0` (the CPU splat's early-return);
    /// else the radius is floored to `0.5` (the CPU `radius.max(0.5)`).
    #[must_use]
    pub fn new(
        cx: f32,
        cy: f32,
        radius: f32,
        water_add: f32,
        color: [f32; 3],
        pigment_mass: f32,
        staining: f32,
    ) -> Option<Self> {
        if radius <= 0.0 {
            return None;
        }
        // Mass-weighted K/S + err + mass channels of the dab (ADR-0080), then the staining
        // accumulator rides along mass-weighted (ADR-0081), exactly like the CPU splat — both
        // scale by `fall` per cell in `cs_splat`, so `stain·mass·fall` lands per touched cell.
        let mut pig = DiffusionGrid::cell_from_color_mass(color, pigment_mass);
        pig[ph2d_painter_brush::diffusion::PIG_STAIN] += staining * pigment_mass;
        Some(Self {
            cx,
            cy,
            r: radius.max(0.5),
            water_add,
            pig,
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

/// The WGSL `Params` UBO, byte-for-byte (28 × 4 = 112 B, 16-aligned). `#[repr(C)]` Pod so a
/// per-frame update is a zero-copy `write_buffer` of `bytemuck::bytes_of` (HR-3). Each shader
/// declares its own smaller `Params` view of this one buffer (a uniform binding only requires
/// `shader struct ≤ buffer`), reading only the fields it needs; only `transfer.wgsl`'s `cs_lift`
/// reads `lift` (offset 24) and only `capillary.wgsl` reads `capillary_branching` (offset 25,
/// ADR-0082). The `region_*` fields scope the diffuse/advect/evaporate dispatch to
/// the wet envelope (ADR-0078 S1) — full-grid `(0, 0, width, height)` is the un-scoped pass.
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
    // ── ADR-0078 S3d shallow-water layer ── these occupy the bytes (offsets 68..84) the
    // diffuse/advect/transfer/combine shaders treat as padding, so those shaders keep a
    // valid 80-B view of this 96-B UBO; only `shallow.wgsl` reads these fields.
    velocity: f32,
    viscosity: f32,
    drag: f32,
    pressure: f32,
    // ── ADR-0078 S5 capillary layer ── occupy the bytes the diffuse/advect/transfer/combine
    // shaders read as padding; only `capillary.wgsl` reads them. `capillary_mobility` = the
    // chromatographic pigment-filtering factor (water wicks ahead of the lagging pigment).
    capillary: f32,
    capillary_mobility: f32,
    // ADR-0078 S5c — BFECC/MacCormack advection sharpness (read by `cs_advect_correct`).
    sharpness: f32,
    // ── ADR-0081 lift ── re-mobilizes NON-staining deposited pigment back into the flowing
    // layer in wet cells (read by `cs_lift`, offset 24). 0 ⇒ the lift pass is dormant.
    lift: f32,
    // ── ADR-0082 branched capillary fringe ── fiber-channeled suppression of the capillary
    // per-face conductance (read by `capillary.wgsl` at offset 25, the slot a `lift` pad held).
    // The 2 trailing pads round the struct to 28 f32 = 112 B (16-aligned — required for the
    // uniform buffer). 0 ⇒ the isotropic capillary is bit-identical (opt-in, ADR-0082).
    capillary_branching: f32,
    _pad_lift1: f32,
    _pad_lift2: f32,
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
    /// Persistent readback staging for the PIPELINED dry-check
    /// ([`Self::read_field_stats_pipelined`]) + its in-flight map. The blocking
    /// sync [`Self::read_field_stats`] (parity tests) allocates its own local
    /// staging; the live drive uses this async pair so the dry-check never does a
    /// per-fire `poll(wait)` queue-drain (the FPS-collapse feedback loop).
    stats_staging: wgpu::Buffer,
    pending_stats: Option<std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>>,
    /// Last stat read back (returned while the next fire's map is in flight). Seeded
    /// "wet, no bbox" so the first fires never spuriously drop the field.
    last_stats: FieldStats,
    /// Pigment-deposition pass (`cs_transfer`, ADR-0078 S3): freezes flowing pigment
    /// (`pig_a`) into the resident `deposited` layer (edge-darkening + granulation).
    /// Own bind-group layout — both pigment buffers are read_write.
    transfer: wgpu::ComputePipeline,
    deposited: wgpu::Buffer,
    bg_transfer: wgpu::BindGroup,
    /// Lift pass (`cs_lift`, ADR-0081 + ADR-0084): the inverse of `cs_transfer` — re-mobilizes
    /// NON-staining `deposited` pigment back into `pig_a` in wet cells, AND (ADR-0084) re-mobilizes
    /// the backdrop-lift donor (`lift_source`) into `pig_a` while accumulating `lifted_frac`. Shares
    /// the `transfer_bgl` layout + `bg_transfer` bind group (flowing/deposited/water + the new
    /// lift_source/lifted_frac bindings). Dormant while `lift` is 0 (the dispatch is gated on it).
    lift: wgpu::ComputePipeline,
    /// **Backdrop-lift donor (ADR-0084)** — `array<vec4<f32>>` (8 vec4/cell = PIG_CH=32), the
    /// downsampled canvas backdrop ("dry paint available to lift"). Seeded once per stroke
    /// ([`Self::upload_lift_source`]) when `lift > 0`, depleted into `pig_a` by `cs_lift`. COPY_SRC
    /// for the parity readback. All-zero while dormant → the backdrop branch is a no-op.
    lift_source: wgpu::Buffer,
    /// **Lifted fraction ∈ [0,1] per cell (ADR-0084)** — `array<f32>`, how much of the backdrop
    /// pigment `cs_lift` re-mobilized out of the paper. Resident (accumulates across substeps); read
    /// by the compositor to drop the backdrop alpha. COPY_SRC for parity readback. ≡ 0 while dormant.
    lifted_frac: wgpu::Buffer,
    /// Combine pass (`cs_combine`, ADR-0078 S3c): `total = flowing + deposited`, the
    /// buffer the compositor reads (so deposited pigment is visible).
    combine: wgpu::ComputePipeline,
    total: wgpu::Buffer,
    bg_combine: wgpu::BindGroup,
    // ── Shallow-water velocity layer (ADR-0078 S3d) ──
    add_forces: wgpu::ComputePipeline,
    divergence_pipe: wgpu::ComputePipeline,
    clear_pressure: wgpu::ComputePipeline,
    jacobi: wgpu::ComputePipeline,
    project: wgpu::ComputePipeline,
    advect_velocity: wgpu::ComputePipeline,
    /// Velocity ping-pong (`vec4`? no — `vec2<f32>` per cell): `vel_a` is canonical
    /// (advect + readback read it), `vel_b` the `add_forces`/`project` scratch.
    vel_a: wgpu::Buffer,
    vel_b: wgpu::Buffer,
    /// Pressure Jacobi ping-pong + the divergence (the Jacobi RHS).
    pressure_a: wgpu::Buffer,
    pressure_b: wgpu::Buffer,
    /// Owned only to keep the GPU buffer alive for the divergence/jacobi bind groups
    /// (recomputed each frame by `cs_divergence`, never cleared/read directly).
    #[allow(dead_code)]
    sw_divergence: wgpu::Buffer,
    bg_add_forces: wgpu::BindGroup,
    bg_divergence: wgpu::BindGroup,
    bg_clear_pa: wgpu::BindGroup,
    bg_clear_pb: wgpu::BindGroup,
    bg_jacobi_ab: wgpu::BindGroup,
    bg_jacobi_ba: wgpu::BindGroup,
    bg_project: wgpu::BindGroup,
    bg_advect_velocity: wgpu::BindGroup,
    // ── BFECC/MacCormack advection sharpness (ADR-0078 S5c) ──
    advect_velocity_rev: wgpu::ComputePipeline,
    advect_correct: wgpu::ComputePipeline,
    /// MacCormack reverse-advected field φ̄ (the round-trip-error estimate). Owned only to keep
    /// the GPU buffer alive for the rev/correct bind groups; only touched when `sharpness > 0`.
    #[allow(dead_code)]
    pig_c: wgpu::Buffer,
    bg_advect_velocity_rev: wgpu::BindGroup,
    bg_advect_correct: wgpu::BindGroup,
    // ── Capillary fringe (ADR-0078 S5) ──
    capillary_pipe: wgpu::ComputePipeline,
    copy_fields_pipe: wgpu::ComputePipeline,
    /// Water ping-pong scratch — `cs_capillary` writes the wicked water field here, `cs_copy_fields`
    /// lands it back in `water` (region-scoped). Owned only to keep the GPU buffer alive for the
    /// two bind groups that reference it (the pigment scratch reuses the existing `pig_b`).
    #[allow(dead_code)]
    water_b: wgpu::Buffer,
    bg_capillary: wgpu::BindGroup,
    bg_copy_fields: wgpu::BindGroup,
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
        // Pigment buffers carry PIG_CH (=32) channels per cell = 8 vec4 (ADR-0080/0081):
        // size = n · 32 · 4 bytes. (Was n · 16 when pigment was one vec4.)
        let vec4n = (n * PIG_CH * 4) as u64;
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
        // ADR-0084 backdrop-lift donor (8 vec4/cell) + lifted-fraction accumulator (f32/cell);
        // COPY_SRC for the parity readback. All-zero (dormant) until a `lift > 0` stroke seeds them.
        let lift_source = buf("fluid lift_source", vec4n, wgpu::BufferUsages::COPY_SRC);
        let lifted_frac = buf("fluid lifted_frac", f32n, wgpu::BufferUsages::COPY_SRC);
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
        let stats_staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-painter-fluid stats staging (pipelined)"),
            size: (5 * core::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
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
                storage(5, false), // lift_source (read_write — ADR-0084, depleted by cs_lift)
                storage(6, false), // lifted_frac (read_write — ADR-0084, accumulated by cs_lift)
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
        // Lift pass (`cs_lift`, ADR-0081) — same module, layout + bind group as transfer.
        let lift = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-painter-fluid lift pipeline"),
            layout: Some(&transfer_layout),
            module: &transfer_module,
            entry_point: Some("cs_lift"),
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
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: lift_source.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: lifted_frac.as_entire_binding(),
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

        // ── Shallow-water velocity layer (`shallow.wgsl`, ADR-0078 S3d) ──
        // One module, focused per-pass bind-group layouts (the ping-pong pattern). The 4
        // S3d UBO fields share `params_buf` (they sit in this 96-B struct's bytes the other
        // shaders read as padding). Velocity is `vec2<f32>` (8 B/cell); pressure/divergence
        // are `f32`.
        let shallow_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-painter-fluid shallow"),
            source: wgpu::ShaderSource::Wgsl(SHALLOW_WGSL.into()),
        });
        let uniform_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let sw_layout = |bgl: &wgpu::BindGroupLayout| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-painter-fluid shallow layout"),
                bind_group_layouts: &[bgl],
                immediate_size: 0,
            })
        };
        let sw_pipe = |entry: &str, layout: &wgpu::PipelineLayout| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ph2d-painter-fluid shallow pipeline"),
                layout: Some(layout),
                module: &shallow_module,
                entry_point: Some(entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let vec2n = (n * 8) as u64;
        let vel_a = buf("fluid vel_a", vec2n, wgpu::BufferUsages::COPY_SRC);
        let vel_b = buf("fluid vel_b", vec2n, wgpu::BufferUsages::empty());
        let pressure_a = buf("fluid pressure_a", f32n, wgpu::BufferUsages::empty());
        let pressure_b = buf("fluid pressure_b", f32n, wgpu::BufferUsages::empty());
        let sw_divergence = buf("fluid sw divergence", f32n, wgpu::BufferUsages::empty());
        fn entry(binding: u32, b: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
            wgpu::BindGroupEntry {
                binding,
                resource: b.as_entire_binding(),
            }
        }

        // add_forces: (params, water r, paper r, vel_in r=vel_a, vel_out rw=vel_b).
        let bgl_af = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid sw add_forces bgl"),
            entries: &[
                uniform_entry(0),
                storage(1, true),
                storage(2, true),
                storage(3, true),
                storage(4, false),
            ],
        });
        let add_forces = sw_pipe("cs_add_forces", &sw_layout(&bgl_af));
        let bg_add_forces = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid sw bg add_forces"),
            layout: &bgl_af,
            entries: &[
                entry(0, &params_buf),
                entry(1, &water),
                entry(2, &paper),
                entry(3, &vel_a),
                entry(4, &vel_b),
            ],
        });

        // divergence: (params, vel_in r=vel_b, divergence rw).
        let bgl_div = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid sw divergence bgl"),
            entries: &[uniform_entry(0), storage(3, true), storage(7, false)],
        });
        let divergence_pipe = sw_pipe("cs_divergence", &sw_layout(&bgl_div));
        let bg_divergence = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid sw bg divergence"),
            layout: &bgl_div,
            entries: &[
                entry(0, &params_buf),
                entry(3, &vel_b),
                entry(7, &sw_divergence),
            ],
        });

        // clear_pressure: (params, pressure_out rw). Two bind groups (seed a, seed b).
        let bgl_clear = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid sw clear bgl"),
            entries: &[uniform_entry(0), storage(6, false)],
        });
        let clear_pressure = sw_pipe("cs_clear_pressure", &sw_layout(&bgl_clear));
        let bg_clear_pa = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid sw bg clear pa"),
            layout: &bgl_clear,
            entries: &[entry(0, &params_buf), entry(6, &pressure_a)],
        });
        let bg_clear_pb = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid sw bg clear pb"),
            layout: &bgl_clear,
            entries: &[entry(0, &params_buf), entry(6, &pressure_b)],
        });

        // jacobi: (params, pressure_in r, divergence rw, pressure_out rw). Two bind groups
        // ping-pong a→b / b→a; RELAX_ITERS even → result lands in pressure_a.
        let bgl_jac = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid sw jacobi bgl"),
            entries: &[
                uniform_entry(0),
                storage(5, true),
                storage(7, false),
                storage(6, false),
            ],
        });
        let jacobi = sw_pipe("cs_jacobi", &sw_layout(&bgl_jac));
        let bg_jacobi_ab = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid sw bg jacobi ab"),
            layout: &bgl_jac,
            entries: &[
                entry(0, &params_buf),
                entry(5, &pressure_a),
                entry(7, &sw_divergence),
                entry(6, &pressure_b),
            ],
        });
        let bg_jacobi_ba = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid sw bg jacobi ba"),
            layout: &bgl_jac,
            entries: &[
                entry(0, &params_buf),
                entry(5, &pressure_b),
                entry(7, &sw_divergence),
                entry(6, &pressure_a),
            ],
        });

        // project: (params, vel_in r=vel_b, pressure_in r=pressure_a, vel_out rw=vel_a).
        let bgl_proj = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid sw project bgl"),
            entries: &[
                uniform_entry(0),
                storage(3, true),
                storage(5, true),
                storage(4, false),
            ],
        });
        let project = sw_pipe("cs_project", &sw_layout(&bgl_proj));
        let bg_project = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid sw bg project"),
            layout: &bgl_proj,
            entries: &[
                entry(0, &params_buf),
                entry(3, &vel_b),
                entry(5, &pressure_a),
                entry(4, &vel_a),
            ],
        });

        // advect_velocity: (params, water r, paper r, vel_in r=vel_a, pig_in r=pig_b,
        // pig_out rw=pig_a) — the velocity-mode pigment transport (B→A, like cs_advect).
        let bgl_av = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid sw advect_velocity bgl"),
            entries: &[
                uniform_entry(0),
                storage(1, true),
                storage(2, true),
                storage(3, true),
                storage(8, true),
                storage(9, false),
            ],
        });
        let advect_velocity = sw_pipe("cs_advect_velocity", &sw_layout(&bgl_av));
        let bg_advect_velocity = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid sw bg advect_velocity"),
            layout: &bgl_av,
            entries: &[
                entry(0, &params_buf),
                entry(1, &water),
                entry(2, &paper),
                entry(3, &vel_a),
                entry(8, &pig_b),
                entry(9, &pig_a),
            ],
        });

        // ── BFECC/MacCormack advection sharpness (`shallow.wgsl`, ADR-0078 S5c) ──
        // The backward pass `cs_advect_velocity_rev` reverse-advects φ̂ (pig_a) → φ̄ (pig_c) with
        // the SAME `bgl_av` layout (negated flow lives in the shader, not a uniform); the
        // `cs_advect_correct` pass forms `φ̂ + sharpness·½(φ−φ̄)` clamped to φ's extrema, reading
        // φ=pig_b (8), φ̄=pig_c (10), φ̂=pig_a (9 rw, in place). Region-scoped like every sw pass.
        let pig_c = buf("fluid pig_c", vec4n, wgpu::BufferUsages::empty());
        let advect_velocity_rev = sw_pipe("cs_advect_velocity_rev", &sw_layout(&bgl_av));
        let bg_advect_velocity_rev = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid sw bg advect_velocity_rev"),
            layout: &bgl_av,
            entries: &[
                entry(0, &params_buf),
                entry(1, &water),
                entry(2, &paper),
                entry(3, &vel_a),
                entry(8, &pig_a), // φ̂ in
                entry(9, &pig_c), // φ̄ out
            ],
        });
        let bgl_correct = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid sw advect_correct bgl"),
            entries: &[
                uniform_entry(0),
                storage(8, true),  // φ (pig_b)
                storage(9, false), // φ̂ in place → φ_new (pig_a)
                storage(10, true), // φ̄ (pig_c)
            ],
        });
        let advect_correct = sw_pipe("cs_advect_correct", &sw_layout(&bgl_correct));
        let bg_advect_correct = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid sw bg advect_correct"),
            layout: &bgl_correct,
            entries: &[
                entry(0, &params_buf),
                entry(8, &pig_b),
                entry(9, &pig_a),
                entry(10, &pig_c),
            ],
        });

        // ── Capillary fringe (`capillary.wgsl`, ADR-0078 S5) ──
        // Two region-scoped passes around a `water_b` ping-pong scratch: `cs_capillary` wicks
        // water a→b, `cs_copy_water` lands it back b→a (the canonical buffer every other pass
        // reads next frame). Shares `params_buf` (`capillary` sits in the bytes the other
        // shaders read as padding). Own per-pass bind-group layouts (the ping-pong idiom).
        let capillary_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-painter-fluid capillary"),
            source: wgpu::ShaderSource::Wgsl(CAPILLARY_WGSL.into()),
        });
        let cap_pipe = |entry_point: &str, layout: &wgpu::PipelineLayout| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ph2d-painter-fluid capillary pipeline"),
                layout: Some(layout),
                module: &capillary_module,
                entry_point: Some(entry_point),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let water_b = buf("fluid water_b", f32n, wgpu::BufferUsages::empty());
        // cs_capillary: (params, water_in r=water, paper r, water_out rw=water_b,
        //                pig_in r=pig_a, pig_out rw=pig_b). Wicks water + co-advects pigment.
        let bgl_cap = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid capillary bgl"),
            entries: &[
                uniform_entry(0),
                storage(1, true),
                storage(2, true),
                storage(3, false),
                storage(4, true),
                storage(5, false),
            ],
        });
        let capillary_pipe = cap_pipe("cs_capillary", &sw_layout(&bgl_cap));
        let bg_capillary = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid bg capillary"),
            layout: &bgl_cap,
            entries: &[
                entry(0, &params_buf),
                entry(1, &water),
                entry(2, &paper),
                entry(3, &water_b),
                entry(4, &pig_a),
                entry(5, &pig_b),
            ],
        });
        // cs_copy_fields: (params, water_in r=water_b, water_out rw=water, pig_in r=pig_b,
        //                  pig_out rw=pig_a) — land both wicked fields back in the canonical buffers.
        let bgl_copy = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid copy_fields bgl"),
            entries: &[
                uniform_entry(0),
                storage(1, true),
                storage(3, false),
                storage(4, true),
                storage(5, false),
            ],
        });
        let copy_fields_pipe = cap_pipe("cs_copy_fields", &sw_layout(&bgl_copy));
        let bg_copy_fields = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluid bg copy_fields"),
            layout: &bgl_copy,
            entries: &[
                entry(0, &params_buf),
                entry(1, &water_b),
                entry(3, &water),
                entry(4, &pig_b),
                entry(5, &pig_a),
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
                velocity: 0.0,
                viscosity: 0.0,
                drag: 0.0,
                pressure: 0.0,
                capillary: 0.0,
                capillary_mobility: 0.0,
                sharpness: 0.0,
                lift: 0.0,
                capillary_branching: 0.0,
                _pad_lift1: 0.0,
                _pad_lift2: 0.0,
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
            stats_staging,
            pending_stats: None,
            last_stats: FieldStats {
                max_water: 1.0,
                bbox: None,
            },
            transfer,
            deposited,
            bg_transfer,
            lift,
            lift_source,
            lifted_frac,
            combine,
            total,
            bg_combine,
            add_forces,
            divergence_pipe,
            clear_pressure,
            jacobi,
            project,
            advect_velocity,
            vel_a,
            vel_b,
            pressure_a,
            pressure_b,
            sw_divergence,
            bg_add_forces,
            bg_divergence,
            bg_clear_pa,
            bg_clear_pb,
            bg_jacobi_ab,
            bg_jacobi_ba,
            bg_project,
            bg_advect_velocity,
            advect_velocity_rev,
            advect_correct,
            pig_c,
            bg_advect_velocity_rev,
            bg_advect_correct,
            capillary_pipe,
            copy_fields_pipe,
            water_b,
            bg_capillary,
            bg_copy_fields,
        }
    }

    /// Upload the initial field. `pigment` is the linear-RGB mass (xyz; w ignored).
    /// Lengths must equal `width * height`.
    pub fn upload(&self, queue: &wgpu::Queue, water: &[f32], paper: &[f32], pigment: &[WetCell]) {
        queue.write_buffer(&self.water, 0, bytemuck::cast_slice(water));
        queue.write_buffer(&self.paper, 0, bytemuck::cast_slice(paper));
        queue.write_buffer(&self.pig_a, 0, bytemuck::cast_slice(&pack_soa(pigment)));
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
            // Shallow-water velocity layer (ADR-0078 S3d) off until `set_shallow_water`.
            velocity: 0.0,
            viscosity: 0.0,
            drag: 0.0,
            pressure: 0.0,
            // Capillary fringe (ADR-0078 S5) off until `set_from_diffusion` drives it per-brush.
            capillary: 0.0,
            capillary_mobility: 0.0,
            sharpness: 0.0,
            // Lift (ADR-0081) off until `set_from_diffusion` drives it per-brush.
            lift: 0.0,
            // Branched capillary (ADR-0082) off until `set_from_diffusion` drives it per-brush.
            capillary_branching: 0.0,
            _pad_lift1: 0.0,
            _pad_lift2: 0.0,
        };
        self.params_cache.set(gp);
        queue.write_buffer(&self.params_buf, 0, bytemuck::bytes_of(&gp));
    }

    /// Enable the **shallow-water velocity layer** (ADR-0078 S3d) on the GPU: mirrors the
    /// CPU `DiffusionParams::{velocity, viscosity, drag, pressure}`. Updates the cached
    /// params (the per-frame region writer carries them forward) and uploads immediately
    /// so a subsequent `step_resident_splat` runs the `move_water` passes + the
    /// velocity-mode advect. `velocity = 0` (the `set_params` default) leaves the layer
    /// dormant — `step_resident_splat` then runs the static gradient-flow advect, so the
    /// existing parity gates are untouched.
    pub fn set_shallow_water(
        &self,
        queue: &wgpu::Queue,
        velocity: f32,
        viscosity: f32,
        drag: f32,
        pressure: f32,
    ) {
        let mut gp = self.params_cache.get();
        gp.velocity = velocity;
        gp.viscosity = viscosity;
        gp.drag = drag;
        gp.pressure = pressure;
        self.params_cache.set(gp);
        queue.write_buffer(&self.params_buf, 0, bytemuck::bytes_of(&gp));
    }

    /// **Upload ALL 15 solver controls from a [`DiffusionParams`] at once (ADR-0079).** The
    /// consolidated per-brush entry the live bridge calls each stroke: the artist's
    /// `WatercolorParams` (projected to `DiffusionParams`) drives the GPU solver directly,
    /// replacing the old `set_params(default)` + `set_deposition(const)` +
    /// `set_shallow_water(const)` trio. Resets the dispatch region to the full grid (the
    /// per-frame `step_resident_splat` re-scopes it). Those three methods stay for the
    /// parity tests; this is the one the live path uses.
    pub fn set_from_diffusion(&self, queue: &wgpu::Queue, dp: &DiffusionParams) {
        let gp = GpuParams {
            width: self.width,
            height: self.height,
            diffusivity: dp.diffusivity,
            evaporation: dp.evaporation,
            downhill: dp.downhill,
            flow_outward: dp.flow_outward,
            w_lo: dp.w_lo,
            w_hi: dp.w_hi,
            perm_valley: dp.perm_valley,
            perm_crest: dp.perm_crest,
            region_ox: 0,
            region_oy: 0,
            region_w: self.width,
            region_h: self.height,
            deposition: dp.deposition,
            deposition_dry: dp.deposition_dry,
            granulation: dp.granulation,
            velocity: dp.velocity,
            viscosity: dp.viscosity,
            drag: dp.drag,
            pressure: dp.pressure,
            capillary: dp.capillary,
            capillary_mobility: dp.capillary_mobility,
            sharpness: dp.sharpness,
            lift: dp.lift,
            capillary_branching: dp.capillary_branching,
            _pad_lift1: 0.0,
            _pad_lift2: 0.0,
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
        let zeros = vec![[0.0f32; PIG_CH]; (self.width as usize) * (self.height as usize)];
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

    /// **Seed the ADR-0084 backdrop-lift donor.** Upload the `width*height` K–M donor cells
    /// (each [`WetCell`] = 32 f32 = 8 vec4, matching the GPU `array<vec4<f32>>` layout) straight
    /// into `lift_source` — the GPU mirror of the CPU `DiffusionGrid::seed_lift_source_from_backdrop`.
    /// Build `cells` with `ph2d_painter_brush::diffusion::backdrop_to_lift_source` (the SAME free fn
    /// the CPU seed uses) so the donor is bit-identical CPU↔GPU (HR-5). Call ONCE at stroke begin
    /// when `lift > 0`; pair with [`Self::clear_lift_gpu`] (to also zero `lifted_frac`) or re-seed.
    pub fn upload_lift_source(&self, queue: &wgpu::Queue, cells: &[WetCell]) {
        queue.write_buffer(&self.lift_source, 0, bytemuck::cast_slice(&pack_soa(cells)));
    }

    /// Zero BOTH the backdrop-lift donor (`lift_source`) AND the lifted-fraction accumulator
    /// (`lifted_frac`) ON the GPU (ADR-0084) — the stroke-begin reset for the backdrop-lift branch,
    /// alongside the pigment/water/deposited clears. Mirrors [`Self::clear_resident_deposited_gpu`].
    /// Leaving both zero keeps the `cs_lift` backdrop branch a no-op + the compositor byte-identical
    /// (the non-destructive `lift = 0` path).
    pub fn clear_lift_gpu(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid clear lift"),
        });
        enc.clear_buffer(&self.lift_source, 0, None);
        enc.clear_buffer(&self.lifted_frac, 0, None);
        queue.submit([enc.finish()]);
    }

    /// The lifted-fraction buffer (`array<f32>`, ADR-0084) — bound by the compositor to drop the
    /// backdrop alpha where paint was lifted, and read back by the parity test. All-zero (so the
    /// compositor output is byte-identical) until a `lift > 0` stroke seeds + runs the backdrop lift.
    #[must_use]
    pub fn lifted_frac_buffer(&self) -> &wgpu::Buffer {
        &self.lifted_frac
    }

    /// The backdrop-lift donor buffer (`array<vec4<f32>>`, ADR-0084) — for the parity readback /
    /// inspection. Depleted in place by `cs_lift`; all-zero while dormant.
    #[must_use]
    pub fn lift_source_buffer(&self) -> &wgpu::Buffer {
        &self.lift_source
    }

    /// The GPU-resident water buffer (`array<f32>`, `width·height`) — bound by the
    /// compositor's preview-texture passes for the wet-paper sheen (darken wet
    /// regions, brighten the meniscus; display-only). Written by `cs_splat` /
    /// `cs_evaporate`; zeroed each stroke begin via [`Self::clear_resident_water_gpu`].
    #[must_use]
    pub fn water_buffer(&self) -> &wgpu::Buffer {
        &self.water
    }

    /// The canonical shallow-water velocity buffer (`vec2<f32>` per cell, ADR-0078 S3d) — the
    /// projected `(u,v)` `cs_advect_velocity` reads. Exposed for the physical-invariant gate
    /// that asserts the field comes to REST under Keep Wet (the C1 surface-tension-pinning
    /// fixed point, ADR-0085). `COPY_SRC`, so a test can read back the magnitude.
    #[must_use]
    pub fn velocity_buffer(&self) -> &wgpu::Buffer {
        &self.vel_a
    }

    /// Zero the resident shallow-water state ON the GPU (ADR-0078 S3d) — the stroke-begin
    /// reset for the velocity + pressure buffers, alongside the pigment/water/deposited
    /// clears, so a reused solver starts a new stroke with no leftover momentum.
    pub fn clear_resident_velocity_gpu(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid clear velocity"),
        });
        enc.clear_buffer(&self.vel_a, 0, None);
        enc.clear_buffer(&self.vel_b, 0, None);
        enc.clear_buffer(&self.pressure_a, 0, None);
        enc.clear_buffer(&self.pressure_b, 0, None);
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
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("fluid splat"),
            });
            self.encode_splat_chunk(queue, &mut enc, chunk);
            queue.submit([enc.finish()]);
        }
    }

    /// Encode ONE chunk's `cs_splat` pass into a caller-owned encoder (no submit).
    /// The shared `dabs_buf`/`splat_params_buf` are `write_buffer`'d here, so a
    /// single chunk is safe to fold into a larger encoder (Watercolor v2 R1 single
    /// submit, ADR-0085 §2.3-I1); MULTIPLE chunks still need a submit between them
    /// (the buffer would be clobbered) — [`Self::splat_dabs`] keeps that contract.
    /// Empty / zero-area chunks no-op.
    fn encode_splat_chunk(&self, queue: &wgpu::Queue, enc: &mut wgpu::CommandEncoder, chunk: &[DabGpu]) {
        if chunk.is_empty() {
            return;
        }
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
            return;
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
        let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("fluid splat pass"),
            timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.splat"),
        });
        pass.set_pipeline(&self.splat);
        pass.set_bind_group(0, &self.bg_splat, &[]);
        pass.dispatch_workgroups(gx, gy, 1);
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
        let enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid resident splat step"),
        });
        let enc = self.encode_resident_splat_step(device, queue, enc, dabs, substeps, region);
        queue.submit([enc.finish()]);
        // No poll: the field stays GPU-resident; the compositor's readback (or a
        // sporadic `read_field_stats`) is the only sync point.
    }

    /// **Encode-only sibling of [`Self::step_resident_splat`]** (Watercolor v2 R1,
    /// ADR-0085 §2.3-I1 — single submit): splat + substeps + combine encoded into the
    /// caller's `enc`, which is RETURNED (not submitted) so the shell folds the sim,
    /// composite and preview copy into ONE `queue.submit`, collapsing the present-stall.
    /// Pass order, params and math are byte-identical to the wrapper, so the parity
    /// gates (which drive the wrapper) are unaffected.
    #[must_use]
    pub fn encode_resident_splat_step(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mut enc: wgpu::CommandEncoder,
        dabs: &[DabGpu],
        substeps: u32,
        region: (u32, u32, u32, u32),
    ) -> wgpu::CommandEncoder {
        // Splat (resident water + pig_a updated), ordered before the step in `enc` so
        // diffuse reads the just-splatted field. A single chunk folds into `enc`; a
        // (rare, non-hot-path) multi-chunk list pre-submits via `splat_dabs` so the
        // shared dab buffer isn't clobbered mid-encoder.
        if dabs.len() > MAX_DABS_PER_DISPATCH {
            self.splat_dabs(device, queue, dabs);
        } else {
            self.encode_splat_chunk(queue, &mut enc, dabs);
        }
        // Pad the envelope so the solver region ⊇ what the compositor reads.
        let p = SOLVER_REGION_PAD;
        let padded = (
            region.0.saturating_sub(p),
            region.1.saturating_sub(p),
            region.2.saturating_add(p),
            region.3.saturating_add(p),
        );
        let (gx, gy) = self.write_params_with_region(queue, padded);
        // One compute pass per pipeline (wgpu inserts the RAW barrier between passes).
        // The per-kernel label feeds the GPU pass profiler (PH2D_FLUID_PROFILE) —
        // `compute_writes` is None (free) when profiling is off.
        let run = |enc: &mut wgpu::CommandEncoder,
                   pipe: &wgpu::ComputePipeline,
                   bg: &wgpu::BindGroup,
                   label: &'static str| {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("fluid resident splat pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes(label),
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, bg, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        };
        // Shallow-water velocity layer on? (ADR-0078 S3d — `set_shallow_water` set it.)
        let shallow = self.params_cache.get().velocity > 0.0;
        // Capillary fringe on? (ADR-0078 S5 — `set_from_diffusion` set it.)
        let capillary = self.params_cache.get().capillary > 0.0;
        // BFECC/MacCormack sharpening of the velocity advect? (ADR-0078 S5c — velocity-mode only.)
        let sharpen = shallow && self.params_cache.get().sharpness > 0.0;
        // Lift on? (ADR-0081 — `set_from_diffusion` set it.) Re-mobilizes deposited pigment.
        let lift = self.params_cache.get().lift > 0.0;
        for _ in 0..substeps {
            // MoveWater (ADR-0078 S3d): accelerate `(u,v)` (a→b), make it divergence-free
            // (divergence → seed pressure → RELAX_ITERS Jacobi sweeps → subtract gradient,
            // b→a) BEFORE the pigment transport. Skipped while the layer is dormant.
            if shallow {
                run(
                    &mut enc,
                    &self.add_forces,
                    &self.bg_add_forces,
                    "fluid.forces",
                );
                run(
                    &mut enc,
                    &self.divergence_pipe,
                    &self.bg_divergence,
                    "fluid.divergence",
                );
                run(
                    &mut enc,
                    &self.clear_pressure,
                    &self.bg_clear_pa,
                    "fluid.clear_p",
                );
                run(
                    &mut enc,
                    &self.clear_pressure,
                    &self.bg_clear_pb,
                    "fluid.clear_p",
                );
                for it in 0..RELAX_ITERS {
                    // ping-pong a→b / b→a; even count lands the result in pressure_a.
                    let bg = if it % 2 == 0 {
                        &self.bg_jacobi_ab
                    } else {
                        &self.bg_jacobi_ba
                    };
                    run(&mut enc, &self.jacobi, bg, "fluid.jacobi");
                }
                run(&mut enc, &self.project, &self.bg_project, "fluid.project");
            }
            // Lift (ADR-0081): re-mobilize NON-staining deposited pigment back into flowing in
            // wet cells, BEFORE diffuse (the CPU `step` order — after move_water, before diffuse)
            // so the lifted pigment blooms/moves this substep. Shares `bg_transfer` (same
            // flowing/deposited/water bindings). Skipped while the lift rate is 0.
            if lift {
                run(&mut enc, &self.lift, &self.bg_transfer, "fluid.lift");
            }
            // Then the CPU `step` order: diffuse A→B, advect B→A (velocity-mode when the
            // layer is on, else the static gradient flow), TRANSFER (flowing→deposited),
            // evaporate water. `cs_transfer` is a no-op while deposition params are 0.
            run(&mut enc, &self.diffuse, &self.bg_diffuse, "fluid.diffuse");
            if shallow {
                // Forward advect φ → φ̂ (B→A). With sharpening (ADR-0078 S5c) add the MacCormack
                // backward pass φ̂ → φ̄ (A→C) + the correction φ̂ + s·½(φ−φ̄) clamped (B,A,C→A).
                run(
                    &mut enc,
                    &self.advect_velocity,
                    &self.bg_advect_velocity,
                    "fluid.advect_v",
                );
                if sharpen {
                    run(
                        &mut enc,
                        &self.advect_velocity_rev,
                        &self.bg_advect_velocity_rev,
                        "fluid.advect_v",
                    );
                    run(
                        &mut enc,
                        &self.advect_correct,
                        &self.bg_advect_correct,
                        "fluid.advect_v",
                    );
                }
            } else {
                run(&mut enc, &self.advect, &self.bg_advect, "fluid.advect");
            }
            run(
                &mut enc,
                &self.transfer,
                &self.bg_transfer,
                "fluid.transfer",
            );
            run(
                &mut enc,
                &self.evaporate,
                &self.bg_evaporate,
                "fluid.evaporate",
            );
            // Capillary fringe (ADR-0078 S5): wick water outward into the dry paper + co-advect
            // a thread of pigment with it, then land both fields back in the canonical buffers.
            // After evaporate (the CPU `step` order), so the wick sees this substep's dried
            // water and the next substep's gate picks up the spread. Skipped while dormant. The
            // grown envelope (driver) keeps the fringe in the composited region; the region pad
            // must cover the wick (SOLVER_REGION_PAD).
            if capillary {
                run(
                    &mut enc,
                    &self.capillary_pipe,
                    &self.bg_capillary,
                    "fluid.capillary",
                );
                run(
                    &mut enc,
                    &self.copy_fields_pipe,
                    &self.bg_copy_fields,
                    "fluid.capillary",
                );
            }
        }
        // Combine once per frame (region-scoped, after the substeps): total = flowing +
        // deposited — the buffer the compositor reads (ADR-0078 S3c). Params still hold
        // the padded region, so `total` is fresh wherever the composite samples it.
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("fluid combine pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.combine"),
            });
            pass.set_pipeline(&self.combine);
            pass.set_bind_group(0, &self.bg_combine, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        enc
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

    /// **Pipelined dry-check (ADR-0078 S2 — no per-frame/per-fire `poll(wait)`).**
    /// The live sibling of [`Self::read_field_stats`]: submits the reduction +
    /// `map_async` WITHOUT a blocking poll, and returns the PREVIOUS fire's stats
    /// (read after a non-blocking `poll(Poll)` fired its callback). The blocking
    /// `poll(wait_indefinitely)` of the sync path drains the ENTIRE GPU queue
    /// (including the UI render) — at ~⅓ s cadence that is a multi-ms stall that
    /// *grows* into a runaway (the stall lengthens the frame → the sim falls behind
    /// → more substeps queued → a deeper queue → a longer drain), which is exactly
    /// the single-layer FPS collapse the profiler pinned to `stats` (0.5 → 6 ms).
    ///
    /// The dry-check tolerates one fire of latency (drying takes ~18 frames; the
    /// drop merely frees the field ~one cadence later — invisible, it is already
    /// dry). Seeded "wet" so the first fires never drop the field prematurely.
    /// Uses the PERSISTENT `stats_staging` (no per-call alloc); the prior map must
    /// be drained before re-submission, like the composite's pipelined readback.
    pub fn read_field_stats_pipelined(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        threshold: f32,
    ) -> FieldStats {
        // Fire any completed map callback WITHOUT blocking (the key vs the sync path).
        let _ = device.poll(wgpu::PollType::Poll);
        if let Some(rx) = self.pending_stats.take() {
            match rx.try_recv() {
                Ok(Ok(())) => {
                    let mapped = self.stats_staging.slice(..).get_mapped_range();
                    let s: [u32; 5] = bytemuck::cast_slice::<u8, u32>(&mapped)[..5]
                        .try_into()
                        .expect("5 u32 stats");
                    drop(mapped);
                    self.stats_staging.unmap();
                    self.last_stats = FieldStats {
                        max_water: f32::from_bits(s[0]),
                        bbox: if s[1] == u32::MAX {
                            None
                        } else {
                            Some((s[1], s[2], s[3], s[4]))
                        },
                    };
                }
                Ok(Err(_)) => {
                    // Map failed — unmap defensively; staging is free to reuse.
                    self.stats_staging.unmap();
                }
                Err(_) => {
                    // GPU not done yet — keep waiting, do NOT re-map the staging
                    // (re-mapping a mapped buffer aborts the process), return the last.
                    self.pending_stats = Some(rx);
                    return self.last_stats;
                }
            }
        }
        // Submit a fresh reduction + async map (staging is free now).
        let init: [u32; 5] = [0, u32::MAX, u32::MAX, 0, 0];
        queue.write_buffer(&self.stats_buf, 0, bytemuck::cast_slice(&init));
        let rp = GpuReduceParams {
            width: self.width,
            height: self.height,
            threshold,
            _pad: 0,
        };
        queue.write_buffer(&self.reduce_params_buf, 0, bytemuck::bytes_of(&rp));
        let (gx, gy) = (self.width.div_ceil(8), self.height.div_ceil(8));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid reduce (pipelined)"),
        });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("fluid reduce pass (pipelined)"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.stats"),
            });
            pass.set_pipeline(&self.reduce);
            pass.set_bind_group(0, &self.bg_reduce, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        enc.copy_buffer_to_buffer(
            &self.stats_buf,
            0,
            &self.stats_staging,
            0,
            self.stats_staging.size(),
        );
        queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        self.stats_staging
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
        self.pending_stats = Some(rx);
        self.last_stats
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
        deposit: &[WetCell],
        substeps: u32,
    ) {
        queue.write_buffer(&self.water, 0, bytemuck::cast_slice(water));
        queue.write_buffer(&self.deposit, 0, bytemuck::cast_slice(&pack_soa(deposit)));
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
    pub fn read_pigment(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<WetCell> {
        let n = (self.width as usize) * (self.height as usize);
        let size = (n * PIG_CH * 4) as u64;
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
        let out = unpack_soa(bytemuck::cast_slice::<u8, f32>(&mapped), n);
        drop(mapped);
        staging.unmap();
        out
    }

    /// Map the current deposited-pigment field back (ADR-0078 S3 — companion to
    /// [`Self::read_pigment`], for the `cs_transfer` parity gate + the pen-up bake).
    #[must_use]
    pub fn read_deposited(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<WetCell> {
        let n = (self.width as usize) * (self.height as usize);
        let size = (n * PIG_CH * 4) as u64;
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
        let out = unpack_soa(bytemuck::cast_slice::<u8, f32>(&mapped), n);
        drop(mapped);
        staging.unmap();
        out
    }

    /// Map the backdrop-lift donor (`lift_source`, ADR-0084) back, unpacked to cell-major
    /// `WetCell`s (the planar→cell transpose, ADR-0085). For the `cs_lift` parity gate.
    #[must_use]
    pub fn read_lift_source(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<WetCell> {
        let n = (self.width as usize) * (self.height as usize);
        let size = (n * PIG_CH * 4) as u64;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluid lift_source readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid lift_source rb"),
        });
        enc.copy_buffer_to_buffer(&self.lift_source, 0, &staging, 0, size);
        queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let mapped = staging.slice(..).get_mapped_range();
        let out = unpack_soa(bytemuck::cast_slice::<u8, f32>(&mapped), n);
        drop(mapped);
        staging.unmap();
        out
    }

    /// Map the combined-pigment field (`total`, ADR-0078 S3c) back — the buffer the
    /// compositor reads. For the `cs_combine` gate + the pen-up bake.
    #[must_use]
    pub fn read_total(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<WetCell> {
        let n = (self.width as usize) * (self.height as usize);
        let size = (n * PIG_CH * 4) as u64;
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
        let out = unpack_soa(bytemuck::cast_slice::<u8, f32>(&mapped), n);
        drop(mapped);
        staging.unmap();
        out
    }

    /// Map the resident velocity field (`vel_a`, `array<vec2<f32>>`) back to the CPU
    /// (ADR-0078 S3d) — for the GPU shallow-water parity gate + the pen-up bake. Returns
    /// `(u, v)` per cell, matching the CPU `DiffusionGrid::velocity()` layout.
    #[must_use]
    pub fn read_velocity(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<[f32; 2]> {
        let n = (self.width as usize) * (self.height as usize);
        let size = (n * 8) as u64;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluid velocity readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid velocity rb"),
        });
        enc.copy_buffer_to_buffer(&self.vel_a, 0, &staging, 0, size);
        queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let mapped = staging.slice(..).get_mapped_range();
        let out = bytemuck::cast_slice::<u8, [f32; 2]>(&mapped).to_vec();
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
}
