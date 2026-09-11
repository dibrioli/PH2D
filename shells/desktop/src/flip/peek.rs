//! **O PEEK do Shift & Trace** (fatia 2 — `docs/Flip/04 §4`, OpenToonz): o flip de papel.
//!
//! Com a tool Flip ativa, **segurar F1/F2/F3** mostra só o desenho **anterior / atual /
//! seguinte** da camada ativa — sem mover o playhead, sem fantasma nenhum na frente
//! (é o animador levantando uma folha do lightbox para conferir o arco); **soltar volta**.
//!
//! Duas metades, cada uma pura e testável:
//!
//! - [`key_transition`] — a política da tecla. Press só ARMA com a tool Flip ativa;
//!   **release SEMPRE desarma** (trocar de tool com a tecla presa não pode deixar o peek
//!   preso — a tecla que armou é a única que desarma, e desarma incondicional).
//! - [`peek_frame`] — o retime. A âncora é a **chave ATIVA** (`active_key`), não o quadro
//!   cru: no meio de um hold, `prev_drawing_key(quadro)` devolve o INÍCIO da exposição
//!   atual — o MESMO desenho que já está na tela, e um peek que mostra o que já se vê
//!   não é um peek. Sem vizinho (primeira/última chave) fica onde está — folhear para
//!   uma folha que não existe é ficar na que se tem.
//!
//! Quem apaga os FANTASMAS durante o peek é o shell (`present.rs` passa `ghosts: None`):
//! o flip é uma folha na mão, não uma pilha translúcida.

use ph2d_flip::{FlipLayer, Frame};
use winit::keyboard::KeyCode;

/// Que folha o peek mostra.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum PeekDir {
    /// F1 — o desenho ANTERIOR.
    Prev,
    /// F2 — o desenho ATUAL, sozinho (sem fantasmas): julgar a pose limpa.
    Here,
    /// F3 — o desenho SEGUINTE.
    Next,
}

/// A política da tecla: `(peek novo, a tecla foi consumida?)`.
///
/// Consumir importa nas duas pontas: um F1 armado não pode cair nos atalhos de editor
/// (viraria dois significados para um aperto), e um F5 solto não pode ser engolido
/// (não é nosso).
#[must_use]
pub(crate) fn key_transition(
    cur: Option<PeekDir>,
    code: KeyCode,
    pressed: bool,
    flip_active: bool,
) -> (Option<PeekDir>, bool) {
    let dir = match code {
        KeyCode::F1 => PeekDir::Prev,
        KeyCode::F2 => PeekDir::Here,
        KeyCode::F3 => PeekDir::Next,
        _ => return (cur, false),
    };
    if pressed {
        if flip_active {
            (Some(dir), true)
        } else {
            (cur, false) // a tecla não é nossa fora da tool Flip
        }
    } else if cur == Some(dir) {
        // Release desarma SEMPRE — mesmo com a tool trocada no meio (senão o peek
        // ficaria preso mostrando uma folha que nada mais explica).
        (None, true)
    } else {
        (cur, false)
    }
}

/// O quadro que a camada ATIVA amostra sob o peek — o quadro dado, quando não há para
/// onde folhear (`Here`, primeira/última chave, camada vazia).
#[must_use]
pub(crate) fn peek_frame(layer: &FlipLayer, frame: Frame, dir: PeekDir) -> Frame {
    if dir == PeekDir::Here {
        return frame;
    }
    let src = layer.source_frame(frame);
    let Some(anchor) = layer.active_key(src) else {
        return frame; // antes da 1ª chave não há folha na mão
    };
    let key = match dir {
        PeekDir::Prev => layer.prev_drawing_key(anchor),
        PeekDir::Next => layer.next_drawing_key(anchor),
        PeekDir::Here => unreachable!("tratado acima"),
    };
    key.unwrap_or(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 🔴 **Press arma (só com a tool Flip), release desarma — e a de OUTRO dedo não.**
    #[test]
    fn press_arms_and_release_disarms_only_its_own_key() {
        assert_eq!(
            key_transition(None, KeyCode::F1, true, true),
            (Some(PeekDir::Prev), true)
        );
        assert_eq!(
            key_transition(None, KeyCode::F2, true, true),
            (Some(PeekDir::Here), true)
        );
        assert_eq!(
            key_transition(None, KeyCode::F3, true, true),
            (Some(PeekDir::Next), true)
        );
        // Soltar a tecla que armou desarma…
        assert_eq!(
            key_transition(Some(PeekDir::Prev), KeyCode::F1, false, true),
            (None, true)
        );
        // …soltar OUTRA não mexe (F1 preso, F3 solto: o peek segue no Prev).
        assert_eq!(
            key_transition(Some(PeekDir::Prev), KeyCode::F3, false, true),
            (Some(PeekDir::Prev), false)
        );
    }

    /// 🔴 **Fora da tool Flip a tecla não é nossa** — F1 num app sem Flip ativo tem de
    /// seguir o caminho de sempre (não consumida, peek intacto).
    #[test]
    fn outside_the_flip_tool_the_key_is_not_ours() {
        assert_eq!(
            key_transition(None, KeyCode::F1, true, false),
            (None, false)
        );
        // Tecla não-F: nunca nossa, com ou sem Flip.
        assert_eq!(
            key_transition(None, KeyCode::KeyA, true, true),
            (None, false)
        );
    }

    /// 🔴 **O release desarma MESMO com a tool trocada no meio** — senão trocar de tool
    /// com F1 preso deixaria o peek armado para sempre, mostrando uma folha que nada
    /// mais explica.
    #[test]
    fn a_release_disarms_even_after_the_tool_switched_away() {
        assert_eq!(
            key_transition(Some(PeekDir::Next), KeyCode::F3, false, false),
            (None, true)
        );
    }
}
