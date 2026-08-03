//! **RESIZE BOX** — o que a alça do gizmo faz ao objeto selecionado (plano UI/UX W3b).
//!
//! ⚠️ A verdade mora no ECS (`ph2d_ecs::resizes_box`, que já compõe o default derivado com o
//! override do artista) e isto é a projeção que a shell publica por frame. O painel não alcança o
//! mundo — e não deve: se alcançasse, a resposta que DESENHA a caixa marcada divergiria da que
//! HONRA o arrasto, e o artista descobriria a divergência esticando os filhos.
//!
//! ⚠️ **O `Option` é a metade que importa.** `None` = a seleção não tem uma resposta *(nada
//! selecionado, ou uma seleção múltipla)* e a linha **não é pintada**: um checkbox que descreve
//! um objeto que não está lá é pior que checkbox nenhum.

use std::cell::Cell;

thread_local! {
    /// `Some(marcado)` para uma seleção com resposta; `None` quando não há o que descrever.
    static RESIZE_BOX: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Publica a resposta deste frame (shell → painel).
pub fn set_resize_box(v: Option<bool>) {
    RESIZE_BOX.with(|c| c.set(v));
}

/// A caixa está marcada? `None` = a linha não é pintada.
#[must_use]
pub(crate) fn resize_box() -> Option<bool> {
    RESIZE_BOX.with(Cell::get)
}
