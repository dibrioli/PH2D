//! **A MOLDURA selecionada** — o fato da cena que decide se a seção Frame existe, e o que ela
//! mostra no chip de *Clip content*.
//!
//! Irmão do [`crate::state_bool`], e com a mesma divisão de donos: a verdade mora no ECS
//! (`ph2d_ecs::VecFrame`) e isto é a projeção que a shell publica por frame. O painel não alcança
//! o mundo — e não deve: se alcançasse, haveria duas respostas para *"a seleção é uma moldura?"*.
//!
//! ⚠️ **Eram uma pergunta só, e viraram duas em 2026-08-21.** O `Option<bool>` respondia a
//! *"é moldura?"* e *"recorta?"* de uma vez, o que era exato enquanto só a moldura podia recortar.
//! Com o recorte a valer para qualquer forma FECHADA, um `Some` deixou de implicar moldura — e as
//! duas seções que dependiam dele querem coisas diferentes: a **Clip** aparece sobre uma estrela,
//! a **Frame** (que traz também *Show as Panel* e os presets de dispositivo) não pode.

use std::cell::Cell;

thread_local! {
    static FRAME_CLIP: Cell<Option<bool>> = const { Cell::new(None) };
    static FRAME_PRESENT: Cell<bool> = const { Cell::new(false) };
}

/// Publica o RECORTE da seleção deste frame (shell → painel). `None` = a seleção não oferece o
/// controlo (não há forma fechada que contenha o que está selecionado); `Some(false)` = oferece e
/// está desligado.
pub fn set_frame_clip(clip: Option<bool>) {
    FRAME_CLIP.with(|c| c.set(clip));
}

/// O recorte da seleção — `None` quando a seleção não o oferece. É isto que faz a seção **Clip**
/// ser pintada.
#[must_use]
pub(crate) fn frame_clip() -> Option<bool> {
    FRAME_CLIP.with(Cell::get)
}

/// Publica se a seleção é uma MOLDURA (shell → painel) — o gate da seção **Frame**.
///
/// ⚠️ Separado do [`set_frame_clip`] porque a resposta divergiu: uma elipse que recorta responde
/// `Some(true)` ao recorte e `false` a isto. Colapsá-los de volta ofereceria os presets de
/// dispositivo e o *Show as Panel* sobre uma elipse.
pub fn set_frame_present(present: bool) {
    FRAME_PRESENT.with(|c| c.set(present));
}

/// A seleção é uma moldura?
#[must_use]
pub(crate) fn frame_present() -> bool {
    FRAME_PRESENT.with(Cell::get)
}

thread_local! {
    static FRAME_PANEL_OPEN: Cell<bool> = const { Cell::new(false) };
}

/// Publica se o painel AUTORADO está aberto (shell → painel).
///
/// ⚠️ **É a visibilidade REAL do painel**, lida do `HeroScreen` a cada frame — não uma cópia que o
/// painel guarde. O X do painel autorado e este chip escrevem o MESMO fato, então fechar por um e
/// olhar o outro nunca pode discordar; um bool próprio aqui seria a segunda resposta que fica
/// acesa depois de o artista fechar o painel.
pub fn set_frame_panel_open(open: bool) {
    FRAME_PANEL_OPEN.with(|c| c.set(open));
}

/// O painel autorado está aberto?
#[must_use]
pub(crate) fn frame_panel_open() -> bool {
    FRAME_PANEL_OPEN.with(Cell::get)
}
