//! A seção **VERTEX** — o tipo do vértice (Corner/Smooth/Symm), o toggle **Chamfer** (o estilo
//! da quina viva, ADR-0121) e o Delete Node. Módulo irmão do [`super`] pelo teto de 600 LOC, no
//! mesmo padrão do `paint_effects`.

use super::*;

impl BodyCtx<'_> {
    /// Seção **VERTEX** — o tipo do vértice selecionado (Corner / Smooth / Symm) + o toggle
    /// Chamfer (quando há quina com raio) + o Delete Node. Só existe com um vértice selecionado:
    /// sem seleção a seção INTEIRA some (nem cabeçalho), e o `step` não emite separador.
    pub(crate) fn vertex_section(&mut self, y: f32) -> f32 {
        let Some(vtype) = state::current_vertex_type() else {
            return y;
        };
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
                vtype == *t,
                bstate,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(*id, rect);
        }
        y += self.row_h + Spacing::Xs.px();
        // **Chamfer** toggle — o ESTILO da quina viva (reta vs arco), oferecido só quando a
        // seleção tem uma quina COM raio (`current_corner_chamfer()` é `Some`). Sem raio não há
        // estilo a mostrar: arrasta-se a alça primeiro (cria o arredondado), depois escolhe-se o
        // chanfro — o modelo do corner widget do Illustrator.
        if let Some(is_chamfer) = state::current_corner_chamfer() {
            let rect = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
            let bstate = self
                .store
                .button_state(ids::VECTOR_VERT_CHAMFER)
                .unwrap_or(ButtonState::Normal);
            paint_segmented_button(
                rect,
                "Chamfer",
                is_chamfer,
                bstate,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(ids::VECTOR_VERT_CHAMFER, rect);
            y += self.row_h + Spacing::Xs.px();
        }
        // Delete-node button (full width). Insert is a canvas gesture (click a
        // segment) — no button.
        self.action_button(ids::VECTOR_VERT_DELETE, "Delete Node", y)
    }
}
