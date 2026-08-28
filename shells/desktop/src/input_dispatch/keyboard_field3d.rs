//! ⭐ **AS TECLAS DO MODELADOR 3D, numa porta só** (ADR-0161).
//!
//! # Por que elas saíram do despachante
//!
//! Eram **seis** blocos consecutivos com a mesma forma — `Pressed` + `PhysicalKey::Code` + uma
//! pergunta ao módulo + `return` — no meio do despachante de teclado do shell. O corte é do teto de
//! LOC (HR-18), e a fronteira estava desenhada: o módulo irmão de escultura já tinha **uma** porta
//! (`sculpt3d_key`), e este passa a ter a dele.
//!
//! ⚠️ **A ORDEM entre elas é lei, e por isso viajou inteira.** A entrada NUMÉRICA vem antes da tecla
//! de verbo: com um campo de número aberto no meio de um gesto do gizmo, um `5` é um cinco e não um
//! pedido de lente. Reordenar aqui é mudar comportamento, não estilo.
//!
//! ⚠️ E o bloco inteiro corre **antes** do `handler.on_key`, que é o que faz cada uma destas teclas
//! ser inerte fora do módulo — cada `field3d_*_key` responde `false` sem o smoke armado, e a nota do
//! `sculpt3d` ao lado regista o dia em que uma porta destas deixou de perguntar o suficiente e comeu
//! os dígitos de todo painel do app.

use winit::event::ElementState;
use winit::keyboard::PhysicalKey;

use crate::App;

impl App {
    /// **Alguma tecla do modelador 3D consumiu este evento?**
    ///
    /// `false` deixa-o seguir para o store e para o `handler`, que é o caminho de toda a gente.
    pub(super) fn field3d_keys(&mut self, physical_key: PhysicalKey, state: ElementState) -> bool {
        if state != ElementState::Pressed {
            return false;
        }
        let PhysicalKey::Code(code) = physical_key else {
            return false;
        };
        // ADR-0161 W4: `Home` repõe a vista da janela 3D de modelagem — a volta que a
        // rotação LIVRE torna necessária (ela inclina o horizonte, de propósito).
        // Inerte sem o smoke armado; ver a nota de `field3d_home_key` sobre o dia em
        // que isso deixar de ser a única porta.
        if self.field3d_home_key(code) {
            return true;
        }
        // ADR-0161 W26: o NÚMERO digitado no meio de um gesto do gizmo (`G X 0,5`). ⚠️ **Antes da
        // tecla de verbo**, e a ordem é a lei: com uma entrada aberta, um `5` é um cinco. Ela exige
        // uma alça AGARRADA, então só pode disparar com o botão do rato em baixo sobre o gizmo.
        if self.field3d_typed_key(code) {
            return true;
        }
        // ADR-0161 W6: `G`/`R`/`S` trocam o verbo do gizmo 3D (mover/rodar/escalar), as letras
        // do Blender. ⚠️ Só com o ponteiro SOBRE a janela 3D — ver a nota de `field3d_mode_key`:
        // sem essa guarda, três letras comuns deixariam de chegar a qualquer campo de texto.
        if self.field3d_mode_key(code) {
            return true;
        }

        // ADR-0161 W15: `Numpad5` alterna a LENTE da janela 3D (convergente ↔ paralela), a tecla
        // do Blender para a mesma coisa. Mesma guarda de ponteiro das outras — ver `over_window`.
        if self.field3d_lens_key(code) {
            return true;
        }

        // ADR-0161 W47: `Numpad1/3/7` (+ `Ctrl` para a oposta) põem a câmera numa VISTA NOMEADA —
        // frente, trás, direita, esquerda, topo, base. As TECLAS são as do Blender; os EIXOS são os
        // nossos (Y para cima). Mesma guarda de ponteiro das outras.
        if self.field3d_view_key(code) {
            return true;
        }

        // ADR-0161 W90: `Ctrl+Alt+Q` abre e fecha a DIVISÃO do canvas em quatro vistas — a tecla
        // do Blender para o *Toggle Quad View*. Mesma guarda de ponteiro das outras.
        if self.field3d_quad_key(code) {
            return true;
        }

        // ADR-0161 W44: `Shift+I` isola o escolhido — ou devolve a peça inteira. A tecla é a do
        // módulo de escultura, lida e não escolhida. ⭐ É ela a **porta de saída** do isolamento: o
        // chip da fileira desaparece com a raiz escolhida, e sem esta tecla a peça isolada não
        // tinha volta. Mesma guarda de ponteiro das outras.
        if self.field3d_isolate_key(code) {
            return true;
        }
        false
    }
}
