//! ⭐⭐⭐ **A APARÊNCIA DO OBJECTO publicada pela shell** — irmão do [`crate::state`] pelo teto de
//! 600 LOC.
//!
//! O que a forma SELECIONADA é, nas duas propriedades que descrevem o objecto inteiro e não uma
//! tinta dele: a **opacidade** e o **modo de mistura** (estudo 42 item 2, v19 do schema).
//!
//! ⚠️ **`None` esconde a seção**, e é a mesma lei das outras seções de forma: sem uma forma
//! selecionada não há sujeito, e uma seção com sliders que não descrevem nada é pior que uma seção
//! ausente — o artista arrasta e nada acontece.

use ph2d_editor_core::zones::Rect;
use ph2d_vec_scene::BlendMode;
use std::cell::{Cell, RefCell};

/// O que a forma selecionada tem, hoje.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Appearance {
    /// `0..=1`, `1` = opaca.
    pub opacity: f32,
    /// O modo de mistura do documento.
    ///
    /// ⚠️ **É o MODO, não o índice na lista do dropdown.** A lista é derivada da tradução para o
    /// Vello (`ph2d_vec_render::blend::offered`) e os dois lados — o painel que a pinta e a shell
    /// que recebe o clique — chamam a MESMA função. Publicar um índice faria a shell ter de
    /// reconstruir a lista para o traduzir, e uma segunda cópia dela é como as duas passam a
    /// discordar sobre o que a linha 7 significa.
    pub blend: BlendMode,
}

thread_local! {
    static CURRENT: RefCell<Option<Appearance>> = const { RefCell::new(None) };
    /// O rect do chip de mistura quando o popover está aberto — o passe diferido do `paint.rs`
    /// consome-o e pinta a lista POR CIMA de todas as seções (mesma lei dos outros quatro slots).
    static PENDING_DD: Cell<Option<Rect>> = const { Cell::new(None) };
}

/// Publica a aparência da forma selecionada; `None` esconde a seção.
pub fn set_current_appearance(a: Option<Appearance>) {
    CURRENT.with(|c| *c.borrow_mut() = a);
}

pub(crate) fn current_appearance() -> Option<Appearance> {
    CURRENT.with(|c| *c.borrow())
}

/// Que LINHA da lista de modos tem este id (`None` se não for uma opção do popover).
///
/// ⚠️ Varre o espaço FIXO de ids (`MAX_BLEND_MODES`), como as outras fábricas deste painel: a
/// resolução não pode depender de quantos modos a lista oferece HOJE, senão um modo novo traria
/// um id que ninguém reconhece.
pub(crate) fn blend_option_index(id: ph2d_a11y::NodeId) -> Option<usize> {
    (0..usize::from(ph2d_vec_scene::MAX_BLEND_MODES))
        .find(|&i| crate::ids::vector_obj_blend_option_id(i) == id)
}

pub(crate) fn set_pending_obj_blend_dd(rect: Option<Rect>) {
    PENDING_DD.with(|c| c.set(rect));
}

pub(crate) fn take_pending_obj_blend_dd() -> Option<Rect> {
    PENDING_DD.with(Cell::take)
}
