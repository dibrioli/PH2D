//! Structural UNDO / SNAPSHOT — capture the editable model, reinstall a
//! snapshot (a structural undo/redo) and record a transition. `impl PainterTool`
//! (one of several blocks in this crate). Split out of the former
//! `tool/layers.rs` god-file (pure move). NB: `crate::undo` is the undo
//! controller crate-module; this `tool::layers::undo` only holds the tool-side
//! glue and refers to it fully-qualified.

use super::super::*;

impl PainterTool {
    /// Capture the full editable model for transactional (structural) undo —
    /// see [`crate::undo::ModelSnapshot`]. `canvas_rgba` is `Arc`-shared (cheap);
    /// `images` deep-copies the non-active layers (a rare, user-paced cost).
    pub(crate) fn snapshot_model(&self) -> crate::undo::ModelSnapshot {
        let (mask_scratch, mask_scratch_target) = self.mask_scratch_for_snapshot();
        let (selection_mask, selection_active, selection_crisp, selection_feather) =
            self.selection_for_snapshot();
        let deform = self.warp_for_snapshot();
        let sculpt = self.sculpt_for_snapshot();
        crate::undo::ModelSnapshot {
            layers: self.layers.clone(),
            images: self.images.clone(),
            heights: self.heights.clone(),
            covers: self.covers.clone(),
            mats: self.mats.clone(),
            vidros: self.vidros.clone(),
            // Este snapshot SEGURA o relevo; quem o descreve sem segurar é
            // [`Self::snapshot_model_eliding_relief`].
            relief_elided: crate::undo::elide::ElidedRelief::default(),
            canvas_elided: None,
            #[cfg(any(test, debug_assertions))]
            canvas_audit: None,
            canvas_rgba: Arc::clone(&self.canvas_rgba),
            // A forma do que o snapshot carrega — o stride de cada plano para o motor de delta.
            canvas_size: self.source_size,
            writes: self.undo_writes,
            selection: self.selection.clone(),
            // Layer ops carry no open shape / live preview; the shape paths override these via
            // `capture_shape_model` (see `tool::paint::shape_snapshot`).
            shape: None,
            offset_norm: self.shape_offset_norm(),
            offset_base_px: self.shape_offset_base_px(),
            papel: self.papel,
            tintas_molhadas: self.tintas_molhadas.clone(),
            preview_patch: None,
            // Layer ops carry no parked stroke shapes; the shape paths override this via `capture_shape_model`.
            parked_shapes: Vec::new(),
            // The ACTIVE shape's boolean op (captured always — cheap; makes the op-cycle tap undoable).
            active_op: self.active_op_wire(),
            mask_scratch,
            mask_scratch_target,
            selection_mask,
            selection_active,
            selection_crisp,
            selection_feather,
            // The parametric shape list is the source of truth the mask derives from — capture it so undo
            // restores the editable shapes (curve points/handles, box params), not just the raster mask.
            selection_shapes: self.selection_shapes_snapshot(),
            deform,
            sculpt,
        }
    }

    /// **O mesmo snapshot, DESCREVENDO o relevo em vez de o segurar** — o degrau 4 do S3
    /// (doc 28 §5.60); o porquê inteiro está em [`crate::undo::elide`].
    ///
    /// ⚠️ **É uma segunda PORTA e não uma segunda RESPOSTA:** as duas devolvem o mesmo modelo, e a
    /// diferença é quem fica dono dos três planos de relevo. Quem precisa deles materializados (o
    /// `live` que o undo/redo passa por base, a rede do audit) chama a de cima; quem só precisa
    /// DESCREVER um passo — o `before` de um traço — chama esta, e é isso que faz o 1º dab escrever
    /// no lugar em vez de forkar o documento (`strong_count > 1` é a pergunta do copiador, §5.15).
    ///
    /// ⚠️ **Ela esvazia os mapas fortes no MESMO passo em que baixa as testemunhas.** Um snapshot que
    /// elidisse e segurasse carregaria o fato duas vezes — e o dono extra que ele deixaria é
    /// exatamente o que a elisão existe para remover.
    ///
    /// ⚠️ **Ela e o JOURNAL do relevo shipam JUNTOS** (doc 28 §5.58.1). O lado `before` de um passo
    /// elidido **é** o journal; elidir sem ele faria todo commit cair no `relief_indescribable` — a
    /// história inteira descartada a cada traço —, e promover o journal sozinho pagaria captura *e*
    /// fork até o fork morrer. A fase B desta wave subiu os dois no mesmo commit.
    pub(crate) fn snapshot_model_eliding_relief(&self) -> crate::undo::ModelSnapshot {
        let mut m = self.snapshot_model();
        #[cfg(test)]
        if !self.undo.elide_relief {
            return m; // a ablação do A/B — ver `UndoController::elide_relief`
        }
        // O vidro (ADR-0205 D2), sob a chave: o lado de antes é o journal dele.
        let vidro = crate::undo::window::vidro_pelo_journal();
        m.relief_elided = crate::undo::elide::ElidedRelief::of(
            &self.heights,
            &self.covers,
            &self.mats,
            vidro.then_some(&self.vidros),
        );
        m.heights.clear();
        m.covers.clear();
        m.mats.clear();
        if vidro {
            m.vidros.clear();
        }
        // E a TELA (B17.4): o lado de antes é o journal dela, então o 1.º dab escreve no lugar.
        if !self.undo.write_state.on_foreign_plane() {
            m.elide_canvas();
        }
        m
    }

    /// A região a que o próximo undo (ou redo) está confinado ([`crate::undo::UndoController::peek_confined_sujo`]),
    /// com a janela da tinta de cada camada cuja marca do Wet vira (o 1.º traço Wet dela, B10c).
    #[cfg(test)]
    pub(crate) fn regiao_do_proximo_passo(&self, redo: bool) -> Option<crate::compositor::Region> {
        self.sujo_do_proximo_passo(redo).map(|(r, _)| r)
    }

    /// [`Self::regiao_do_proximo_passo`] e as caixas que a cobrem (B17.5; vazia quando uma só basta).
    pub(crate) fn sujo_do_proximo_passo(
        &self,
        redo: bool,
    ) -> Option<(crate::compositor::Region, Vec<crate::compositor::Region>)> {
        self.undo
            .peek_confined_sujo(redo, &|id| self.alcance_da_marca_molhada(id))
    }

    /// Onde virar a marca do Wet da camada `id` muda a luz: os píxeis com alfa dela (`papel_a_vista`). Nada onde
    /// a luz não a lê — o filtro do `tinta` de `relief_fields_for` (máscara, escondida, não raster, tamanho
    /// errado, sem substrato, a sprite crua que é o papel: [`Self::a_marca_molhada_e_lida`]). Também o pen-down (B18.4).
    pub(crate) fn alcance_da_marca_molhada(
        &self,
        id: crate::layers::LayerId,
    ) -> crate::undo_delta::confine::PlaneReach {
        use crate::undo_delta::confine::{PlaneReach, janela_da_tinta};
        let lida = self.layers.get(id).is_some_and(|l| {
            matches!(l.kind, crate::layers::LayerKind::Raster(_))
                && self.layers.effectively_visible(id)
                && !self.layers.is_mask(id)
        }) && self.a_marca_molhada_e_lida(id);
        let px: Option<&[u8]> = if self.layers.active() == Some(id) {
            Some(self.canvas_rgba.as_slice())
        } else {
            self.images.get(&id).map(|i| i.rgba8.as_slice())
        };
        let (w, h) = self.source_size;
        match px {
            Some(px) if lida && px.len() == (w as usize) * (h as usize) * 4 => {
                janela_da_tinta(px, w as usize)
            }
            _ => PlaneReach::Untouched,
        }
    }

    /// As camadas cujos píxeis um passo pode trocar (B17.5): as duas ativas (a ativa lê a tela, que
    /// o re-carimbo do restauro reescreve) e as de `images` cujo `Arc` difere. As outras são o MESMO buffer
    /// nos dois lados: a versão delas fica, e a GPU não re-carrega a fatia. `None` = a pilha mudou: todas.
    fn camadas_que_o_passo_troca(
        &self,
        m: &crate::undo::ModelSnapshot,
    ) -> Option<std::collections::BTreeSet<RtLayerId>> {
        if !self.layers.all_ids().eq(m.layers.all_ids()) {
            return None;
        }
        let mut ids: std::collections::BTreeSet<RtLayerId> =
            [self.layers.active(), m.layers.active()]
                .into_iter()
                .flatten()
                .collect();
        for id in self.images.keys().chain(m.images.keys()) {
            match (self.images.get(id), m.images.get(id)) {
                (Some(a), Some(b)) if Arc::ptr_eq(a, b) => {}
                _ => {
                    ids.insert(*id);
                }
            }
        }
        Some(ids)
    }

    /// Reinstall a model snapshot (a structural undo/redo). Restores the layer
    /// tree (incl. the active target), the per-layer pixel store, the active
    /// working buffer, and the panel selection, then refreshes every derived
    /// cache so the composite + GPU preview rebuild.
    pub(crate) fn restore_model(&mut self, m: crate::undo::ModelSnapshot) {
        self.restore_model_confined(m, None);
    }

    /// [`Self::restore_model`] sabendo **a que região o passo estava confinado**.
    ///
    /// `None` = *não confinado*, e aí isto É o `restore_model` de sempre: derruba o cache de composite
    /// e o dirty-rect, e a próxima drenagem refaz a tela inteira. `Some(r)` promete que **fora de `r` a
    /// figura é a mesma**, e a drenagem pode usar a pista PARCIAL — medido, um quadro pós-Ctrl+Z a
    /// 4096² com impasto cai de **381 ms para a ordem de 0,6** (doc 28 §5.62).
    ///
    /// ⚠️ **Quem responde é a HISTÓRIA, não este método** ([`crate::undo::UndoController::peek_confined_sujo`]):
    /// a promessa exige os dezenove planos e os metadados dos DOIS endpoints, e nada disso está
    /// disponível aqui — o snapshot que chega já é o resultado.
    pub(crate) fn restore_model_confined(
        &mut self,
        m: crate::undo::ModelSnapshot,
        confined: Option<(crate::compositor::Region, Vec<crate::compositor::Region>)>,
    ) {
        // Confinado ou não: a camada cujo `Arc` é o mesmo nos dois lados tem os mesmos píxeis, e a fatia dela na
        // placa continua certa (um Ctrl+Z não confinado a 4K subia 33 MB POR camada; a pilha que muda segue `None`).
        let trocadas = self.camadas_que_o_passo_troca(&m);
        self.layers = m.layers;
        self.images = m.images;
        self.heights = m.heights;
        self.covers = m.covers;
        self.mats = m.mats;
        self.vidros = m.vidros;
        // As duas sessões molhadas acabam neste restauro (abaixo); o journal delas descreve um plano trocado.
        self.vidro_congelado.larga_as_sessoes(None);
        self.tintas_molhadas = m.tintas_molhadas;
        // The live-edit buffer describes a stroke on a ground that no longer exists — forget it, or the
        // next Depth drag would rebuild an undone stroke out of thin air.
        self.drop_live_relief();
        // And the in-progress ENVELOPE too. The shape editors reset it before each re-stamp, so when
        // the restored snapshot still carries a shape, `restore_shape_overlay` below rebuilds it in
        // lock-step with the pixels. But the LAST undo restores a snapshot with NO shape — that path
        // only clears the editors, and the previous re-stamp's envelope survived with no pigment
        // under it: the light kept shading a Line whose colour was gone (Enio, live smoke
        // 2026-07-15: "desfez a cor mas restou o relevo"). A gesture cannot be in flight during an
        // undo, so there is never a live envelope this reset could legitimately lose.
        self.reset_stroke_height();
        // …and the protection session, one plane over and for a stronger reason: its `base` is a canvas
        // that no longer exists, and a projection against it would blend the restored pixels toward the
        // undone ones. The canvas is replaced on the very next line, so nothing here could survive as a
        // valid base — dropping it makes the next gated batch re-seed from the restored truth.
        self.end_gate_session();
        self.instala_a_tela_do_restauro(m.canvas_rgba);
        self.selection = m.selection;
        // Reinstate the Mask brush scratch + target so an undo/redo across a mask stroke restores the
        // live mask-in-progress in lock-step with the pixels (the composite rebuild below reads it).
        self.restore_mask_scratch(m.mask_scratch, m.mask_scratch_target);
        // Reinstate the parametric selection SHAPES (the source of truth) BEFORE the mask, so the editable
        // curve points/handles + box params roll back with the mask and the next recompose can't regenerate
        // the undone edit (Enio 2026-07-03).
        self.restore_selection_shapes(m.selection_shapes);
        // Reinstate the Selection mask + active flag so an undo/redo across a selection edit restores the
        // selected region in lock-step with the pixels (ADR-0103).
        self.restore_selection(
            m.selection_mask,
            m.selection_active,
            m.selection_crisp,
            m.selection_feather,
        );
        self.set_shape_offset_norm(m.offset_norm);
        self.set_shape_offset_base_px(m.offset_base_px);
        // A cor do papel não é história, a existência dele é ([`Self::poe_o_papel`]).
        if self.papel.is_none() != m.papel.is_none() {
            self.papel = m.papel;
            self.invalidate_composite(); // o papel cobre a tela inteira
        }
        // Reinstate the ACTIVE shape's boolean op so undoing a centre-square op-cycle tap rolls it back.
        self.set_active_op_wire(m.active_op);
        // The **Sculpt** session — and it must land BEFORE the shape overlay below, which RE-STAMPS.
        //
        // The sculpt writes the layer's relief LIVE, so a snapshot captures an already-carved plane. Two
        // things therefore have to be true when that re-stamp runs, and only the second is obvious:
        //
        //   1. `heights` is the snapshot's (done above), and
        //   2. the SESSION is the snapshot's too — because the re-stamp restores the window from the
        //      session's frozen `pre` before re-accumulating.
        //
        // Restore the session *after* the re-stamp and (2) is false: the re-stamp finds no session, opens a
        // fresh one, and freezes the **carved** plane as its source — then runs the kernel on top of it.
        // Undo, redo, and the ridge has been smoothed twice; the pixels are perfect the whole time, so
        // nothing else in the system so much as blinks. (`docs/Painter/18…` §10.4; the same
        // source-of-truth-before-its-derivative ordering the selection shapes get, four lines up.)
        self.restore_sculpt(m.sculpt);
        // Wet Paint (doc 21): an undo is a wholesale foreign swap — the water DIES, eagerly and
        // HERE, before `restore_shape_overlay` below re-stamps the reinstated editor: that refill's
        // stash tail re-arms the guard as an OWNED write, and a lazy guard-kill would lose the race
        // (the session would surf the undo on the refill's re-arm — caught red by G10). Sibling of
        // the watercolor teardown at the end of this fn.
        self.wetpaint_end_session();
        // Reinstate (or clear) the open shape overlay: peel the snapshot canvas back to its pristine
        // baseline (strip the preview patch) and re-stamp the editor's geometry, so dots + pixels stay in
        // sync. A `None` shape just clears the editors. See `tool::paint::shape_snapshot`.
        self.restore_shape_overlay(m.shape, m.preview_patch, m.parked_shapes);
        // Every layer's pixels may have changed identity → bump all content
        // versions so the GPU compositor re-uploads each slice, and drop the CPU
        // composite cache + bump the panel `layers_revision`.
        // Reinstate the Deform session (displacement + `pre` + active) in lock-step with the pixels, so
        // undoing a deform stroke rolls the warp back AND keeps Reconstruct able to un-warp what remains.
        self.restore_deform(m.deform);
        let antes = (self.layers_revision, self.pixel_clock);
        match trocadas {
            Some(ids) => ids
                .into_iter()
                .for_each(|id| self.bump_layer_pixels(Some(id))),
            None => self.bump_all_layer_pixels(),
        }
        match confined {
            // O caminho de sempre: um passo que pode ter mudado QUALQUER coisa (opacidade, blend,
            // ordem, visibilidade, um ajuste) muda o composite fora de retângulo nenhum.
            None => self.invalidate_composite(),
            // ⚠️ **O passo é confinado, então o cache de composite SOBREVIVE e a região é publicada.**
            // As duas metades andam juntas: a pista parcial da drenagem exige `composited.is_some()`
            // **e** um `dirty_rect`, e derrubar um dos dois manda a tela inteira ser refeita.
            //
            // O que o `invalidate_composite` faz e continua sendo feito aqui: a revisão de publicação
            // (o painel re-clona a pilha), o `edited_since_bind`, e o cache de CORTE dos ajustes — este
            // último é dropado inteiro de propósito, porque é um cache que se re-preenche a frio e
            // custa menos que raciocinar sobre quais cortes a região tocou.
            Some((r, caixas)) => {
                self.mark_dirty_caixas(r, caixas);
                self.edited_since_bind = true;
                self.compositor_cache
                    .invalidate_from(crate::layers::LayerId(0), &self.layers);
                self.bump_layers_revision();
                let depois = (self.layers_revision, self.pixel_clock);
                self.aplicada.segue_o_passo_confinado(antes, depois);
            }
        }
        // Watercolor: the canvas identity just changed under the wet session — its moisture map (+ frozen
        // base + union buffers) is now stale (the overlay showed the undone stroke's damp; the guard Arc no
        // longer matches). Tear it down so undo/redo clears the wetness (Enio 2026-07-11). No-op when dry.
        self.dry_session_now();
        self.preview_dirty = true;
        // **O journal esquece o passo** — e é a reinstalação inteira que o exige, não uma cortesia.
        //
        // Todo plano acabou de ser trocado por outro, então os bytes que o journal guardava descrevem um
        // passado que não existe mais. Pior: as substituições acima passaram pela porta
        // ([`crate::tool::paint::plane_fork`]), então elas CAPTURARAM os planos de antes do undo — que
        // são exatamente o estado que o undo está desfazendo. Sem esta linha o journal jura que o passo
        // seguinte começou no estado desfeito (o censo do degrau 2 pegou as 3 últimas divergências aqui).
        self.undo.write_state.reset_journal();
        // E a janela também: o passo seguinte começa no modelo restaurado (a troca do plano acima não o varre).
        let mut w = self.undo.write_state.get();
        w.reset(self.undo_writes);
        self.undo.write_state.set(w);
        // O passo pode ter tirado uma camada: a tinta adormecida dela não acorda na que reusar o id (B02.1).
        self.wet_poda_as_adormecidas();
    }

    /// Record a STRUCTURAL transition from `before` (captured at the edit's start)
    /// to the CURRENT model, making the edit undoable in chronological order. Every
    /// structural site (add/delete/duplicate layer, mask/adjustment create, active
    /// switch) calls this with the model it snapshotted before mutating.
    pub(crate) fn commit_structural_edit(&mut self, before: crate::undo::ModelSnapshot) {
        // A janela declarada desde o último commit — aceita só se este `before` for posterior a ele (aí
        // ela é um SUPERCONJUNTO do que ele não viu) e se nenhum acesso de escrita ficou sem declarar.
        let hint = self.undo.write_state.get().hint_for(before.writes);
        self.audit_commit_window(before.writes, hint);
        self.audit_journal_matches_the_before(&before);
        // ⚠️ **A identidade do §5.28 NÃO é perguntável aqui, e a tentativa está registrada.** Neste
        // ponto o `absorb_foreign_writes` ainda não rodou (ele mora dentro do `record_*` abaixo),
        // então o cursor é o de ANTES da reconciliação — perguntar se ele é reconstruível é
        // perguntar sobre um estado que a linha seguinte vai mover. Medido: 588 de 589 chamadas
        // concordam e a exceção é o re-stamp do Deform, exatamente o caso que a absorção existe
        // para consertar. O lugar onde a pergunta cabe é `begin_undo_step`, ANTES do passo (doc
        // 28 §5.28), e é lá que ela está.
        let after = self.snapshot_model();
        self.undo.record_structural_hinted(before, after, hint);
        let mut w = self.undo.write_state.get();
        w.reset(self.undo_writes);
        self.undo.write_state.set(w);
    }

    /// [`Self::commit_structural_edit`] for a COALESCIBLE action (see [`crate::undo::CoalesceKind`]): a run
    /// of repeated same-kind actions collapses into one undo entry spanning first-before → latest-after.
    pub(crate) fn commit_structural_edit_coalesced(
        &mut self,
        kind: crate::undo::CoalesceKind,
        before: crate::undo::ModelSnapshot,
    ) {
        let after = self.snapshot_model();
        self.undo.record_structural_coalesced(kind, before, after);
        let mut w = self.undo.write_state.get();
        w.reset(self.undo_writes);
        self.undo.write_state.set(w);
    }
}

#[cfg(test)]
mod donos_entre_gestos {
    use super::super::super::*;
    use ph2d_editor_core::tool::{CanvasPointer, PointerPhase, RasterEditTool};

    fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
        CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        }
    }

    fn traco(t: &mut PainterTool, y: f32) {
        t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
        for k in 1..=4u8 {
            t.on_canvas_pointer(cp([40.0 + f32::from(k) * 30.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([200.0, y], PointerPhase::Up));
    }

    /// **B17.4 (PX-19/PP-01):** no pen-down a tela tem UM dono forte — o cursor e o `before` do traço a DESCREVEM
    /// (testemunha + journal), então o 1.º dab escreve no lugar em vez de copiar 33 MB a 4K. Entre gestos o cursor
    /// ainda a segura (os editores de forma reinstalam outra tela depois do commit, ver `UndoController::begin_step`).
    /// RED visto: antes da cura o pen-down tinha 3 donos (tool + cursor + `before`). E o undo do traço volta ao byte.
    #[test]
    fn at_pen_down_every_canvas_plane_has_one_strong_owner() {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 256 * 256 * 4], 256, 256);
        traco(&mut t, 80.0);
        traco(&mut t, 160.0);
        let entre = std::sync::Arc::strong_count(&t.canvas_rgba);
        let antes: Vec<u8> = t.canvas_rgba.as_ref().clone();
        t.on_canvas_pointer(cp([40.0, 220.0], PointerPhase::Down));
        let no_pen_down = std::sync::Arc::strong_count(&t.canvas_rgba);
        for k in 1..=4u8 {
            t.on_canvas_pointer(cp([40.0 + f32::from(k) * 30.0, 220.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([200.0, 220.0], PointerPhase::Up));
        assert_ne!(*t.canvas_rgba, antes, "o 3.º traço pintou");
        assert!(t.undo_last(), "o 3.º traço desfaz-se");
        assert_eq!(*t.canvas_rgba, antes, "o undo volta ao byte");
        assert_eq!(entre, 2, "entre gestos o cursor segura a tela");
        assert_eq!(no_pen_down, 1, "no pen-down a tela tem {no_pen_down} donos");
    }
}
