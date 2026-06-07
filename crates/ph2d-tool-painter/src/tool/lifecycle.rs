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
const WET_FIELD_SCALE: u32 = 2;
const WET_WATER_DEPOSIT: f32 = 0.55;
const WET_PIGMENT_DEPOSIT: f32 = 0.5;
const WET_SUBSTEPS_PAINTING: u32 = 1;
const WET_SUBSTEPS_IDLE: u32 = 2;
const WET_DRY_THRESHOLD: f32 = 0.045;
/// Coverage rate: `alpha = 1 − exp(−amount · K)`, where `amount` is the
/// COLOUR-INDEPENDENT pigment load. The grid stores colour×amount, so the raw
/// channel sum is `amount · Σcolour` — luminance-weighted, which made bright
/// pigments (yellow/magenta) read as fully opaque while blue/red stayed a proper
/// translucent wash. Normalising by the stroke colour's linear sum recovers
/// `amount`; `K` is anchored to the (already-correct) blue/red look — their
/// `Σcolour ≈ 0.53`, so `K = old 2.0 × 0.53 ≈ 1.06` keeps them unchanged.
const WET_COVERAGE_K: f32 = 1.06;

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

        // **W15 fluid:** (re)allocate the live wet-on-wet diffusion field when the
        // brush opts in. A fresh field per stroke (v1 — cross-stroke wet-on-wet is a
        // W15.3 refinement); the previous stroke's wash was already composited into
        // the canvas, so resetting just "sets" it. `None` for non-fluid brushes.
        if self.brush.rendering.fluid_enabled {
            let (sw, sh) = self.source_size;
            let gw = (sw / WET_FIELD_SCALE).max(1);
            let gh = (sh / WET_FIELD_SCALE).max(1);
            self.wet_field = Some(ph2d_painter_brush::diffusion::DiffusionGrid::new(
                gw,
                gh,
                WET_FIELD_SCALE as f32,
            ));
            // W15.3: signal the shell GPU drive that a FRESH field began, so it
            // resets the resident pigment (a reused solver must not inherit the
            // previous stroke's bloom).
            self.fluid_stroke_epoch = self.fluid_stroke_epoch.wrapping_add(1);
            // The wash composites over the pre-stroke canvas for the WHOLE life of
            // the field — including the many frames it keeps blooming after pen-up.
            // `pending_pre_stroke` can't serve this (it's consumed by the undo stack
            // at `end_stroke`), so snapshot a dedicated backdrop here.
            self.wet_backdrop = Some(self.canvas_rgba.as_ref().clone());
        } else {
            self.wet_field = None;
            self.wet_backdrop = None;
        }
        // Fresh stroke ⇒ no previous wet region to union against.
        self.wet_composite_bbox = None;

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
    /// **W15 idle tick** (driven by `Tool::on_tick` each frame). Advances the live
    /// wet field while it stays wet — the wash keeps blooming + drying after pen-up
    /// ("the paint stays wet"). No-op once the field has dried + been dropped.
    pub(crate) fn on_tick_diffusion(&mut self) {
        // GPU-driven: the shell steps + composites the field (skip the CPU path).
        if self.gpu_fluid_driven || self.wet_field.is_none() {
            return;
        }
        self.tick_wet_field(WET_SUBSTEPS_IDLE);
        self.preview_dirty = true;
        self.dirty_rect = None;
    }

    /// Step the live wet field `substeps` times (CPU) + composite + settle. Shared
    /// by the painting splat and the idle tick.
    fn tick_wet_field(&mut self, substeps: u32) {
        let params = ph2d_painter_brush::diffusion::DiffusionParams::default();
        match self.wet_field.as_mut() {
            Some(grid) => {
                for _ in 0..substeps {
                    grid.step(&params);
                }
            }
            None => return,
        }
        self.composite_and_settle_fluid();
    }

    /// Composite the (already-stepped) wet field into the canvas + drop it once it
    /// dries (wettest cell < `WET_DRY_THRESHOLD`). The **step-agnostic** half of the
    /// live tick: the CPU path calls it right after a CPU `step`; the shell GPU path
    /// (W15.3) calls it after [`ph2d_painter_fluid::FluidSolver::step_grid`] has
    /// written the GPU-evolved grid back (`fluid_grid_mut`). Sets `preview_dirty`.
    pub fn composite_and_settle_fluid(&mut self) {
        let dry = match self.wet_field.as_ref() {
            // Dry only when the WETTEST cell is below the gate — a mean reads "dry"
            // instantly (the grid is mostly bare paper).
            Some(grid) => grid.water().iter().copied().fold(0.0f32, f32::max) < WET_DRY_THRESHOLD,
            None => return,
        };
        self.composite_wet_field();
        self.preview_dirty = true;
        if dry {
            // Final pigment is baked into the canvas (the composite above ran first).
            self.wet_field = None;
            self.wet_backdrop = None;
            self.wet_composite_bbox = None;
        }
    }

    /// W15.3 shell GPU drive: set by the shell when it is stepping the wet field on
    /// the GPU, so the tool skips its CPU diffusion (`on_tick`/`queue_pointer`).
    pub fn set_gpu_fluid_driven(&mut self, v: bool) {
        self.gpu_fluid_driven = v;
    }

    /// Mutable access to the live wet field so the shell GPU stepper can advance it
    /// in place; `None` when there is no live field (dry / non-fluid brush).
    pub fn fluid_grid_mut(
        &mut self,
    ) -> Option<&mut ph2d_painter_brush::diffusion::DiffusionGrid> {
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

    // ───────────────── W15.3 GPU resident-composite hooks (shell-facing) ─────────────────

    /// Bumps each `begin_stroke` that allocates a fresh fluid field — the shell
    /// resets the GPU-resident pigment + re-uploads paper on change. See
    /// [`Self::fluid_stroke_epoch`] field doc.
    #[must_use]
    pub fn fluid_stroke_epoch(&self) -> u64 {
        self.fluid_stroke_epoch
    }

    /// Canvas/grid ratio (the field runs at 1/scale of the canvas).
    #[must_use]
    pub fn fluid_field_scale(&self) -> u32 {
        WET_FIELD_SCALE
    }

    /// Density→alpha coverage rate for the composite (`WET_COVERAGE_K`).
    #[must_use]
    pub fn fluid_coverage_k(&self) -> f32 {
        WET_COVERAGE_K
    }

    /// Per-step evaporation (the shell evaporates the CPU water mirror by
    /// `this × substeps` per frame — the GPU never touches water in the resident path).
    #[must_use]
    pub fn fluid_evaporation(&self) -> f32 {
        ph2d_painter_brush::diffusion::DiffusionParams::default().evaporation
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

    /// The stroke colour in linear sRGB — the composite derives `pcol` + the coverage
    /// normaliser from it (the resident path's grid holds only the per-frame deposit,
    /// so the chromaticity comes from the stroke, not a grid total; they're equal).
    #[must_use]
    pub fn fluid_stroke_color_linear(&self) -> [f32; 3] {
        ph2d_painter_brush::cpu_render::oklab_to_linear_srgb(
            self.stroke_color_oklab[0],
            self.stroke_color_oklab[1],
            self.stroke_color_oklab[2],
        )
    }

    /// The pre-stroke backdrop the wash composites over (canvas-res RGBA8), or `None`.
    #[must_use]
    pub fn fluid_backdrop(&self) -> Option<&[u8]> {
        self.wet_backdrop.as_deref()
    }

    /// Pull this frame's GPU-drive inputs: the dab `deposit` (the grid pigment, then
    /// CLEARED), the `water` mirror (then EVAPORATED by `evap_per_frame`), the wet
    /// `region` (water bbox ∪ last frame, stored), and the grid `dims`. `None` when
    /// the field is dry / absent (the shell then drops it). The water is captured
    /// PRE-evaporation (the GPU gate/flow read it), then evaporated for next frame +
    /// the dry-check.
    #[must_use]
    pub fn fluid_frame_step_inputs(
        &mut self,
        evap_per_frame: f32,
    ) -> Option<crate::tool::FluidFrameInputs> {
        let prev_bbox = self.wet_composite_bbox;
        let grid = self.wet_field.as_mut()?;
        let (gw, gh) = grid.dims();
        // Any wetness ⇒ the gate may have spread pigment there; a small threshold so
        // the region is a superset of the pigment (the composite short-circuits dry
        // pixels anyway).
        let cur = grid.water_bbox(1.0e-3);
        let region = match (cur, prev_bbox) {
            (Some(a), Some(b)) => (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => {
                // Bare field — nothing wet. Let the shell drop it via the dry-check.
                self.wet_composite_bbox = None;
                return None;
            }
        };
        self.wet_composite_bbox = cur;
        let deposit: Vec<[f32; 4]> = grid.pigment().iter().map(|p| [p[0], p[1], p[2], 0.0]).collect();
        let water = grid.water().to_vec();
        grid.clear_pigment();
        grid.evaporate(evap_per_frame);
        Some(crate::tool::FluidFrameInputs {
            deposit,
            water,
            region,
            dims: (gw, gh),
        })
    }

    /// Blit a GPU-composited canvas-width **row band** (`rows` = `(py_hi-py_lo)*cw*4`
    /// RGBA8) over `canvas_rgba` rows `[py_lo, py_hi)` — the resident path's apply
    /// (the GPU composited; the shell read back only this band). Marks the preview
    /// dirty + bumps the active layer so the existing preview upload re-blits it.
    pub fn fluid_apply_gpu_composite_rows(&mut self, rows: &[u8], py_lo: u32, py_hi: u32) {
        let (cw, _ch) = self.source_size;
        let row_bytes = (cw * 4) as usize;
        let start = py_lo as usize * row_bytes;
        let end = py_hi as usize * row_bytes;
        let n4 = self.canvas_rgba.len();
        if cw == 0 || end <= start || end > n4 || rows.len() < end - start {
            return;
        }
        let canvas = Arc::make_mut(&mut self.canvas_rgba);
        canvas[start..end].copy_from_slice(&rows[..end - start]);
        self.preview_dirty = true;
        self.dirty_rect = None;
        let active = self.layers.active();
        self.bump_layer_pixels(active);
    }

    /// Dry-check on the CPU water mirror: if the wettest cell is below the dry
    /// threshold, drop the field (its final pigment was already composited this
    /// frame). Returns `true` when it dropped. The GPU-resident twin of the dry half
    /// of [`Self::composite_and_settle_fluid`].
    pub fn fluid_dry_check_and_drop(&mut self) -> bool {
        let dry = match self.wet_field.as_ref() {
            Some(grid) => grid.max_water() < WET_DRY_THRESHOLD,
            None => return false,
        };
        if dry {
            self.wet_field = None;
            self.wet_backdrop = None;
            self.wet_composite_bbox = None;
        }
        dry
    }

    /// Composite the low-res wet field over the pre-stroke backdrop into the canvas
    /// (**bicubic upsample + Kubelka–Munk subtractive glaze**, scoped to the wet
    /// bbox). The per-pixel math is the shared CPU reference
    /// ([`ph2d_painter_brush::wet_composite::composite_wet_field_cpu`]) — ONE
    /// definition the GPU port mirrors band-for-band (W15.3 parity gate). This is
    /// the CPU fallback; the shell drives the GPU compositor when a real device is
    /// available (ADR-0049). Bare-paper pixels short-circuit to the backdrop, so the
    /// spectral solve already tracks the wet region.
    fn composite_wet_field(&mut self) {
        use ph2d_painter_brush::wet_composite::{
            composite_wet_field_cpu, prepare_wet_composite, wet_pigment_bbox,
        };
        let (cw, ch) = self.source_size;
        let n4 = (cw as usize) * (ch as usize) * 4;
        // The dedicated fluid backdrop — outlives the stroke, unlike
        // `pending_pre_stroke` (consumed by the undo stack at `end_stroke`).
        let Some(backdrop) = self.wet_backdrop.as_ref() else {
            return;
        };
        let Some(grid) = self.wet_field.as_ref() else {
            return;
        };
        if backdrop.len() != n4 {
            return;
        }
        let (gw, gh) = grid.dims();
        let pig = grid.pigment();
        // Wet bbox (grid cells), unioned with last frame's so cells that just DRIED
        // get their canvas pixel reset to the backdrop. Scopes the whole composite to
        // the wash neighbourhood.
        let cur_bbox = wet_pigment_bbox(pig, gw, gh);
        let region = match (cur_bbox, self.wet_composite_bbox) {
            (Some(a), Some(b)) => Some((a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        };
        self.wet_composite_bbox = cur_bbox;
        let Some(region) = region else {
            return;
        };
        // Amortised K–M brush prep (a fluid stroke is one colour): chromaticity from
        // the total pigment mass + the colour-independent coverage normaliser from
        // the stroke colour — derived ONCE per composite.
        let scol = ph2d_painter_brush::cpu_render::oklab_to_linear_srgb(
            self.stroke_color_oklab[0],
            self.stroke_color_oklab[1],
            self.stroke_color_oklab[2],
        );
        let brush = prepare_wet_composite(pig, scol);
        let canvas = Arc::make_mut(&mut self.canvas_rgba);
        if canvas.len() != n4 {
            return;
        }
        composite_wet_field_cpu(
            canvas,
            backdrop,
            pig,
            gw,
            gh,
            cw,
            ch,
            WET_FIELD_SCALE,
            WET_COVERAGE_K,
            &brush,
            region,
        );
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
            // **W15 fluid (ADR-0049 / ADR-0077 D11):** when the brush opts into the
            // live wet-on-wet field, the dabs splat into the diffusion grid instead
            // of the canvas; this call (+ `on_tick`) steps the diffusion + composites
            // it out, so the wash blooms wet-on-wet AS you paint and keeps evolving
            // after pen-up. `stamps` borrows `self.scheduler`; `self.wet_field` is a
            // disjoint field, so the splat coexists with that borrow.
            if self.wet_field.is_some() {
                {
                    let scale = WET_FIELD_SCALE as f32;
                    let grid = self.wet_field.as_mut().expect("wet_field present");
                    for stamp in stamps {
                        let rgb = ph2d_painter_brush::cpu_render::oklab_to_linear_srgb(
                            stamp.color_oklab[0],
                            stamp.color_oklab[1],
                            stamp.color_oklab[2],
                        );
                        let dep = WET_PIGMENT_DEPOSIT * stamp.opacity.clamp(0.0, 1.0);
                        grid.splat(
                            stamp.position_world[0] / scale,
                            stamp.position_world[1] / scale,
                            (stamp.size_px * 0.5 / scale).max(0.5),
                            WET_WATER_DEPOSIT,
                            [rgb[0] * dep, rgb[1] * dep, rgb[2] * dep],
                        );
                    }
                }
                // GPU-driven (W15.3): only splat here; the shell's per-frame GPU
                // stepper advances + composites the field. CPU path steps inline.
                if !self.gpu_fluid_driven {
                    self.tick_wet_field(WET_SUBSTEPS_PAINTING);
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
        let edge_style = if self.brush.rendering.wet_edges {
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
        }
        // A NEW committed stroke invalidates the redo branch (standard linear
        // history). The texture redo stack is cleared inside `record_pre_stroke`;
        // clear the parallel semantic redo records here so a later redo can
        // never resurrect a stale record from a discarded branch.
        self.undo_redo_records.clear();
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
            P::Fluid => b.rendering.fluid_enabled = v >= 0.5,
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
        self.preview_dirty = true;
        // GPU preview: the active layer's pixels were restored → bump version.
        let active = self.layers.active();
        self.bump_layer_pixels(active);
        true
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
