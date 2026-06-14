//! See `tool/mod.rs` — this is stroke lifecycle + journal + UI edit + undo/redo + base accessors,
//! split out of the former `tool.rs` god-object (pure mechanical move).

use super::*;

/// Wet-edge rim width as a fraction of the brush diameter (the settle blur
/// radius). ~15% of the diameter matches the soft, size-relative rim of a real
/// wash; clamped to a sane pixel range at the call site.
const WET_EDGE_RIM_FRACTION: f32 = 0.15;

/// **W15 live-diffusion tuning.** The wet field runs at 1/`WET_FIELD_SCALE` of the
/// canvas (the budget-feasible low-res sim grid; GPU/full-res is W15.3). Per-dab
/// water + pigment deposit, the diffusion sub-steps run while painting vs idle, the
/// dryness threshold (per cell) below which the field is dropped, and the
/// density→alpha exponent for the composite.
pub(super) const WET_FIELD_SCALE: u32 = 2;

const WET_WATER_DEPOSIT: f32 = 0.55;
const WET_PIGMENT_DEPOSIT: f32 = 0.5;
// Diffusion sub-steps per frame. **Raised 1→3 / 2→3 (ADR-0085 — wet-on-wet bloom).** The diffuse
// pass works + `Diffusivity` has a strong effect (proven:
// `physical_invariants::diffusivity_actually_diffuses`, peak 1.0→0.43 in 40 steps); more sub-steps
// = more diffusion per frame, so freshly-laid pigment visibly blooms into the wet field as you
// paint. Kept MODEST (3, not 6): the sub-step loop ALSO runs deposition + evaporation, so a high
// count would over-deposit/over-dry the ratified DRY look. The real wet-on-wet fix is suppressing
// deposition while the field is wet (`fluid_diffusion_params`), which keeps the pigment mobile so
// it actually diffuses instead of freezing into the `deposited` layer. Region-scoped +
// single-submit, so the extra diffuse kernels are cheap (the cost was always submit/copy).
const WET_SUBSTEPS_PAINTING: u32 = 3;
const WET_SUBSTEPS_IDLE: u32 = 3;
const WET_DRY_THRESHOLD: f32 = 0.045;
/// Minimal evaporation Keep Wet keeps (instead of a hard 0) so the wet edge recedes through the
/// gate into a SOFT rim rather than pinning into a crisp/pixelated step (Enio 2026-06-12). Small
/// enough that the wash stays workable (the drop-guard never lets it disappear); the slider raises
/// it. ≈ ⅓ of the 0.012 preset, so the bulk dries far slower than a normal (non-keep-wet) wash.
const KEEP_WET_EVAP: f32 = 0.004;
/// Coverage rate: `alpha = 1 − exp(−amount · K)`, where `amount` is the
/// COLOUR-INDEPENDENT pigment load. The grid stores colour×amount, so the raw
/// channel sum is `amount · Σcolour` — luminance-weighted, which made bright
/// pigments (yellow/magenta) read as fully opaque while blue/red stayed a proper
/// translucent wash. Normalising by the stroke colour's linear sum recovers
/// `amount`; `K` is anchored to the (already-correct) blue/red look — their
/// `Σcolour ≈ 0.53`, so `K = old 2.0 × 0.53 ≈ 1.06` keeps them unchanged.
const WET_COVERAGE_K: f32 = 1.06;

/// Cached paper-tooth field entry: `(gw, gh, scale, paper)` — the key `(gw,gh,scale)` plus the
/// shared field (see [`PAPER_CACHE`]).
type PaperCacheEntry = (u32, u32, u32, std::sync::Arc<Vec<f32>>);

/// This frame's fluid dabs + the composite envelope `(x0,y0,x1,y1)` (see [`PainterTool::fluid_take_dabs`]).
type FluidDabBatch = (Vec<crate::tool::FluidDab>, (u32, u32, u32, u32));

thread_local! {
    /// **Cached paper-tooth field (perf fix 2026-06-08).** `DiffusionGrid::new`
    /// generates the paper via `grain_noise` PER CELL — O(grid) on the CPU, ~⅓ s on a
    /// large/4K canvas. It's deterministic in `(gw, gh, scale)`, so a fresh field per
    /// stroke regenerated it needlessly (the "delay entre o clique e o início do
    /// traço"). Cache it keyed by those, so begin_stroke only pays a cheap clone after
    /// the first stroke on a canvas. Thread-local (the tool runs single-threaded).
    static PAPER_CACHE: std::cell::RefCell<Option<PaperCacheEntry>> =
        const { std::cell::RefCell::new(None) };
}

/// Get-or-generate the cached paper-tooth field for `(gw, gh, scale)` (the O(grid)
/// `grain_noise` that was the per-stroke ~⅓ s hitch). Generated at most once per
/// canvas/scale; the one-time generation cost is logged. Pre-warm calls this during
/// hover so the first stroke is cheap too.
fn cached_fluid_paper(gw: u32, gh: u32, scale: u32) -> std::sync::Arc<Vec<f32>> {
    PAPER_CACHE.with(|cell| {
        let mut c = cell.borrow_mut();
        if let Some((cw, ch, cs, p)) = c.as_ref()
            && *cw == gw
            && *ch == gh
            && *cs == scale
        {
            return std::sync::Arc::clone(p);
        }
        let t = std::time::Instant::now();
        let p = std::sync::Arc::new(
            ph2d_painter_brush::diffusion::DiffusionGrid::generate_paper(gw, gh, scale as f32),
        );
        eprintln!(
            "[fluid] paper generated {gw}x{gh} (scale {scale}) in {:.1}ms — cached for reuse",
            t.elapsed().as_secs_f64() * 1000.0
        );
        *c = Some((gw, gh, scale, std::sync::Arc::clone(&p)));
        p
    })
}

impl PainterTool {
    /// Inicia um novo stroke. Caller deriva `seed` de inputs determinísticos
    /// (e.g., `pointer_down_time_ms ^ entity_bits ^ brush_hash`).
    ///
    /// **Q-2 (audit T1.9):** se já há stroke ativo (caller esqueceu
    /// `end_stroke`), o anterior é **cancelado** sem consumir `next_seq` —
    /// `next_seq` só avança quando um stroke vai virar `StrokeRecord`
    /// (ou seja, fim deste método com PartialStroke materializado).
    pub fn begin_stroke(&mut self, seed: u64) {
        if self.stroke_active {
            // Defensive: previous stroke didn't close cleanly. Encerra-o
            // implicitamente sem commit pra evitar state corruption.
            self.scheduler.end_stroke();
            // T1.9: descarta PartialStroke + journal active stroke também.
            // Sem commit = stroke aborted; journal sample-batch buffer
            // limpo via cancel_stroke (não suja WAL).
            if let Some(journal) = self.stroke_journal.as_mut()
                && journal.current_stroke.is_some()
            {
                let _ = journal.cancel_stroke();
            }
            // Q-2: stroke cancelado NÃO consumiu next_seq — reciclar o seq
            // do PartialStroke abandonado pra próximo stroke usar.
            if let Some(p) = self.current_partial.take() {
                self.next_seq = p.seq;
            }
            self.current_samples.clear();
        }
        self.scheduler.begin_stroke(seed);
        self.stroke_active = true;
        // **W2.T2.2 undo capture:** snapshot the layer texture BEFORE any
        // stamp of this stroke lands (painting happens later in
        // `queue_pointer` via `apply_stamps`). Held in `pending_pre_stroke`
        // and committed to the undo stack only if the stroke is non-empty
        // (`end_stroke`); the empty-stroke recycle path discards it so a
        // no-paint gesture never creates a phantom undo slot (mirrors the V-1
        // empty-stroke gate for `stroke_history`). Cloning the texture per
        // stroke-begin is the inherent cost of snapshot-based undo (CPU-MVP
        // W2; W11 `T-replay` replaces it with delta/replay).
        self.pending_pre_stroke = Some(self.canvas_rgba.as_ref().clone());
        // R4-LG-4: cache OKLab color at stroke boundary. UI updates to
        // `params.active_color` between strokes — recomputing the two
        // transcendentals per pointer event is pure waste otherwise.
        // `effective_active_color` grays it when painting into a mask (§2.7).
        self.stroke_color_oklab = oklch_to_oklab(self.effective_active_color());
        // **W2.T2.1 Day-7 — stroke-level opacity wire:** Stamp.opacity é
        // hardcoded 1.0 no scheduler (T1.7 TODO "taper + stroke-level
        // opacity"). Aplicamos `params.opacity` como pre-multiply no
        // alpha do color (STRAIGHT alpha → shader premultiplies). Per-
        // stamp opacity dynamics (taper) vem em W5+ Brush Studio.
        //
        // **W5 wash vs build-up (orthogonal to pigment).** When the brush is NOT
        // in accumulate mode, strokes route through `apply_stamps_wash`, where
        // opacity is the per-STROKE *cap* on coverage — NOT a per-dab multiply
        // (that builds up unbounded). We keep the colour alpha at the UI value and
        // hand `params.opacity` to the wash path as the cap; coverage is a fresh
        // zeroed per-pixel buffer over the source. `accumulate=true` falls through
        // to the normal per-dab build-up path (opacity baked into the alpha).
        let wash = !self.brush.rendering.accumulate;
        // The coverage buffer feeds the wash composite AND the wet/burnt edge
        // settle. So it is needed whenever EITHER is active — including build-up
        // (`accumulate=true`) strokes with edges on, where it is a pure side
        // output (the build-up render is byte-identical; it just also records the
        // stroke's coverage for the pen-up settle). Without this, edges only
        // worked in wash mode.
        let edges_on = self.brush.rendering.wet_edges || self.brush.rendering.burnt_edges;
        // Build-up also needs the coverage buffer when the global paper tooth is on
        // (it routes through `apply_stamps_buildup`, which carries the tooth).
        let paper_on = self.params.paper_grain > 0.0;
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if wash || edges_on || paper_on {
            // Reuse the existing buffer (same source size) to avoid a per-stroke
            // realloc; otherwise allocate a fresh zeroed coverage map.
            match self.wash_coverage.as_mut() {
                Some(buf) if buf.len() == n => buf.iter_mut().for_each(|c| *c = 0.0),
                _ => self.wash_coverage = Some(vec![0.0; n]),
            }
        } else {
            self.wash_coverage = None;
        }
        if wash {
            self.wash_opacity_cap = self.params.opacity.clamp(0.0, 1.0);
            // Parallel per-pixel accumulated brush colour (Color Dynamics blend) —
            // only the wash composite uses this; build-up bakes the colour live.
            match self.wash_color.as_mut() {
                Some(buf) if buf.len() == n => buf.iter_mut().for_each(|c| *c = [0.0; 3]),
                _ => self.wash_color = Some(vec![[0.0; 3]; n]),
            }
        } else {
            self.wash_color = None;
            self.stroke_color_oklab[3] *= self.params.opacity.clamp(0.0, 1.0);
        }

        // **W15 fluid (ADR-0080 cross-stroke wet-on-wet):** the live wet-on-wet diffusion
        // field persists ACROSS strokes while it is still WET. A new stroke deposits its dabs
        // into the surviving wet pigment, so colours mix subtractively across strokes (pinte
        // azul, pinte amarelo molhado por cima → verde). The dry-drop
        // ([`Self::fluid_dry_check_and_drop_gpu`]) clears the field (`wet_field = None`) ONCE
        // it has dried + the stroke ended, so a surviving `Some` here means "still wet → reuse".
        // Only a missing field (dry/baked) or a resolution change builds a FRESH field — which
        // snapshots the backdrop + bumps the GPU-reset epoch (the bridge clears the resident
        // pigment) + restarts the composite envelope. `None` for non-fluid brushes.
        //
        // **ADR-0085 (GPU-first, GPU-only):** the live watercolor sim is GPU-resident — there is
        // no CPU fallback. A wet field is only allocated on a GPU-capable device (`fluid_hires`,
        // set by the shell before `begin_stroke`); without a GPU the fluid brush gracefully
        // degrades to the normal wash path (watercolor OFF), so `wet_field` stays `None`.
        let mut wet_field_reused = false;
        // ADR-0087: a `wash_enabled` brush reuses the SAME wet-field carrier lifecycle (dab list,
        // backdrop snapshot, epoch, dims) — the shell's `drive_wash_gpu` drives the WashSolver
        // over it instead of the FluidSolver. The two are mutually exclusive (the apply handler).
        if (self.brush.rendering.fluid_enabled || self.brush.rendering.wash_enabled) && self.fluid_hires {
            // **W15.3 full-res on a capable GPU.** The field runs at full canvas
            // resolution (`scale=1`) for fine bleeds + sharp edges.
            let scale = self.live_field_scale();
            let (sw, sh) = self.source_size;
            let gw = (sw / scale).max(1);
            let gh = (sh / scale).max(1);
            // Reuse iff a still-wet field of the SAME resolution survived the previous stroke.
            wet_field_reused = self.wet_field_scale == scale
                && self.wet_field.as_ref().map(|g| g.dims()) == Some((gw, gh));
            if !wet_field_reused {
                self.wet_field_scale = scale;
                // **Perf (2026-06-08):** reuse the cached deterministic paper field instead
                // of regenerating `grain_noise` per cell every stroke (O(grid), ~⅓ s at 4K —
                // the click→stroke delay). Pre-warm (hover) usually populated the cache.
                let paper = cached_fluid_paper(gw, gh, scale);
                self.wet_field = Some(ph2d_painter_brush::diffusion::DiffusionGrid::with_paper(
                    gw,
                    gh,
                    (*paper).clone(),
                ));
                // Signal the shell GPU drive that a FRESH field began, so it resets the
                // resident pigment (a reused solver must not inherit a dried bloom). NOT bumped
                // on reuse → the resident field persists for cross-stroke mixing.
                self.fluid_stroke_epoch = self.fluid_stroke_epoch.wrapping_add(1);
                // The wash composites over the pre-stroke canvas for the WHOLE life of the
                // field. `pending_pre_stroke` can't serve this (consumed by undo at `end_stroke`),
                // so snapshot a dedicated backdrop here — and KEEP it across a reused stroke (the
                // wet field still holds the prior stroke's pigment over the same backdrop).
                self.wet_backdrop = Some(self.canvas_rgba.as_ref().clone());
            }
        } else {
            self.wet_field = None;
            self.wet_backdrop = None;
        }
        // Fresh stroke ⇒ reset the composite envelope; a REUSED (still-wet) field keeps its
        // envelope so the surviving pigment stays composited (don't clip it to the new dabs).
        if !wet_field_reused {
            self.wet_composite_bbox = None;
            self.wet_pigment_envelope = None;
        }
        self.fluid_dabs.clear();

        // T1.9: construir PartialStroke + wire journal se ativo.
        //
        // **S-8 NaN guard:** `params.active_color` é pub field; bridge PCA
        // pode escrever NaN/Inf inadvertently. OKLCH NaN persistido em WAL +
        // canon explode em replay W12 (postcard preserva bits, mas brush
        // matemática faz NaN propagation cascading). Sanitize aqui.
        let primary = sanitize_oklch_or_default(self.params.active_color);
        // **S-3:** `secondary_color` (Procreate long-press slot ADR-0046 §2.2)
        // SEMPRE persistido em T1.9 wire — caller W2 sidebar refinará "in
        // use" semantics em ADR-0046-amendment-2 (carry-over W11 S-15).
        let secondary = Some(sanitize_oklch_or_default(self.params.secondary_color));
        // **R-5:** cache do `params_blake3` (postcard alloc + blake3 ~32B).
        // `set_brush` invalida; entre strokes, hit cache = 0 alloc.
        let brush_hash = match self.cached_brush_hash {
            Some(h) => h,
            None => {
                let h = self.brush.params_blake3();
                self.cached_brush_hash = Some(h);
                h
            }
        };
        let brush_handle_canon = brush_handle_stub_to_canon(self.params.active_brush);
        let mut partial = PartialStroke::new(
            self.next_seq,
            self.canvas_id,
            self.layer_target,
            brush_handle_canon,
            brush_hash,
            primary,
        );
        partial.rng_seed = seed;
        // **S-1 wall-clock:** ADR-0046 §2.2 + ADR-0052 §2.2 prescrevem
        // `started_at_ms` populated em Begin. Pré-S1, ficava 0 → quebrava
        // time-lapse W11 + Inspector W14 chronology + audit log. Wall-clock
        // é EXPLICITAMENTE não-determinístico (ADR-0046 §3 Neutras) ⇒ não
        // entra em det-replay (vide U-9 doc em StrokeRecord).
        partial.started_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        // **S-2 ToolMode mapping (ADR-0043 §2.6.1 congelado):**
        partial.tool_mode = painter_mode_to_tool_mode(self.params.mode);
        partial.secondary_color = secondary;
        // **Q-3/R-3 + Q-9 + V-2:** se WAL falhar, captura em `last_wal_error`
        // pro bridge W11 surface. NÃO materializa current_partial — assim
        // queue_pointer/end_stroke não chamam add_sample/commit em journal
        // sem Begin, evitando cascade `NoActiveStroke` que clobbed o erro
        // original (V-2). PainterTool fica em modo "CPU-only painting; nem
        // WAL nem in-memory history pra esse stroke" — explicit doc no
        // `last_wal_error()` accessor.
        let mut wal_accepted = true;
        if let Some(journal) = self.stroke_journal.as_mut()
            && let Err(e) = journal.begin_stroke(partial.clone())
        {
            debug_assert!(
                false,
                "WAL begin_stroke failed: {:?} — degrading to in-memory mode",
                e
            );
            self.last_wal_error = Some(e);
            wal_accepted = false;
        }
        if wal_accepted || self.stroke_journal.is_none() {
            self.current_partial = Some(partial);
        } else {
            // WAL anexado mas rejeitou begin → manter current_partial = None.
            // queue_pointer/end_stroke vão tratar como "in-memory only stroke
            // sem record" (V-2 cascade fix: nenhum WAL call subsequent).
            self.current_partial = None;
        }
        self.current_samples.clear();
        // **V-7:** reserve cap lazy — Default mantém Vec::new() (V-5 fix);
        // begin_stroke pre-aloca quando vai começar a popular.
        if self.current_samples.capacity() < 256 {
            self.current_samples.reserve(256);
        }
        // Q-10/R-10: next_seq avança IFF PartialStroke foi materializado.
        // `checked_add` em vez de saturating: u64 overflow é fisicamente
        // impossível em uso humano (~580 anos a 1B strokes/s), mas LOUD
        // panic é melhor que silent dedup violando ADR-0046 §2.2.
        if self.current_partial.is_some() {
            self.next_seq = self
                .next_seq
                .checked_add(1)
                .expect("next_seq u64 overflow — canvas needs canonical reset");
        }
    }

    /// Empilha um pointer sample no scheduler e aplica os stamps gerados
    /// sobre `canvas_rgba` (CPU path T1.5). No-op se nenhum stroke ativo.
    ///
    /// **Audit T1.6 R7 L1-5:** silent no-op when `!self.stroke_active`
    /// historically masked a bridge bug — a pointer-down handler that
    /// forgot to call `begin_stroke` (or had its dispatch path
    /// early-return before reaching it) would silently drop every drag
    /// sample, and the subsequent `drain_painter` would emit a
    /// `Toast::info("Painter: no strokes to apply")` indistinguishable
    /// from "Apply ran with no paint". The `debug_assert!` here surfaces
    /// the missed `begin_stroke` immediately in dev/test builds without
    /// changing release behavior. Pair with the toast-wording follow-up
    /// in `drain_painter` (separate dispatch site) for the full UX fix.
    /// **W15 idle tick** (driven by `Tool::on_tick` each frame). ADR-0085: the live wet
    /// field is GPU-resident and the shell steps + composites + dries it on its per-frame
    /// drive, so the tool's heartbeat has nothing to do — a no-op. (Kept as the trait's
    /// `on_tick` target so the heartbeat contract is unchanged.)
    pub(crate) fn on_tick_diffusion(&mut self) {}

    /// W15.3 shell GPU drive: set by the shell when it is stepping the wet field on
    /// the GPU, so the tool skips its CPU diffusion (`on_tick`/`queue_pointer`).
    pub fn set_gpu_fluid_driven(&mut self, v: bool) {
        self.gpu_fluid_driven = v;
    }

    /// Mutable access to the live wet field so the shell GPU stepper can advance it
    /// in place; `None` when there is no live field (dry / non-fluid brush).
    pub fn fluid_grid_mut(&mut self) -> Option<&mut ph2d_painter_brush::diffusion::DiffusionGrid> {
        self.wet_field.as_mut()
    }

    /// `true` when a live wet field exists (the shell checks before driving the GPU).
    #[must_use]
    pub fn has_wet_field(&self) -> bool {
        self.wet_field.is_some()
    }

    /// Idle diffusion sub-steps per frame — the count the shell passes to the GPU
    /// stepper so it matches the CPU idle cadence.
    #[must_use]
    pub fn fluid_idle_substeps(&self) -> u32 {
        WET_SUBSTEPS_IDLE
    }

    /// **Painting** diffusion sub-steps per frame the shell runs on the GPU while a stroke is
    /// live (Watercolor v2, ADR-0085). The GPU drive once used the IDLE count (2) for both,
    /// doubling the whole per-frame pass chain (~40 passes) while painting; the intended 1
    /// halves the live sim cost.
    #[must_use]
    pub fn fluid_painting_substeps(&self) -> u32 {
        WET_SUBSTEPS_PAINTING
    }

    // ───────────────── W15.3 GPU resident-composite hooks (shell-facing) ─────────────────

    /// Bumps each `begin_stroke` that allocates a fresh fluid field — the shell
    /// resets the GPU-resident pigment + re-uploads paper on change. See
    /// [`Self::fluid_stroke_epoch`] field doc.
    #[must_use]
    pub fn fluid_stroke_epoch(&self) -> u64 {
        self.fluid_stroke_epoch
    }

    /// Canvas/grid ratio of the LIVE field (`1` hires GPU, else `WET_FIELD_SCALE`).
    #[must_use]
    pub fn fluid_field_scale(&self) -> u32 {
        self.wet_field_scale
    }

    /// Set by the shell when a capable GPU is present, so the NEXT fluid stroke runs
    /// at full canvas resolution (`scale=1`). No effect on the current field.
    pub fn set_fluid_hires(&mut self, v: bool) {
        self.fluid_hires = v;
    }

    /// **Watercolor v2 (ADR-0085 §2.3-I4) — the canvas/grid ratio for the NEXT field.**
    /// The sim is O(grid cells); the composite bicubic-upsamples (ADR-0080 §2.4), so the
    /// field is meant to be COARSE. Full canvas-res (`fluid_hires` ⇒ scale 1) is 4-16× the
    /// cells of the low-res field the architecture is built around — the dominant per-frame
    /// cost (a 308k-cell wet region at scale 1 is ~19k at scale 4). `PH2D_FLUID_SCALE`
    /// overrides for live perf bisection (e.g. `PH2D_FLUID_SCALE=4`); the upsample covers
    /// the look. Clamped ≥ 1.
    fn live_field_scale(&self) -> u32 {
        if self.fluid_hires { 1 } else { WET_FIELD_SCALE }
    }

    /// **Real-pigment palette (ADR-0081).** Pick a curated artist pigment by PALETTE index, or
    /// clear back to raw colour with `None`.
    ///
    /// `Some(i)` (with `i < PALETTE.len()`): loads the pigment's **masstone** into the brush
    /// colour (`params.active_color`), folds its **granulation** into the brush watercolor slider
    /// (`apply_granulation`), and records the index so each dab carries the pigment's **staining**
    /// (see [`Self::active_staining`]). `None` (or an out-of-range index) clears the active pigment
    /// and leaves the colour + params exactly as they are (the bit-identical raw-colour path).
    pub fn set_active_pigment(&mut self, idx: Option<u8>) {
        match idx {
            Some(i) if (i as usize) < ph2d_painter_brush::PALETTE.len() => {
                let p = &ph2d_painter_brush::PALETTE[i as usize];
                let [r, g, b] = p.srgb;
                self.params.active_color = crate::color::srgb8_to_painter_oklch([r, g, b, 255]);
                p.apply_granulation(&mut self.brush.rendering.watercolor);
                self.active_pigment = Some(i);
            }
            _ => self.active_pigment = None,
        }
    }

    /// The active real pigment's PALETTE index, or `None` for raw colour (ADR-0081).
    #[must_use]
    pub fn active_pigment(&self) -> Option<u8> {
        self.active_pigment
    }

    /// The active pigment's staining ∈ [0,1] (ADR-0081) — `0.0` when no pigment is selected. Rides
    /// every fluid dab so the pigment keeps staining/lifting per its real-world character.
    pub(crate) fn active_staining(&self) -> f32 {
        self.active_pigment
            .and_then(|i| ph2d_painter_brush::PALETTE.get(i as usize))
            .map_or(0.0, |p| p.staining)
    }

    /// `true` when the active brush opts into the live wet field — the shell uses this
    /// to PRE-WARM the GPU solver+compositor (compile the big composite shader) before
    /// the first dab, so the stroke doesn't hitch on first use.
    #[must_use]
    pub fn fluid_brush_enabled(&self) -> bool {
        self.brush.rendering.fluid_enabled
    }

    /// Whether the active brush selects the minimal watercolor core (ADR-0087). The shell's
    /// `drive_wash_gpu` gates on this; mutually exclusive with [`Self::fluid_brush_enabled`].
    pub fn wash_brush_enabled(&self) -> bool {
        self.brush.rendering.wash_enabled
    }

    /// Whether the active brush selects the **spectral Kubelka–Munk** colour model (`PigmentMode::
    /// Subtractive`, the "Pigment" toggle) vs. RGB Beer–Lambert (`Linear`). The wash composite reads
    /// this to pick its colour path; the two are a per-brush CHOICE (ADR-0086 §8.1), not a swap.
    pub fn wash_subtractive(&self) -> bool {
        self.brush.rendering.pigment_mode == ph2d_painter_brush::PigmentMode::Subtractive
    }

    /// The grid dims a fresh field WOULD use right now (`source_size / scale`, scale
    /// from the current hires flag) — for the shell to pre-warm the solver at the
    /// right size before `begin_stroke`. `None` if there's no source yet.
    #[must_use]
    pub fn fluid_prewarm_dims(&self) -> Option<(u32, u32)> {
        let (sw, sh) = self.source_size;
        if sw == 0 || sh == 0 {
            return None;
        }
        let scale = self.live_field_scale();
        Some(((sw / scale).max(1), (sh / scale).max(1)))
    }

    /// Pre-generate + cache the paper-tooth field for the next stroke's dims/scale, so
    /// `begin_stroke` (the click) pays only a cheap clone instead of the O(grid)
    /// `grain_noise` (the ~⅓ s click→stroke delay). The shell calls this from the
    /// GPU-drive PRE-WARM (while hovering, before the first dab), moving the one-time
    /// cost off the click path. No-op for non-fluid brushes / no source.
    pub fn fluid_prewarm_paper(&self) {
        // ADR-0087: the wash brush shares the same paper-tooth cache + begin_stroke path, so it
        // ALSO pre-warms here (else the O(grid) `grain_noise` runs on the first click → ~0.5 s delay).
        if !(self.brush.rendering.fluid_enabled || self.brush.rendering.wash_enabled) {
            return;
        }
        let (sw, sh) = self.source_size;
        if sw == 0 || sh == 0 {
            return;
        }
        let scale = self.live_field_scale();
        let gw = (sw / scale).max(1);
        let gh = (sh / scale).max(1);
        let _ = cached_fluid_paper(gw, gh, scale);
    }

    /// Density→alpha coverage rate for the composite (`WET_COVERAGE_K`).
    #[must_use]
    pub fn fluid_coverage_k(&self) -> f32 {
        WET_COVERAGE_K
    }

    /// Per-step evaporation (the shell evaporates the CPU water mirror by
    /// `this × substeps` per frame — the GPU never touches water in the resident path).
    /// Per-brush (ADR-0079): reads the active brush's watercolor `evaporation` so the CPU
    /// water mirror dries at the same rate the artist set for the GPU solver.
    #[must_use]
    pub fn fluid_evaporation(&self) -> f32 {
        if self.keep_wet {
            return 0.0; // keep-wet: the water mirror must not dry either
        }
        self.brush.rendering.watercolor.evaporation
    }

    /// The active brush's full watercolor tuning projected onto the solver's
    /// [`DiffusionParams`] (ADR-0079) — the bridge uploads ALL 15 controls (diffusion +
    /// deposition + shallow-water flow) to the GPU solver per stroke via
    /// `FluidSolver::set_from_diffusion`, replacing the old global `WATERCOLOR_*` consts.
    ///
    /// **Keep-wet override:** while [`Self::fluid_keep_wet`] is on, `evaporation` is
    /// forced to `0.0` — the single chokepoint for the GPU solver upload, so the field
    /// never dries (the dry-check threshold is never crossed and the field never drops).
    /// The wash still reaches a STABLE wet extent (it does not creep forever): the
    /// shallow-water FlowOutward force is surface-tension-PINNED (ADR-0085 C1,
    /// `shallow.wgsl`), so the front stops where the film thins — equilibrium by physics,
    /// not by evaporation.
    #[must_use]
    pub fn fluid_diffusion_params(&self) -> ph2d_painter_brush::diffusion::DiffusionParams {
        let mut dp = self.brush.rendering.watercolor.to_diffusion();
        if self.keep_wet {
            // Keep Wet keeps a MINIMAL evaporation (NOT 0). A perfectly static (un-drying) front
            // pins into a HARD water step ⇒ a crisp, pixelated rim; a trace of evaporation lets the
            // thin edge water recede through the wet gate, so the rim becomes a soft gradient — a
            // real watercolor edge (Enio: "se há o mínimo de evaporação, a borda fica boa"). The
            // keep-wet drop-guard ([`Self::fluid_dry_check_and_drop_gpu`]) still keeps the field
            // alive, and `evaporation > 0` lets the DRY-gated deposition run, so the edge gets the
            // soft rim while the wet interior stays mobile (wet-on-wet still blends). Keep Wet
            // OVERRIDES the brush's evaporation with this minimal value (the slider is inert while
            // it's on — turn Keep Wet off to dry faster), like it forced 0 before.
            dp.evaporation = KEEP_WET_EVAP;
        }
        // **Deposition follows DRYING (ADR-0085 — the "marked edges don't dissolve when wet" fix).**
        // Pigment strands on the paper as the water LEAVES; the base deposition (0.012/substep) +
        // edge-darkening (`deposition_dry`·dry) otherwise freeze pigment into the non-diffusing
        // `deposited` layer EVEN on a fully wet field — marking stroke edges that never re-dissolve
        // and making wet-on-wet "behave like dry". With no evaporation (Keep Wet, or the Evaporation
        // slider at 0) the wash stays wet, so the pigment must stay MOBILE: zero deposition, no
        // frozen edges, wet-on-wet keeps blending. Evaporation > 0 (the ratified default look)
        // restores deposition + edge-darkening as the wash dries.
        if dp.evaporation <= 0.0 {
            dp.deposition = 0.0;
            dp.deposition_dry = 0.0;
        }
        dp
    }

    /// Keep-wet toggle (watercolor UX): `true` = evaporation paused indefinitely — the
    /// live wash stays wet + re-workable until toggled off (or the canvas changes).
    #[must_use]
    pub fn fluid_keep_wet(&self) -> bool {
        self.keep_wet
    }

    /// Show-wet toggle (watercolor UX): `true` = the wet-paper sheen (subtle darkening
    /// of wet regions + bright meniscus at the wet boundary) renders in the live
    /// preview. VIEW-ONLY — the bridge feeds it to the compositor's preview-texture
    /// flag each frame; the baked composite never sees it. Default ON.
    #[must_use]
    pub fn fluid_show_wet(&self) -> bool {
        self.show_wet
    }

    /// Whether the active brush's capillary fringe (ADR-0078 S5) is on. The drive grows the
    /// composite envelope to follow the outward water-wick only when this is true, so a
    /// non-capillary brush keeps the (cheaper) dab-bbox envelope bit-for-bit unchanged.
    #[must_use]
    pub fn fluid_capillary_active(&self) -> bool {
        self.brush.rendering.watercolor.capillary > 0.0
    }

    /// The static paper-height field of the live grid (clone) — the shell uploads it
    /// ONCE per stroke (on an epoch change) to the resident solver. `None` if no field.
    #[must_use]
    pub fn fluid_paper(&self) -> Option<Vec<f32>> {
        self.wet_field.as_ref().map(|g| g.paper().to_vec())
    }

    /// Grid (low-res) dimensions of the live field, or `None`.
    #[must_use]
    pub fn fluid_grid_dims(&self) -> Option<(u32, u32)> {
        self.wet_field.as_ref().map(|g| g.dims())
    }

    /// The pre-stroke backdrop the wash composites over (canvas-res RGBA8), or `None`.
    #[must_use]
    pub fn fluid_backdrop(&self) -> Option<&[u8]> {
        self.wet_backdrop.as_deref()
    }

    /// **E4 (ADR-0078 S2) — is the layer stack trivial?** `true` when the composite is
    /// byte-identical to `canvas_rgba` (single visible opaque Normal raster, no mask/clip).
    /// The fluid bridge's zero-readback texture path shows the FLUID composite (active layer
    /// over its backdrop) as the whole preview — only valid when the stack IS that one layer;
    /// a non-trivial stack must keep the readback path so the CPU layer compositor flattens
    /// the full stack each frame.
    #[must_use]
    pub fn preview_is_trivial_stack(&self) -> bool {
        self.is_trivial_stack()
    }

    /// **ADR-0084 paper-reveal lift — the "paper" canvas** (canvas-res RGBA8): the active edit
    /// target's content when it became the edit target. The lift donor seeds from
    /// `backdrop − paper` (only PAINT lifts) and the compositor lerps lifted pixels back toward
    /// this (revealing the substrate, never punching alpha holes). `None` when the snapshot
    /// doesn't match the canvas size (defensive) — callers treat that as paper == backdrop
    /// (lift inert).
    #[must_use]
    pub fn fluid_paper_base(&self) -> Option<&[u8]> {
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize) * 4;
        (self.paper_base.len() == n).then(|| self.paper_base.as_slice())
    }

    /// **GPU-resident path (4K real-time arch §4): drain this frame's dab list +
    /// return the composite region.** Replaces [`Self::fluid_frame_step_inputs`] (no
    /// O(grid) deposit/water alloc, no CPU evaporate/scan): `cs_splat` consumes the
    /// dabs onto the resident field; the compositor uses the region. The region is the
    /// MONOTONIC envelope (grown from the dab bboxes in `queue_pointer`), so it never
    /// recedes and never clips the conserved pigment (§2 lesson). `None` when the
    /// field has never been wet (the shell drops it). On idle frames after pen-up the
    /// dab list is empty but the region persists, so the field keeps blooming +
    /// compositing until the GPU dry-check drops it.
    #[must_use]
    pub fn fluid_take_dabs(&mut self) -> Option<FluidDabBatch> {
        let region = self.wet_pigment_envelope?;
        Some((std::mem::take(&mut self.fluid_dabs), region))
    }

    /// Blit a GPU-composited **row band** over `canvas_rgba`, but ONLY the bbox
    /// columns `[px_lo, px_hi)` of `rect = (px_lo, py_lo, px_hi, py_hi)`. `band` is the
    /// full-width readback `(py_hi-py_lo)*cw*4` RGBA8; we copy just the rect's columns
    /// per row — pigment OUTSIDE the active bbox is frozen and must be left untouched
    /// (a full-width blit would erase parts of the stroke that share rows with the
    /// current wet front — the rectangular-cut bug). Marks the preview dirty + bumps
    /// the active layer so the existing preview upload re-blits it.
    pub fn fluid_apply_gpu_composite_rows(&mut self, band: &[u8], rect: (u32, u32, u32, u32)) {
        let (px_lo, py_lo, px_hi, py_hi) = rect;
        let (cw, _ch) = self.source_size;
        let row_bytes = (cw * 4) as usize;
        let n4 = self.canvas_rgba.len();
        let col_lo = px_lo as usize * 4;
        let col_hi = px_hi as usize * 4;
        let band_rows = (py_hi.saturating_sub(py_lo)) as usize;
        if cw == 0
            || px_hi <= px_lo
            || py_hi <= py_lo
            || col_hi > row_bytes
            || py_hi as usize * row_bytes > n4
            || band.len() < band_rows * row_bytes
        {
            return;
        }
        let copy_len = col_hi - col_lo;
        let canvas = Arc::make_mut(&mut self.canvas_rgba);
        for ry in 0..band_rows {
            let band_off = ry * row_bytes + col_lo;
            let canvas_off = (py_lo as usize + ry) * row_bytes + col_lo;
            canvas[canvas_off..canvas_off + copy_len]
                .copy_from_slice(&band[band_off..band_off + copy_len]);
        }
        self.preview_dirty = true;
        self.dirty_rect = None;
        let active = self.layers.active();
        self.bump_layer_pixels(active);
    }

    /// **GPU-resident dry-check (4K real-time arch §4, ADR-0085).** Drop the field when
    /// the GPU-reduced `max_water` ([`ph2d_painter_fluid::FluidSolver::read_field_stats`])
    /// is below the dry threshold AND the stroke has ended — the field's water is
    /// GPU-resident, so the wetness comes from the GPU reduction, not a CPU O(grid) scan.
    /// The shell reads the stats SPORADICALLY (drying is slow), so this is only called on
    /// those frames. Returns `true` when it dropped.
    ///
    /// **Gated on `!stroke_active`:** a mid-stroke PAUSE (button held, no dabs) dries
    /// the field in ~0.3s; dropping it then would make a resumed drag find no field
    /// and paint non-fluid (Enio's pause bug). Keeping it lets a resumed dab re-wet it.
    ///
    /// **Gated on `!keep_wet` (ADR-0085 — the "Keep Wet stops working" fix):** Keep Wet means the
    /// wash stays workable INDEFINITELY, so under it the field must NEVER drop. Forcing evaporation
    /// to 0 (`fluid_diffusion_params`) was supposed to keep `max_water` above the threshold, but
    /// the capillary layer REDISTRIBUTES water outward (thinning the pool) and a thin wash can sit
    /// near the threshold — so `max_water` can dip below it even with evaporation off, dropping the
    /// field despite Keep Wet. Guarding the drop on `keep_wet` makes the feature reliable.
    pub fn fluid_dry_check_and_drop_gpu(&mut self, max_water: f32) -> bool {
        if self.wet_field.is_none() || self.keep_wet {
            return false;
        }
        if max_water < WET_DRY_THRESHOLD && !self.stroke_active {
            self.wet_field = None;
            self.wet_backdrop = None;
            self.wet_composite_bbox = None;
            self.wet_pigment_envelope = None;
            self.fluid_dabs.clear();
            return true;
        }
        false
    }

    pub fn queue_pointer(&mut self, sample: PointerSample) {
        debug_assert!(
            self.stroke_active || self.canvas_rgba.is_empty(),
            "queue_pointer called without an active stroke on a non-empty \
             canvas — the bridge wired pointer-move but not pointer-down. \
             Sample {:?} silently dropped. Audit R7 L1-5.",
            sample
        );
        if !self.stroke_active || self.canvas_rgba.is_empty() {
            return;
        }
        // **Raw-input vs render-output separation (root-cause fix, 2026-05-28):**
        // the vectorial history (`current_samples` + WAL) is the SOURCE record
        // for deterministic replay (W12 Reproject), Inspector (W14) and MCP
        // queries (W13). It must capture EVERY finite input sample —
        // independent of whether the brush *spacing* happened to emit a stamp
        // on THIS event. Pré-fix, an early `return` on `stamps.is_empty()`
        // silently dropped sub-spacing samples (slow drag where consecutive
        // points fall inside `spacing_px`) and stationary pressure-only samples
        // from history, making reproject lossy and breaking the
        // `current_samples_len` / tilt-flag contracts. Stamp emission gates the
        // CANVAS paint only; it must NEVER gate the input record below.
        //
        // Non-finite samples ARE still rejected (mirrors the scheduler's entry
        // guard): NaN/Inf positions are driver garbage, never legitimate input,
        // and would poison the Q16.16 history + WAL replay.
        if !sample.position[0].is_finite()
            || !sample.position[1].is_finite()
            || !sample.pressure.is_finite()
        {
            return;
        }
        // T3.7: alpha-lock restricts paint to the active layer's existing alpha.
        // Read it before the scheduler `&mut` borrow that `stamps` extends (a
        // whole-`self` method can't coexist with that partial mutable borrow).
        let alpha_lock = self.active_alpha_locked();
        let size_px = self.effective_size_px();
        // **ADR-0081:** the active pigment's staining rides every fluid dab (0 for raw colour).
        // Captured here — before the scheduler `&mut` borrow that `stamps` extends — so the
        // per-stamp loops below can use it without a whole-`self` re-borrow.
        let staining = self.active_staining();
        let stamps = self
            .scheduler
            .advance(&self.brush, sample, size_px, self.stroke_color_oklab);
        // Render path: touch the canvas ONLY when the scheduler emitted stamps.
        // Sub-spacing / stationary samples advance scheduler state
        // (residual_dist, pressure) but paint nothing this event — the input
        // record below still captures them.
        if !stamps.is_empty() {
            // **W3 perf:** accumulate the stamped bbox so the next preview drain
            // recomposites only this region, not the full canvas.
            if let Some(bbox) = stamps_bbox(stamps, self.source_size.0, self.source_size.1) {
                self.dirty_rect = Some(match self.dirty_rect {
                    Some(prev) => union_region(prev, bbox),
                    None => bbox,
                });
            }
            // **W15 fluid (ADR-0049 / ADR-0077 D11, ADR-0085 GPU-only):** when the brush
            // opts into the live wet-on-wet field on a GPU-capable device, the dabs are
            // captured as a small list — `cs_splat` adds them to the GPU-resident field on
            // the shell's per-frame drive, which also steps + composites it, so the wash
            // blooms wet-on-wet AS you paint and keeps evolving after pen-up. `wet_field` is
            // only `Some` on a GPU device (gated in `begin_stroke`), so this never touches
            // the canvas. `stamps` borrows `self.scheduler`; `self.wet_field` is disjoint.
            if self.wet_field.is_some() {
                let scale = self.wet_field_scale as f32;
                // **Brush opacity = pigment DILUTION for the fluid path (2026-06-08).** The wash/
                // build-up paths apply `params.opacity` (lines above), but the fluid dab used the
                // scheduler's hardcoded `stamp.opacity` (1.0), so the Opacity slider did nothing
                // to a fluid stroke. Scale the pigment deposit by it — the WATER deposit stays
                // fixed, so lower opacity = the same wetness with LESS pigment = a more dilute,
                // transparent wash (watercolor "aguado"). 1.0 ⇒ unchanged (the validated look).
                let brush_opacity = self.params.opacity.clamp(0.0, 1.0);
                // **Water brush / rewetting (ADR-0079 control 19, 2026-06-09).** `Water` scales
                // the PIGMENT deposit by `1 − water` while the WATER deposit stays full: 1 = a
                // pure-water brush (pre-wet the paper, soften/bloom existing wet paint, and —
                // with Lift — lift dry paint without laying colour); 0 = the validated paint
                // look bit-for-bit. Continuous (0.5 = half-load dilute wash).
                let pigment_load = 1.0 - self.brush.rendering.watercolor.water.clamp(0.0, 1.0);
                // **GPU-resident path (4K real-time arch §4, ADR-0085):** capture the dabs as
                // a small list — `cs_splat` adds them to the resident water/pigment on the
                // shell's per-frame drive, so the CPU never splats into (or scans) the O(grid)
                // field. Grow the MONOTONIC composite envelope from each dab's cell bbox (the
                // §2 lesson: never recede; a superset of the old water bbox, padded by the
                // compositor). `wet_field` is only allocated on a GPU device, so this is the
                // only path — there is no CPU sim fallback.
                debug_assert!(
                    self.fluid_hires,
                    "wet_field allocated without fluid_hires — begin_stroke gate violated"
                );
                {
                    let (gw, gh) = self.wet_field.as_ref().expect("wet_field present").dims();
                    for stamp in stamps {
                        let dep = WET_PIGMENT_DEPOSIT
                            * stamp.opacity.clamp(0.0, 1.0)
                            * brush_opacity
                            * pigment_load;
                        let cx = stamp.position_world[0] / scale;
                        let cy = stamp.position_world[1] / scale;
                        let r =
                            (stamp.size_px * 0.5 / scale).max(if scale > 1.0 { 1.5 } else { 0.5 });
                        // **ADR-0080:** the dab carries the per-stamp COLOUR (with Color Dynamics
                        // jitter) + a colour-independent coverage `mass = dep`. The field stores
                        // it as mass-weighted K/S, so dabs of different colours mix subtractively
                        // (blue+yellow→green) — both within a stroke and across strokes (the wet
                        // field persists). `mass = dep` keeps coverage value-driven (ADR-0079); a
                        // dark/black colour still deposits mass (so black paints).
                        let color = ph2d_painter_brush::cpu_render::oklab_to_linear_srgb(
                            stamp.color_oklab[0],
                            stamp.color_oklab[1],
                            stamp.color_oklab[2],
                        );
                        self.fluid_dabs.push(crate::tool::FluidDab {
                            cx,
                            cy,
                            r,
                            water: WET_WATER_DEPOSIT,
                            color,
                            mass: dep,
                            staining,
                        });
                        let x0 = ((cx - r).floor().max(0.0) as u32).min(gw - 1);
                        let y0 = ((cy - r).floor().max(0.0) as u32).min(gh - 1);
                        let x1 = ((cx + r).ceil().max(0.0) as u32).min(gw - 1);
                        let y1 = ((cy + r).ceil().max(0.0) as u32).min(gh - 1);
                        self.wet_pigment_envelope = Some(match self.wet_pigment_envelope {
                            Some((a, b, c, d)) => (a.min(x0), b.min(y0), c.max(x1), d.max(y1)),
                            None => (x0, y0, x1, y1),
                        });
                    }
                }
                self.preview_dirty = true;
                self.dirty_rect = None;
                self.has_painted_since_source = true;
                let active = self.layers.active();
                self.bump_layer_pixels(active);
            } else {
                // R4-LG-1: `Arc::make_mut` ⇒ unique-borrow path when refcount==1
                // (zero alloc), clone-once when bridge cached the prior Arc
                // (16 MB once per preview drain cycle, NOT per pointer event).
                // Explicit `Vec<u8>` annotation prevents the compiler from
                // coercing through `Arc<[u8]>::make_mut` (different impl).
                let (w, h) = self.source_size;
                let opacity_cap = self.wash_opacity_cap;
                let pigment = self.brush.rendering.pigment_mode
                    == ph2d_painter_brush::PigmentMode::Subtractive;
                let paper_grain = self.params.paper_grain.clamp(0.0, 1.0);
                let canvas_vec: &mut Vec<u8> = Arc::make_mut(&mut self.canvas_rgba);
                // **W5 wash path (accumulate OFF):** composite each dab against the
                // pre-stroke backdrop with opacity-capped coverage — stable, no build-up.
                // `pigment` selects subtractive K-M vs a plain linear lerp (orthogonal).
                // `accumulate ON` clears `wash_coverage` at begin_stroke → falls to the
                // per-dab build-up path. The fields are disjoint from `canvas_rgba`, so
                // the borrow checker permits all three at once.
                match (
                    self.wash_coverage.as_mut(),
                    self.wash_color.as_mut(),
                    self.pending_pre_stroke.as_deref(),
                ) {
                    (Some(coverage), Some(color), Some(backdrop))
                        if backdrop.len() == canvas_vec.len() =>
                    {
                        ph2d_painter_brush::apply_stamps_wash(
                            canvas_vec,
                            backdrop,
                            coverage,
                            color,
                            w,
                            h,
                            stamps,
                            opacity_cap,
                            pigment,
                            alpha_lock,
                            paper_grain,
                        );
                    }
                    // Build-up (`accumulate`) WITH coverage allocated (edges on OR paper
                    // tooth on): render build-up while recording coverage (for the pen-up
                    // edge settle) and applying the global paper tooth.
                    (Some(coverage), None, _) => {
                        ph2d_painter_brush::apply_stamps_buildup(
                            canvas_vec,
                            coverage,
                            w,
                            h,
                            stamps,
                            alpha_lock,
                            paper_grain,
                        );
                    }
                    _ => {
                        apply_stamps_with_options(canvas_vec, w, h, stamps, alpha_lock);
                    }
                }
                self.preview_dirty = true;
                self.has_painted_since_source = true;
                // GPU preview: the active layer's pixels just changed → bump its
                // content version so the compositor re-uploads only its slice.
                let active = self.layers.active();
                self.bump_layer_pixels(active);
            }
        }

        // T1.9: persist sample em history vetorial (in-memory) + journal.
        // Conversão `PointerSample (f32)` → `RawPointerSample (Q16.16 + Q8.8)`
        // via helpers HR-5 cross-OS. Cap u16::MAX preserva semântica
        // ADR-0046 §2.2; sample 65536+ é silenciosamente dropado pra
        // não-bloquear UI (caller raramente atinge — strokes reais têm
        // <512 samples).
        //
        // **Q-5/R-8 clock contract:** `pseudo_now_ms = len * 16` é
        // SYNTHETIC tick — válido só enquanto T1.9 não wire wall-clock.
        // Quando W11+ shell wirar `Instant::elapsed().as_millis()`,
        // `attach_journal` DEVE ser invocado de novo (a journal nova
        // nasce com `last_flush_ms = 0`, evitando flush storm da
        // diferença sintético→wall-clock). Vide doc de `attach_journal`.
        let Some(raw) = pointer_to_raw_sample(sample) else {
            // **B.4 audit-2:** position out of the Q16.16 useful window — drop it
            // (no record, no clamp). Off-window also means off-canvas, so any
            // stamp emitted above already painted nothing.
            return;
        };
        if self.current_samples.len() < u16::MAX as usize {
            self.current_samples.push(raw);
            // **V-2 cascade fix:** only call journal.add_sample if journal
            // has the corresponding Begin (current_stroke.is_some()). Sem
            // o gate, journal.begin_stroke fail em begin_stroke leva todo
            // add_sample subsequente a retornar NoActiveStroke, clobbando
            // last_wal_error original (e.g. PayloadTooLarge → NoActiveStroke).
            if let Some(journal) = self.stroke_journal.as_mut()
                && journal.current_stroke.is_some()
            {
                let pseudo_now_ms = (self.current_samples.len() as u64) * 16;
                // **Q-3/R-3:** captura erro WAL em `last_wal_error` em vez
                // de silenciar. Bridge W11 surface via `last_wal_error()`.
                if let Err(e) = journal.add_sample(raw, pseudo_now_ms) {
                    self.last_wal_error = Some(e);
                }
            }
        } else {
            // **Q-7:** cap u16::MAX hit. Drop silencioso é foot-gun pra
            // MCP path que injeta 70k samples. T1.9 ship: warn via
            // eprintln (until `tracing` joins workspace, R6-LN-1).
            // W11+: auto-split em begin_stroke novo + sentinel
            // `SAMPLE_FLAG_STROKE_SPLIT_BOUNDARY` no último sample
            // pré-split. Carry-over em W11 stitched-stroke.
            eprintln!(
                "[ph2d-painter] queue_pointer: stroke hit MAX_SAMPLES_PER_STROKE \
                 ({}); subsequent samples dropped until end_stroke. Long-stroke \
                 split is W11 carry-over (audit T1.9 Q-7).",
                u16::MAX
            );
        }
    }

    /// Finaliza o stroke atual. Idempotente — chamar duas vezes é seguro.
    ///
    /// **T1.9 — wire history + journal commit:** se houver `PartialStroke`
    /// em-progresso, materializa pra `StrokeRecord` + push em
    /// `stroke_history` + journal.commit_stroke (WAL Commit entry +
    /// fsync). Sem PartialStroke ativo (e.g., end_stroke chamado 2× OR
    /// begin_stroke seguiu sem queue_pointer), é no-op silencioso.
    ///
    /// Fsync síncrono (ADR-0052 §2.2 N-7 doc): caller mobile DEVE wire
    /// em worker thread shell-side. T1.9 ship com path síncrono —
    /// carry-over W11.
    pub fn end_stroke(&mut self) {
        self.scheduler.end_stroke();
        self.stroke_active = false;
        // T1.9: materializar PartialStroke → StrokeRecord + history push.
        let Some(mut partial) = self.current_partial.take() else {
            // Nenhum PartialStroke ⇒ no-op. **Q-10:** defesa-em-profundidade
            // — se journal por algum motivo tem stroke ativo, cancela
            // explicitamente pra não deixar Begin órfão no WAL.
            if let Some(journal) = self.stroke_journal.as_mut()
                && journal.current_stroke.is_some()
            {
                let _ = journal.cancel_stroke();
            }
            self.current_samples.clear();
            // No PartialStroke ⇒ nothing committed ⇒ drop the pre-image so it
            // doesn't leak into the next stroke's commit (W2.T2.2).
            self.pending_pre_stroke = None;
            return;
        };
        // **V-7:** mem::take em vez de mem::replace+Vec::with_capacity —
        // begin_stroke pre-aloca, end_stroke não precisa re-alocar.
        let samples = std::mem::take(&mut self.current_samples);
        // **V-1/V-3 empty-stroke gate (CRITICAL):** pré-V1, set_source
        // mid-stroke + begin_stroke implicit-cancel materializavam empty
        // records → canon polluído com phantom strokes; W2 undo veria
        // ghosts; W12 Reproject re-pintaria nada com seq consumido; W14
        // Inspector mostraria rows vazias.
        //
        // Fix: se samples empty, RECYCLE seq + emit WAL Cancel em vez de
        // Commit. Espelha o Q-2 recycle de begin_stroke implicit-cancel
        // path. Preserva pre-T1.9 semantic "end_stroke sem painting = cancel".
        if samples.is_empty() {
            if let Some(journal) = self.stroke_journal.as_mut()
                && journal.current_stroke.is_some()
            {
                let _ = journal.cancel_stroke();
            }
            // Recycle seq pra próximo stroke usar (Q-2 parity).
            self.next_seq = partial.seq;
            // Empty stroke painted nothing ⇒ drop the pre-image (no undo slot;
            // mirrors the V-1 phantom-record gate above) — W2.T2.2.
            self.pending_pre_stroke = None;
            return;
        }
        // **Wet edges (watercolor) — Phase-B settle / pen-up "dry-down".** The
        // live stroke showed honest wet paint (interior only); now that the stroke
        // is finished, its wet-region boundary is known, so the edge-darkening rim
        // settles in once — pigment transported to the receding water front
        // (`cpu_render::apply_wash_settle`; Curtis et al. SIGGRAPH 1997). This is
        // the Procreate / DiVerdi grow-then-bake lifecycle: a one-frame rim
        // appearing on pen-up, NOT a per-stamp contour filter. Wash-mode only — the
        // coverage buffer IS the wet-region field; build-up brushes have none.
        // Wet (watercolor) takes precedence if both are somehow on; burnt is the
        // dry-media (charcoal / sumi-e) variant of the same transport band.
        let edge_style = if self.brush.rendering.wash_enabled {
            // The GPU wash (ADR-0089) owns edge-darkening (FlowOutward → coverage) and bakes
            // `canvas_rgba` itself at the stroke boundary; skip the CPU settle so the two paths never
            // both write the flat canvas (which would race the undo pre-image).
            None
        } else if self.brush.rendering.wet_edges {
            Some(ph2d_painter_brush::EdgeStyle::Wet)
        } else if self.brush.rendering.burnt_edges {
            Some(ph2d_painter_brush::EdgeStyle::Burnt)
        } else {
            None
        };
        let mut wet_settled = false;
        if let Some(style) = edge_style
            && let Some(coverage) = self.wash_coverage.as_ref()
        {
            let (w, h) = self.source_size;
            if let Some(bbox) = ph2d_painter_brush::coverage_bbox(coverage, w, h) {
                let rim_px = (self.params.size_px * WET_EDGE_RIM_FRACTION).clamp(2.0, 32.0) as u32;
                let seed = partial.rng_seed as u32;
                let strength = self.brush.rendering.edge_intensity.clamp(0.0, 1.0);
                // Granulation strength rides the Paper slider — granulation IS pigment
                // sedimenting into the paper tooth, so its depth tracks the tooth depth
                // (v1.5; Curtis §4.5). The masstone (`wash_color`) + pre-stroke backdrop
                // enable the physically-grounded K–M dry-down; build-up (no wash_color)
                // falls back to the gamma rim inside `apply_wash_settle`.
                let granulation = self.params.paper_grain.clamp(0.0, 1.0);
                let backdrop = self.pending_pre_stroke.as_deref();
                let wash_color = self.wash_color.as_deref();
                let canvas_vec: &mut Vec<u8> = Arc::make_mut(&mut self.canvas_rgba);
                ph2d_painter_brush::apply_wash_settle(
                    canvas_vec,
                    backdrop,
                    coverage,
                    wash_color,
                    w,
                    h,
                    bbox,
                    strength,
                    rim_px,
                    granulation,
                    seed,
                    style,
                );
                wet_settled = true;
            }
        }
        if wet_settled {
            // The rim repainted a sub-rect; force a full recompose on the next
            // preview drain (pen-up, not the hot path) so it shows everywhere.
            self.preview_dirty = true;
            self.dirty_rect = None;
            let active = self.layers.active();
            self.bump_layer_pixels(active);
        }
        partial.samples_count_in_journal = samples.len() as u32;
        // Journal commit primeiro (preserva ordering "wrote to WAL before
        // history"); se WAL falhar, history ainda recebe (caller mobile
        // pode optar perder durability vs. perder stroke commit).
        //
        // **V-2 cascade fix:** só chama commit_stroke se journal tem o
        // Begin desse stroke. Sem o gate, journal_begin_stroke fail em
        // begin_stroke (current_partial=None branch) levaria todo commit
        // subsequent a retornar NoActiveStroke, clobbando last_wal_error.
        if let Some(journal) = self.stroke_journal.as_mut()
            && journal.current_stroke.is_some()
        {
            let pseudo_now_ms = (samples.len() as u64) * 16;
            // **Q-3/R-3:** captura WAL commit failure pra surface bridge.
            if let Err(e) = journal.commit_stroke(pseudo_now_ms) {
                self.last_wal_error = Some(e);
            }
        }
        let committed_seq = partial.seq;
        let record = partial_to_record(partial, samples);
        self.stroke_history.push(record);
        // **W2.T2.2 undo commit:** the stroke painted pixels onto `canvas_rgba`
        // and is now canonical; record the pre-stroke pre-image (captured at
        // `begin_stroke`) so an undo can restore the layer to its pre-stroke
        // state. Keyed by the committed seq for checkpoint thinning. If the
        // pre-image is somehow absent (e.g. a direct `end_stroke` without a
        // matching `begin_stroke` — guarded against, but fail-safe), skip
        // rather than push a bogus slot.
        if let Some(pre) = self.pending_pre_stroke.take() {
            self.undo.record_pre_stroke(committed_seq, &pre);
            // **ADR-0088:** tag this undo entry wash-or-not so undo/redo can keep the wash GPU
            // pigment field in lock-step (a non-wash stroke must NOT shift the wash count).
            let is_wash = self.brush.rendering.wash_enabled;
            self.wash_undo_flags.push(is_wash);
            if is_wash {
                self.wash_active_strokes += 1;
                self.wash_last_change_redo = false; // a fresh commit, NOT a redo (ADR-0089)
            }
            // Stay aligned with the controller's (depth-thinned) stack: a stroke dropped off the
            // FRONT is no longer undoable but is still applied, so drop the flag WITHOUT touching the
            // count (undo can never reach back that far ⇒ the bridge never needs its snapshot).
            let depth = self.undo.undo_depth();
            while self.wash_undo_flags.len() > depth {
                self.wash_undo_flags.remove(0);
            }
        }
        // A NEW committed stroke invalidates the redo branch (standard linear
        // history). The texture redo stack is cleared inside `record_pre_stroke`;
        // clear the parallel semantic redo records here so a later redo can
        // never resurrect a stale record from a discarded branch.
        self.undo_redo_records.clear();
        self.wash_redo_flags.clear(); // ADR-0088: new stroke discards the wash redo branch too
        // **S-5 WAL rotation:** após commit, checka se journal precisa
        // rotacionar (cap 500 MiB OR 100 commits). T1.9 ship: rotate só
        // depois de canon flush (caller W11 contract). Aqui apenas surface
        // o sinal — caller polla via `should_rotate_journal()` accessor.
        // Implementação completa = W11 carry-over S-5 (precisa coordenar
        // com PaintProject save_canon).
    }

    /// Anexa um WAL `StrokeJournal` ao PainterTool. Caller (shell W11+)
    /// invoca após `Tool::on_activate` quando canvas tem persistence path.
    /// `flush_policy` default = `Hybrid { n: 8, ms: 100 }`.
    ///
    /// Retorna erro se:
    /// - `journal_path` já está locked (outro processo / outra thread do
    ///   mesmo processo via in-process registry).
    /// - I/O falha (path inválido, permissions).
    /// - **Audit Q-8/R-1:** `is_stroke_active()` (caller chamou attach
    ///   no meio de um stroke) → `JournalError::AttachDuringActiveStroke`.
    ///
    /// **Audit Q-5/R-8 — clock contract:** o novo journal nasce com
    /// `last_flush_ms = 0`. Caller que migra de sintético (T1.9) pra
    /// wall-clock real (W11+) DEVE chamar `detach_journal` + `attach_journal`
    /// pra reset baseline, evitando flush storm.
    ///
    /// **Audit R-7 — sync fsync na UI thread:** T1.9 wira `commit_stroke` +
    /// `add_sample` DIRETO (sem worker thread). Mobile DEVE wire em worker
    /// thread via canal SPSC (carry-over W11).
    pub fn attach_journal(
        &mut self,
        journal_path: PathBuf,
        flush_policy: FlushPolicy,
    ) -> Result<(), JournalError> {
        // Q-8/R-1: refuse mid-stroke attach pra não deixar stroke órfão.
        if self.stroke_active || self.current_partial.is_some() {
            return Err(JournalError::AttachDuringActiveStroke);
        }
        // **S-6 baseline check:** ADR-0046 §2.2 "seq cresce sempre, replay
        // determinístico depende" + `set_next_seq` doc instrui caller passar
        // `last_persisted_seq + 1` antes de attach. Sem o gate, caller que
        // chame `load(canon)` → `attach_journal(...)` sem `set_next_seq` re-
        // pinta com seq=0 colidindo com strokes históricos.
        //
        // debug_assert apenas em dev; release segue (caller deg risk).
        debug_assert!(
            self.next_seq > 0 || self.stroke_history.is_empty(),
            "attach_journal: set_next_seq(last_persisted_seq + 1) obrigatório \
             ANTES quando stroke_history não-vazio — ADR-0046 §2.2 monotonicity \
             (audit T1.9 S-6)"
        );
        // Defensive: se já tinha journal, drop ele primeiro (libera lock).
        // Q-5/R-8: reset baseline ms naturalmente via journal novo.
        self.stroke_journal = None;
        // Q-3/R-3: limpa estado degradado em re-attach.
        self.last_wal_error = None;
        let journal = StrokeJournal::open(journal_path, flush_policy)?;
        self.stroke_journal = Some(journal);
        Ok(())
    }

    /// **S-4 (audit T1.9):** emit WAL `Heartbeat` entry pra distinguir
    /// "app crashou agora" de "WAL órfão de processo morto há semanas"
    /// (ADR-0052 §2.3). Caller (shell W11+) DEVE invocar a 1Hz idle
    /// (e.g., `Tool::on_tick` extension futuro) — sem heartbeat,
    /// `CrashRecovery::scan` boot trata stale journal como recoverable
    /// strokes velhos, mostrando UX prompt "Recover N strokes?" com
    /// strokes irrelevantes.
    ///
    /// `journal::StrokeJournal::heartbeat` faz rate-limit interno (1Hz);
    /// caller pode chamar mais frequentemente sem custo extra.
    ///
    /// Returns `true` se WAL heartbeat write failed (caller can poll
    /// [`Self::last_wal_error`] pro tipo de erro). Returns `false` quando
    /// journal não anexado OU sucesso. `JournalError` não impl `Clone`
    /// (postcard::Error ABI), por isso bool-return + borrow accessor.
    pub fn heartbeat_journal(&mut self, now_ms: u64) -> bool {
        if let Some(journal) = self.stroke_journal.as_mut()
            && let Err(e) = journal.heartbeat(now_ms)
        {
            self.last_wal_error = Some(e);
            return true;
        }
        false
    }

    /// **S-5 (audit T1.9):** check se WAL atingiu cap rotation (100 commits
    /// OR 50 MiB). Caller (shell W11+) consulta ANTES de chamar
    /// `rotate_journal` pra coordenar com canon save — rotate sem canon
    /// flush pode perder strokes não-persistidos. Carry-over W11: PainterTool
    /// implementar `rotate_journal_if_safe` que coordena com PaintProject.
    #[must_use]
    pub fn should_rotate_journal(&self) -> bool {
        self.stroke_journal
            .as_ref()
            .is_some_and(|j| j.should_rotate())
    }

    /// **W2.T2.1 — `PainterUiSnapshot` projection.** Snapshot read-only
    /// que o sidebar (`ph2d-panel-painter-sidebar`) pinta a cada frame
    /// (shell publica via `set_current_painter_snapshot` antes de paint).
    ///
    /// ADR-0043 §2.3 cap 18 fields; ADR-0040 TG-B unidirecional —
    /// snapshot é display projection, edits voltam via `apply_ui_edit`.
    #[must_use]
    pub fn ui_snapshot(&self) -> crate::params::PainterUiSnapshot {
        crate::params::PainterUiSnapshot {
            size01: crate::params::px_to_size01(self.params.size_px),
            opacity01: self.params.opacity.clamp(0.0, 1.0),
            active_color: self.params.active_color,
            secondary_color: self.params.secondary_color,
            active_brush_thumb: crate::params::ThumbHandle::default(),
            active_brush_name: format!("brush_{}", self.params.active_brush.0),
            mode: self.params.mode,
            eyedropper_armed: self.params.eyedropper_armed,
            symmetry_enabled: self.params.symmetry.is_some(),
            // **W2.T2.2:** reflect the snapshot-undo controller, the actual
            // source of truth for whether undo/redo will do anything (the
            // texture stacks). `stroke_history.is_empty()` could disagree on
            // the `set_source` reset boundary; `can_undo`/`can_redo` are exact.
            undo_enabled: self.undo.can_undo(),
            redo_enabled: self.undo.can_redo(),
            stroke_in_flight: self.stroke_active,
            takeover_active: self.params.takeover_active,
            active_layer_name: String::new(),
            active_layer_locked: false,
            pigment_enabled: self.brush.rendering.pigment_mode
                == ph2d_painter_brush::PigmentMode::Subtractive,
            accumulate_enabled: self.brush.rendering.accumulate,
            grain_type: {
                use ph2d_painter_brush::{GrainSource, ProceduralGrain};
                match &self.brush.grain.grain_source {
                    GrainSource::Procedural(ProceduralGrain::SimplexNoise { .. }) => 1,
                    GrainSource::Procedural(ProceduralGrain::GaborNoise { .. }) => 2,
                    GrainSource::Procedural(ProceduralGrain::PaperWeave { .. }) => 3,
                    GrainSource::Procedural(ProceduralGrain::SprayDot { .. }) => 4,
                    _ => 0, // None / Bitmap / Imported
                }
            },
        }
    }

    /// Re-bake the cached `stroke_color_oklab` from the current
    /// `params.active_color` + `params.opacity` **if a stroke is live**.
    ///
    /// `stroke_color_oklab` is snapshotted at `begin_stroke` (R4-LG-4) to
    /// avoid recomputing two transcendentals per pointer event. A color
    /// edit (`SetColor`/`SetColorSrgb`) or opacity edit mid-stroke must
    /// refresh this cache, otherwise the in-flight stroke silently keeps
    /// the old color until the *next* stroke — the same live-edit contract
    /// `Opacity` established (audit X-7). Between strokes this is a no-op:
    /// `begin_stroke` re-reads `active_color` anyway.
    pub(crate) fn refresh_stroke_color_if_in_flight(&mut self) {
        if self.stroke_active {
            self.stroke_color_oklab = oklch_to_oklab(self.effective_active_color());
            self.stroke_color_oklab[3] *= self.params.opacity.clamp(0.0, 1.0);
        }
    }

    /// **W2.T2.1 — `PainterUiEdit` dispatch.** Mapeia ADR-0043 §2.3
    /// sidebar edits para canon params. Caller (shell drains via
    /// `EditorAction::ToolPanelEvent` → `handle_panel_event`) NÃO
    /// re-implementa semântica — vive aqui pra single source of truth.
    ///
    /// **Anti-pattern (audit T1.6 R7 L1-4 / R8 M1-3):** caller que
    /// muta `params.active_brush` diretamente perde o brush runtime
    /// sync. Use `apply_ui_edit(PainterUiEdit::SelectBrush(h))` (que
    /// chama `set_brush` internamente).
    pub fn apply_ui_edit(&mut self, edit: crate::params::PainterUiEdit) {
        match edit {
            crate::params::PainterUiEdit::Size(v01) => {
                // Sidebar normalizado 0..1 → size_px (SSOT map, audit Y-6).
                self.params.size_px = crate::params::size01_to_px(v01);
            }
            crate::params::PainterUiEdit::Opacity(v01) => {
                self.params.opacity = v01.clamp(0.0, 1.0);
                // Audit X-7: opacity is baked into `stroke_color_oklab[3]`
                // at begin_stroke; refresh it mid-stroke so a live slider
                // edit takes effect on the in-flight stroke instead of
                // silently deferring to the next stroke (T2.1 made this
                // edit live-draggable). Shared with the SetColor path
                // (W2.T2.3) via `refresh_stroke_color_if_in_flight`.
                self.refresh_stroke_color_if_in_flight();
            }
            crate::params::PainterUiEdit::SetColor(c) => {
                // Audit S-8: NaN guard happens em `begin_stroke`; aqui
                // só armazena (caller pode ter passado válido).
                self.params.active_color = c;
                self.refresh_stroke_color_if_in_flight();
            }
            crate::params::PainterUiEdit::SetColorSrgb(rgba) => {
                // W2.T2.3: picker/hex wire format is sRGB8. Convert to the
                // painter-native OKLCH (hue in radians) at the single
                // bridge so the caller never touches the hue convention.
                self.params.active_color = crate::color::srgb8_to_painter_oklch(rgba);
                self.refresh_stroke_color_if_in_flight();
            }
            crate::params::PainterUiEdit::SelectBrush(_h) => {
                // T1.6 R7 L1-4 — `set_brush` exige Brush runtime alongside
                // handle. Sidebar W2 ainda não tem brush library lookup;
                // wire completo é W5 Brush Studio (carry-over T2.3).
            }
            crate::params::PainterUiEdit::ToggleBrushMode => {
                self.params.mode = crate::params::PainterMode::Brush;
            }
            crate::params::PainterUiEdit::ToggleSmudgeMode => {
                self.params.mode = crate::params::PainterMode::Smudge;
            }
            crate::params::PainterUiEdit::ToggleEraserMode => {
                self.params.mode = crate::params::PainterMode::Eraser;
            }
            crate::params::PainterUiEdit::ToggleEyedropper => {
                self.params.eyedropper_armed = !self.params.eyedropper_armed;
            }
            crate::params::PainterUiEdit::Undo => {
                self.undo_last_stroke();
            }
            crate::params::PainterUiEdit::Redo => {
                self.redo_last_stroke();
            }
            crate::params::PainterUiEdit::ResetSidebar => {
                // Long-press reset (ADR-0043 §2.3). Restore defaults
                // EXCEPT history (preservado).
                let defaults = crate::params::PainterParams::default();
                self.params.size_px = defaults.size_px;
                self.params.opacity = defaults.opacity;
                self.params.active_color = defaults.active_color;
                self.params.secondary_color = defaults.secondary_color;
                self.params.mode = defaults.mode;
                self.params.eyedropper_armed = false;
                self.params.symmetry = None;
            }
            crate::params::PainterUiEdit::ToggleSymmetry => {
                self.params.symmetry = match self.params.symmetry {
                    None => Some(crate::params::SymmetryAxis::Vertical),
                    Some(_) => None,
                };
            }
            crate::params::PainterUiEdit::TogglePigment => {
                // Flip the active brush's pigment mode (Linear ↔ Subtractive).
                // Brush param changed → invalidate the cached hash (R-5), same as
                // `set_brush`. A live stroke keeps its baked mode until end_stroke
                // (the wash buffer is set up at begin_stroke from this field).
                use ph2d_painter_brush::PigmentMode;
                self.brush.rendering.pigment_mode = match self.brush.rendering.pigment_mode {
                    PigmentMode::Linear => PigmentMode::Subtractive,
                    _ => PigmentMode::Linear,
                };
                self.cached_brush_hash = None;
            }
            crate::params::PainterUiEdit::ToggleAccumulate => {
                // Flip wash ↔ build-up (orthogonal to pigment). Takes effect at the
                // next begin_stroke (which reads this to set up the wash buffer).
                self.brush.rendering.accumulate = !self.brush.rendering.accumulate;
                self.cached_brush_hash = None;
            }
            crate::params::PainterUiEdit::ToggleGrain => {
                // Cycle the procedural grain generator: Off → Simplex → Gabor →
                // PaperWeave → SprayDot → Off. The scheduler bakes the type into
                // each stamp; the render textures the footprint. Per-type param
                // dials (scale/depth/seed) graduate into the Brush Studio.
                use ph2d_painter_brush::{GrainSource, ProceduralGrain};
                self.brush.grain.grain_source = match &self.brush.grain.grain_source {
                    GrainSource::Procedural(ProceduralGrain::SimplexNoise { .. }) => {
                        GrainSource::Procedural(ProceduralGrain::GaborNoise {
                            frequency: 1.0,
                            orientation: 0.4,
                            anisotropy: 0.5,
                            seed: 0,
                        })
                    }
                    GrainSource::Procedural(ProceduralGrain::GaborNoise { .. }) => {
                        GrainSource::Procedural(ProceduralGrain::PaperWeave {
                            fiber_density: 1.0,
                            fiber_anisotropy: 0.5,
                            crossweave: true,
                            seed: 0,
                        })
                    }
                    GrainSource::Procedural(ProceduralGrain::PaperWeave { .. }) => {
                        GrainSource::Procedural(ProceduralGrain::SprayDot {
                            dot_density: 1.0,
                            dot_size: 0.35,
                            dot_jitter: 0.5,
                            seed: 0,
                        })
                    }
                    GrainSource::Procedural(ProceduralGrain::SprayDot { .. }) => GrainSource::None,
                    _ => GrainSource::Procedural(ProceduralGrain::default_simplex()),
                };
                self.cached_brush_hash = None;
            }
            crate::params::PainterUiEdit::SetGrainDepth(v) => {
                self.brush.grain.grain_depth = v.clamp(0.0, 1.0);
                self.cached_brush_hash = None;
            }
            crate::params::PainterUiEdit::OpenBrushStudio => {
                // W5: unlike OpenLayersPopover/OpenColorPopover (pure shell
                // affordances), the Brush Studio's visibility is tool state
                // (`show_brush_studio`, mirror of `dock_shows_layers`) so all
                // three dock panels + the shell agree without a panel→panel dep.
                self.open_brush_studio();
            }
            crate::params::PainterUiEdit::SetBrushParam(param, v) => {
                self.set_brush_param(param, v);
            }
            // OpenLayersPopover / OpenColorPopover são affordances visuais
            // geridas shell-side (não mutam tool state).
            _ => {}
        }
    }

    /// Write a single scalar Brush field from the Brush Studio (W5). Routed via
    /// [`crate::params::PainterUiEdit::SetBrushParam`]; clamps per field (bools
    /// via `>= 0.5`, `shape_count` rounded, `rendering_mode` decoded by index)
    /// and invalidates `cached_brush_hash` so the next `begin_stroke` re-bakes.
    /// A live stroke keeps its baked params until `end_stroke` (the stamp
    /// scheduler reads these at `begin_stroke`), matching the Toggle* contract.
    fn set_brush_param(&mut self, param: crate::params::BrushParam, v: f32) {
        use crate::params::BrushParam as P;
        let b = &mut self.brush;
        match param {
            P::Spacing => b.stroke_path.spacing = v.clamp(0.01, 1.0),
            P::SpacingJitter => b.stroke_path.spacing_jitter = v.clamp(0.0, 1.0),
            P::JitterLateral => b.stroke_path.jitter_lateral = v.clamp(0.0, 1.0),
            P::Falloff => b.stroke_path.falloff = v.clamp(0.0, 1.0),
            // Slider 0..1 → taper_length_start 0..0.5 (the spec range); the tip
            // defaults (size/opacity start = 0) make it a clean pointed entry.
            P::TaperLength => b.taper.taper_length_start = v.clamp(0.0, 1.0) * 0.5,
            P::StreamlineAmount => b.stabilization.streamline_amount = v.clamp(0.0, 1.0),
            P::Stabilization => b.stabilization.stabilization = v.clamp(0.0, 1.0),
            P::MotionFiltering => b.stabilization.motion_filtering_amount = v.clamp(0.0, 1.0),
            P::MotionExpression => b.stabilization.motion_filtering_expression = v.clamp(0.0, 1.0),
            // Bipolar: slider 0..1 → speed_* −1..1 (0.5 = neutral / off).
            P::SpeedSize => b.dynamics.speed_size = (v * 2.0 - 1.0).clamp(-1.0, 1.0),
            P::SpeedOpacity => b.dynamics.speed_opacity = (v * 2.0 - 1.0).clamp(-1.0, 1.0),
            P::SpeedSpacing => b.dynamics.speed_spacing = (v * 2.0 - 1.0).clamp(-1.0, 1.0),
            P::ShapeScatter => b.shape.shape_scatter = v.clamp(0.0, 1.0),
            // Sliders always emit 0..1; map to the field's natural range here
            // (the panel inverts it for display) — same split as `size01_to_px`.
            P::ShapeCount => b.shape.shape_count = (1.0 + v.clamp(0.0, 1.0) * 15.0).round() as u32,
            P::ShapeCountJitter => b.shape.shape_count_jitter = v.clamp(0.0, 1.0),
            P::ShapeRoundness => b.shape.shape_roundness = v.clamp(0.0, 1.0),
            P::ShapeRotationFollow => b.shape.shape_rotation_follow = v >= 0.5,
            P::ShapeRandomized => b.shape.shape_randomized = v >= 0.5,
            P::ShapeFlipX => b.shape.shape_flip_x = v >= 0.5,
            P::ShapeFlipY => b.shape.shape_flip_y = v >= 0.5,
            P::Flow => b.rendering.flow = v.clamp(0.0, 1.0),
            P::AlphaThreshold => b.rendering.alpha_threshold = v.clamp(0.0, 1.0),
            P::WetEdges => b.rendering.wet_edges = v >= 0.5,
            P::BurntEdges => b.rendering.burnt_edges = v >= 0.5,
            // Live watercolor fluid diffusion (ADR-0049 / ADR-0077 D11). Takes
            // effect at the next begin_stroke, which allocates `wet_field` when
            // this is set (the live stroke keeps its baked mode until end_stroke).
            P::Fluid => {
                b.rendering.fluid_enabled = v >= 0.5;
                if b.rendering.fluid_enabled {
                    b.rendering.wash_enabled = false; // mutually exclusive (ADR-0087)
                }
            }
            // Minimal watercolor core (ADR-0087) — parallel to Fluid, mutually exclusive.
            P::Wash => {
                b.rendering.wash_enabled = v >= 0.5;
                if b.rendering.wash_enabled {
                    b.rendering.fluid_enabled = false;
                }
            }
            P::EdgeIntensity => b.rendering.edge_intensity = v.clamp(0.0, 1.0),
            // Substrate property → tool params (disjoint field from `b`).
            P::Paper => self.params.paper_grain = v.clamp(0.0, 1.0),
            P::RenderingMode => {
                b.rendering.rendering_mode =
                    ph2d_painter_brush::RenderingMode::from_u32(v.round().max(0.0) as u32);
            }
            P::GrainScale => b.grain.grain_scale = 0.1 + v.clamp(0.0, 1.0) * 3.9,
            P::HueJitter => b.color_dynamics.stamp_hue_jitter = v.clamp(0.0, 1.0),
            P::SaturationJitter => b.color_dynamics.stamp_saturation_jitter = v.clamp(0.0, 1.0),
            P::LightnessJitter => b.color_dynamics.stamp_lightness_jitter = v.clamp(0.0, 1.0),
            P::DarknessJitter => b.color_dynamics.stamp_darkness_jitter = v.clamp(0.0, 1.0),
            P::SizeJitter => b.dynamics.jitter_size = v.clamp(0.0, 1.0),
            P::OpacityJitter => b.dynamics.jitter_opacity = v.clamp(0.0, 1.0),
            // Watercolor control `i` (ADR-0079): the slider's normalized 0..1 maps onto the
            // control's physical range. Takes effect at the next begin_stroke (the bridge
            // reads `watercolor` then), like the other fluid params.
            P::Watercolor(i) => b.rendering.watercolor.set_normalized(i as usize, v),
            // Watercolor UX toggles — TOOL-level state (like `Paper`), not brush fields.
            // KeepWet takes effect immediately (the bridge re-uploads the solver params
            // when it flips); ShowWet is read by the bridge per frame (view-only sheen).
            P::KeepWet => self.keep_wet = v >= 0.5,
            P::ShowWet => self.show_wet = v >= 0.5,
        }
        self.cached_brush_hash = None;
    }

    /// Build the [`crate::params::BrushStudioSnapshot`] the shell publishes to
    /// the `ph2d-panel-brush-studio` panel each frame (W5). Read-only projection
    /// of the active [`ph2d_painter_brush::Brush`] — the studio's uncapped
    /// companion to [`Self::ui_snapshot`].
    #[must_use]
    pub fn brush_studio_snapshot(&self) -> crate::params::BrushStudioSnapshot {
        use ph2d_painter_brush::{GrainSource, PigmentMode, ProceduralGrain};
        let b = &self.brush;
        crate::params::BrushStudioSnapshot {
            spacing: b.stroke_path.spacing,
            spacing_jitter: b.stroke_path.spacing_jitter,
            jitter_lateral: b.stroke_path.jitter_lateral,
            falloff: b.stroke_path.falloff,
            taper_length: b.taper.taper_length_start * 2.0,
            streamline_amount: b.stabilization.streamline_amount,
            stabilization: b.stabilization.stabilization,
            motion_filtering_amount: b.stabilization.motion_filtering_amount,
            motion_filtering_expression: b.stabilization.motion_filtering_expression,
            shape_scatter: b.shape.shape_scatter,
            shape_count: b.shape.shape_count,
            shape_count_jitter: b.shape.shape_count_jitter,
            shape_roundness: b.shape.shape_roundness,
            shape_rotation_follow: b.shape.shape_rotation_follow,
            shape_randomized: b.shape.shape_randomized,
            shape_flip_x: b.shape.shape_flip_x,
            shape_flip_y: b.shape.shape_flip_y,
            flow: b.rendering.flow,
            alpha_threshold: b.rendering.alpha_threshold,
            wet_edges: b.rendering.wet_edges,
            burnt_edges: b.rendering.burnt_edges,
            fluid_enabled: b.rendering.fluid_enabled,
            wash_enabled: b.rendering.wash_enabled,
            edge_intensity: b.rendering.edge_intensity,
            pigment_enabled: b.rendering.pigment_mode == PigmentMode::Subtractive,
            accumulate_enabled: b.rendering.accumulate,
            rendering_mode: b.rendering.rendering_mode as u8,
            grain_type: match &b.grain.grain_source {
                GrainSource::Procedural(ProceduralGrain::SimplexNoise { .. }) => 1,
                GrainSource::Procedural(ProceduralGrain::GaborNoise { .. }) => 2,
                GrainSource::Procedural(ProceduralGrain::PaperWeave { .. }) => 3,
                GrainSource::Procedural(ProceduralGrain::SprayDot { .. }) => 4,
                _ => 0,
            },
            grain_scale: b.grain.grain_scale,
            grain_depth: b.grain.grain_depth,
            // Paper tooth is a substrate property → tool params, not the brush.
            paper_grain: self.params.paper_grain,
            stamp_hue_jitter: b.color_dynamics.stamp_hue_jitter,
            stamp_saturation_jitter: b.color_dynamics.stamp_saturation_jitter,
            stamp_lightness_jitter: b.color_dynamics.stamp_lightness_jitter,
            stamp_darkness_jitter: b.color_dynamics.stamp_darkness_jitter,
            jitter_size: b.dynamics.jitter_size,
            jitter_opacity: b.dynamics.jitter_opacity,
            speed_size: b.dynamics.speed_size,
            speed_opacity: b.dynamics.speed_opacity,
            speed_spacing: b.dynamics.speed_spacing,
            watercolor: core::array::from_fn(|i| b.rendering.watercolor.normalized(i)),
            active_pigment: self.active_pigment,
            keep_wet: self.keep_wet,
            show_wet: self.show_wet,
            brush_name: format!("brush_{}", self.params.active_brush.0),
        }
    }

    // =========================================================================
    // W2.T2.2 — undo / redo public API (driven by gesture dispatch).
    //
    // The gesture wiring is foundational/shell (Coordinator): a 2-finger tap
    // resolves to `EditorAction::Undo` and a 3-finger tap to `EditorAction::
    // Redo`. The shell calls these methods (or routes through
    // `apply_ui_edit(PainterUiEdit::Undo / ::Redo)`, which delegates here).
    // Both restore the layer texture in place and keep `stroke_history` (the
    // semantic canon) in sync, so any consumer of either stays consistent.
    // =========================================================================

    /// Undo the most recently committed stroke: restore the layer texture to
    /// its pre-stroke pre-image and pop the matching semantic record onto the
    /// redo branch. Returns `true` if a stroke was undone (the canvas changed),
    /// `false` if there was nothing to undo. After a successful undo the live
    /// preview is marked dirty so the shell re-blits the restored texture.
    ///
    /// **Active-stroke contract:** an in-flight stroke is committed
    /// (`end_stroke`) before undoing, so the undo lands on a clean stroke
    /// boundary (mirrors the implicit-close in `begin_stroke`).
    pub fn undo_last_stroke(&mut self) -> bool {
        if self.stroke_active {
            self.end_stroke();
        }
        // Pass the live (post-stroke) canvas so the controller can stash it for
        // a later redo, and hand back the pre-stroke pre-image to restore.
        let current = self.canvas_rgba.as_ref().clone();
        let Some(pixels) = self.undo.undo(&current) else {
            return false;
        };
        // Restore pixels in place; keep the Arc allocation when uniquely owned.
        let canvas = Arc::make_mut(&mut self.canvas_rgba);
        *canvas = pixels;
        // **W15 fluid:** a live wash keeps blooming over `pending_pre_stroke`
        // after pen-up (`on_tick`). Undo backs the stroke out, so the bloom must
        // stop — otherwise it re-composites the wash onto the restored pre-image.
        // Dropping the field neutralises the composite (it no-ops without one).
        self.wet_field = None;
        self.wet_backdrop = None;
        self.wet_composite_bbox = None;
        // Keep the semantic canon in lock-step: pop the record the undo backed
        // out of, holding it so `redo` can re-insert it (StrokeHistory::redo
        // needs the popped record — the gap this task closed).
        if let Some(rec) = self.stroke_history.undo() {
            self.undo_redo_records.push(rec);
        }
        // **ADR-0088:** move the wash flag from the undo side to the redo side; drop the wash count
        // only if the undone entry was a wash stroke (the bridge restores its field snapshot).
        if let Some(was_wash) = self.wash_undo_flags.pop() {
            if was_wash {
                self.wash_active_strokes = self.wash_active_strokes.saturating_sub(1);
            }
            self.wash_redo_flags.push(was_wash);
        }
        self.preview_dirty = true;
        // GPU preview: the active layer's pixels were restored → bump version.
        let active = self.layers.active();
        self.bump_layer_pixels(active);
        true
    }

    /// Redo the most recently undone stroke: re-apply its post-stroke texture
    /// and re-insert its semantic record. Returns `true` if a stroke was
    /// redone, `false` if the redo branch was empty.
    pub fn redo_last_stroke(&mut self) -> bool {
        if self.stroke_active {
            self.end_stroke();
        }
        // Pass the live (pre-stroke) canvas so the controller can stash it for a
        // later undo, and hand back the post-stroke image to restore.
        let current = self.canvas_rgba.as_ref().clone();
        let Some(pixels) = self.undo.redo(&current) else {
            return false;
        };
        let canvas = Arc::make_mut(&mut self.canvas_rgba);
        *canvas = pixels;
        // **W15 fluid:** redo restores a frozen post-stroke image; any field that
        // was still blooming belongs to a different timeline. Drop it so it can't
        // re-composite over the restored pixels (symmetric with `undo_last_stroke`).
        self.wet_field = None;
        self.wet_backdrop = None;
        self.wet_composite_bbox = None;
        // Re-insert the semantic record we held back during the matching undo.
        if let Some(rec) = self.undo_redo_records.pop() {
            self.stroke_history.redo(rec);
        }
        // **ADR-0088:** symmetric to undo — move the wash flag back to the undo side, re-adding the
        // wash count if the redone entry was a wash stroke (the bridge re-applies its snapshot).
        if let Some(was_wash) = self.wash_redo_flags.pop() {
            if was_wash {
                self.wash_active_strokes += 1;
                self.wash_last_change_redo = true; // this count rise IS a redo (ADR-0089)
            }
            self.wash_undo_flags.push(was_wash);
        }
        self.preview_dirty = true;
        // GPU preview: the active layer's pixels were restored → bump version.
        let active = self.layers.active();
        self.bump_layer_pixels(active);
        true
    }

    /// **ADR-0088:** committed-and-not-undone WASH stroke count — the shell's `drive_wash_gpu` polls
    /// this to keep the GPU pigment field synced with undo/redo (restoring per-stroke snapshots).
    #[must_use]
    pub fn wash_active_strokes(&self) -> usize {
        self.wash_active_strokes
    }

    /// **ADR-0089:** `true` iff the most recent `wash_active_strokes` rise was a REDO (not a fresh
    /// commit). Lets `drive_wash_gpu` distinguish a new stroke after undo from a redo without a
    /// frame-timing heuristic (a fast stroke could bump the count before the bridge saw its dabs).
    #[must_use]
    pub fn wash_last_change_redo(&self) -> bool {
        self.wash_last_change_redo
    }

    /// Generation that bumps when the wash's canvas base is invalidated (new source / layer switch).
    /// The shell drops + rebuilds its persistent wash session on a change.
    #[must_use]
    pub fn wash_reset_generation(&self) -> u64 {
        self.wash_reset_generation
    }

    /// `true` if there is at least one committed stroke to undo (drives the
    /// sidebar `undo_enabled` affordance).
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.undo.can_undo()
    }

    /// `true` if there is at least one undone stroke to redo (drives the
    /// sidebar `redo_enabled` affordance — previously hardcoded `false`).
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.undo.can_redo()
    }

    /// Desfaz o anexo de journal — drop libera advisory lock + registry.
    ///
    /// **Audit R-2:** se há stroke em-progresso quando detach acontece,
    /// emite `Cancel` no WAL ANTES do drop. Sem isso, o Begin permanecia
    /// órfão → próxima `CrashRecovery::scan` via re-attach false-positiva
    /// o stroke como `InProgressAtCrash`.
    pub fn detach_journal(&mut self) {
        if let Some(journal) = self.stroke_journal.as_mut()
            && journal.current_stroke.is_some()
        {
            let _ = journal.cancel_stroke();
        }
        self.stroke_journal = None;
    }

    /// Configura `CanvasId` deste PainterTool. Caller (shell W11+) invoca
    /// quando canvas ativo muda. Default = `CanvasId(0)` (single-canvas).
    ///
    /// **S-7 (audit T1.9):** ADR-0052 §2.6 state machine `[idle] ↔
    /// [stroke_active]` proíbe mutações cross-canvas mid-stroke. Sem
    /// guard, mudar canvas_id mid-stroke confunde recovery (filtra
    /// canvas antigo) + cross-canvas stroke pollution.
    pub fn set_canvas_id(&mut self, id: CanvasId) {
        debug_assert!(
            !self.stroke_active,
            "set_canvas_id called mid-stroke — ADR-0052 §2.6 lifecycle \
             violation (audit T1.9 S-7)"
        );
        self.canvas_id = id;
    }

    /// Configura `LayerId` alvo pra próximos strokes. W3 layers nasce ⇒
    /// caller wire conforme active layer selection.
    ///
    /// **S-7 (audit T1.9):** mid-stroke = lifecycle violation. Vide
    /// [`Self::set_canvas_id`] rationale.
    pub fn set_layer_target(&mut self, layer: LayerId) {
        debug_assert!(
            !self.stroke_active,
            "set_layer_target called mid-stroke — ADR-0052 §2.6 lifecycle \
             violation (audit T1.9 S-7)"
        );
        self.layer_target = layer;
    }
}
