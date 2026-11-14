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

    /// ⭐⭐ **ESCOLHE o modo do dock** — o que um grupo segmentado precisa, e o que um *toggle* não
    /// sabe fazer.
    ///
    /// ⛔ Com um botão só, clicar era sempre INVERTER; um segmento que já está escolhido tem de
    /// ser um no-op, senão a fileira pisca ao ser tocada duas vezes no mesmo sítio. *Um alternador
    /// e um selector respondem a perguntas diferentes, e o desenho do controlo escolhe qual é.*
    pub fn set_dock_shows_layers(&mut self, shows_layers: bool) {
        self.dock_shows_layers = shows_layers;
    }

    /// Decode a per-row layers-panel widget [`NodeId`] back to its
    /// `(layer, kind)` by recomputing [`painter_layer_widget_id`] for every
    /// current layer × kind and matching. `None` if `id` isn't a per-row
    /// widget of any layer. (≤8 layers × kinds = cheap FNV hashes; the layers
    /// panel is not a hot path.)
    pub(crate) fn decode_layer_widget(
        &self,
        id: ph2d_a11y::NodeId,
    ) -> Option<(RtLayerId, crate::ids::PainterLayerWidget)> {
        super::layer_edit::layer_widget_in(&self.layers, id)
    }

    // ── Structural undo / redo (layer model) ────────────────────────────

    /// Undo the most recent structural layer edit, reinstalling the prior model.
    /// Returns `true` if an edit was undone. Driven by the shell's undo gesture /
    /// shortcut.
    pub fn undo_last(&mut self) -> bool {
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::Desfaz, {
            self.undo_last_corpo()
        })
    }

    fn undo_last_corpo(&mut self) -> bool {
        // A LIVE Deform Transform owns undo while it's up: step the gizmo back through its gestures (and
        // finally un-lift), all WITHOUT touching the structural timeline — the whole transform is one
        // structural entry, committed only when it ends (Enio 2026-07-04).
        if self.transform_undo_step() {
            return true;
        }
        // E a CANETA da seleção, pela MESMA razão: as âncoras em voo não estão no `ProjectState` (a
        // sessão inteira é um passo, gravado só no fechamento), então sem este dono o Ctrl+Z desfazia a
        // seleção ANTERIOR enquanto o artista desenhava esta (Enio, 2026-08-07).
        if self.selection_pen_undo() {
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
        // O VIVO é a base de que o relevo é materializado (degrau 3) — e ele é o MESMO snapshot que a
        // absorção acabou de construir, então recebê-lo não acrescenta trabalho.
        let mut live = self.snapshot_model();
        // ⚠️ **A região é perguntada ANTES de aplicar o passo** — depois, a entrada já saiu da pilha.
        // `None` = não confinado, e aí o restore é o de sempre (doc 28 §5.62).
        let confined = self.sujo_do_proximo_passo(false);
        // A tinta viva sai do caminho do restore (que acaba a sessão) e volta com as grades de antes do passo.
        let viva = self.wet_solta_para_o_undo(false);
        let presa = if viva.is_none() {
            self.wet_tira_para_o_restauro()
        } else {
            None
        };
        let mut relevo = self.tira_o_relevo_para_o_undo();
        let desfez = if let Some(model) =
            self.undo
                .undo_na_tela(&mut live, &mut self.canvas_rgba, &mut relevo)
        {
            self.restore_model_confined(*model, confined);
            true
        } else {
            self.devolve_o_relevo_do_undo(relevo);
            false
        };
        if let Some(v) = viva {
            self.wet_volta_do_undo(v, desfez);
        }
        if let Some(p) = presa {
            self.wet_volta_do_restauro(p, false);
        }
        desfez
    }

    /// Redo the most recently undone edit on the unified timeline. Returns `true` if an edit was redone.
    pub fn redo_last(&mut self) -> bool {
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::Refaz, {
            self.redo_last_corpo()
        })
    }

    fn redo_last_corpo(&mut self) -> bool {
        // A live / just-un-lifted Transform owns redo: re-lift the gizmo (recreate it) or step a gizmo pose
        // FORWARD — mirroring transform_undo_step, WITHOUT touching the structural timeline (Enio 2026-07-04).
        if self.transform_redo_step() {
            return true;
        }
        // A caneta viva é dona do redo pela mesma razão — e ela o ENGOLE com a pilha vazia, como o float
        // do Transform faz logo abaixo: refazer um passo estrutural sob um caminho em voo é a confusão do
        // undo pela outra ponta.
        if self.selection_pen_redo() {
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
        let mut live = self.snapshot_model();
        // ⚠️ **A região é perguntada ANTES de aplicar o passo** — depois, a entrada já saiu da pilha.
        // `None` = não confinado, e aí o restore é o de sempre (doc 28 §5.62).
        let confined = self.sujo_do_proximo_passo(true);
        let viva = self.wet_solta_para_o_undo(true);
        let presa = if viva.is_none() {
            self.wet_tira_para_o_restauro()
        } else {
            None
        };
        let mut relevo = self.tira_o_relevo_para_o_undo();
        let refez = if let Some(model) =
            self.undo
                .redo_na_tela(&mut live, &mut self.canvas_rgba, &mut relevo)
        {
            self.restore_model_confined(*model, confined);
            true
        } else {
            self.devolve_o_relevo_do_undo(relevo);
            false
        };
        if let Some(v) = viva {
            self.wet_volta_do_undo(v, refez);
        }
        if let Some(p) = presa {
            self.wet_volta_do_restauro(p, true);
        }
        refez
    }

    /// O relevo de impasto ENTREGUE ao Ctrl+Z (B17.5): sem este dono, os tiles do passo vão no lugar.
    fn tira_o_relevo_para_o_undo(&mut self) -> crate::undo_planes::Relevo {
        crate::undo_planes::Relevo {
            heights: std::mem::take(&mut self.heights),
            covers: std::mem::take(&mut self.covers),
            mats: std::mem::take(&mut self.mats),
        }
    }

    /// A recusa do Ctrl+Z devolve o relevo intacto (no sucesso o restauro instala o do modelo).
    fn devolve_o_relevo_do_undo(&mut self, r: crate::undo_planes::Relevo) {
        self.heights = r.heights;
        self.covers = r.covers;
        self.mats = r.mats;
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

    /// **Um passo de undo COMEÇA aqui** — a porta única do ciclo de vida de um passo, chamada onde o
    /// `before` é capturado.
    ///
    /// A ordem das duas metades carrega o peso:
    ///
    /// 1. **a cadeia primeiro.** Se alguém escreveu desde o último commit (a gota que a sim do Wet Paint
    ///    composita depois do pen-up), aquilo pertence ao passo **anterior** — foi ele que a causou — e
    ///    tem de entrar nele **antes** de o journal esquecer que a viu.
    /// 2. **e só então o journal passa a descrever ESTE passo.** Ancorado no último *commit* ele mistura
    ///    as duas coisas, e nos tiles que ambas tocam a primeira captura é a da gota: o lado `before`
    ///    sairia do passo anterior. Ancorado no *passo*, ele responde a pergunta certa.
    ///
    /// ⚠️ **Um sítio que ainda não a chama não fica errado, fica lento:** a proveniência
    /// ([`crate::undo::window::WriteState::journal_describes_step_at`]) recusa o journal e o commit cai
    /// no caminho de sempre. É a mesma política do contador de acessos não-declarados do S1, e é ela que
    /// torna a migração incremental em vez de tudo-ou-nada.
    pub(crate) fn begin_undo_step(&mut self) {
        self.absorb_foreign_writes_now();
        self.undo.marca.comeca_o_passo();
        self.audit_cursor_is_reconstructible();
        self.undo.begin_step(self.undo_writes, &self.canvas_rgba);
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

    /// O orçamento do histórico em vigor ([`crate::undo::history_budget_here`]).
    #[must_use]
    pub fn undo_max_bytes(&self) -> usize {
        self.undo.max_bytes()
    }

    // ── Apply / preview drive ───────────────────────────────────────────

    /// Requisita commit (Apply): the bridge fires `EditorAction::OneShotImageOp`
    /// next frame, which calls `run_full` to bake the layer composite into the
    /// sprite.
    pub fn request_commit(&mut self) {
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::PedeCommit, {
            self.request_commit_corpo()
        })
    }

    fn request_commit_corpo(&mut self) {
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
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::Assado, {
            self.mark_baked_corpo()
        })
    }

    fn mark_baked_corpo(&mut self) {
        self.edited_since_bind = false;
    }

    /// Take (and clear) the "deactivation deferred a bake" flag set by [`crate::tool::PainterTool`]'s
    /// `on_deactivate` when it kept the canvas. `true` → the shell must bake the kept canvas back to
    /// the last-bound sprite, then call `deactivate()` to finish the teardown.
    pub fn take_deferred_bake(&mut self) -> bool {
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::TiraAssadoAdiado, {
            self.take_deferred_bake_corpo()
        })
    }

    fn take_deferred_bake_corpo(&mut self) -> bool {
        std::mem::take(&mut self.deferred_bake)
    }

    /// Dimensions of the working canvas in pixels (`set_source` sets, `deactivate`
    /// zeroes).
    #[must_use]
    pub fn canvas_size(&self) -> (u32, u32) {
        self.source_size
    }

    /// Os píxeis de cada camada (RGBA8 straight, do tamanho da tela), pela ordem dos ids — a activa
    /// sai do `canvas_rgba`. Só leitura (o tutorial «fazer» compara-as com as do estado inicial).
    #[must_use]
    pub fn camadas_rgba(&self) -> Vec<(u64, &[u8])> {
        let n = self.source_size.0 as usize * self.source_size.1 as usize * 4;
        let active = self.layers.active();
        let mut out: Vec<(u64, &[u8])> = self
            .images
            .iter()
            .filter(|(id, img)| Some(**id) != active && img.rgba8.len() == n)
            .map(|(id, img)| (id.0, img.rgba8.as_slice()))
            .collect();
        if let Some(a) = active
            && self.canvas_rgba.len() == n
        {
            out.push((a.0, self.canvas_rgba.as_slice()));
        }
        out.sort_unstable_by_key(|(id, _)| *id);
        out
    }

    /// O documento volta ao de `t0` (o estado inicial de uma macro, do mesmo tamanho): um passo de
    /// undo estrutural. A água em curso assenta antes; a idade dela volta com o desenho.
    pub(crate) fn recomeca_com(&mut self, t0: &PainterTool) {
        self.wetpaint_end_session();
        let before = self.snapshot_model();
        self.restore_model(t0.snapshot_model());
        self.desde_o_ultimo_up = t0.desde_o_ultimo_up;
        self.edited_since_bind = true;
        // O «Recomeçar» é da macro: não marca (W9).
        self.undo.marca.suspensa = true;
        self.commit_structural_edit(before);
        self.undo.marca.suspensa = false;
        self.invalidate_composite();
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
        let full = Region { x: 0, y: 0, w, h };
        let mut rgba = self.composto_pelo_vidro(&src, full).unwrap_or_else(|| {
            let mut c = composite(&self.layers, &src, w, h);
            self.compoe_sobre_o_papel(&mut c, full);
            c
        });
        // ⚠️ **O relevo entra AQUI, e pela mesma função que o BAKE chama.**
        //
        // Esta porta responde *"com o que este documento se parece?"* para o Grain, o Paper e a
        // Granulation do menu "Use as …" — e o `run_full` (o Apply) responde a MESMA pergunta,
        // dizendo no próprio doc-comment por quê: *"o campo de altura não sobrevive ao Apply, então
        // a sombra tem de ser assada, senão o Apply jogaria o relevo fora em silêncio e devolveria
        // tinta chapada"*. Sem esta linha as duas portas discordavam exatamente pela luz (medido:
        // 523 de 3600 texels, pior delta 68), e o sintoma era o do report do Enio (2026-08-09) — um
        // sprite JÁ assado da hierarquia levava a sombra do relevo, e o documento ATIVO não.
        //
        // Sem relevo o passe multiplica por 1 e soma 0, então isto é byte-idêntico ao que já shipava.
        self.apply_impasto_light(&mut rgba, Region { x: 0, y: 0, w, h });
        let lum = rgba
            .as_chunks::<4>()
            .0
            .iter()
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
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::Preview, {
            self.take_preview_arc_corpo()
        })
    }

    /// Drains the dirty bbox `(x, y, w, h)` of the LAST [`Self::take_preview_arc`]
    /// — `Some` iff that drain was a partial fast-lane update the bridge may
    /// upload as a sub-rect; `None` = upload the full texture.
    pub fn take_preview_upload_bbox(&mut self) -> Option<(u32, u32, u32, u32)> {
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::PreviewBbox, {
            self.take_preview_upload_bbox_corpo()
        })
    }

    /// **As caixas do próximo [`Self::take_preview_upload_bbox`]** (B17.5): espie-as ANTES dele. Vazia =
    /// o bbox sozinho; só valem se a união delas é esse bbox (um bbox posto por outro caminho as cala).
    pub fn preview_upload_caixas(
        &self,
    ) -> impl ExactSizeIterator<Item = (u32, u32, u32, u32)> + '_ {
        let uniao = self
            .preview_upload_caixas
            .iter()
            .copied()
            .reduce(crate::tool::paint::region::union_region);
        let caixas = if self.preview_upload_caixas.len() > 1
            && uniao.is_some()
            && uniao == self.preview_upload_bbox
        {
            self.preview_upload_caixas.as_slice()
        } else {
            &[]
        };
        caixas.iter().map(|r| (r.x, r.y, r.w, r.h))
    }

    fn take_preview_upload_bbox_corpo(&mut self) -> Option<(u32, u32, u32, u32)> {
        self.preview_upload_por_tomar = false;
        self.preview_upload_caixas.clear();
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

    /// A revisão do CONTEÚDO do documento: `(layers_revision, pixel_clock)`. Toda escrita de píxeis
    /// passa por `mark_dirty` → `bump_layer_pixels` (a testemunha da época do traço depende disso) e
    /// toda edição de estrutura/undo por `invalidate_composite`: um dos dois anda. A chave da
    /// avaliação do tutorial «fazer» (Macros W6).
    #[must_use]
    pub fn content_revision(&self) -> (u64, u64) {
        (self.layers_revision, self.pixel_clock)
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
