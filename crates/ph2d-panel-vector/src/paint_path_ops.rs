//! **A seção PATH** do painel de Vector — remodelar o caminho selecionado inteiro.
//!
//! ⚠️ Saiu de `paint_arrange.rs` em 2026-09-16, pelo mesmo corte da `paint_align` (tecto de LOC
//! depois da migração dos rótulos). É uma secção própria, com cabeçalho próprio.

use crate::paint_sections::BodyCtx;
use crate::state;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;

impl BodyCtx<'_> {
    /// Seção **PATH** — remodela o path selecionado inteiro. Smooth / Sharpen numa linha
    /// de 2 colunas; Simplify (menos pontos) / Subdivide (mais pontos) na segunda; o
    /// toggle Close/Open (rótulo vindo do `closed` publicado); e, quando a seleção tem
    /// uma forma VIVA (paramétrica / texto), o **Convert to Curves** — que é justamente
    /// o que PRODUZ um path cru editável, e por isso mora aqui.
    pub(crate) fn path_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ph2d_tool_vector::ids::VECTOR_SECTION_PATH,
            tr("panel.vector.section.path"),
            y,
        );
        if collapsed {
            return y;
        }
        let gap = Spacing::Xs.px();
        let w = ((self.inner_w - gap) / 2.0).max(1.0);
        if state::convertible() {
            y = self.action_button(
                crate::ids::VECTOR_CONVERT_TO_CURVES,
                tr("panel.vector.arrange.convert_to_curves"),
                y,
            );
        }
        y = self.row2(
            w,
            gap,
            [
                (
                    ph2d_tool_vector::ids::VECTOR_PATH_SMOOTH,
                    tr("panel.vector.arrange.smooth"),
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_PATH_SHARPEN,
                    tr("panel.vector.arrange.sharpen"),
                ),
            ],
            y,
        );
        y = self.row2(
            w,
            gap,
            [
                (
                    ph2d_tool_vector::ids::VECTOR_PATH_SIMPLIFY,
                    tr("panel.vector.arrange.simplify"),
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_PATH_SUBDIVIDE,
                    tr("panel.vector.arrange.subdivide"),
                ),
            ],
            y,
        );
        // Close/Open toggle — label reflects the current state (default "Close"
        // when no path is selected / not yet closed).
        let label = if state::current_path_closed() == Some(true) {
            tr("panel.vector.arrange.open_path")
        } else {
            tr("panel.vector.arrange.close_path")
        };
        y = self.action_button(crate::ids::VECTOR_PATH_CLOSE, label, y);
        // **Reverse** (W4) — o sentido do caminho é um fato autorado, e até aqui não havia gesto
        // nenhum que o mudasse: o `reverse_path` existia com UM chamador interno e nenhum id.
        y = self.action_button(
            crate::ids::VECTOR_PATH_REVERSE,
            tr("panel.vector.arrange.reverse"),
            y,
        );
        // **Join** — ⚠️ só com 2+ selecionados. Com um caminho só a resposta é o `Close Path` logo
        // acima; oferecer o Join ali seria um segundo botão para a mesma pergunta, e ele
        // devolveria `false` em silêncio (a lei do botão morto).
        if state::current_selection_count() >= 2 {
            y = self.action_button(
                crate::ids::VECTOR_PATH_JOIN,
                tr("panel.vector.arrange.join"),
                y,
            );
        }
        // ⭐⭐⭐ **Soldar** (plano 39, ideia do Enio) — colado no *Join* porque são o mesmo verbo em
        // dois sítios: aquele solda **duas pontas**, este solda **os cruzamentos**. Lidos juntos,
        // ensinam a diferença sem uma linha de ajuda.
        //
        // ⚠️ **Sem o piso de 2**, ao contrário do *Join*: um caminho sozinho pode ter
        // AUTO-cruzamento, e ali soldar tem o que fazer.
        if state::current_selection_count() >= 1 {
            y = self.action_button(
                crate::ids::VECTOR_PATH_WELD,
                tr("panel.vector.arrange.weld"),
                y,
            );
        }
        y
    }
}
