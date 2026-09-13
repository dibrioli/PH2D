//! **O movimento do cursor, os ARRASTOS em curso** — ramos do `on_cursor_moved` ([`super`]): cada arrasto vivo é
//! dono do ponteiro até ao Up e devolve cedo; sem arrasto, cada porta é no-op. Os corpos MUDARAM-SE verbatim
//! (`line/input-dispatch`, 2026-09-13) e correm no sítio da chamada, pela mesma ordem.
//!
//! ⚠️ Os blocos eram de nível de topo do handler, e continuam à MESMA indentação dentro dos ramos: as agulhas
//! de coluna que os gates lêem (o fecho do `if` da preview) casam no texto emendado como casavam no ficheiro.

impl crate::App {
    /// Os arrastos da ferramenta vetorial: a região, o Build, o osso posado, a gaiola, a caneta, o gradiente, a forma,
    /// o lápis, o Width, o conector e as alças (conector, texto, fichas), e a âncora do motion path.
    pub(super) fn ramo_mover_vetor(&mut self) -> bool {
        // ADR-0108 Fase 1: o gesto de REGIÃO do modo Node — o canto vivo segue, e o LAÇO grava
        // mais um ponto se andou o bastante. Early-return para não panar / desenhar. No-op parado.
        if let Some(m) = self.vec.marquee.as_mut() {
            m.advance(self.last_pointer);
            return true;
        }
        // Shape Builder: o realce segue o cursor mesmo SEM botão apertado (é o que
        // deixa o artista ver as regiões antes de escolher uma), e com o botão ele PINTA.
        if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Build
            && let Some(w) = self.vec_world_at(self.last_pointer)
            && self.build_move(w)
        {
            return true;
        }
        // ⭐⭐⭐ **POSAR um osso** (estudo 42 item 5): a mesma disciplina de early-return dos irmãos,
        // e no-op sem osso agarrado.
        if self.vec_bone_pose_move() {
            return true;
        }
        // ADR-0129 Fatia 1: arrastar um canto da gaiola do Envelope (modo Node).
        // Mesma disciplina de early-return do pen; no-op sem um canto agarrado.
        if self.vec_envelope_corner_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // ADR-0108 Fase 1.2: Pen NOVO — arrastar após a âncora puxa os handles
        // Bézier (simétricos). Early-return: não pan/gizmo. No-op sem drag ativo.
        if self.vec_pen_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // ADR-0111: não há gizmo vetorial próprio. O gizmo de sprite move o
        // `Transform` da entidade do path, pelo mesmo caminho de qualquer objeto.
        // Gradient group 3b: dragging a multi-point gradient handle. Same
        // early-return discipline; no-op unless a grad drag is live.
        if self.vec_grad_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // ADR-0108 Fase 1: shape drag-to-size (Rectangle/Ellipse/Polygon). Same
        // early-return discipline as the pen; no-op unless a shape drag is live.
        if self.vec_shape_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // **O LÁPIS**: a mão livre acumula amostras e re-ajusta a curva. Mesma disciplina.
        if self.vec_pencil_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // **O Width Tool**: a alça de largura agarrada segue o cursor. Mesma disciplina.
        if self.vec_width_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Conector (modo Connect): a 2ª ponta segue o cursor e GRUDA na forma sob ele — o
        // que se vê é o conector de verdade, re-cozido pela mesma `route`. Mesma disciplina
        // de early-return; no-op sem gesto vivo.
        if self.connector_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Alça de ponta (modo Select): a ponta agarrada segue o cursor. Durante o arrasto ela é
        // uma ponta SOLTA no ponteiro, e o re-cook do frame já desenha a linha inteira seguindo
        // a mão — não há caminho de preview separado, o preview é o conector.
        if self.conn_handle_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // W5: arrastar a alça do texto em caminho (modo Select) — irmã da alça do conector
        // acima, mesma disciplina de early-return; no-op sem a alça agarrada.
        if self.vec_textpath_handle_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // W4 do pattern: arrastar a ficha de Start/End (modo Select) — irmã da do texto.
        if self.vec_patternpath_handle_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // A ÂNCORA do motion path (ADR-0141): leva a trajetória para o cursor. Irmã das
        // alças acima na disciplina, e sem gate de ferramenta — a trajetória é do
        // documento de animação, não de uma tool.
        if self.motion_path_anchor_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        false
    }
}
