//! ⭐ **QUAL POPOVER ESTÁ ABERTO neste quadro** — irmão do [`crate::state`] pelo teto de 600 LOC.
//!
//! O corte é por **assunto**: os quatro slots respondem a uma pergunta só — *que chip guardou o
//! rect para o passe diferido?* O body-paint guarda; o passe diferido do `paint.rs` consome, e
//! pinta a lista **por cima** de todas as seções.
//!
//! ⚠️ **Sem o passe diferido o `push_clip` do scroll cortaria o popover na borda da seção** — é
//! essa a razão de o slot existir, e é a mesma para os quatro.
//!
//! ⚠️ **O quinto vive noutro sítio de propósito:** o menu da condição de uma seta do Morph mora em
//! [`crate::state_morph_states`], porque *ali* é que as setas vivem. Um slot é do assunto dele.

use ph2d_editor_core::zones::Rect;
use std::cell::Cell;

thread_local! {
    /// Chip rect stashed by the body paint when the font dropdown is open, taken by
    /// the deferred popover pass so the list paints ON TOP of every section.
    static PENDING_FONT_DD: Cell<Option<Rect>> = const { Cell::new(None) };
    /// Idem para o chip de CATEGORIA do catálogo de formas.
    static PENDING_GROUP_DD: Cell<Option<Rect>> = const { Cell::new(None) };
    /// Idem para os chips de PONTA do traço — mas guardando também o SLOT (0 = começo, 1 = fim):
    /// os dois seletores compartilham o passe diferido, e o popover precisa saber de quem ele é.
    /// Só um fica aberto por vez (abrir o outro fecha este, pelo dispatch genérico do dropdown).
    static PENDING_MARKER_DD: Cell<Option<(usize, Rect)>> = const { Cell::new(None) };
    /// A LINHA da pilha de filtros cujo chip de mistura está aberto, + o rect dele.
    static PENDING_BLEND_DD: Cell<Option<(usize, Rect)>> = const { Cell::new(None) };
}

pub(crate) fn set_pending_font_dd(rect: Option<Rect>) {
    PENDING_FONT_DD.with(|c| c.set(rect));
}

pub(crate) fn take_pending_font_dd() -> Option<Rect> {
    PENDING_FONT_DD.with(|c| c.take())
}

pub(crate) fn set_pending_group_dd(rect: Option<Rect>) {
    PENDING_GROUP_DD.with(|c| c.set(rect));
}

pub(crate) fn take_pending_group_dd() -> Option<Rect> {
    PENDING_GROUP_DD.with(|c| c.take())
}

pub(crate) fn set_pending_marker_dd(slot_rect: Option<(usize, Rect)>) {
    PENDING_MARKER_DD.with(|c| c.set(slot_rect));
}

pub(crate) fn take_pending_marker_dd() -> Option<(usize, Rect)> {
    PENDING_MARKER_DD.with(|c| c.take())
}

pub(crate) fn set_pending_blend_dd(row_rect: Option<(usize, Rect)>) {
    PENDING_BLEND_DD.with(|c| c.set(row_rect));
}

pub(crate) fn take_pending_blend_dd() -> Option<(usize, Rect)> {
    PENDING_BLEND_DD.with(|c| c.take())
}
