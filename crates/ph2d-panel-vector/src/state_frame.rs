//! **A MOLDURA selecionada** — o fato da cena que decide se a seção Frame existe, e o que ela
//! mostra no chip de *Clip content*.
//!
//! Irmão do [`crate::state_bool`], e com a mesma divisão de donos: a verdade mora no ECS
//! (`ph2d_ecs::VecFrame`) e isto é a projeção que a shell publica por frame. O painel não alcança
//! o mundo — e não deve: se alcançasse, haveria duas respostas para *"a seleção é uma moldura?"*.
//!
//! ⚠️ Um único `Option<bool>` responde às DUAS perguntas, e é de propósito: `None` = a seleção não
//! é moldura (a seção não é pintada), `Some(c)` = é, e recorta ou não. Dois flags separados
//! admitiriam o estado *"não é moldura mas recorta"*, que não quer dizer nada — e é o estado em
//! que um deles fica quando alguém esquece de limpar o outro.

use std::cell::Cell;

thread_local! {
    static FRAME_CLIP: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Publica a moldura da seleção deste frame (shell → painel). `None` = a seleção não é moldura.
pub fn set_frame_clip(clip: Option<bool>) {
    FRAME_CLIP.with(|c| c.set(clip));
}

/// A moldura da seleção — `None` quando não há. É isto que faz a seção Frame ser oferecida.
#[must_use]
pub(crate) fn frame_clip() -> Option<bool> {
    FRAME_CLIP.with(Cell::get)
}
