//! `impl Tool` + `impl RasterEditTool` for `PainterTool`, split out of
//! the former `tool.rs` god-object (pure mechanical move).

use super::*;

impl Tool for PainterTool {
    fn id(&self) -> ToolId {
        ToolId::new("painter")
    }

    fn label(&self) -> &str {
        "Painter"
    }

    fn icon_slug(&self) -> &str {
        "painter"
    }

    fn build_panel(&self) -> FloatingPanel {
        // T1.1 stub: panel vazio com title só. Sidebar Procreate-style
        // ship em W2 (ph2d-panel-painter trait-driven; ADR-0029 padrão).
        FloatingPanel::new(self.id(), "Painter")
    }

    fn on_activate(&mut self) {
        // T1.2 implementará: pushar `painter_active = true` no HeroScreen
        // para acionar takeover (suprime chrome PH2D normal; ADR-0043 §1.1).
        self.params.takeover_active = true;
        // **R6-LN-1 doc honesty:** `println!` é um marker T1.2 smoke
        // temporário — remove quando o `tracing` crate joins the
        // workspace (ADR pendente). NÃO é "convenção canônica do
        // projeto" como a versão anterior do comentário afirmava: grep
        // confirmou que nenhum outro tool de produção usa `println!`
        // em runtime path. Painter é o único; é uma stub T1.5, não um
        // estabelecido pattern.
        println!("painter activated");
    }

    fn on_deactivate(&mut self) {
        self.params.takeover_active = false;
        // Audit T1.5 round 1 B-M2: full RasterEditTool teardown when the
        // registry switches tools — clears canvas_rgba + source_size +
        // dirty/commit flags too, not just stroke state. Without this,
        // re-activation runs against stale canvas from prior session and
        // the bridge's `last_painter_pushed_entity` reset (set None in
        // `painter_bridge::dispatch` inactive path) wouldn't be enough.
        <Self as RasterEditTool>::deactivate(self);
        // R6-LN-1: T1.2 smoke marker, temporário (vide on_activate).
        println!("painter deactivated");
    }

    fn handle_panel_event(&mut self, event: ph2d_editor_core::tool::PanelEvent) {
        // **W2.T2.1:** ADR-0040 TG-B canal genérico — sidebar emite
        // PanelEvent::SetValue(NodeId, f64). Routing pra PainterUiEdit
        // semantic + apply_ui_edit single source of truth (ADR-0043 §2.3).
        //
        // **Audit Y-7/Y-9/Z-2 (2026-05-28):** apenas os SLIDERS são
        // roteados. O slider armazena o `0..1` canônico; o chip espelha
        // via `link_slider_number_mapped` (display px/%) e o commit do
        // chip propaga de volta pro slider, cuja `ValueChanged` chega aqui
        // já normalizada — rotear o chip diretamente injetaria valor
        // display-space (px/%) no `Size`/`Opacity` que esperam `0..1`
        // (trap de unidade). Undo/Redo (T2.2 replay engine) e modifier
        // square (T2.4 eyedropper-while-held) não têm paint nesta wave;
        // o roteamento delas volta junto do paint correspondente.
        use crate::params::PainterUiEdit;
        use ph2d_editor_core::ids::{self as core_ids, PainterLayerWidget};
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            // ── Brush sidebar sliders (W2.T2.1) ───────────────────────────
            PanelEvent::SetValue(id, v) if id == core_ids::PAINTER_SIDEBAR_SIZE_SLIDER => {
                self.apply_ui_edit(PainterUiEdit::Size(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == core_ids::PAINTER_SIDEBAR_OPACITY_SLIDER => {
                self.apply_ui_edit(PainterUiEdit::Opacity(v as f32));
            }
            // ── Dock toggle (mode C) — either panel's header button ────────
            PanelEvent::Click(id)
                if id == core_ids::PAINTER_LAYERS_TOGGLE_DOCK
                    || id == core_ids::PAINTER_SIDEBAR_TOGGLE_DOCK =>
            {
                self.toggle_dock();
            }
            // ── Layers panel: "+ Layer" (create + activate a raster on top) ─
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_ADD => {
                let name = format!("Layer {}", self.layers.len() + 1);
                self.add_raster_layer(name);
            }
            // ── Header actions: duplicate / delete / group the active layer ─
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_DUPLICATE => {
                if let Some(active) = self.layers.active() {
                    self.duplicate_layer(active);
                }
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_DELETE => {
                if let Some(active) = self.layers.active() {
                    self.delete_layer(active);
                }
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_GROUP => {
                self.group_selected();
            }
            // ── Modifier toolbar (acts on the ACTIVE layer) ────────────────
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_MASK => {
                self.add_mask_to_active();
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_CLIP => {
                if let Some(a) = self.layers.active() {
                    let now = self.layers.get(a).is_some_and(|l| l.clipping);
                    self.set_layer_clipping(a, !now);
                }
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_ALPHA_LOCK => {
                if let Some(a) = self.layers.active() {
                    let now = self.layers.get(a).is_some_and(|l| l.alpha_locked);
                    self.set_layer_alpha_locked(a, !now);
                }
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_REFERENCE => {
                if let Some(a) = self.layers.active() {
                    let now = self.layers.get(a).is_some_and(|l| l.is_reference);
                    self.set_layer_reference(a, !now);
                }
            }
            // ── Apply CTA (either panel) — commit the composite to the sprite.
            // The bridge's drive_pending_commit bakes it (run_full) next frame.
            PanelEvent::Click(id) if id == core_ids::PAINTER_APPLY => {
                self.request_commit();
            }
            // ── Layers panel: per-row click (row select / visibility eye) ──
            PanelEvent::Click(id) => {
                if let Some((layer, kind)) = self.decode_layer_widget(id) {
                    match kind {
                        // Multi-select: the panel stashed the Cmd/Shift state for
                        // this row click (frozen PanelEvent + store-less
                        // handle_panel_event can not carry it). Shift = range,
                        // Cmd/Ctrl = toggle additive, plain = single.
                        PainterLayerWidget::Row => {
                            let (cmd, shift) = take_pending_select_mods(id);
                            if shift {
                                self.select_range(layer);
                            } else if cmd {
                                self.select_additive(layer);
                            } else {
                                self.select_single(layer);
                            }
                        }
                        PainterLayerWidget::Visibility => {
                            let now = self.layers.get(layer).map(|l| l.visible).unwrap_or(true);
                            self.set_layer_visible(layer, !now);
                        }
                        PainterLayerWidget::MoveUp => self.move_layer_up(layer),
                        PainterLayerWidget::MoveDown => self.move_layer_down(layer),
                        // Mask row affordances (§2.7).
                        PainterLayerWidget::MaskInvert => self.toggle_mask_inverted(layer),
                        PainterLayerWidget::MaskApply => {
                            self.apply_mask(layer);
                        }
                        // Opacity slider/chip emit SetValue; Blend emits
                        // SelectOption — neither arrives as a Click.
                        _ => {}
                    }
                }
            }
            // ── Layers panel: per-row sliders (opacity + adjustment H/S/B),
            // all stored 0..1; the tool maps each to its target. ───────────
            PanelEvent::SetValue(id, v) => {
                if let Some((layer, kind)) = self.decode_layer_widget(id) {
                    match kind {
                        PainterLayerWidget::Opacity => self.set_layer_opacity(layer, v as f32),
                        PainterLayerWidget::AdjParam0 => {
                            self.set_adjustment_param(layer, 0, v as f32)
                        }
                        PainterLayerWidget::AdjParam1 => {
                            self.set_adjustment_param(layer, 1, v as f32)
                        }
                        PainterLayerWidget::AdjParam2 => {
                            self.set_adjustment_param(layer, 2, v as f32)
                        }
                        PainterLayerWidget::AdjParam3 => {
                            self.set_adjustment_param(layer, 3, v as f32)
                        }
                        PainterLayerWidget::AdjParam4 => {
                            self.set_adjustment_param(layer, 4, v as f32)
                        }
                        PainterLayerWidget::AdjParam5 => {
                            self.set_adjustment_param(layer, 5, v as f32)
                        }
                        _ => {}
                    }
                }
            }
            // ── "+ Adjustment" kind pick (W4 T4.15): value = index into
            // `AdjustmentKind::ALL`; create that kind's layer ───────────────
            PanelEvent::SelectOption(id, value)
                if id == core_ids::PAINTER_LAYERS_ADD_ADJUSTMENT =>
            {
                if let Ok(idx) = value.parse::<usize>()
                    && let Some(&kind) =
                        ph2d_painter_brush::adjustments::AdjustmentKind::ALL.get(idx)
                {
                    self.add_adjustment_layer(kind);
                }
            }
            // ── Curves editor 2-D point drag (W4 §3): value =
            // "layer:channel:index:x:y" (the panel drained it from the store's
            // curve_point_drag slot and re-derived the layer). ──────────────
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_CURVE_EDIT => {
                let mut it = value.split(':');
                if let (Some(l), Some(c), Some(i), Some(xs), Some(ys)) =
                    (it.next(), it.next(), it.next(), it.next(), it.next())
                    && let (Ok(layer), Ok(ch), Ok(idx), Ok(x), Ok(y)) = (
                        l.parse::<u64>(),
                        c.parse::<u8>(),
                        i.parse::<usize>(),
                        xs.parse::<f32>(),
                        ys.parse::<f32>(),
                    )
                {
                    self.set_curve_point(RtLayerId(layer), ch, idx, x, y);
                }
            }
            // ── Curves editor add a point (W4 §3): value = "layer:channel" ──
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_CURVE_ADD => {
                let mut it = value.split(':');
                if let (Some(l), Some(c)) = (it.next(), it.next())
                    && let (Ok(layer), Ok(ch)) = (l.parse::<u64>(), c.parse::<u8>())
                {
                    self.add_curve_point(RtLayerId(layer), ch);
                }
            }
            // ── Curves editor remove a point (W4 §3): value = "layer:channel:index" ─
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_CURVE_REMOVE => {
                let mut it = value.split(':');
                if let (Some(l), Some(c), Some(i)) = (it.next(), it.next(), it.next())
                    && let (Ok(layer), Ok(ch), Ok(idx)) =
                        (l.parse::<u64>(), c.parse::<u8>(), i.parse::<usize>())
                {
                    self.remove_curve_point(RtLayerId(layer), ch, idx);
                }
            }
            // ── Layers panel: per-row blend-mode pick (value = wire u8) ────
            PanelEvent::SelectOption(id, value) => {
                if let Some((layer, PainterLayerWidget::Blend)) = self.decode_layer_widget(id)
                    && let Ok(mode) = value.parse::<u8>()
                {
                    self.set_layer_blend_mode(layer, BlendMode::from_u8(mode));
                }
            }
            // **W2.T2.4:** the eyedropper icon no longer routes through here
            // — the sidebar arms the picker's `eyedropper_pending` directly
            // (panel `event.rs`), so the sampled pixel flows back via the
            // shared picker → painter_bridge path (`SetColorSrgb`).
            PanelEvent::Toggle(_, _) => {}
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
        Some(self)
    }

    fn is_default(&self) -> bool {
        // Brush proto-tool (editor-core) continua default em W1. Quando
        // Painter substituir o proto (T1.X close W1), is_default() flipa
        // para true e brush proto é deletado.
        false
    }
}

impl RasterEditTool for PainterTool {
    /// Inicializa o working canvas a partir do source do sprite. RGBA8
    /// straight; é o estado base sobre o qual os stamps depositam.
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        // **R4-LH-3 fix:** production-grade length guard. A debug_assert
        // here lets a release-build caller pass a mismatched buffer that
        // then triggers undefined behavior in `apply_stamps` (or worse,
        // silently produces a corrupted commit through the Apply-without-
        // paint chain).
        assert_eq!(
            rgba.len(),
            (width as usize) * (height as usize) * 4,
            "set_source rgba length must equal width*height*4"
        );
        self.canvas_rgba = Arc::new(rgba);
        self.source_size = (width, height);
        self.preview_dirty = true;
        // **W3 (ADR-0046-amд-1 Option A):** a fresh source = a fresh single-
        // raster stack. The lone layer's pixels ARE `canvas_rgba` (the active
        // working buffer), so `images` stays empty and the stack is trivial →
        // `current_preview` keeps the exact T1.5 fast path. Multi-layer state
        // only appears once the layers panel adds layers.
        self.layers = LayerStack::new();
        self.layers.add_raster("Layer 1", width, height);
        self.images.clear();
        // GPU preview: a fresh `LayerStack` restarts layer ids at 1, so a key
        // can be REUSED for a different image. Clear the version map and bump
        // the new active layer; `pixel_clock` stays monotonic, so the bumped
        // version is strictly greater than anything the compositor cached for
        // the previous source under the same key → no stale slice.
        self.layer_pixel_versions.clear();
        let active = self.layers.active();
        self.bump_layer_pixels(active);
        self.composited = None;
        // B.5: a fresh single-raster stack is a published-structure change, and
        // `set_source` resets `composited` directly (not via invalidate_composite).
        self.layers_revision = self.layers_revision.wrapping_add(1);
        // R3-LF-5: reset "painted since source" — fresh source = clean
        // slate for the Apply-emptiness check.
        self.has_painted_since_source = false;
        // Source switched mid-stroke → encerra stroke pra não pintar no
        // canvas errado. Bridge garante ordem (source push antes de
        // queue_pointer) mas defesa-em-profundidade.
        self.end_stroke();
        // **W2.T2.2:** a fresh source is a different working canvas (the bridge
        // only re-pushes when the selected sprite entity changes —
        // `drive_source_push`). Undo/redo over the OLD canvas is meaningless on
        // the NEW one, so reset the snapshot stacks + the parallel semantic
        // redo branch. `stroke_history` itself is the persisted canon and is
        // reset by the caller's project/canvas lifecycle, not here.
        self.undo.clear();
        self.undo_redo_records.clear();
        self.pending_pre_stroke = None;
    }

    /// Devolve referência ao composite atual iff houve update desde a última
    /// call. Comportamento ADR-0041 — drena `preview_dirty`.
    ///
    /// **W3 (ADR-0046-amд-1 Option A):** for the trivial stack (single
    /// visible, opaque, Normal raster — the common case, and the only state
    /// reachable until the layers panel adds layers) this returns
    /// `canvas_rgba` byte-for-byte (exact T1.5 fast path, zero composite, zero
    /// alloc). For a non-trivial stack (a hidden/faded/blended/multi layer) it
    /// composites the runtime `LayerStack` (active layer = `canvas_rgba`,
    /// others = `images`) into a cached buffer — the CPU reference; the GPU
    /// compositor (Block 2) is the real-time path the shell uses for big stacks.
    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
        if !std::mem::take(&mut self.preview_dirty) || self.canvas_rgba.is_empty() {
            return None;
        }
        let (w, h) = self.source_size;
        if self.is_trivial_stack() {
            return Some((&self.canvas_rgba, w, h));
        }
        // Non-trivial: composite the runtime stack (CPU reference). Compute
        // with immutable borrows first, then cache, then hand back the cache.
        let composited = {
            let active = self.layers.active().unwrap_or(RtLayerId(0));
            let src = ToolPixelSource {
                active_id: active,
                active_rgba: &self.canvas_rgba,
                images: &self.images,
            };
            composite(&self.layers, &src, w, h)
        };
        self.composited = Some(Arc::new(composited));
        Some((
            self.composited
                .as_ref()
                .map(|a| a.as_slice())
                .unwrap_or(&[]),
            w,
            h,
        ))
    }

    fn take_pending_commit(&mut self) -> bool {
        std::mem::take(&mut self.pending_commit)
    }

    /// Bake final do canvas para commit. T1.5 = retorna o working canvas
    /// (`canvas_rgba`) inteiro, já com todos os stamps já depositados in-
    /// place via CPU path. Não toca state — bridge é responsável por
    /// chamar `deactivate` se for o ciclo de fim-de-tool.
    fn run_full(&mut self) -> (Vec<u8>, u32, u32) {
        let (w, h) = self.source_size;
        // **W3 multi-layer Apply:** bake the full layer COMPOSITE (exactly what
        // the live preview shows — base layer + every layer above with their
        // opacity / blend / visibility), not just the active layer's working
        // buffer. Without this, Apply with ≥2 layers silently discards the
        // non-active layers (data loss + the baked sprite would not match the
        // preview). Mirror of `take_preview_arc` / `current_preview`.
        if !self.is_trivial_stack() {
            let active = self.layers.active().unwrap_or(RtLayerId(0));
            let src = ToolPixelSource {
                active_id: active,
                active_rgba: &self.canvas_rgba,
                images: &self.images,
            };
            return (composite(&self.layers, &src, w, h), w, h);
        }
        // Trivial single-layer fast path. **R5-LI-C:** `mem::replace` takes the
        // original Arc out (leaving an empty stub) BEFORE `unwrap_or_clone`, so
        // the take-out arc has refcount=1 → `unwrap_or_clone` returns the inner
        // Vec without allocation (the bridge already dropped its preview Arc in
        // the Apply teardown path). The stub is harmless: `deactivate` replaces
        // it immediately on the standard Apply teardown.
        let canvas = std::mem::replace(&mut self.canvas_rgba, Arc::new(Vec::new()));
        (Arc::unwrap_or_clone(canvas), w, h)
    }

    fn deactivate(&mut self) {
        self.canvas_rgba = Arc::new(Vec::new());
        self.source_size = (0, 0);
        self.preview_dirty = false;
        self.pending_commit = false;
        self.scheduler.end_stroke();
        self.stroke_active = false;
        self.has_painted_since_source = false;
        // W3 (audit L-2): reset the dock-mode flag so re-activating on another
        // sprite opens the default brush sidebar, not a stale Layers dock.
        // (`set_source` rebuilds layers/images/composited on the next push, but
        // it never touches this flag, so reset it here.)
        self.dock_shows_layers = false;
        // T1.9: drop journal (libera flock + in-process registry) +
        // descarta PartialStroke em-progresso. `stroke_history` é
        // PRESERVADO — caller decide se persiste no canon antes do
        // Tool::on_deactivate.
        //
        // **Audit R-2:** se há stroke ativo no WAL, cancela ANTES de drop
        // pra não deixar Begin órfão (recovery false-positivo
        // InProgressAtCrash). Same fix que `detach_journal`.
        if let Some(journal) = self.stroke_journal.as_mut()
            && journal.current_stroke.is_some()
        {
            let _ = journal.cancel_stroke();
        }
        self.stroke_journal = None;
        self.current_partial = None;
        self.current_samples.clear();
        self.last_wal_error = None;
    }
}
