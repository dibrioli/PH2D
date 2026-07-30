//! A seção **VERTEX** — o tipo do vértice (Corner/Smooth/Symm) e o Delete Node. Módulo irmão do
//! [`super`] pelo teto de 600 LOC, no mesmo padrão do `paint_effects`.
//!
//! O **estilo de quina** (arredondado vs chanfro) NÃO mora mais aqui: virou o par de ferramentas
//! **Fillet / Chamfer** no rail (clicar-e-arrastar sobre a quina). O toggle Chamfer que ficava
//! nesta seção foi removido junto com a alça de raio do Node — a consolidação que o Enio pediu.

use super::*;

impl BodyCtx<'_> {
    /// Seção **VERTEX** — o tipo do vértice selecionado (Corner / Smooth / Symm) + o Delete Node.
    /// Só existe com um vértice selecionado: sem seleção a seção INTEIRA some (nem cabeçalho), e o
    /// `step` não emite separador.
    pub(crate) fn vertex_section(&mut self, y: f32) -> f32 {
        let Some(sel) = state::current_vertex_type() else {
            return y;
        };
        let active = active_chip(sel);
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_VERTEX,
            tr("panel.vector.section.vertex"),
            y,
        );
        if collapsed {
            return y;
        }
        let verts = [
            (ids::VECTOR_VERT_CORNER, "Corner", VertexType::Corner),
            (ids::VECTOR_VERT_SMOOTH, "Smooth", VertexType::Smooth),
            (ids::VECTOR_VERT_SYMMETRIC, "Symm", VertexType::Symmetric),
        ];
        let vseg_gap = Spacing::Sm.px();
        let vseg_w =
            ((self.inner_w - vseg_gap * (verts.len() as f32 - 1.0)) / verts.len() as f32).max(1.0);
        for (i, (id, label, t)) in verts.iter().enumerate() {
            let rx = self.inner_x + i as f32 * (vseg_w + vseg_gap);
            let rect = Rect::new(rx, y, vseg_w, self.row_h);
            let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            paint_segmented_button(
                rect,
                label,
                active == Some(*t),
                bstate,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(*id, rect);
        }
        y += self.row_h + Spacing::Xs.px();
        // Delete-node button (full width). Insert is a canvas gesture (click a
        // segment) — no button.
        self.action_button(ids::VECTOR_VERT_DELETE, "Delete Node", y)
    }
}

/// **Qual chip a seção acende** — `None` = nenhum.
///
/// ⚠️ Seleção MISTA não acende chip nenhum: acender o do vértice PRIMÁRIO faria o painel afirmar um
/// tipo sobre uma seleção que tem três (auditoria do plano 25, item 5). Os botões continuam **VIVOS**
/// — retipar é justamente o gesto de *tornar* a seleção uniforme, e desligá-los tiraria a única saída
/// do estado misto.
///
/// Função pura, e é o padrão `hit_plan` do repo: a decisão fica testável e o laço de pintura só a
/// honra.
pub(crate) fn active_chip(sel: VertexSel) -> Option<VertexType> {
    match sel {
        VertexSel::Uniform(t) => Some(t),
        VertexSel::Mixed => None,
    }
}

#[cfg(test)]
mod tests {
    use super::active_chip;
    use ph2d_tool_vector::{VertexSel, VertexType};

    /// **Uniforme acende o chip do tipo; MISTO não acende nenhum.**
    ///
    /// Mutação que tem de sangrar: dobrar o `Mixed` no tipo do primário (o código que shipava — o
    /// painel publicava só ele).
    #[test]
    fn a_mixed_selection_lights_no_chip() {
        assert_eq!(
            active_chip(VertexSel::Uniform(VertexType::Smooth)),
            Some(VertexType::Smooth)
        );
        assert_eq!(
            active_chip(VertexSel::Mixed),
            None,
            "selecao MISTA acendeu um chip — o painel esta' a afirmar um tipo sobre uma selecao que \
             tem varios"
        );
    }
}
