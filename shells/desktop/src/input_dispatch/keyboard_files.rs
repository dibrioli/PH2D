//! **Os acordes de ARQUIVO** — Ctrl+S, Ctrl+O e os dois da escultura.
//!
//! Irmão (`#[path]`-livre, módulo normal) de [`super::keyboard`], e o corte é
//! por RESPONSABILIDADE: aqui mora tudo o que este app responde à pergunta *"que
//! arquivo?"*. Ele nasceu quando o `keyboard.rs` cruzou o cap de 600 LOC do
//! HR-18 — o mesmo motivo e o mesmo precedente do `keyboard_timeline.rs`, que a
//! integração de 2026-07-27 criou quando duas linhas somaram sobre um arquivo
//! que nenhuma das duas cruzava sozinha.
//!
//! ⚠️ **A ORDEM continua sendo do chamador.** Este bloco precede o de clipboard
//! (que também usa Ctrl), e mover a decisão para cá não pode mover o lugar dela
//! na cadeia — por isso a chamada ficou exatamente onde o bloco estava.

use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::App;

impl App {
    /// Consumiu a tecla? `true` = ninguém mais a vê.
    ///
    /// ⚠️ **A ORDEM DOS BRAÇOS é a asserção inteira do arch-gate** que vigia esta
    /// função: um `match` escolhe o PRIMEIRO padrão que casa, e um
    /// `KeyCode::KeyO` sem guarda posto acima deixaria o braço com `if shift`
    /// **inalcançável** — sem o compilador dizer nada, porque ele não prova
    /// cobertura através de uma guarda.
    pub(crate) fn file_chords(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        repeat: bool,
    ) -> bool {
        // Ctrl+S / Ctrl+O — save/load de PROJETO, GLOBAL (qualquer tool). O projeto é
        // mundo + geometria + pixels (`crate::project`). Cede a um campo de texto
        // focado (senão Ctrl+S num nome roubaria o atalho).
        if state == ElementState::Pressed
            && !repeat
            && (self.modifiers.control_key() || self.modifiers.super_key())
            && !self.text_entry_focused()
            && let PhysicalKey::Code(code) = physical_key
        {
            match code {
                // ⚠️ **Ctrl+Shift+S é o `Save As…`**, e o par é o mesmo do `Ctrl+Shift+O` (import
                // de malha) e do `Ctrl+Shift+Z` — o `shift` é PERGUNTADO, senão os dois gestos
                // ficam indistinguíveis, que é acidente e não desenho.
                //
                // ⚠️ E os dois chamam a MESMA função do menu Ficheiro (`crate::project_io`): duas
                // portas para o mesmo gesto é como um `Save` do menu acabaria a gravar noutro
                // sítio que o `Ctrl+S`.
                KeyCode::KeyS => {
                    self.project_save_gesture(self.modifiers.shift_key());
                    return true;
                }
                // ⚠️ **Ctrl+Shift+O importa uma MALHA** (ADR-0150 W8.4). Ele mora
                // aqui, e não no `sculpt3d_key`, por um motivo que decide a
                // feature: aquele handler sai no `sculpt3d_scene_mut()` quando
                // não há cena — e o caso que importa é justamente o de não haver
                // uma, porque **trazer um arquivo é o que ARMA o módulo**.
                //
                // ⚠️ E o `shift` passou a ser PERGUNTADO: até aqui este braço
                // ignorava o modificador, então Ctrl+Shift+O carregava projeto —
                // indistinguível do Ctrl+O, o que é acidente e não desenho. O par
                // é o mesmo do Ctrl+Z/Ctrl+Shift+Z que a própria cena 3D usa.
                #[cfg(feature = "sculpt3d")]
                KeyCode::KeyO if self.modifiers.shift_key() => {
                    self.sculpt3d_pick_and_import();
                    return true;
                }
                // ⚠️ **Ctrl+Shift+E escreve a cena 3D num arquivo** — o par
                // exato do Ctrl+Shift+O acima, e mora ao lado dele pelo mesmo
                // motivo: aqui é onde este app responde *"que arquivo?"*.
                // Ctrl+E sozinho segue livre, então o `shift` não está a
                // desviar nada.
                #[cfg(feature = "sculpt3d")]
                KeyCode::KeyE if self.modifiers.shift_key() => {
                    self.sculpt3d_export();
                    return true;
                }
                // ⚠️ **Abrir PERGUNTA sempre** — é o gesto que deita fora o trabalho não
                // gravado, e fazê-lo com uma tecla e sem pergunta é o modo de falha caro.
                KeyCode::KeyO => {
                    self.project_open_gesture();
                    return true;
                }
                _ => {}
            }
        }
        false
    }
}
