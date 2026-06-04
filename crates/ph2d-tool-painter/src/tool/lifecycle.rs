//! See `tool/mod.rs` — this is stroke lifecycle + journal + UI edit + undo/redo + base accessors,
//! split out of the former `tool.rs` god-object (pure mechanical move).

use super::*;

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
        self.stroke_color_oklab[3] *= self.params.opacity.clamp(0.0, 1.0);

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
            // R4-LG-1: `Arc::make_mut` ⇒ unique-borrow path when refcount==1
            // (zero alloc), clone-once when bridge cached the prior Arc
            // (16 MB once per preview drain cycle, NOT per pointer event).
            // Explicit `Vec<u8>` annotation prevents the compiler from
            // coercing through `Arc<[u8]>::make_mut` (different impl).
            let canvas_vec: &mut Vec<u8> = Arc::make_mut(&mut self.canvas_rgba);
            apply_stamps_with_options(
                canvas_vec,
                self.source_size.0,
                self.source_size.1,
                stamps,
                alpha_lock,
            );
            self.preview_dirty = true;
            self.has_painted_since_source = true;
            // GPU preview: the active layer's pixels just changed → bump its
            // content version so the compositor re-uploads only its slice.
            let active = self.layers.active();
            self.bump_layer_pixels(active);
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
            // OpenLayersPopover / OpenColorPopover / OpenBrushStudio são
            // affordances visuais geridas shell-side (não mutam tool state).
            _ => {}
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
