//! See `tool/mod.rs` — this is dock toggle + accessors + preview-composite drive,
//! split out of the former `tool.rs` god-object (pure mechanical move).

use super::*;

impl PainterTool {
    // ── W3.T3.4 dock toggle (mode C) ────────────────────────────────────

    /// Which painter panel is shown in the shared right-dock slot —
    /// `false` = brush sidebar, `true` = layers panel. The shell
    /// `painter_bridge` reads this to compute `panel_visibility`.
    #[must_use]
    pub fn dock_shows_layers(&self) -> bool {
        self.dock_shows_layers
    }

    /// Flip the dock between the brush sidebar and the layers panel. Raised by
    /// either panel's header toggle button (`handle_panel_event`).
    pub fn toggle_dock(&mut self) {
        self.dock_shows_layers = !self.dock_shows_layers;
    }

    /// Decode a per-row layers-panel widget [`NodeId`] back to its
    /// `(layer, kind)` by recomputing [`painter_layer_widget_id`] for every
    /// current layer × kind and matching. `None` if `id` isn't a per-row
    /// widget of any layer. (≤8 layers × 5 kinds = ≤40 cheap FNV hashes; the
    /// layers panel is not a hot path.)
    pub(crate) fn decode_layer_widget(
        &self,
        id: ph2d_a11y::NodeId,
    ) -> Option<(RtLayerId, ph2d_editor_core::ids::PainterLayerWidget)> {
        use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
        for layer in self.layers.all_ids() {
            for kind in PainterLayerWidget::ALL {
                if painter_layer_widget_id(layer.0, kind) == id {
                    return Some((layer, kind));
                }
            }
        }
        None
    }

    /// Configura `next_seq` baseline pra anchor monotonic per-canvas.
    /// Caller invoca após `load(canon_bytes)` passando
    /// `paint_project.last_persisted_seq().map(|s| s + 1).unwrap_or(0)`
    /// — anti-replay defense via `CrashRecovery::scan_with_baseline`.
    ///
    /// **S-7 (audit T1.9):** mid-stroke override é silent footgun —
    /// current_partial já consumiu o next_seq antigo; override perdoa
    /// progress de monotonicity ADR-0046 §2.2.
    pub fn set_next_seq(&mut self, seq: u64) {
        debug_assert!(
            !self.stroke_active,
            "set_next_seq called mid-stroke — current_partial consumiu o \
             seq antigo; override viola ADR-0046 §2.2 monotonicity \
             (audit T1.9 S-7)"
        );
        self.next_seq = seq;
    }

    /// Read-only borrow do history (pra Reproject W12 / Inspector W14 /
    /// MCP W13 queries).
    #[must_use]
    pub fn stroke_history(&self) -> &StrokeHistory {
        &self.stroke_history
    }

    /// Take ownership do history (move-out — útil quando caller persiste
    /// canon + drop tool). Substitui por `StrokeHistory::default()`.
    pub fn take_stroke_history(&mut self) -> StrokeHistory {
        std::mem::take(&mut self.stroke_history)
    }

    /// **Audit Q-1 getter** — testes/inspectors querem checar buffer
    /// in-progress sem expor o `Vec` mutável.
    #[must_use]
    pub fn current_samples_len(&self) -> usize {
        self.current_samples.len()
    }

    /// **Audit Q-1 getter** — equivalente a `current_samples_len() == 0`,
    /// nome explícito pra assertions em tests.
    #[must_use]
    pub fn current_samples_is_empty(&self) -> bool {
        self.current_samples.is_empty()
    }

    /// **Audit Q-3/R-3 surface** — último erro WAL desde o último
    /// [`Self::attach_journal`]. `Some` = durability degraded (bridge W11
    /// deve emitir toast "Stroke history WAL unavailable — recent strokes
    /// in-memory only"). `None` após `attach_journal` bem-sucedido.
    #[must_use]
    pub fn last_wal_error(&self) -> Option<&JournalError> {
        self.last_wal_error.as_ref()
    }

    /// **Audit Q-3/R-3** — limpa o flag de degraded durability após o
    /// caller (bridge W11) ter mostrado o toast OU recuperado.
    pub fn clear_last_wal_error(&mut self) {
        self.last_wal_error = None;
    }

    /// Swap the active brush atomically with the public-contract
    /// `params.active_brush` handle. Guards against mid-stroke brush
    /// changes by calling `end_stroke` first (the scheduler's
    /// `residual_dist` / `stroke_rotation_base` / `last_follow_angle` /
    /// `stamp_index` carry brush-specific state — replaying them under
    /// a new brush's `spacing` / `shape_count` / `shape_rotation_follow`
    /// is undefined and produces visible seams or double-stamps at the
    /// switch boundary).
    ///
    /// **Audit T1.6 R7 L1-4:** `PainterParams.active_brush` (the
    /// `BrushHandle` published in ADR-0043 §2.3 §2.8 stub) was never
    /// read by `PainterTool` before this helper — `self.brush` was the
    /// sole runtime source of truth. A W2 sidebar implementer reading
    /// only the params surface (the published contract) would write a
    /// `SelectBrush(handle)` handler that updates the snapshot but
    /// leaves the rendered brush stale — silent regression with no
    /// gate to catch it. This helper makes the two writes inseparable
    /// AND adds the L1-6 end_stroke guard. `apply_ui_edit`'s future
    /// `PainterUiEdit::SelectBrush` handler MUST go through here.
    ///
    /// Calling this with `is_stroke_active() == true` emits a
    /// `debug_assert!` so the caller knows a stroke was silently
    /// dropped; release builds end the stroke without warning. The
    /// scheduler's `pool` (uncommitted stamps) is discarded along with
    /// the rest of `end_stroke` state — the caller is responsible for
    /// flushing visible stamps via `request_commit` BEFORE switching
    /// if mid-stroke flush is desired.
    ///
    /// **Audit T1.6 R8 M1-3 — `stroke_color_oklab` is intentionally
    /// NOT refreshed here.** That cache lives across `set_brush`
    /// because the color is part of `params.active_color`, not the
    /// brush, and brush-switch shouldn't mutate it. The unconditional
    /// `end_stroke()` above sets `stroke_active = false`, so the L1-5
    /// `debug_assert!` in `queue_pointer` will fire before the stale
    /// cache could be read by a stamp; the next `begin_stroke` then
    /// refreshes the cache from the current `params.active_color`.
    /// A future refactor that removes the `end_stroke()` call here
    /// would re-open the staleness window — keep it.
    pub fn set_brush(&mut self, handle: BrushHandle, brush: Brush) {
        debug_assert!(
            !self.stroke_active,
            "set_brush called mid-stroke — the scheduler residual state \
             carries brush-specific assumptions (spacing, rotation_base, \
             follow_angle); switching under it produces visible seams. \
             Audit R7 L1-6."
        );
        if self.stroke_active {
            self.end_stroke();
        }
        self.params.active_brush = handle;
        self.brush = brush;
        // R-5: brush mudou → hash cache stale.
        self.cached_brush_hash = None;
    }

    /// Public accessor for the active `BrushHandle`. Equivalent to
    /// `params.active_brush`, exposed as a function so future refactors
    /// (e.g., derive `self.brush` lazily from a library lookup keyed by
    /// handle — collapsing the two sources of truth) keep the public
    /// API stable. Audit T1.6 R7 L1-4.
    #[must_use]
    pub fn active_brush_handle(&self) -> BrushHandle {
        self.params.active_brush
    }

    /// Borrow the active runtime [`Brush`]. Audit T1.6 R8 P1-3.
    ///
    /// W2 sidebar widgets that need to populate shape / color-
    /// dynamics controls from the active brush state should call
    /// this — without it, every widget had to either store a
    /// duplicate copy of the brush or call back into a hardcoded
    /// `match handle → constructor()` block (the latter defeats the
    /// L1-4 / set_brush sync contract by re-introducing the manual
    /// handle-to-construction mapping).
    ///
    /// Companion to [`Self::active_brush_handle`]: the handle is the
    /// serializable identity, the brush is the runtime parameter
    /// vector. The two must stay in sync via [`Self::set_brush`].
    #[must_use]
    pub fn active_brush(&self) -> &Brush {
        &self.brush
    }

    /// "Brush lifted" — cursor saiu do footprint do sprite mid-drag.
    /// O próximo `queue_pointer` tratará o sample como ponto novo (sem
    /// interpolar uma linha reta no gap). Mantém o stroke ativo + o
    /// `stamp_index` counter. Audit T1.5 round 3 R3-LE-1.
    pub fn break_stroke_segment(&mut self) {
        if self.stroke_active {
            self.scheduler.break_segment();
        }
    }

    /// True iff um stroke está ativo (entre `begin_stroke` e `end_stroke`).
    #[must_use]
    pub fn is_stroke_active(&self) -> bool {
        self.stroke_active
    }

    /// Requisita commit (apply): bridge dispara `EditorAction::OneShotImageOp`
    /// no próximo frame, que aciona `run_full` para baking final.
    pub fn request_commit(&mut self) {
        self.pending_commit = true;
    }

    /// Tamanho efetivo do stamp em pixels, clampado ao limite ABI.
    pub(crate) fn effective_size_px(&self) -> f32 {
        self.params.size_px.clamp(1.0, MAX_STAMP_SIZE_PX as f32)
    }

    /// Dimensões do working canvas em pixels (`set_source` define;
    /// `deactivate` zera). Usado pelo input dispatch para mapear cursor
    /// screen-px → canvas-pixel coords.
    #[must_use]
    pub fn canvas_size(&self) -> (u32, u32) {
        self.source_size
    }

    /// **R4-LG-1 fast path:** zero-copy preview drain via Arc clone (1
    /// atomic increment). Drains `preview_dirty` and returns the SAME
    /// underlying `Arc<Vec<u8>>` the tool holds (no `to_vec`). The
    /// bridge stashes this Arc in its `painter_preview` cache; the
    /// canvas Vec stays shared between tool and bridge until the next
    /// `queue_pointer` triggers `Arc::make_mut`, at which point ONE
    /// `Vec::clone` happens.
    ///
    /// Prefer this over `RasterEditTool::current_preview` for the
    /// per-frame cache drain path (`drive_preview_cache` does
    /// `pixels.to_vec()` which at 60 fps × 16 MB is ~960 MB/s of
    /// allocator churn — audit R4-LG-1).
    #[must_use]
    pub fn take_preview_arc(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        if !std::mem::take(&mut self.preview_dirty) || self.canvas_rgba.is_empty() {
            return None;
        }
        let (w, h) = self.source_size;
        // **W3 multi-layer fix:** the trivial stack (single visible opaque
        // Normal raster — the T1.5 single-layer case) stays the zero-copy fast
        // path. Any non-trivial stack (≥2 layers, or a layer with opacity<1 /
        // non-Normal blend / hidden) MUST be composited here, otherwise the
        // on-canvas preview shows only the active layer's working buffer and
        // overlapping layers / per-layer opacity / blend modes are invisible.
        // Mirror of [`RasterEditTool::current_preview`]'s composite branch.
        if self.is_trivial_stack() {
            // Trivial single-layer path stays a full upload (the T1.5 fast path,
            // smoke-validated). Partial upload is scoped to the multi-layer
            // composite below — the case the dirty-rect win targets.
            self.preview_upload_bbox = None;
            return Some((Arc::clone(&self.canvas_rgba), w, h));
        }
        let active = self.layers.active().unwrap_or(RtLayerId(0));
        // **W3 perf — dirty-rect fast lane.** When a valid full composite is
        // cached AND only the active layer changed (a stroke) within a known
        // bbox, recomposite ONLY that region and blit it into the cache —
        // O(N×bbox) vs O(N×W×H). Otherwise (no cache after a structural edit,
        // or first drain) do a full recompose. The bridge still uploads the
        // full texture; the win is the composite itself (the dominant cost).
        let dirty = self.dirty_rect.take();
        let stroke_dirtied = dirty.is_some();
        match (self.composited.is_some(), dirty) {
            (true, Some(bbox)) => {
                let region = {
                    let src = ToolPixelSource {
                        active_id: active,
                        active_rgba: &self.canvas_rgba,
                        images: &self.images,
                    };
                    composite_region(&self.layers, &src, w, h, bbox)
                };
                // W5: a stroke changed the active layer's pixels — any adjustment
                // cut above it is now stale. Drop all cuts (next slider-drag cold-
                // fills) + the pending flag (the stroke supersedes it).
                self.compositor_cache.invalidate_from(active, &self.layers);
                self.adjustment_cache_pending = false;
                // `make_mut`: unique-borrow (zero-copy) once the bridge dropped
                // its prior Arc; clone-once if it's still holding the cache.
                let cache = Arc::make_mut(self.composited.as_mut().expect("checked is_some"));
                blit_region(cache, w, &region, bbox);
                // B.1: only `bbox` changed in the cache since the last drain (and
                // the last drain uploaded), so the bridge may upload just this
                // sub-rect on top of the already-synced GPU texture.
                self.preview_upload_bbox = Some(bbox);
            }
            _ => {
                let src = ToolPixelSource {
                    active_id: active,
                    active_rgba: &self.canvas_rgba,
                    images: &self.images,
                };
                let composed =
                    if std::mem::take(&mut self.adjustment_cache_pending) && !stroke_dirtied {
                        // W5 slider-drag: restart from the cut-point cache — the layers
                        // below the changed adjustment are unchanged, so only this
                        // adjustment + the layers above recompose. Bit-identical to a
                        // full `composite` (gate `cache_matches_full_recompose`).
                        composite_with_cache(&self.layers, &src, w, h, &mut self.compositor_cache)
                    } else {
                        // Cold full recompose (first drain / a stroke with no cache to
                        // blit into / structural edit): the cuts are stale — drop them.
                        self.compositor_cache.invalidate_from(active, &self.layers);
                        composite(&self.layers, &src, w, h)
                    };
                self.composited = Some(Arc::new(composed));
                // Full recompose (first drain / post-structural-or-metadata edit)
                // → the bridge must upload the whole texture to re-sync.
                self.preview_upload_bbox = None;
            }
        }
        Some((
            Arc::clone(self.composited.as_ref().expect("just set")),
            w,
            h,
        ))
    }

    /// Drains the dirty bbox `(x, y, w, h)` of the LAST [`Self::take_preview_arc`]
    /// — `Some` iff that drain was a partial fast-lane update the bridge may
    /// upload as a sub-rect (B.1); `None` = upload the full texture. Call right
    /// after `take_preview_arc` on the same frame. `take` so a later not-dirty
    /// drain can't replay a stale partial.
    pub fn take_preview_upload_bbox(&mut self) -> Option<(u32, u32, u32, u32)> {
        self.preview_upload_bbox
            .take()
            .map(|r| (r.x, r.y, r.w, r.h))
    }

    /// B.5: monotonic revision of the published layer structure. The bridge
    /// publishes the `LayerStack` snapshot only when this changes, instead of
    /// deep-cloning it every frame. Bumped by `invalidate_composite` (all
    /// structural/metadata edits) + `set_source`; strokes don't bump it.
    #[must_use]
    pub fn layers_revision(&self) -> u64 {
        self.layers_revision
    }

    /// B.5: the active primary color as sRGB8 `[r, g, b, a]`, direct from
    /// `params.active_color` — so the bridge reads the color without building a
    /// full `ui_snapshot` (a `String`-allocating call) every frame. Identical to
    /// [`PainterUiSnapshot::active_color_srgb8`].
    #[must_use]
    pub fn active_color_srgb8(&self) -> [u8; 4] {
        crate::color::painter_oklch_to_srgb8(self.params.active_color)
    }

    /// True iff at least one stamp landed in `canvas_rgba` since the
    /// last `set_source`. Used by `drain_painter` (Apply) to skip the
    /// bake when the user clicked Apply without painting anything,
    /// avoiding a wasted Individual texture + no-op undo entry. Audit
    /// T1.5 round 3 R3-LF-5.
    #[must_use]
    pub fn has_painted_since_source(&self) -> bool {
        self.has_painted_since_source
    }

    /// Derive a deterministic, HR-5 cross-OS-stable `stroke_seed` from
    /// the canonical inputs of a pointer-down event.
    ///
    /// `canvas_px / canvas_py` are the position in CANVAS-pixel coords
    /// (not screen-px) so the seed is invariant under camera zoom/pan.
    /// `src_w / src_h` distinguish strokes on differently-sized sprites
    /// at the same logical canvas pixel. `entity_bits` distinguishes
    /// strokes on different sprites at the exact same dimensions.
    ///
    /// Mixer is a wyhash-style fold (3× multiply + xorshift) — same
    /// family as the scheduler's `det_random`; bit-identical across
    /// Mac/Linux/Windows. No dependency on `rand`'s `SmallRng` (whose
    /// seeding varies cross-platform).
    ///
    /// Audit T1.5 round 1 A-H4 + B-M3: canonical helper replaces the
    /// ad-hoc XOR formula previously inlined in `painter_input.rs`.
    #[must_use]
    pub fn derive_seed(
        canvas_px: f32,
        canvas_py: f32,
        src_w: u32,
        src_h: u32,
        entity_bits: u64,
    ) -> u64 {
        // Quantize canvas-px to u32 bits for stable hashing (NaN/Inf
        // canvas positions short-circuit upstream in `painter_pointer_uv`
        // but we defensively replace non-finite with zero here).
        let qx = if canvas_px.is_finite() {
            canvas_px.to_bits()
        } else {
            0
        };
        let qy = if canvas_py.is_finite() {
            canvas_py.to_bits()
        } else {
            0
        };
        let mut h = (qx as u64) | ((qy as u64) << 32);
        h ^= entity_bits;
        h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^= h >> 32;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= (src_w as u64) | ((src_h as u64) << 32);
        h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
        h
    }
}
