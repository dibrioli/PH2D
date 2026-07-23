//! **O canal de REQUESTS shell→paint** — pedidos one-shot que o shell levanta e o
//! próximo paint consome (fit / reveal / aba Keys).
//!
//! Split de `state.rs` quando ele cruzou o cap de 600 LOC do painel (HR-18), e uma
//! unidade por direito próprio: tudo aqui é a mesma dança — um `Cell<bool>` armado
//! por um `request_*` público e drenado por um `take_*` do paint. Os statics moram
//! aqui COM os acessores (um bloco `thread_local!` próprio), então o par armar/drenar
//! de cada pedido não pode se separar do estado que ele toca.

use std::cell::Cell;

thread_local! {
    /// A pending "fit the view to the keys" request (`F` over the panel). The
    /// shortcut is read by the shell's keyboard handler, but the view transform
    /// it fits is panel state — so the shell raises the request here and `paint`
    /// consumes it once the time area's pixel width is known.
    static FIT_REQUESTED: Cell<bool> = const { Cell::new(false) };
    /// A pending "pan the time axis until the playhead is visible" request,
    /// raised by the shell after a transport command that jumps the playhead
    /// (go-to-start/end, a frame step, a typed time). Consumed by `paint`, which
    /// alone knows the visible span.
    static REVEAL_REQUESTED: Cell<bool> = const { Cell::new(false) };
    /// A pending "switch to the Keys tab" request — raised by the shell when a NEW
    /// object becomes selected ([`request_keys_tab`]), consumed (or, hidden,
    /// dropped) by the next paint.
    static KEYS_TAB_REQUESTED: Cell<bool> = const { Cell::new(false) };
}

/// Ask the panel to fit its time view to the extent of the keys on the next
/// paint (the `F` shortcut, raised by the shell while the cursor is over the
/// panel — Blender's per-area focus).
pub fn request_fit() {
    FIT_REQUESTED.with(|c| c.set(true));
}

/// Consume a pending fit request.
pub(crate) fn take_fit_request() -> bool {
    FIT_REQUESTED.with(|c| c.replace(false))
}

/// Ask the panel to pan the time axis until the playhead is visible on the next
/// paint. Raised by the shell right after it queues a transport command that
/// jumps the playhead — the panel only page-follows while PLAYING, so a paused
/// go-to-end would otherwise leave the view where it was.
pub fn request_reveal_playhead() {
    REVEAL_REQUESTED.with(|c| c.set(true));
}

/// Consume a pending reveal request.
pub(crate) fn take_reveal_request() -> bool {
    REVEAL_REQUESTED.with(|c| c.replace(false))
}

/// **Ask the panel to land on the Keys tab on its next paint** — raised by the shell
/// when a NEW object becomes selected (Enio, 2026-07-22: *"quando um objeto novo é
/// selecionado, a timeline deve ir para aba keys"*). Selecting an object is saying
/// "I want to work on THIS one", and its keys are where that work happens.
///
/// A request, not a live rule: it fires once at the selection EDGE
/// (`timeline_bridge::selection_jumps_to_keys`), so the animator is free to walk to
/// Containers/Arrange afterwards without the selection dragging them back. Consumed
/// by the same paint that honours `F` — and DROPPED by a hidden panel's paint, so a
/// selection made with the timeline closed cannot yank the tab when it reopens.
pub fn request_keys_tab() {
    KEYS_TAB_REQUESTED.with(|c| c.set(true));
}

/// Consume a pending Keys-tab request.
pub(crate) fn take_keys_tab_request() -> bool {
    KEYS_TAB_REQUESTED.with(|c| c.replace(false))
}
