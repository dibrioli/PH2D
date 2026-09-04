//! ⭐⭐ **AS TECLAS DO PALETTE DE NÓS** — irmão dos outros `keyboard_*`, e pela mesma lei:
//! uma FAMÍLIA de teclas vive numa porta própria e o `key_input` chama-a.
//!
//! ⚠️ **O corte foi obrigado pela INTEGRAÇÃO de 2026-09-04** — duas linhas acrescentaram
//! ramos ao `key_input` no mesmo dia (as teclas da Hierarquia e as do redesenho) e o
//! ficheiro passou a `602 / 600`. ⛔ Nenhuma delas o via sozinha; *um tecto de LOC é a
//! única coisa deste repo que só a árvore COMBINADA acusa*.
//!
//! ⭐ Escolheu-se ESTE bloco por ser o mais auto-contido do corpo: ele é **modal** (engole
//! a tecla inteira, press e release) e não partilha estado com nenhum ramo vizinho.

use crate::App;
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

impl App {
    /// `true` ⇒ o palette consumiu a tecla e o `key_input` devolve JÁ.
    pub(crate) fn command_palette_keys(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        text: Option<&str>,
    ) -> bool {
        // O palette de "Add Node" (tela cheia, Motion) é MODAL — enquanto aberto ele COME toda tecla:
        // caracteres imprimíveis vão pro campo de busca, Enter escolhe o topo do filtro, Backspace apaga,
        // Escape fecha. Vem PRIMEIRO (antes dos atalhos de painel/ferramenta) para uma letra digitada nunca
        // vazar num atalho de grafo embaixo. O `A` que ABRIU o palette foi capturado no quadro ANTERIOR
        // pelo painel (o palette só abre no quadro seguinte, na ponte), então a tecla de abertura flui normal.
        if !self.command_palette_open() {
            return false;
        }
        if state == ElementState::Pressed {
            if let PhysicalKey::Code(code) = physical_key {
                match code {
                    KeyCode::Escape => {
                        self.command_palette_close();
                        return true;
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        self.command_palette_confirm();
                        return true;
                    }
                    KeyCode::Backspace => {
                        self.command_palette_backspace();
                        return true;
                    }
                    _ => {}
                }
            }
            if let Some(s) = text {
                for ch in s.chars() {
                    self.command_palette_type(ch);
                }
            }
        }
        // Modal: engole TODA a tecla (press e release), aberto ou não haja o que digitar.
        true
    }
}
