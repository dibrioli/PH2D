//! Minimal watercolor core drive (ADR-0086/0087, feature `wash`) — the PARALLEL, drastically
//! simpler counterpart to `painter_fluid_bridge`. Reuses the tool's fluid lifecycle (wet-field
//! carrier, dab list, backdrop snapshot, epoch/dims) and ports the v2 zero-stall hot path:
//!
//! - **ONE submit/frame**: `cs_splat` (region) + `cs_step`×substeps (region) + the to-texture
//!   composite (region) + the preview-slot copy (full SEED once, then DIRTY-RECT) all in one encoder
//!   → a `PreviewOverride` the renderer samples in place of the sprite. NO per-frame readback / poll.
//! - **Region = the monotonic wet envelope** (everywhere painted this stroke). A moving window would
//!   step cells an inconsistent number of times under Evaporation 0 (the field keeps diffusing) and
//!   etch rectangular tonal seams; the envelope steps every wet cell uniformly. Bounded by the
//!   painted footprint, released ACTIVE_WINDOW frames after pen-up (finalize).
//! - **Backdrop built ONLY on stroke start** (not every frame — that full-canvas `Vec` copy was a
//!   per-frame CPU stall).
//! - **Finalize**: a short bloom window after pen-up, then ONE readback bakes `canvas_rgba` and the
//!   session idles for free until the next stroke.
//!
//! Set `PH2D_WASH_PROFILE=1` to print the per-frame CPU cost + region size every 120 frames.
//! Downcasts to the concrete `PainterTool` (allowlisted bridge).
#![cfg(feature = "wash")]

use super::sim_extract::PreviewOverride;
use ph2d_editor::ToolRegistry;
use ph2d_gpu::GpuContext;
use ph2d_painter_wash::{Dab, WashCompositor, WashParams, WashSolver};
use ph2d_render::SpriteRenderer;
use std::cell::RefCell;
use std::time::Instant;

/// Frames a dab's area stays in the per-frame work region after it lands (then it freezes — its
/// last composite persists in the preview slot). Doubles as the post-pen-up bloom window.
const ACTIVE_WINDOW: u64 = 30;
/// Wick pad (grid cells) grown around the active region.
const REGION_PAD: u32 = 8;

thread_local! {
    static WASH_SESSION: RefCell<Option<WashSession>> = const { RefCell::new(None) };
    static PROFILE: RefCell<Profile> = const { RefCell::new(Profile::new()) };
}

struct Profile {
    on: Option<bool>,
    frames: u64,
    cpu_us: u64,
    gpu_us: u64,
    seed: u64,
    dirty: u64,
    errs: u64,
    last_region_cells: u64,
}
impl Profile {
    const fn new() -> Self {
        Self { on: None, frames: 0, cpu_us: 0, gpu_us: 0, seed: 0, dirty: 0, errs: 0, last_region_cells: 0 }
    }
    fn enabled(&mut self) -> bool {
        *self.on.get_or_insert_with(|| std::env::var("PH2D_WASH_PROFILE").is_ok())
    }
}

struct WashSession {
    solver: WashSolver,
    compositor: WashCompositor,
    dims: (u32, u32),
    /// Active colour model (0=RGB Beer–Lambert, 1=K–M). A change re-renders (the field is always
    /// pigment concentrations, so it's a pure composite switch — the live transform).
    color_model: u32,
    /// Spectral K–M model — drives the dab concentration encoding (both modes encode concentrations).
    km: ph2d_painter_wash::km::KmModel,
    slot: Option<(u32, u32, u32)>,
    /// Base backdrop captured + `begin_stroke` done — the PERSISTENT pigment field composites over a
    /// single fixed base (the canvas before any wash), accumulating across strokes.
    initialized: bool,
    /// Any pigment has been deposited (the field is worth showing / re-rendering on a model flip).
    has_pigment: bool,
    seeded: bool,
    /// Frames since the last dab — the field keeps settling (bloom/edge recession) for a short
    /// window, then stops costing GPU (the cached slot keeps displaying the persistent wash).
    idle_frames: u32,
    /// **ADR-0088 undo:** per-stroke pigment-field snapshots — `committed[i]` is the field after
    /// `i+1` settled wash strokes. `applied` = how many the GPU field currently realises. The bridge
    /// polls the tool's `wash_active_strokes()`; a mismatch (undo/redo) restores `committed[want-1]`.
    committed: Vec<Vec<[f32; 4]>>,
    applied: usize,
    /// The tool reset generation this session was built for (mismatch ⇒ rebuild).
    reset_gen: u64,
}

impl WashSession {
    /// Build a fresh session — this is where the compute pipelines COMPILE (the click-path stall
    /// the pre-warm moves to hover) + the buffers allocate.
    fn new(device: &wgpu::Device, dims: (u32, u32)) -> Self {
        Self {
            solver: WashSolver::new(device, dims.0, dims.1),
            compositor: WashCompositor::new(device),
            dims,
            color_model: 0,
            km: ph2d_painter_wash::km::KmModel::new(),
            slot: None,
            initialized: false,
            has_pigment: false,
            seeded: false,
            idle_frames: 0,
            committed: Vec::new(),
            applied: 0,
            reset_gen: 0,
        }
    }
    fn release_slot(&mut self, renderer: &mut SpriteRenderer) {
        if let Some((id, _, _)) = self.slot.take() {
            renderer.individual_mut().release(id);
        }
    }
}

fn wash_params_from(dp: &ph2d_painter_brush::diffusion::DiffusionParams) -> WashParams {
    let evap = dp.evaporation.max(0.0);
    // FlowOutward (edge-darkening) is a DRYING phenomenon — pigment strands on the paper as the
    // water LEAVES, marking the receding rim. With no drying (Keep Wet, or the Evaporation slider
    // at 0) there is no receding front, so lateral pumping is unphysical: left on, it bleeds the
    // whole interior out to the rim FOREVER → a hollow/white centre + a pixelated over-concentrated
    // edge (the v2 bug). Ramp edge-darkening in with the drying rate: ~0 at keep-wet's trace evap,
    // full at the default drying rate. Keep-wet/evap-0 ⇒ a flat, stable stain (diffusion only, and
    // since water never spreads here, it stays inside the stroke footprint — no growing puddle).
    let dry_drive = (((evap - 0.004) / (0.012 - 0.004)).clamp(0.0, 1.0)).powi(2);
    WashParams {
        diffusivity: dp.diffusivity.clamp(0.0, 0.20),
        flow_outward: dp.flow_outward.max(0.0) * dry_drive,
        evaporation: evap,
        w_lo: dp.w_lo,
        w_hi: dp.w_hi,
        perm_valley: dp.perm_valley,
        perm_crest: dp.perm_crest,
        ..WashParams::default()
    }
}

fn drop_session(renderer: &mut SpriteRenderer) {
    WASH_SESSION.with(|c| {
        let mut b = c.borrow_mut();
        if let Some(s) = b.as_mut() {
            s.release_slot(renderer);
        }
        *b = None;
    });
}

pub(crate) fn drive_wash_gpu(
    tools: &mut ToolRegistry,
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    override_entity: Option<u64>,
) -> Option<PreviewOverride> {
    let painter = tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<ph2d_tool_painter::PainterTool>())?;

    if !painter.wash_brush_enabled() {
        // Keep the persistent session ALIVE (its undo snapshots + the tool's stroke count must stay
        // in sync across a brush switch); just don't drive it this frame.
        return None;
    }
    let capable = !matches!(gpu.adapter.get_info().device_type, wgpu::DeviceType::Cpu);
    painter.set_fluid_hires(capable);
    if !capable {
        drop_session(renderer);
        return None;
    }
    // Drop + rebuild the session on a tool reset (new source / layer switch) so the base backdrop,
    // pigment field and undo snapshots are rebuilt from the current canvas (kept in sync with the
    // tool's reset of `wash_active_strokes`).
    let reset_gen = painter.wash_reset_generation();
    // ── PRE-WARM ── generate the paper-tooth field while hovering (off the click path) so the first
    // stroke doesn't pay the O(grid) `grain_noise` (~0.5 s first-stroke delay). Cheap if cached.
    let has_field = painter.has_wet_field();
    if !has_field {
        painter.fluid_prewarm_paper();
    }
    let (cw, ch) = painter.source_size();
    if cw == 0 || ch == 0 {
        return None;
    }
    let dims = (cw, ch); // hires (scale 1) ⇒ grid == canvas
    let scale = 1u32;
    let active = painter.is_stroke_active();
    // Colour model: 0 = RGB Beer–Lambert, 1 = spectral K–M ("Pigment" toggle). The PERSISTENT
    // pigment field is encoding-agnostic (always concentrations), so a flip is a pure re-render.
    let color_model = u32::from(painter.wash_subtractive());
    let coverage_k = painter.fluid_coverage_k();
    let substeps = if active { painter.fluid_painting_substeps() } else { painter.fluid_idle_substeps() };
    let wp = wash_params_from(&painter.fluid_diffusion_params());
    // Base backdrop (the canvas BEFORE any wash pigment) is captured ONCE; the persistent field
    // composites over it forever. Fetch lazily — only while a session still needs to initialise.
    let need_init = WASH_SESSION.with(|c| c.borrow().as_ref().map(|s| s.initialized && s.dims == dims) != Some(true));
    let backdrop: Option<Vec<u32>> = if need_init {
        painter
            .fluid_backdrop()
            .map(|b| b.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
    } else {
        None
    };
    // ADR-0088: committed-and-not-undone wash stroke count — drives the undo/redo field sync.
    let want = painter.wash_active_strokes();
    // Dabs only when a live field is producing them (None when hovering / settled).
    let dabs_envelope = if has_field { painter.fluid_take_dabs() } else { None };
    let profile = PROFILE.with(|p| p.borrow_mut().enabled());
    let t0 = profile.then(Instant::now);

    let out = WASH_SESSION.with(|cell| {
        let mut slot_cell = cell.borrow_mut();
        if slot_cell.as_ref().map(|s| (s.dims, s.reset_gen)) != Some((dims, reset_gen)) {
            if let Some(old) = slot_cell.as_mut() {
                old.release_slot(renderer);
            }
            let mut s = WashSession::new(&gpu.device, dims);
            s.reset_gen = reset_gen;
            *slot_cell = Some(s);
        }
        let sess = slot_cell.as_mut()?;

        // Initialise the persistent base backdrop + begin_stroke ONCE (needs a backdrop = 1st stroke).
        if !sess.initialized {
            let bd = backdrop.as_deref()?; // no backdrop yet (pure hover) ⇒ wait for the first stroke
            sess.compositor.begin_stroke(
                &gpu.device, &gpu.queue, dims.0, dims.1, cw, ch, bd, coverage_k, color_model,
                sess.solver.pig_buffer(),
            );
            sess.color_model = color_model;
            sess.initialized = true;
            sess.seeded = false;
        }

        // Colour-model flip ⇒ live re-render of the WHOLE persistent field (no clear, no re-encode).
        let model_flipped = sess.color_model != color_model;
        if model_flipped {
            sess.compositor.set_color_model(color_model);
            sess.color_model = color_model;
            sess.seeded = false; // forces a full re-composite below
        }

        // ── ADR-0088 undo/redo sync ── the tool's wash stroke count moved (undo lowered it, redo
        // raised it back). Restore the GPU pigment field to the matching per-stroke snapshot. A count
        // ABOVE `applied` with no matching snapshot yet is just the in-flight stroke settling (NOT a
        // redo) — guarded by `committed.len() >= want`.
        let is_redo = want > sess.applied && sess.committed.len() >= want;
        let mut restored_now = false;
        if want < sess.applied || is_redo {
            if want == 0 {
                sess.solver.clear(&gpu.device, &gpu.queue);
            } else {
                let snap = sess.committed[want - 1].clone();
                sess.solver.upload_pigment(&gpu.queue, &snap);
            }
            sess.applied = want;
            sess.has_pigment = want > 0;
            sess.seeded = false; // one full re-composite of the restored field
            // Mark settled so NO physics runs on the restore: stepping would re-diffuse the restored
            // pigment through the STALE water field (full at Evaporation 0 ⇒ the undo drifts/spreads —
            // the reported evap-0 undo bug). The snapshot is already the settled state; just show it.
            sess.idle_frames = ACTIVE_WINDOW as u32 + 1;
            restored_now = true;
        }

        // Encode dabs — ALWAYS 4 pigment concentrations (both colour models read the same field).
        let dabs: &[ph2d_tool_painter::FluidDab] = dabs_envelope.as_ref().map_or(&[], |(d, _)| d.as_slice());
        let gpu_dabs: Vec<Dab> = dabs
            .iter()
            .map(|d| {
                let mut conc = sess.km.rgb_to_concentrations(d.color);
                for c in &mut conc {
                    *c *= d.mass;
                }
                Dab::from_concentrations(d.cx, d.cy, d.r.max(0.5), d.water, conc)
            })
            .collect();
        if !gpu_dabs.is_empty() {
            sess.has_pigment = true;
        }

        // Idle bookkeeping: the field keeps settling (bloom/edge recession) a short window after the
        // last dab, then stops costing GPU — the cached slot keeps displaying the persistent wash.
        if active || !gpu_dabs.is_empty() {
            sess.idle_frames = 0;
        } else {
            sess.idle_frames = sess.idle_frames.saturating_add(1);
        }
        let settling = sess.idle_frames <= ACTIVE_WINDOW as u32;

        // Nothing to draw yet.
        if !sess.has_pigment {
            return None;
        }
        let entity_bits = override_entity?;

        // Touch the GPU only when something changed; otherwise return the cached slot (≈0 idle cost).
        let full = !sess.seeded || model_flipped; // a fresh seed / a model flip re-renders everything
        let need_gpu = full || !gpu_dabs.is_empty() || settling;
        if !need_gpu {
            return sess
                .slot
                .map(|(id, _, _)| PreviewOverride { entity_bits, texture_id: id, premultiplied: true });
        }

        sess.solver.set_params(&gpu.queue, wp);

        // Region: full canvas on a full re-render, else the painted envelope (padded).
        let (g_region, cx0, cy0, cw_r, ch_r) = if full {
            ((0, 0, dims.0, dims.1), 0, 0, cw, ch)
        } else {
            let (ex0, ey0, ex1, ey1) = dabs_envelope.as_ref().map_or((0, 0, dims.0 - 1, dims.1 - 1), |(_, e)| *e);
            let gx0 = ex0.saturating_sub(REGION_PAD);
            let gy0 = ey0.saturating_sub(REGION_PAD);
            let gx1 = (ex1 + REGION_PAD).min(dims.0 - 1);
            let gy1 = (ey1 + REGION_PAD).min(dims.1 - 1);
            let cx0 = (gx0 * scale).min(cw);
            let cy0 = (gy0 * scale).min(ch);
            let cx1 = ((gx1 + 1) * scale).min(cw);
            let cy1 = ((gy1 + 1) * scale).min(ch);
            ((gx0, gy0, gx1 - gx0 + 1, gy1 - gy0 + 1), cx0, cy0, cx1 - cx0, cy1 - cy0)
        };
        PROFILE.with(|p| p.borrow_mut().last_region_cells = u64::from(cw_r) * u64::from(ch_r));

        let (id, fresh) = match sess.slot {
            Some((id, w, h)) if w == cw && h == ch => (id, false),
            _ => {
                if let Some((old, _, _)) = sess.slot.take() {
                    renderer.individual_mut().release(old);
                }
                let id = renderer.acquire_individual_empty(cw, ch);
                sess.slot = Some((id, cw, ch));
                (id, true)
            }
        };
        let seed = fresh || full;

        // ── ONE encoder: step + composite + slot copy ──
        // A restore re-renders the snapshot with ZERO physics (no re-diffusion through stale water).
        let step_n = if restored_now { 0 } else { substeps };
        let enc = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash frame") });
        let mut enc = sess.solver.encode_step(&gpu.queue, enc, &gpu_dabs, step_n, g_region);
        let tex_ok = if seed {
            sess.compositor.encode_composite(&gpu.queue, &mut enc, (0, 0, cw, ch));
            sess.compositor.preview_texture().is_some_and(|t| renderer.encode_copy_into_individual(&mut enc, id, t, cw, ch).is_ok())
        } else {
            sess.compositor.encode_composite(&gpu.queue, &mut enc, (cx0, cy0, cw_r, ch_r));
            sess.compositor
                .preview_texture()
                .is_some_and(|t| renderer.encode_copy_region_into_individual(&mut enc, id, t, cx0, cy0, cx0, cy0, cw_r, ch_r).is_ok())
        };
        gpu.queue.submit([enc.finish()]);
        if profile {
            let tg = Instant::now();
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            PROFILE.with(|p| {
                let mut p = p.borrow_mut();
                p.gpu_us += tg.elapsed().as_micros() as u64;
                if seed { p.seed += 1 } else { p.dirty += 1 }
                if !tex_ok { p.errs += 1 }
            });
        }
        if !tex_ok {
            sess.release_slot(renderer);
            sess.seeded = false;
            return None;
        }
        sess.seeded = true;

        // ── ADR-0088 undo snapshot ── the tool committed a stroke (its count ran past `applied`, and
        // it's not a redo). Snapshot the field NOW (at pen-up, one per stroke ⇒ no fast-stroke
        // collapse), discarding any redo branch first. `committed[i]` = field after stroke i+1.
        if want > sess.applied && sess.committed.len() < want {
            sess.committed.truncate(sess.applied);
            let snap = sess.solver.read_pigment(&gpu.device, &gpu.queue);
            while sess.applied < want {
                sess.committed.push(snap.clone());
                sess.applied += 1;
            }
        }

        // Bake into canvas_rgba (for save / layer thumbnail / interop) once the wash settles, or on a
        // restore. The slot override keeps DISPLAYING the live wash; this just keeps the flat canvas
        // in sync (and lets the tool's snapshot-undo restore a coherent pre-image).
        let do_bake = restored_now || (!active && sess.idle_frames == ACTIVE_WINDOW as u32);
        if let Some(words) = do_bake.then(|| sess.compositor.read_preview(&gpu.device, &gpu.queue)).flatten() {
            let band: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
            painter.fluid_apply_gpu_composite_rows(&band, (0, 0, cw, ch));
        }
        Some(PreviewOverride { entity_bits, texture_id: id, premultiplied: true })
    });

    if let Some(t0) = t0 {
        PROFILE.with(|p| {
            let mut p = p.borrow_mut();
            p.frames += 1;
            p.cpu_us += t0.elapsed().as_micros() as u64;
            if p.frames % 120 == 0 {
                eprintln!(
                    "[wash] cpu {:.2}ms gpu {:.2}ms/frame | seed {} dirty {} err {} | region {} cells dims {}x{}",
                    p.cpu_us as f64 / 120.0 / 1000.0,
                    p.gpu_us as f64 / 120.0 / 1000.0,
                    p.seed,
                    p.dirty,
                    p.errs,
                    p.last_region_cells,
                    dims.0,
                    dims.1,
                );
                p.cpu_us = 0;
                p.gpu_us = 0;
                p.seed = 0;
                p.dirty = 0;
                p.errs = 0;
            }
        });
    }
    out
}
