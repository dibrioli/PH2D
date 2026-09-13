//! **O movimento do cursor, os ARRASTOS em curso** — ramos do `on_cursor_moved` ([`super`]), corpos verbatim pela mesma
//! ordem: cada arrasto vivo é dono do ponteiro até ao Up. ⚠️ Os blocos ficam à MESMA indentação: as agulhas de coluna
//! dos gates (o fecho do `if` da preview) casam no texto emendado.

use super::*;

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
        if self.collider_gizmo_move(self.last_pointer.0, self.last_pointer.1) {
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

    /// O topo dos arrastos: a borda da coluna, a biblioteca, o pie menu, o cursor do conta-gotas, as guias, o hover da
    /// preview (sem consumir), o conta-gotas arrastado, as órbitas 3D e os arrastos do editor de áudio.
    pub(super) fn ramo_mover_arrastos_de_topo(&mut self) -> bool {
        // ⚠️ **A ORDEM destes dois foi decidida na integracao de 2026-09-04**, e nao
        // e' arbitraria: o da BORDA tem `return` e uma guarda estreita (so' responde com
        // `dock_seam_drag` armado), logo so' toma o quadro quando ha' um redimensionamento
        // a serio -- e nesse quadro nao pode existir arrasto de biblioteca. Invertidos, o
        // da biblioteca correria em todo quadro de um resize.
        // ⭐ **O arrasto da BORDA de uma coluna** — dono do ponteiro até o Up, como todo arrasto
        // desta shell. Vem cedo pelo mesmo motivo que o Down dele.
        if self.dock_seam_move(self.last_pointer.0) {
            return true;
        }

        // ⭐ **O arrasto da biblioteca anda AQUI**, e não no `on_mouse_input` — é a mesma doutrina
        // das guias logo abaixo: um arrasto em curso responde ao movimento, não ao botão.
        {
            let (x, y) = self.last_pointer;
            self.asset_drag_move(x, y);
        }
        // ⭐ **O PIE MENU acende pela DIRECÇÃO** (estudo de UI viva, E4) — aqui, no movimento, e não
        // no frame: o menu tem de responder ao gesto em curso, e um acender que espera o quadro
        // seguinte é um menu que a mão sente como pesado. No-op sem menu aberto.
        self.radial_point();
        // Reflect the colour-picker eyedropper in the OS cursor (a crosshair "target" while armed).
        self.update_eyedropper_cursor();
        // **AS GUIAS** (plano 25 §9, a W6.2): um arrasto de guia em curso é DONO do ponteiro,
        // então ele vem antes de todo o resto — a mesma doutrina dos `*_move` abaixo.
        //
        // ⚠️ **É AQUI que um arrasto de guia anda, e não no `on_mouse_input`.** A primeira
        // versão desta wave pôs o braço `PointerKind::Move` junto do Down/Up, num handler que
        // só produz Down e Up: o braço era **inalcançável**, `guide_pointer_move` ficou sem
        // chamador nenhum, e o produto criava a guia e a deixava onde nasceu — o que se lê
        // como *"criar funciona, mover não"*, dois sintomas de um defeito só.
        if self.guide_pointer_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // **O MODO DE PREVIEW** (plano UI/UX W7r): o cursor decide que hospedeiro está aceso.
        //
        // ⚠️ **Ele NÃO consome, e a assimetria com o Down/Up é deliberada.** Um `Down` primário
        // abriria um arrasto de edição e por isso é da preview; um movimento não abre nada, e
        // consumi-lo mataria o pan e o zoom — que o Figma mantém vivos no modo de apresentação
        // dele, pela mesma razão: olhar de perto não é editar.
        //
        // ⚠️ E ele corre **antes** dos `*_move` abaixo de propósito: um arrasto de gizmo não pode
        // existir aqui dentro (o `Down` que o abriria foi consumido), então nada a jusante tem
        // opinião sobre este movimento — mas se um dia tiver, o hover da UI é quem manda.
        if self.ui_preview.is_on() {
            // ⚠️ **`Primary`, e não `is_some()`**: o `held_button` guarda QUALQUER botão entre o
            // Down e o Up, então um pan de botão do meio sobre um controle o mostraria `Pressed` —
            // o papel errado por um gesto que nem é dele.
            let pressed = self.held_button == Some(ph2d_host::PointerButton::Primary);
            self.ui_preview_point(self.last_pointer.0, self.last_pointer.1, pressed);
        }
        // BgRemoval eyedropper drag (SHELL-only): while the primary
        // button is held with the eyedropper armed, every motion
        // samples another colour. Early-return so the move does not
        // also drive a gizmo drag / panel slider.
        if self.eyedropper_dragging {
            self.try_eyedropper_sample(self.last_pointer.0, self.last_pointer.1);
            return true;
        }
        // ADR-0150 W1/M2: a órbita da cena 3D. Só consome com um arrasto EM
        // CURSO — a porta devolve `false` sem cena armada e sem botão preso, e
        // é por isso que ela não rouba o hover do app 2D.
        // ⭐ A shell procura a cena; a lei do gesto é função livre da família (W2/L3-A2).
        // ⚠️ **O `false` sem cena armada continua a ser a resposta** — ele mudou de sítio
        // (era o `else` do `let Some` lá dentro), não de valor: sem cena esta porta não rouba
        // o hover do app 2D, e é essa a promessa que a linha de cima descreve.
        #[cfg(feature = "sculpt3d")]
        {
            let (px, py) = self.last_pointer;
            if self
                .sculpt3d_scene_mut()
                .is_some_and(|scene| ph2d_app_sculpt3d::pointer_move(scene, px, py))
            {
                return true;
            }
        }
        // ADR-0161 W4: a órbita da janela 3D de MODELAGEM (irmã da de cima, e com
        // a mesma lei: só consome com um arrasto EM CURSO).
        if self.field3d_pointer_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Audio Editor piece drag (SHELL-only): Move / Scale own the pointer while they are live.
        #[cfg(feature = "panel-audio-editor")]
        if self.audio_piece_drag_move(self.last_pointer.0) {
            return true;
        }
        // Audio Editor waveform selection drag (SHELL-only): while a selection is
        // being dragged over the overlay waveform, every motion extends it. Early-
        // return so it doesn't also pan / drive a gizmo.
        #[cfg(feature = "panel-audio-editor")]
        if self.audio_sel_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return true;
        }
        // Audio Editor playhead scrub drag (SHELL-only): dragging the time ruler seeks
        // the preview. Early-return so it doesn't also pan.
        #[cfg(feature = "panel-audio-editor")]
        if self.audio_scrub_move(self.last_pointer.0) {
            return true;
        }
        false
    }
}
