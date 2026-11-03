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
use std::cell::Cell;

// CartoesUI C4: os tipos da aparência desceram para a ferramenta — o Inspector em cartões também os lê.
pub use ph2d_tool_vector::appearance::{
    Appearance, PaintRow, RowMixed, close_open_layer, current_appearance, open_layer_index,
    set_current_appearance, set_open_layer, set_paint_move, take_paint_move,
};

thread_local! {
    /// O rect do chip de mistura quando o popover está aberto — o passe diferido do `paint.rs`
    /// consome-o e pinta a lista POR CIMA de todas as seções (mesma lei dos outros quatro slots).
    static PENDING_DD: Cell<Option<Rect>> = const { Cell::new(None) };
    /// O rect do chip de mistura da CAMADA, para o passe diferido.
    static PENDING_PAINT_DD: Cell<Option<Rect>> = const { Cell::new(None) };
}

pub(crate) fn open_layer() -> Option<usize> {
    open_layer_index()
}

/// Que LINHA da lista de modos da CAMADA tem este id.
pub(crate) fn paint_blend_option_index(id: ph2d_a11y::NodeId) -> Option<usize> {
    (0..usize::from(ph2d_vec_scene::MAX_BLEND_MODES))
        .find(|&i| crate::ids::vector_paint_blend_option_id(i) == id)
}

pub(crate) fn set_pending_paint_blend_dd(rect: Option<Rect>) {
    PENDING_PAINT_DD.with(|c| c.set(rect));
}

pub(crate) fn take_pending_paint_blend_dd() -> Option<Rect> {
    PENDING_PAINT_DD.with(Cell::take)
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
