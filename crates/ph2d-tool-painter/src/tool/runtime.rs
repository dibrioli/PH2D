//! See `tool/mod.rs` — dock toggle + accessors + preview-composite drive +
//! structural undo/redo, split out of the former `tool.rs` god-object.

use super::*;

impl PainterTool {
    // ── Dock toggle (mode C) ────────────────────────────────────────────

    /// Whether the layers panel occupies the shared right-dock slot. The shell
    /// `painter_bridge` reads this to compute `panel_visibility`.
    #[must_use]
    pub fn dock_shows_layers(&self) -> bool {
        self.dock_shows_layers
    }

    /// Flip the dock slot (the layers panel header toggle button).
    pub fn toggle_dock(&mut self) {
        self.dock_shows_layers = !self.dock_shows_layers;
    }

    /// Decode a per-row layers-panel widget [`NodeId`] back to its
    /// `(layer, kind)` by recomputing [`painter_layer_widget_id`] for every
    /// current layer × kind and matching. `None` if `id` isn't a per-row
    /// widget of any layer. (≤8 layers × kinds = cheap FNV hashes; the layers
    /// panel is not a hot path.)
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

    // ── Structural undo / redo (layer model) ────────────────────────────

    /// Undo the most recent structural layer edit, reinstalling the prior model.
    /// Returns `true` if an edit was undone. Driven by the shell's undo gesture /
    /// shortcut.
    pub fn undo_last(&mut self) -> bool {
        // A LIVE Deform Transform owns undo while it's up: step the gizmo back through its gestures (and
        // finally un-lift), all WITHOUT touching the structural timeline — the whole transform is one
        // structural entry, committed only when it ends (Enio 2026-07-04).
        if self.transform_undo_step() {
            return true;
        }
        // ONE unified timeline: shape authoring (create / point-edit / reshape / Offset) and pixel bakes
        // (Apply / Apply & Keep) are all `ModelSnapshot` entries, so undo walks them in reverse chronological
        // order regardless of kind. Each entry carries the open-shape editor state, so a restore reinstates
        // the overlay together with the pixels — they can never desync (Enio 2026-06-28). First close any
        // coalesced Offset drag still in flight so it undoes as its own step.
        self.flush_shape_txn();
        // **A cadeia é estabelecida aqui também, não só no commit** (doc 28 §5.24).
        //
        // O delta se sustenta em *`entry[topo].after` == o estado que este passo encontra*. O
        // `absorb_foreign_writes` estabelecia isso na entrada dos `record_*`; o UNDO é o **segundo**
        // consumidor do mesmo invariante e o assumia. Medido: 92 undos/redos na suíte, **duas** em que
        // o vivo não era o cursor (o re-stamp do shape · o escorrido do Wet Paint) — e absorver aqui as
        // leva a **zero**.
        //
        // ⚠️ **Custo zero no caso comum, por construção:** a pergunta usa o MESMO `PlaneDeltas::split`,
        // que começa por `Arc::ptr_eq` em cada plano ⇒ sem escrita estrangeira ele não lê um byte.
        self.absorb_foreign_writes_now();
        self.audit_live_is_the_cursor("undo");
        if let Some(model) = self.undo.undo() {
            self.restore_model(*model);
            true
        } else {
            false
        }
    }

    /// Redo the most recently undone edit on the unified timeline. Returns `true` if an edit was redone.
    pub fn redo_last(&mut self) -> bool {
        // A live / just-un-lifted Transform owns redo: re-lift the gizmo (recreate it) or step a gizmo pose
        // FORWARD — mirroring transform_undo_step, WITHOUT touching the structural timeline (Enio 2026-07-04).
        if self.transform_redo_step() {
            return true;
        }
        // While a Transform float is live with nothing left to redo, the structural timeline is frozen (the
        // transform is one pending entry) — swallow redo so it can't reinstate a stale structural state.
        if self.deform_transform_live() {
            return false;
        }
        self.flush_shape_txn();
        // **A cadeia é estabelecida aqui também, não só no commit** (doc 28 §5.24).
        //
        // O delta se sustenta em *`entry[topo].after` == o estado que este passo encontra*. O
        // `absorb_foreign_writes` estabelecia isso na entrada dos `record_*`; o UNDO é o **segundo**
        // consumidor do mesmo invariante e o assumia. Medido: 92 undos/redos na suíte, **duas** em que
        // o vivo não era o cursor (o re-stamp do shape · o escorrido do Wet Paint) — e absorver aqui as
        // leva a **zero**.
        //
        // ⚠️ **Custo zero no caso comum, por construção:** a pergunta usa o MESMO `PlaneDeltas::split`,
        // que começa por `Arc::ptr_eq` em cada plano ⇒ sem escrita estrangeira ele não lê um byte.
        self.absorb_foreign_writes_now();
        self.audit_live_is_the_cursor("redo");
        if let Some(model) = self.undo.redo() {
            self.restore_model(*model);
            true
        } else {
            false
        }
    }

    /// **Reconcilia a história com escritas estrangeiras, AGORA** — a porta única dos dois consumidores
    /// do invariante da cadeia (o commit tem a sua no `commit_structural_edit`).
    ///
    /// O snapshot é feito de clones de `Arc`, então ele custa refcounts; e o `split` que responde
    /// *"alguém escreveu?"* começa por `Arc::ptr_eq`, então no caso comum nada é lido.
    pub(crate) fn absorb_foreign_writes_now(&mut self) {
        let live = self.snapshot_model();
        self.undo.absorb_foreign_writes(&live);
    }

    /// **O RE-CENSO da premissa do S3** — *o estado vivo serve de base para o delta?* (doc 28 §7).
    ///
    /// Chamado no instante de todo undo/redo, com `PH2D_UNDO_AUDIT=1` a suíte inteira desta crate vira
    /// a rede: ela roda em debug em ~4 s e exercita fill, seleção, warp, sculpt, máscara, inpaint,
    /// clone, aquarela, Wet Paint e os shape editors, então o censo cobre **todo gesto que algum teste
    /// encena**. A medição de 2026-07-27 deu **81 chamadas · 79 concordam · 2 divergem**, e as duas
    /// exceções estão pinadas em `paint::undo_live_base_tests`.
    ///
    /// ⚠️ **`cargo test` CAPTURA o stderr de teste que passa** — sem `--nocapture` o censo sai vazio e
    /// *vazio parece concordância*. O controle positivo (imprimir também quando não diverge) é o que
    /// separou as duas coisas na primeira rodada.
    ///
    /// ```text
    /// PH2D_UNDO_AUDIT=1 cargo test -p ph2d-tool-painter -- --nocapture 2>&1 | grep S3-AUDIT
    /// ```
    #[cfg(any(test, debug_assertions))]
    fn audit_live_is_the_cursor(&self, tag: &str) {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var_os("PH2D_UNDO_AUDIT").is_some()) {
            return;
        }
        let Some(cursor) = self.undo.cursor_for_audit() else {
            return;
        };
        let d = crate::undo_planes::PlaneDeltas::divergences(&self.snapshot_model(), cursor);
        eprintln!("[S3-AUDIT] {tag}: divergencias={} {d:?}", d.len());
    }

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(clippy::unused_self, reason = "no-op em release; a rede é de debug")]
    fn audit_live_is_the_cursor(&self, _tag: &str) {}

    /// **O commit recebeu a janela, ou varreu?** — o readout que separa os dois custos possíveis dos
    /// 12,2 ms que sobraram do S2 (doc 28 §5.19): *extração* de uma janela pequena contra *varredura*
    /// dos quatro planos porque algum sítio deixou um acesso sem declarar.
    ///
    /// ⚠️ O contador é **um só para todos os planos** (uma `WriteState` no tool), então um único sítio
    /// esquecido tira a janela de TODOS eles. É por isso que a pergunta se faz aqui, no consumidor, e
    /// não por sítio: o que decide é o total.
    ///
    /// `PH2D_UNDO_AUDIT=1 cargo test -p ph2d-tool-painter -- --nocapture 2>&1 | grep S3-AUDIT`
    /// (⚠️ o `--nocapture` é parte da medição — o `cargo test` engole o stderr de teste que passa).
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn audit_commit_window(
        &self,
        snapshot_writes: u64,
        hint: Option<crate::compositor::Region>,
    ) {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var_os("PH2D_UNDO_AUDIT").is_some()) {
            return;
        }
        let w = self.undo.write_state.get();
        let verdict = match hint {
            Some(r) => format!("JANELA {}x{} em ({},{})", r.w, r.h, r.x, r.y),
            None => "VARRE".to_string(),
        };
        eprintln!(
            "[S3-AUDIT] commit: {verdict} · nao-declarados={} · snapshot_writes={snapshot_writes}",
            w.undeclared()
        );
    }

    /// **A REDE DO JOURNAL — todo escritor de canvas passa pela porta?** (doc 28 §7, degrau S3).
    ///
    /// O journal captura os bytes velhos do canvas na primeira vez que a porta
    /// ([`super::paint::plane_fork::fork_canvas`]) é atravessada, e é zerado em cada commit. Logo o
    /// que ele guarda é, por construção, **o canvas como ele estava no ÚLTIMO COMMIT** — e quem
    /// carrega esse estado hoje é o `cursor` do histórico. Esta função afirma essa igualdade.
    ///
    /// ⚠️ **O alvo é o `cursor`, não o `before` do passo, e a primeira versão errou isso.** Mirado no
    /// `before` a rede disparou em 16 testes na 1ª rodada, com o journal em 255 (tela virgem) contra
    /// um `before` já pintado: o journal descreve um passado *mais antigo*, porque escritas entre dois
    /// commits (um preview re-stampado, um tick de água) o armam antes de o passo abrir. Isso **não é
    /// defeito** — é a mesma reconciliação que o
    /// [`crate::undo::UndoController::absorb_foreign_writes`] faz hoje por diff, e que o journal
    /// passa a fazer **por construção**: ele capturou aquelas escritas também.
    ///
    /// ⚠️ **O que ela pega é o escritor que escreve sem passar pela porta**: aí a captura pega bytes
    /// já modificados e o journal passa a descrever um passado que nunca existiu. É a única forma de
    /// ele estar errado enquanto captura o plano inteiro, e é justamente o que precisa estar provado
    /// antes de ele virar a FONTE do undo em vez de uma rede.
    ///
    /// ⚠️ **A rede só se aplica a um passo que COMEÇA no cursor**, e a razão está no
    /// [`crate::undo::UndoController::absorb_foreign_writes`]: se o `before` que chega não é o
    /// cursor, alguém escreveu no meio — e o histórico responde a isso **re-partindo a entrada do
    /// topo e movendo o cursor**. Até essa reconciliação acontecer o journal está ancorado num
    /// commit que deixou de descrever a tela, então a pergunta que ele responderia é sobre outro
    /// passado. Perguntar mesmo assim seria afirmar mais do que o produto promete.
    ///
    /// O discriminante é o MESMO fato que a absorção usa (o `before` é o cursor?), perguntado por
    /// **identidade de `Arc`**: no caso comum nada escreveu desde o commit, então o `before` clona o
    /// ponteiro do cursor e a igualdade é exata e barata.
    ///
    /// ```text
    /// PH2D_UNDO_AUDIT=1 cargo test -p ph2d-tool-painter --lib 2>&1 | grep "journal do canvas"
    /// ```
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn audit_journal_matches_the_before(&self, before: &crate::undo::ModelSnapshot) {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var_os("PH2D_UNDO_AUDIT").is_some()) {
            return;
        }
        if !self.undo.write_state.get().covers(before.writes) {
            return; // um `before` anterior ao último reset: o journal não fala do passado dele
        }
        let Some(cursor) = self.undo.cursor_for_audit() else {
            return; // história vazia: não há "último commit" com que comparar
        };
        if !std::sync::Arc::ptr_eq(&before.canvas_rgba, &cursor.canvas_rgba) {
            return; // escrita estrangeira: a absorção move o cursor, e só depois dela há o que conferir
        }
        let mut differ = 0usize;
        let mut first = None;
        for (i, &want) in cursor.canvas_rgba.iter().enumerate() {
            if let Some(got) = self.undo.write_state.canvas_before(i)
                && got != want
            {
                differ += 1;
                first.get_or_insert((i, got, want));
            }
        }
        assert!(
            differ == 0,
            "o journal do canvas descreve um passado diferente do estado do ULTIMO COMMIT: {differ} \
             byte(s) divergem, o 1o em {first:?} (indice, journal, cursor). Isso significa que algo \
             escreveu no canvas SEM passar pela porta `fork_canvas` — o journal capturou bytes ja \
             modificados, e como FONTE do undo ele devolveria um estado que nunca existiu (doc 28 §7)"
        );
    }

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(clippy::unused_self, reason = "no-op em release; a rede é de debug")]
    pub(crate) fn audit_journal_matches_the_before(&self, _before: &crate::undo::ModelSnapshot) {}

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(clippy::unused_self, reason = "no-op em release; a rede é de debug")]
    pub(crate) fn audit_commit_window(
        &self,
        _snapshot_writes: u64,
        _hint: Option<crate::compositor::Region>,
    ) {
    }

    /// `true` if there is at least one edit to undo (an open in-flight shape transaction counts).
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.has_pending_shape_txn() || self.undo.can_undo()
    }

    /// `true` if there is at least one undone edit to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.undo.can_redo()
    }

    /// **Os bytes que o histórico de undo retém** — o número que o cap em BYTES consulta
    /// ([`crate::undo::DEFAULT_MAX_BYTES`]) e que `tests/measure_undo_memory.rs` reconcilia contra o
    /// dhat. Um budget que só é declarado não pode ser violado, só excedido em silêncio (HR-13, emenda
    /// do ADR-0117): esta é a porta pela qual ele é observado.
    #[must_use]
    pub fn undo_retained_bytes(&self) -> usize {
        self.undo.retained_bytes()
    }

    // ── Apply / preview drive ───────────────────────────────────────────

    /// Requisita commit (Apply): the bridge fires `EditorAction::OneShotImageOp`
    /// next frame, which calls `run_full` to bake the layer composite into the
    /// sprite.
    pub fn request_commit(&mut self) {
        self.pending_commit = true;
    }

    /// `true` when the working canvas/composite has edits not yet baked into the sprite (any stroke,
    /// layer op or adjustment since the last `set_source` bind). The shell uses this to auto-persist
    /// the painting when the selection leaves the sprite or the tool deactivates (Enio 2026-06-24).
    #[must_use]
    pub fn has_unbaked_edits(&self) -> bool {
        self.edited_since_bind && !self.canvas_rgba.is_empty()
    }

    /// Mark the working canvas as fully baked into the sprite — clears the unbaked-edits flag without
    /// touching the canvas (the shell calls this right after a successful auto-bake).
    pub fn mark_baked(&mut self) {
        self.edited_since_bind = false;
    }

    /// Take (and clear) the "deactivation deferred a bake" flag set by [`crate::tool::PainterTool`]'s
    /// `on_deactivate` when it kept the canvas. `true` → the shell must bake the kept canvas back to
    /// the last-bound sprite, then call `deactivate()` to finish the teardown.
    pub fn take_deferred_bake(&mut self) -> bool {
        std::mem::take(&mut self.deferred_bake)
    }

    /// Dimensions of the working canvas in pixels (`set_source` sets, `deactivate`
    /// zeroes).
    #[must_use]
    pub fn canvas_size(&self) -> (u32, u32) {
        self.source_size
    }

    /// Composite the live layers to a flat `Rec.601` luminance WITHOUT baking — the source for "Use as
    /// Brush Grain" on the active document, so the grain reflects the LIVE painting without re-pushing
    /// the sprite (a re-push runs `set_source`, which resets the layer stack and would destroy the
    /// user's layers). `None` for an empty canvas. Read-only (never mutates the document).
    #[must_use]
    pub fn composite_to_lum(&self) -> Option<(Vec<u8>, u32, u32)> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        let active = self.layers.active().unwrap_or(RtLayerId(0));
        let src = ToolPixelSource {
            active_id: active,
            active_rgba: &self.canvas_rgba,
            images: &self.images,
        };
        let rgba = composite(&self.layers, &src, w, h);
        let lum = rgba
            .chunks_exact(4)
            .map(|p| {
                ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29) >> 8) as u8
            })
            .collect();
        Some((lum, w, h))
    }

    /// Sample the visible layer COMPOSITE at normalized `(u, v)` ∈ `[0, 1]` → straight-sRGB8 RGBA —
    /// the displayed pixel, integrated with the layer stack (opacity / blend / masks / adjustments).
    /// Drives the colour-picker eyedropper so it reads the painted colour, not the transparent Vello
    /// overlay the generic GPU readback hits. Composites only the 1×1 pixel (the eyedrop is a one-off).
    #[must_use]
    pub fn sample_composite_at_uv(&self, u: f32, v: f32) -> Option<[u8; 4]> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || self.canvas_rgba.is_empty() {
            return None;
        }
        let ix = ((u.clamp(0.0, 1.0) * w as f32) as u32).min(w - 1);
        let iy = ((v.clamp(0.0, 1.0) * h as f32) as u32).min(h - 1);
        let src = ToolPixelSource {
            active_id: self.layers.active().unwrap_or(RtLayerId(0)),
            active_rgba: &self.canvas_rgba,
            images: &self.images,
        };
        let px = composite_region(
            &self.layers,
            &src,
            w,
            h,
            Region {
                x: ix,
                y: iy,
                w: 1,
                h: 1,
            },
        );
        (px.len() >= 4).then(|| [px[0], px[1], px[2], px[3]])
    }

    /// **Zero-copy preview drain** via Arc clone (1 atomic increment). Drains
    /// `preview_dirty` and returns the SAME underlying `Arc<Vec<u8>>` for a
    /// trivial stack, or a freshly-composited cache for a multi-layer stack. The
    /// bridge stashes this Arc in its `painter_preview` cache.
    #[must_use]
    pub fn take_preview_arc(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        if !std::mem::take(&mut self.preview_dirty) || self.canvas_rgba.is_empty() {
            self.last_drain_branch = crate::tool::DrainBranch::Idle;
            return None;
        }
        // Past the guard we are returning a fresh preview, so the CONTENT changed since the last drain:
        // bump the version the shell keys its upload on. This is the ONE place a Some is produced, so a
        // single bump here covers the trivial, composite and mask-view returns below. (See the field doc
        // on `preview_version`: the version replaces the shell's old `Arc::as_ptr` compare, which forced a
        // whole-canvas copy per move.)
        self.preview_version += 1;
        // Esta pista vai CONSUMIR agora o `dirty_rect` que a pista GPU também usaria para saber
        // o que mudou. A GPU não vê este frame, então tudo o que sai daqui é uma edição que o cache
        // dela (fatias por-camada no compositor + as texturas de plano do impasto) não conhece:
        // marque-o, para a próxima drenagem dela recusar confinamento e refazer inteiro. Espelho do
        // `composited = None` que a pista GPU faz na direção oposta (`take_preview_dirty`).
        self.gpu_lane_stale = true;
        let (w, h) = self.source_size;
        // A mask row's grayscale-VIEW eye is open → show that mask's grayscale instead of the composite.
        if let Some(gray) = self.mask_grayscale_view_pixels() {
            self.composited = Some(Arc::new(gray));
            self.preview_upload_bbox = None;
            self.last_drain_branch = crate::tool::DrainBranch::Gray;
            return Some((
                Arc::clone(self.composited.as_ref().expect("just set")),
                w,
                h,
            ));
        }
        // The trivial stack (single visible opaque Normal raster) stays the
        // zero-copy fast path. Any non-trivial stack (≥2 layers, or a layer with
        // opacity<1 / non-Normal blend / hidden / an adjustment) MUST be
        // composited here so per-layer opacity / blend / adjustments are visible.
        // Impasto forces the composite path: this lane hands back the RAW `canvas_rgba` Arc, which the
        // light pass must never write into (those are the artist's PIXELS, not a preview buffer) — and a
        // single-layer document is the common case, so without this guard the most ordinary way to use
        // Impasto would show no relief at all.
        if self.is_trivial_stack() && !self.mask_scratch_active() && !self.impasto_visible() {
            // Carry the accumulated dirty bbox into the bridge so a stroke uploads
            // only the touched sub-rect (B.1 partial lane), NOT the whole canvas
            // each frame. `None` here forced a full clone + premul + full texture
            // upload per painted frame, O(W×H) regardless of the 10px dab — the
            // 300→150 fps stroke regression. The first drain (source-push, no paint
            // yet) has no `dirty_rect` → stays `None` → the bridge seeds the GPU
            // texture with one full upload; paint frames then patch just the dab
            // footprint. If the texture isn't seeded yet the bridge's guard falls
            // back to a full upload anyway, so an early bbox can never desync it.
            self.preview_upload_bbox = self.dirty_rect.take();
            self.last_drain_branch = crate::tool::DrainBranch::TrivialFast;
            return Some((Arc::clone(&self.canvas_rgba), w, h));
        }
        let active = self.layers.active().unwrap_or(RtLayerId(0));
        // Dirty-rect fast lane: when a valid full composite is cached AND only a
        // known bbox changed, recomposite ONLY that region and blit it into the
        // cache — O(N×bbox) vs O(N×W×H). Otherwise do a full recompose.
        let dirty = self.dirty_rect.take();
        let stroke_dirtied = dirty.is_some();
        // A live Mask scratch does NOT force the full path. Its protection-overlay tint is PER-PIXEL
        // (`apply_mask_overlay`: a texel's tint depends only on its own coverage + colour, no global
        // term), and a mask dab changes coverage only inside its `dirty_rect` (the stamp edits the
        // scratch swapped into `canvas_rgba` — `stamp_dabs_mask`). The genuinely global overlay changes
        // (colour swatch, canvas-op, first scratch) all `invalidate_composite()` → the full arm below.
        // So the partial arm re-tints only the dab region (`apply_mask_overlay_region`), byte-identical
        // to a full re-tint there. Forcing the full path here was a whole-canvas recompose + full 16 MiB
        // upload per mask-painted frame (measured: 17 ms preview + 6.9 ms upload @ 2048² — the mask FPS
        // drop Enio reported), when only a dab-sized rect changed.
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
                // Impasto: light the FRESHLY-composited region (never the cache — lighting is not
                // idempotent, and re-lighting the cached pixels would compound the shading a little
                // more every frame). The normal reads across the region's edge into the full height
                // field, so the border is lit exactly as a full recompose would light it.
                let mut region = region;
                self.apply_impasto_light(&mut region, bbox);
                // Mask overlay: re-tint ONLY this region by the (dab-updated) scratch coverage — the
                // partial twin of the full arm's `apply_mask_overlay`, same per-pixel kernel, no-op
                // without a live scratch. This is what lets a mask stroke take the fast lane.
                self.apply_mask_overlay_region(&mut region, bbox);
                let region = region;
                self.compositor_cache.invalidate_from(active, &self.layers);
                self.adjustment_cache_pending = false;
                let cache = Arc::make_mut(self.composited.as_mut().expect("checked is_some"));
                blit_region(cache, w, &region, bbox);
                // The COMPOSITE stays partial (the 17 ms → ~2 ms win). But the mask overlay is a
                // TRANSLUCENT tint, and the shell's PARTIAL GPU upload of a sub-rect leaves visible
                // seams for it on the real device (Enio smoke 2026-07-24) — even though the CPU cache
                // is byte-identical to a full recompose (proven headless; a real-wgpu-only artifact,
                // mechanism not yet isolated). So force a FULL upload while a scratch is live: at 2048²
                // it is ~6 ms, well under the 16.7 ms frame budget (measured: dispatch ~8 ms, 60 fps).
                // Follow-up (doc 25 §13): isolate the partial-upload device seam so the fast path is
                // clean too (matters at 4096², where a full upload would overrun the budget).
                self.preview_upload_bbox = if self.mask_scratch_active() {
                    None
                } else {
                    Some(bbox)
                };
                self.last_drain_branch = crate::tool::DrainBranch::PartialComposite;
            }
            _ => {
                // Mask brush: the active layer composites NORMALLY (fully visible) — the protection scratch
                // never hides it; only `apply_mask_overlay` below tints the frozen region.
                let src = ToolPixelSource {
                    active_id: active,
                    active_rgba: &self.canvas_rgba,
                    images: &self.images,
                };
                let mut composed =
                    if std::mem::take(&mut self.adjustment_cache_pending) && !stroke_dirtied {
                        // Adjustment slider-drag: restart from the cut-point cache —
                        // bit-identical to a full `composite` (gate
                        // `cache_matches_full_recompose`).
                        composite_with_cache(&self.layers, &src, w, h, &mut self.compositor_cache)
                    } else {
                        self.compositor_cache.invalidate_from(active, &self.layers);
                        composite(&self.layers, &src, w, h)
                    };
                // Impasto: light the whole freshly-composited canvas (see the dirty-rect lane above).
                self.apply_impasto_light(&mut composed, Region { x: 0, y: 0, w, h });
                // Mask overlay: tint the composite by the active mask's coverage (no-op otherwise).
                self.apply_mask_overlay(&mut composed);
                self.composited = Some(Arc::new(composed));
                self.preview_upload_bbox = None;
                self.last_drain_branch = crate::tool::DrainBranch::FullComposite;
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
    /// upload as a sub-rect; `None` = upload the full texture.
    pub fn take_preview_upload_bbox(&mut self) -> Option<(u32, u32, u32, u32)> {
        self.preview_upload_bbox
            .take()
            .map(|r| (r.x, r.y, r.w, r.h))
    }

    /// Monotonic revision of the published layer structure. The bridge publishes
    /// the `LayerStack` snapshot only when this changes. Bumped by
    /// `invalidate_composite` (all structural/metadata edits) + `set_source`.
    #[must_use]
    pub fn layers_revision(&self) -> u64 {
        self.layers_revision
    }

    /// Monotonic CONTENT version of the preview, bumped once per dirty
    /// [`Self::take_preview_arc`]. The shell keys its GPU-slot upload on this
    /// instead of the drained `Arc`'s pointer, so it never has to hold a clone of
    /// `canvas_rgba` across the frame — which is what forced `stamp_dabs` to copy
    /// the whole canvas per move. Read it right after the drain that produced the
    /// preview (its value pairs with the `Arc` that drain returned).
    #[must_use]
    pub fn canvas_version(&self) -> u64 {
        self.preview_version
    }

    /// `PH2D_PAINT_PERF` diagnostic: `(which arm the last drain took, impasto_visible, mask_scratch)`.
    /// The branch names the cost (the O(W×H) `FullComposite` arm vs the O(1) `TrivialFast`); the two
    /// predicates name WHY a trivial stack fell to the full arm — the coarse `is_trivial_stack` flag
    /// the shell already logs cannot tell those apart. Purely observational; no state change.
    #[must_use]
    pub fn preview_drain_diag(&self) -> (crate::tool::DrainBranch, bool, bool) {
        (
            self.last_drain_branch,
            self.impasto_visible(),
            self.mask_scratch_active(),
        )
    }
}
