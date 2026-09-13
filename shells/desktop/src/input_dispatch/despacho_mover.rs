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

    /// O anel do pincel de protecção e os arrastos de pintura (protecção, Falloff, Painter), do Flip (traço, borracha,
    /// Reshape, Colorize, Edit, Trace, gizmos de pose e selecção), dos gizmos de nó (warp, field) e dos cartões (Fill, modais).
    pub(super) fn ramo_mover_pintura_flip_e_modais(&mut self) -> bool {
        // Keep the brush-size ring gizmo following the cursor while the
        // protection brush is armed (published for the on-canvas overlay).
        self.update_protect_brush_cursor(self.last_pointer.0, self.last_pointer.1);
        // BgRemoval protection brush drag (SHELL-only): while a dab is in
        // progress, every motion paints/erases another disc into the keep
        // mask. Early-return so it doesn't also drive a gizmo drag / slider.
        if self.protect_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Painter Falloff add-drag (SHELL-only): while a freshly click-added
        // control point is grabbed, motion drags it. Early-return so it doesn't
        // pan / drive a gizmo. No-ops unless an add-drag is live.
        if self.painter_falloff_drag(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Painter brush stroke (SHELL-only): while a canvas stroke is open, every
        // motion feeds another `CanvasPointer` to the active PainterTool. Early-
        // return so it doesn't also drive a gizmo drag / pan / slider.
        if self.painter_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Flip stroke (ADR-0114 W2, SHELL-only): while a Flip canvas stroke is open,
        // every motion adds a world sample. Early-return so it doesn't also drive a
        // gizmo drag / pan. No-op unless a Flip stroke is in progress.
        if self.flip_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Flip eraser (ADR-0114 W2 T2.9): while an erase gesture is open, every
        // motion erases under the cursor. Early-return like the stroke. No-op
        // unless a Flip erase is in progress.
        if self.flip_erase_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Flip sculpt (ADR-0114 W5): while a reshape gesture is open, every motion is
        // ONE more brush sample (a dose é por amostra — mover devagar aplica mais).
        // No-op unless a sculpt gesture is in progress.
        if self.flip_reshape_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // ADR-0114 C2: enquanto um rabisco do Colorize está aberto, cada movimento é mais
        // uma amostra da polilinha. No-op sem gesto.
        if self.flip_colorize_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Flip Edit Mode (ADR-0114 W6.1): enquanto um gesto de seleção está aberto, cada
        // movimento arrasta a CAIXA do marquee ou TRANSLADA a seleção. No-op sem gesto.
        if self.flip_edit_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Shift & Trace: enquanto um arrasto de trace está aberto, cada movimento
        // desloca (ou gira) a folha do fantasma pego. No-op sem gesto.
        if self.flip_trace_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Flip W7.5: arrasto do gizmo de POSE em curso — cada movimento recomputa a
        // pose da chave (rotate/scale) a partir do snapshot do Down. No-op sem gesto.
        if self.flip_pose_gizmo_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Flip §4.A: arrasto do gizmo de SELEÇÃO em curso — cada movimento recomputa os
        // pontos selecionados (rotate/scale) a partir do snapshot do Down. No-op sem gesto.
        if self.flip_selection_gizmo_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Motion Nodes: arrasto do gizmo de FIELD em curso — cada movimento recomputa o TRS
        // (do snapshot do Down) e escreve os params do NÓ. No-op sem gesto.
        if self.warp_gizmo_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        if self.field_gizmo_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Fill (Bucket) ColorDrop drag (SHELL-only): while a colour is being dragged from the Fill rail
        // button onto the canvas, deliver it to the painter's Fill. Early-return so it doesn't pan.
        if self.fill_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Fill "Fill adjust" modal title-band drag (SHELL-only): while the card is grabbed, motion moves
        // it. Early-return so it doesn't pan / drive a gizmo. No-ops unless a modal drag is armed.
        // ⚠️ A janela do Input Map ANTES do Fill: as duas são cartões flutuantes, e quem está a
        // arrastar um não pode ver o outro reclamar o movimento.
        if self.input_map_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        if self.fill_modal_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Onion settings modal title-band drag (ADR-0142 W3b) — same shape as the Fill modal's.
        if self.onion_modal_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        false
    }
}
