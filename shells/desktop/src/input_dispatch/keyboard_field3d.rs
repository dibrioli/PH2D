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
        // ⛔⛔ **COM A PALETA ABERTA, NENHUMA TECLA DAQUI EXISTE** (W100).
        //
        // ⚠️ **Este roteador corre ANTES da captura modal da paleta** (ver `keyboard.rs`), e a
        // guarda de cada tecla é o **ponteiro sobre a janela 3D** — que continua verdadeira com o
        // modal por cima. ⇒ escrever «capsule» na busca disparava o `S` (escalar), o `A`
        // (reabrir) e o `U`… e as letras nunca chegavam ao campo.
        //
        // ⚠️ **A família é MAIOR do que a tecla que a revelou, e é PRÉ-EXISTENTE:** `G`/`R`/`S`,
        // `I`, `Q` e `Home` já eram comidos com o `Ctrl+K` ou a biblioteca do Motion abertos sobre
        // o módulo armado. Curar **na entrada do roteador** apanha as seis e a próxima — um remendo
        // dentro do `field3d_add_key` curaria só a que se viu.
        //
        // ⚠️ A tecla que ABRE não é afetada: ela é capturada num quadro em que a paleta ainda não
        // existe (o pedido atravessa por caixa de correio e a paleta abre na ponte, no quadro
        // seguinte) — a mesma nota que o `keyboard.rs` já tinha escrito para o `A` do Motion.
        if self.field3d_yields_to_modal() {
            return false;
        }
        // ADR-0161 W4: `Home` repõe a vista da janela 3D de modelagem — a volta que a
        // rotação LIVRE torna necessária (ela inclina o horizonte, de propósito).
        // ADR-0161 W109: com o menu do cabeçalho aberto, `Escape` é dele. ⚠️ **PRIMEIRO no
        // roteador**, e a ordem é a mesma lei do `field3d_typed_key`: enquanto um popup está aberto,
        // a tecla de desistir pertence-lhe. Ele devolve `false` sem menu, logo é inerte no resto do
        // tempo.
        if self.field3d_view_menu_key(code) {
            return true;
        }
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

        // ADR-0161 W100: `A` abre a PALETA DE FORMAS — a mesma tecla e o mesmo widget da biblioteca
        // de nós do Motion. Mesma guarda de ponteiro das outras: uma letra solta sem ela roubaria
        // todo `a` digitado num campo de texto.
        //
        // ⚠️ **Por último no roteador**, e a ordem é a mesma lei do `field3d_typed_key`: com uma
        // entrada numérica aberta no meio de um gesto do gizmo, a tecla pertence a ela.
        if self.field3d_add_key(code) {
            return true;
        }
        false
    }
}
