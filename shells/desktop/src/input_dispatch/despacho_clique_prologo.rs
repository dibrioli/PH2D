//! **O clique, o PRÓLOGO** — ramos do `on_mouse_input` ([`super`]), corpos verbatim pela mesma ordem: o arrasto da
//! biblioteca, o aperto que solta o teclado do painel, as janelas 3D e a alça do gizmo de âncora, e o editor de áudio.
//! ⚠️ A soltura das mãos NÃO está aqui: é a primeira coisa do handler, e fica no índice à vista.

use super::*;

impl crate::App {
    /// O editor de áudio: a régua (scrub), o corpo da onda pela ferramenta armada (Select/Move/Scale) e os Up que
    /// largam a peça, o scrub e a selecção.
    #[cfg(feature = "panel-audio-editor")]
    pub(super) fn ramo_editor_audio(&mut self, kind: PointerKind) -> bool {
        // Audio Editor waveform selection (SHELL-only): a primary press INSIDE the
        // overlay waveform starts a selection (cleared to a point); release ends
        // it. Early-return so the press doesn't drive the canvas/gizmo underneath.
        // Presses on the overlay's title-bar / resize handles fall through (they're
        // outside the waveform rect) to the shared BlenderHit dispatch.
        #[cfg(feature = "panel-audio-editor")]
        match kind {
            // Press on the RULER strip → grab the playhead and scrub (seek).
            PointerKind::Down
                if let Some(frame) =
                    self.audio_ruler_frame_at(self.last_pointer.0, self.last_pointer.1) =>
            {
                self.audio_scrub_drag = true;
                if let Some(a) = self.audio.as_mut() {
                    a.editor_scrub_to_frame(frame);
                }
                return true;
            }
            // Press on the WAVE body → what it means depends on the armed tool (the Edit
            // section's toolbar). Select drags a time range, which is what the waveform has
            // always done; Move drags a piece onto another seam; Scale drags a piece's edge.
            PointerKind::Down
                if let Some(hit) =
                    self.audio_wave_frame_at(self.last_pointer.0, self.last_pointer.1) =>
            {
                use ph2d_panel_audio_editor::tool_state::{EditTool, tool};
                let frame = hit.0 as usize;
                match tool() {
                    EditTool::Move => {
                        if let Some(a) = self.audio.as_mut() {
                            a.editor_piece_grab(frame);
                        }
                    }
                    EditTool::Scale => {
                        if let Some(a) = self.audio.as_mut() {
                            a.editor_piece_scale_grab(frame);
                        }
                    }
                    EditTool::Select => {
                        self.audio_sel_drag = Some(hit);
                        if let Some(a) = self.audio.as_mut() {
                            a.editor_clear_selection();
                        }
                    }
                }
                return true;
            }
            // Let go of a piece: THIS is where the reorder / stretch lands, as one undo step.
            PointerKind::Up
                if self
                    .audio
                    .as_ref()
                    .is_some_and(|a| a.editor_piece_drag().is_some()) =>
            {
                if let Some(a) = self.audio.as_mut() {
                    a.editor_piece_release();
                }
                return true;
            }
            PointerKind::Up if self.audio_scrub_drag => {
                self.audio_scrub_drag = false;
                // Hand the playhead back to playback if it's advancing; else the
                // manual position stays where it was dropped.
                if let Some(a) = self.audio.as_mut() {
                    a.editor_end_scrub();
                }
                return true;
            }
            PointerKind::Up if self.audio_sel_drag.take().is_some() => return true,
            _ => {}
        }
        false
    }

    /// A cena de escultura 3D e a janela de modelagem tomam o botão para navegar; a alça do gizmo de âncora toma-o
    /// antes do resto do `Down`.
    pub(super) fn ramo_navegacao_3d_e_ancora(
        &mut self,
        state: ElementState,
        button: MouseButton,
    ) -> bool {
        // ADR-0150 W1/M2: a cena 3D toma o botão para navegar. Inerte (e
        // portanto invisível) sem cena armada.
        #[cfg(feature = "sculpt3d")]
        {
            let taken = match state {
                ElementState::Pressed => self.sculpt3d_pointer_down(button),
                // ⚠️ **Os dois lados do `match` deixaram de ter a mesma FORMA** (W2/L3-A2), e
                // é mensagem, não descuido: o pen-up só precisa da CENA e é função livre; o
                // pen-down arbitra quem fica com o gesto e para isso lê `gfx`, `last_pointer`
                // e `modifiers` — logo continua em `impl App`, com a razão escrita lá.
                ElementState::Released => self
                    .sculpt3d_scene_mut()
                    .is_some_and(ph2d_app_sculpt3d::pointer_up),
            };
            if taken {
                return true;
            }
        }
        // ADR-0161 W4: a janela 3D de modelagem toma o botão para navegar. Inerte
        // (e portanto invisível) sem o smoke armado, e ela só reclama o gesto que
        // começa DENTRO da área que ela desenhou.
        {
            let taken = match state {
                ElementState::Pressed => self.field3d_pointer_down(button),
                ElementState::Released => self.field3d_pointer_up(),
            };
            if taken {
                return true;
            }
        }
        // **§12 — a alça do gizmo de âncora toma o botão** (ADR-0072 §2.3).
        //
        // ⚠️ Antes do resto do `Down`, e com `return`: agarrar uma alça **não** é selecionar um
        // sprite, não é começar um marquee e não é entregar o ponteiro à ferramenta. O
        // `try_open_…` já recusou tudo o que não é canvas (painel por cima, seção fechada, nenhuma
        // linha aberta), então chegar aqui e devolver `true` significa que o gesto é este.
        if state == ElementState::Pressed
            && button == MouseButton::Left
            && let (px, py) = self.last_pointer
            && self.try_open_anchor_gizmo_drag(px, py)
        {
            return true;
        }
        false
    }

    /// Um aperto fora do chrome solta o foco de teclado que um campo do painel segurava — antes de quem toma o aperto.
    pub(super) fn ramo_aperto_solta_teclado(&mut self, state: ElementState) {
        // ⭐⭐⭐ **UM APERTO NO CANVAS SOLTA O TECLADO QUE UM CAMPO DO PAINEL SEGURAVA.**
        //
        // ⛔ **Ele vem ANTES dos três consumidores abaixo, e é aí que está a cura.** Os três
        // — a cena de escultura, a janela de modelagem, a alça do gizmo de âncora — TOMAM o
        // aperto e devolvem `return` antes do `forward_to_hero`, que é o único sítio onde a
        // partida de foco corre. Sem esta linha, tocar num chip numérico de painel e voltar
        // ao canvas deixava `focus_id` preso naquele chip **para o resto da sessão**, e com
        // ele morriam `Delete`, `Ctrl+Z` e todo atalho do módulo que tomou o gesto (Enio,
        // 2026-09-07: *"a tecla del parou de funcionar e não temos undo/redo para Cloth"* —
        // dois relatos, um defeito, nenhum deles do pincel de tecido).
        //
        // ⚠️ **A guarda é a MESMA que os consumidores usam** (`pointer_over_chrome`): um
        // aperto SOBRE o chrome não é um aperto no canvas, e para esse o despachante lá
        // abaixo continua a decidir sozinho — inclusive quando ele cai em espaço morto de
        // painel, que é blur pela lei dele.
        //
        // ⚠️ **E ela não presume que alguém vá consumir**: a porta é idempotente, então
        // quando ninguém toma o gesto o `dispatch_down` a seguir não tem o que refazer.
        // *Condicionar a soltura a QUEM tomou seria uma lista de consumidores a apodrecer no
        // dia em que nasce o quarto.*
        if state == ElementState::Pressed
            && !crate::chrome_hit::pointer_over_chrome(
                self.gfx.as_ref(),
                self.last_pointer.0,
                self.last_pointer.1,
            )
        {
            forward_blur_to_hero(self.gfx.as_mut());
        }
    }

    /// O arrasto de um asset da biblioteca: o `Down` arma sem consumir, o `Up` larga sem consumir.
    pub(super) fn ramo_arrasto_biblioteca(&mut self, state: ElementState, button: MouseButton) {
        // ⭐⭐⭐ **O ARRASTO DA BIBLIOTECA** (plano `docs/Components/07`, etapa B).
        //
        // ⛔⛔ **E ele vem DEPOIS da soltura das mãos, não antes — a 1.ª versão tinha-o antes e o
        // comentário logo acima descreve exactamente o defeito que isso cria:** este handler tem
        // muitos early-returns, e uma mão que sobrevive ao release fica colada ao cursor para
        // sempre. Eu acrescentei um `return` **à frente** da própria linha que existe para o
        // evitar. *Ler a regra não é o mesmo que estar do lado certo dela.*
        //
        // ⚠️ **O `Down` NÃO consome**: enquanto o limiar não for passado isto ainda é um clique, e
        // o clique do cartão tem de chegar ao painel como sempre (ele escolhe; o duplo-clique
        // instancia).
        //
        // ⚠️ **O `Up` consome, e só quando o gesto foi de facto um arrasto.** Sem isso o mesmo
        // gesto largaria o asset na tela **e** contaria como clique no cartão — o `forward_to_hero`
        // que emite o `Click` corre mais abaixo neste mesmo handler.
        if button == MouseButton::Left {
            match state {
                ElementState::Pressed => {
                    let (x, y) = self.last_pointer;
                    self.asset_drag_down(x, y);
                }
                ElementState::Released => {
                    let (x, y) = self.last_pointer;
                    // ⛔⛔ **E ele NÃO consome, e a 1.ª versão consumia.** Um `return` aqui salta o
                    // resto deste handler — e com ele o `held_button = None` e, mais abaixo, o
                    // `forward_to_hero` que é o **único** sítio do app que faz `set_active(None)`.
                    // Consequências medidas na auditoria: o cartão fica preso em `Pressed`, o
                    // widget activo aponta para ele para sempre, e o `post_frame_undo` recusa-se a
                    // registar um passo enquanto `held_button.is_some()` ⇒ **a queda não era
                    // desfazível** até ao clique seguinte.
                    //
                    // ⚠️ **E não há nada a suprimir:** o `Click` que o despachante emite a seguir
                    // cai num cartão, e o `apply_event` do navegador **não tem braço para
                    // `Click(cartão)`** — só para `DoubleClick`. *Suprimir um evento inofensivo
                    // custou quatro fugas de estado.*
                    self.asset_drag_up(x, y);
                }
            }
        }
    }
}
