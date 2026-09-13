//! **Fase do quadro: OS VERBOS DE NÓ E DE ARRANJO** — tipo de vértice, apagar, seleccionar, média/unir/inverter, soldar, simetria, cortar,
//! arranjar, duplicar, espelhar e rodar (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct NodeAndArrangeVerbsIntents {
    pub(super) pending_vec_select_subpath: bool,
    pub(super) pending_vec_select_same: bool,
    pub(super) pending_vec_join: bool,
    pub(super) pending_vec_weld: bool,
    pub(super) pending_vec_cut: bool,
    pub(super) pending_vec_symmetry_apply: bool,
    pub(super) pending_vec_cut_discard: bool,
    pub(super) pending_vec_reverse: bool,
    pub(super) pending_vec_average: bool,
    pub(super) pending_vec_vertex_kind: Option<ph2d_vec_scene::VertexKind>,
    pub(super) pending_vec_delete_vertex: bool,
    pub(super) pending_vec_reorder: Option<ph2d_vec_scene::ZOrder>,
    pub(super) pending_vec_duplicate: bool,
    pub(super) pending_vec_flip: Option<ph2d_vec_scene::FlipAxis>,
    pub(super) pending_vec_rotate: Option<ph2d_vec_scene::Rotate90>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_node_and_arrange_verbs(
        &mut self,
        intents: NodeAndArrangeVerbsIntents,
        vec_px_to_world: f64,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        let NodeAndArrangeVerbsIntents {
            pending_vec_select_subpath,
            pending_vec_select_same,
            pending_vec_join,
            pending_vec_weld,
            pending_vec_cut,
            pending_vec_symmetry_apply,
            pending_vec_cut_discard,
            pending_vec_reverse,
            pending_vec_average,
            pending_vec_vertex_kind,
            pending_vec_delete_vertex,
            pending_vec_reorder,
            pending_vec_duplicate,
            pending_vec_flip,
            pending_vec_rotate,
        } = intents;
        if let Some(kind) = pending_vec_vertex_kind {
            crate::input_dispatch::apply_vec_vertex_kind(vec_scene, &mut self.vec.pen, kind);
        }
        if pending_vec_delete_vertex {
            crate::input_dispatch::apply_vec_delete_vertex(vec_scene, &mut self.vec.pen);
        }
        // ⚠️ Os dois só mudam QUEM está selecionado, e a seleção não é estado de documento — o
        // undo global (por diff) não vê passo nenhum neles.
        if pending_vec_select_subpath {
            self.vec.pen.select_subpath_verts(vec_scene);
        }
        if pending_vec_select_same {
            self.vec.pen.select_verts_of_same_kind(vec_scene);
        }
        // **As três da W4.** O passo de undo é o da fila global (por diff do quadro): ele só
        // nasce se algo de fato mudou.
        for (armed, op) in [
            (pending_vec_join, 0u8),
            (pending_vec_reverse, 1),
            (pending_vec_average, 2),
        ] {
            if !armed {
                continue;
            }
            match op {
                0 => self.vec.pen.join_selection(vec_scene),
                1 => self.vec.pen.reverse_selected_paths(vec_scene),
                _ => self.vec.pen.average_selected_verts(vec_scene),
            };
        }
        // ⭐⭐⭐ **SOLDAR** (plano 39) — ao lado das três acima, e **fora** do laço delas porque
        // ela precisa das POSES: dois traços só se cruzam depois de o `Transform` os pôr no
        // lugar, e medir na geometria local diria que eles não se encontram.
        // ⚠️ Um comando que não cortou nada não muda a cena, e por isso não gasta um Ctrl+Z (o
        // undo global regista por diff).
        if pending_vec_weld {
            let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            crate::vec_weld::apply_vec_weld(
                vec_scene,
                &mut self.vec.pen,
                &xf,
                crate::vec_snap::vec_weld_tolerance(vec_px_to_world),
            );
        }
        // **O CORTE e o DESCARTE da lâmina.** Como as três acima: o passo é o do undo global, e
        // só nasce se algo de fato mudou (uma lâmina que não atravessa nada não deixa linha na
        // fila do Ctrl+Z).
        // **Apply Symmetry** — o único gesto desta seção que toca o documento. Até aqui as
        // cópias eram DESENHO; a partir daqui são geometria, e a simetria sai com a
        // forma-fonte (o `sync` do frame seguinte despawna a entidade dela).
        if pending_vec_symmetry_apply {
            let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            // ⚠️ TODA forma armada, não a seleção: a simetria é um MODO, e *"consolidar a
            // forma e desativar a simetria"* vale para o que o modo produziu. O porquê de
            // consolidar só o selecionado ser destrutivo está na `armed_paths`.
            let ids = crate::symmetry_live::armed_paths(sim, &self.vec.entities, vec_scene);
            crate::symmetry_live::materialise(
                vec_scene,
                sim,
                &mut self.vec.pen,
                &self.vec.entities,
                &xf,
                &ids,
            );
        }
        if pending_vec_cut {
            // A seleção que o corte exige é a da LÂMINA, e ela pode chegar por qualquer das
            // duas listas do pen (a de objeto e a de caminho) — perguntar só a uma delas faria
            // o botão recusar um gesto legítimo.
            let mut selected = self.vec.pen.selected_paths().to_vec();
            if let Some(one) = self.vec.pen.selected()
                && !selected.contains(&one)
            {
                selected.push(one);
            }
            let armed = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Cut;
            if crate::vec_cut_line::apply_cut(sim, vec_scene, &self.vec.entities, &selected, armed)
                > 0
            {
                // A seleção descreve formas que já não existem — as peças as substituíram.
                self.vec.pen.select(None);
            }
        }
        if pending_vec_cut_discard {
            crate::vec_cut_line::discard(sim, vec_scene, &self.vec.entities);
        }
        // **Os botões Arrange escrevem na ÁRVORE** (Enio, 2026-08-04). Eles chamavam o
        // `VecScene::reorder_path`, que mexe na ordem do VETOR da cena — e essa é reescrita a
        // cada frame pela projeção da árvore (ADR-0110), então os quatro estavam MORTOS:
        // acendiam, mexiam, e o frame seguinte desfazia. O undo é o GLOBAL por diff (a árvore
        // é `ProjectState`). (Até 2026-09-12 esta nota contrastava-o com a `vec_history`, a
        // pilha do vetor que guardava a CENA — ela morreu sem nunca ter tido leitor.)
        if let Some(order) = pending_vec_reorder
            && let Some(sel) = self.vec.pen.selected()
        {
            ph2d_vec_entities::entities::zorder::reorder(sim, &self.vec.entities, sel, order);
        }
        if pending_vec_duplicate {
            // Offset the clone by a fixed SCREEN distance (px → world) so it's
            // visibly separated at any zoom.
            //
            // ⚠️ A const é a PARTILHADA. Ela vivia aqui como cópia local de 12.0 ao lado do
            // `PASTE_OFFSET_PX`, que é o mesmo número pela mesma razão — exatamente a
            // divergência contra a qual o doc de `screen_offset_world` avisa.
            let off = crate::input_dispatch::PASTE_OFFSET_PX * vec_px_to_world;
            crate::input_dispatch::apply_vec_duplicate(vec_scene, &mut self.vec.pen, off, off);
        }
        if let Some(axis) = pending_vec_flip {
            crate::input_dispatch::apply_vec_flip(vec_scene, &self.vec.pen, axis);
        }
        if let Some(dir) = pending_vec_rotate {
            crate::input_dispatch::apply_vec_rotate(vec_scene, &self.vec.pen, dir);
        }
    }
}
