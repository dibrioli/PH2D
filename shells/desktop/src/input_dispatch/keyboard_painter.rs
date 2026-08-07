//! **A cadeia de teclas do Painter que APAGA** — o que o Delete tira, e em que ordem.
//!
//! Há uma tecla só e três donos possíveis, então a precedência **é** a feature: o alvo mais
//! ESPECÍFICO vence. Com um nó selecionado numa curva o Delete tira o nó; sem nó, tira a FIGURA em
//! mãos (Enio, 2026-08-07: *"permita usar del para deletar a forma selecionada por último"*); e o
//! falloff é o dono restante. É a divisão que o Illustrator faz com *duas ferramentas* (seta branca ×
//! seta preta) — aqui é uma tecla, então a ordem é o que a expressa.
//!
//! ⚠️ **A cadeia inteira corta ANTES do hero**, cujo caminho genérico de Delete apaga a **ENTIDADE**:
//! sem o degrau da figura, um Delete com uma figura viva na tela levava o sprite inteiro, com a arte
//! dentro.
//!
//! Os VERBOS moram aqui junto com a cadeia, e não no `painter_canvas_input`: eles não são entrada de
//! CANVAS — são o que uma TECLA faz —, e vê-los ao lado da ordem que os chama é o que torna a
//! precedência auditável de um relance.
//!
//! Cortado de `keyboard.rs` pelo teto de LOC, e por ASSUNTO: os três degraus só significam alguma
//! coisa JUNTOS, e vê-los num arquivo é o que torna a ordem legível.

use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

impl crate::App {
    /// Roda a cadeia; `true` quando algum degrau consumiu a tecla.
    pub(crate) fn painter_delete_chain(
        &mut self,
        state: ElementState,
        physical_key: PhysicalKey,
    ) -> bool {
        // Painter Curve: Delete/Backspace drops the selected control point (kept ≥ 2). Tried BEFORE
        // the Falloff delete — curve editing is on-canvas, falloff editing is in the Brush dock, so
        // only one can have a selection. Consumed only when a curve point was removed.
        if state == ElementState::Pressed
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Delete | KeyCode::Backspace)
            )
            && self.painter_curve_delete_selected_point()
        {
            return true;
        }

        // Painter: Delete/Backspace apaga a FIGURA em mãos — a última selecionada. Depois do delete de
        // ÂNCORA acima (o alvo mais específico vence) e ANTES do falloff + do caminho genérico do hero,
        // que apagaria a ENTIDADE (o sprite inteiro) com uma figura viva na tela.
        if state == ElementState::Pressed
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Delete | KeyCode::Backspace)
            )
            && self.painter_delete_active_shape()
        {
            return true;
        }

        // Painter Falloff curve: Delete/Backspace drops the selected control
        // point. Consumed only when a Falloff point is selected (the helper gates
        // on it), so the key falls through to other tools otherwise.
        if state == ElementState::Pressed
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Delete | KeyCode::Backspace)
            )
            && self.painter_delete_selected_falloff_point()
        {
            return true;
        }
        false
    }

    /// Delete/Backspace: remove the selected Curve control point. Returns `true` (consuming) iff a
    /// point was removed, so the key falls through to the Falloff-point delete / other tools when
    /// no curve point is selected.
    pub(crate) fn painter_curve_delete_selected_point(&mut self) -> bool {
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        // Either curve owner: the stroke Shape curve or the selection Convert-to-Curve editor.
        painter.curve_delete_selected() || painter.selection_curve_delete_selected_point()
    }

    /// **A peça COLADA que flutua: Enter aplica, Esc descarta.** `true` quando havia peça viva — só
    /// então a tecla é consumida, senão o Enter/Esc segue para os donos de sempre.
    ///
    /// ⚠️ **Não há terceira saída, de propósito.** Um clique perdido no canvas não aplica nem cancela
    /// (o roteador de ponteiro o consome sem agir): aplicar num toque acidental viraria tinta, e
    /// cancelar jogaria fora o posicionamento. As duas saídas são teclas porque são DECISÕES.
    pub(crate) fn painter_paste_patch_key(
        &mut self,
        state: ElementState,
        physical_key: PhysicalKey,
        repeat: bool,
    ) -> bool {
        if state != ElementState::Pressed || repeat {
            return false;
        }
        let PhysicalKey::Code(code) = physical_key else {
            return false;
        };
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        match code {
            KeyCode::Enter | KeyCode::NumpadEnter => painter.paste_commit(),
            KeyCode::Escape => painter.paste_cancel(),
            _ => false,
        }
    }

    /// **Os atalhos de área de transferência da SELEÇÃO do Painter** — Ctrl+X / C / V, mais Ctrl+A
    /// (tudo), Ctrl+D (nada) e Ctrl+Shift+I (inverter). `true` quando um deles consumiu a tecla.
    ///
    /// ⚠️ **Gateado em `is_selection_mode`, e é isso que o torna seguro:** Ctrl+A já é *selecionar
    /// todos os nós* do vetor e Ctrl+C/V já são o clipboard do grafo e da timeline. A seleção do
    /// Painter é **modo-exclusiva** (a seção do painel só existe nela), então perguntar pelo MODO é o
    /// que dá a estes atalhos um dono sem tirá-los de ninguém.
    ///
    /// ⚠️ **E ele corre DEPOIS da cadeia do Delete** de propósito: as duas famílias não se cruzam
    /// (aquela é Delete/Backspace, esta é Ctrl+letra), mas a ordem declarada é o que impede a próxima
    /// tecla de nascer ambígua.
    pub(crate) fn painter_selection_clipboard_chain(
        &mut self,
        state: ElementState,
        physical_key: PhysicalKey,
    ) -> bool {
        if state != ElementState::Pressed {
            return false;
        }
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        if !ctrl {
            return false;
        }
        let shift = self.modifiers.shift_key();
        let PhysicalKey::Code(code) = physical_key else {
            return false;
        };
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        if !painter.is_selection_mode() {
            return false;
        }
        match code {
            KeyCode::KeyX => painter.selection_cut(),
            KeyCode::KeyC => painter.selection_copy(),
            KeyCode::KeyV => painter.selection_paste(),
            KeyCode::KeyA => painter.selection_select_all(),
            KeyCode::KeyD => painter.clear_selection(),
            KeyCode::KeyI if shift => painter.invert_selection(),
            _ => return false,
        }
        true
    }

    /// **Delete apaga a FIGURA em mãos** (Enio, 2026-08-07: *"permita usar del para deletar a forma
    /// selecionada por último"*). `true` quando havia uma — só então a tecla é consumida.
    ///
    /// ⚠️ **Roda DEPOIS do delete de âncora**, e a precedência é a convenção do ofício: o alvo mais
    /// ESPECÍFICO vence. Com um nó selecionado numa curva o Delete tira o nó (o `curve_delete_selected`
    /// já se gateia nisso, e recusa quando sobrariam menos de 2 pontos); sem nó selecionado, ele tira a
    /// figura. É a divisão que o Illustrator faz com duas ferramentas — aqui é uma tecla só, então a
    /// ordem é o que a expressa.
    ///
    /// ⚠️ **E ele fecha um buraco que existia antes:** sem esta rota, um Delete com figura viva caía no
    /// caminho genérico do hero e apagava a **ENTIDADE** — o sprite inteiro, com a arte dentro.
    pub(crate) fn painter_delete_active_shape(&mut self) -> bool {
        self.painter_tool_mut()
            .is_some_and(ph2d_tool_painter::PainterTool::delete_active_shape)
    }
}
