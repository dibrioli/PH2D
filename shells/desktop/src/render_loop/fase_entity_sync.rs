//! **Fase do quadro: A SINCRONIZAÇÃO DAS ENTIDADES E AS FORMAS VIVAS** — o `sync` das entidades vectoriais e do Flip, a receita do balde recém-nascido, o
//! `VecShape::Text` da sessão viva e a forma que nasce viva (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_entity_sync(&mut self, vec_cfg: ph2d_tool_vector::VectorDrawConfig) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            vec_scene,
            flip,
            ..
        } = FrameGfx::of(gfx);
        // ADR-0110 — a árvore do editor é a Hierarquia. Reconcilia documento e
        // entidades (path novo ⇒ entidade; entidade apagada ⇒ path), projeta a
        // ordem de z da árvore na pilha, e lê visibilidade/trava herdadas.
        ph2d_vec_entities::entities::sync(sim, vec_scene, &mut self.vec.entities);
        // ⭐⭐⭐ **O BALDE** (plano 40): a entidade do preenchimento acabou de nascer — é agora
        // que a RECEITA (a semente) lhe é presa e que ele vai para o FUNDO. ⚠️ O
        // `insert_path(0, …)` NÃO é o fundo: quem manda no desenho é o `RootOrder` da entidade,
        // e o `sync` dá a toda entidade nova **o maior**.
        if !self.vec.bucket_new.is_empty() {
            crate::vec_bucket::arm_new_fills(sim, &self.vec.entities, &mut self.vec.bucket_new);
        }
        // ADR-0114: idem para os objetos Flip (objeto novo ⇒ entidade; entidade
        // apagada ⇒ objeto). No W0 é no-op (nenhuma tool cria objetos ainda); a
        // tool do W2 passa a populá-lo.
        ph2d_flip_entities::entities::sync(sim, flip, &mut self.flip_state.entities);
        // Live Shapes: mantém o `VecShape::Text` na entidade do texto ativo (a
        // entidade já existe pós-sync) para o objeto lembrar que é texto — re-cook,
        // painel, Convert e save/undo. Idempotente; só com sessão viva.
        if let Some(edit) = self.vec.text_edit.as_ref() {
            crate::vec_text::upsert_text_shape(sim, &self.vec.entities, edit);
        }
        // Live Shapes: a forma recém-desenhada NASCE VIVA — geometria re-cozida
        // centrada (pivô no centro), pose no `Transform`, `VecShape` na entidade.
        // Antes do `settle` (que pula formas vivas). Idempotente.
        crate::vec_shape_live::make_committed_shape_live(
            sim,
            vec_scene,
            &self.vec.entities,
            &mut self.vec.shape,
            // O gesto foi o da ferramenta MOLDURA? A forma nasce igual e ganha o `VecFrame`.
            vec_cfg.mode == ph2d_tool_vector::DrawMode::Frame,
        );
    }
}
