//! **O ÍMÃ e as RÉGUAS** — as cinco chaves que a seção Snap pinta (plano 25 §9 e W6.2).
//!
//! Irmão do [`crate::state_frame`] pelo teto de 600 LOC do painel, e o corte é por ASSUNTO: aqui
//! mora *a que a ponta se agarra, e o que a borda do canvas mostra*. São **três** perguntas
//! independentes e por isso três publicadores — colapsá-las faria esconder a régua desligar o
//! ímã, que é o oposto do que o artista pede.

use std::cell::Cell;

thread_local! {
    /// Whether shape-snapping is on (mirrored from the shell). The GRID toggle
    /// lives in the editor's universal Grid Snap panel, not here.
    static CURRENT_SNAP: Cell<bool> = const { Cell::new(true) };
    static CURRENT_SNAP_PATH: Cell<bool> = const { Cell::new(false) };
    static CURRENT_SNAP_CROSS: Cell<bool> = const { Cell::new(false) };
    /// O ímã das GUIAS (W6.2). Nasce LIGADO — num documento sem guias ele é inerte.
    static CURRENT_SNAP_GUIDES: Cell<bool> = const { Cell::new(true) };
    /// As RÉGUAS à mostra (W6.2). Nasce LIGADO: elas são o gesto de onde as guias nascem, e
    /// uma afordância que ninguém acha é uma que não existe.
    static CURRENT_RULERS: Cell<bool> = const { Cell::new(true) };
}

/// Publish whether shape-snapping is on, so the Snap section reflects it.
pub fn set_current_snap(on: bool) {
    CURRENT_SNAP.with(|c| c.set(on));
}

/// Whether shape-snapping is on this frame.
pub(crate) fn current_snap() -> bool {
    CURRENT_SNAP.with(|c| c.get())
}

/// Publish the two POSITION claims of the Snap section (plano 25 §9). They are separate from
/// `set_current_snap` because they answer a different question: that one aligns one axis at a
/// time, these two land the point somewhere.
pub fn set_current_snap_position(path: bool, crossings: bool) {
    CURRENT_SNAP_PATH.with(|c| c.set(path));
    CURRENT_SNAP_CROSS.with(|c| c.set(crossings));
}

/// Whether snapping ONTO the geometry is on this frame.
pub(crate) fn current_snap_path() -> bool {
    CURRENT_SNAP_PATH.with(|c| c.get())
}

/// Whether snapping to curve crossings is on this frame.
pub(crate) fn current_snap_crossings() -> bool {
    CURRENT_SNAP_CROSS.with(|c| c.get())
}

/// Publish the two switches of the W6.2 — o ímã das GUIAS e a visibilidade das RÉGUAS.
///
/// ⚠️ São publicados juntos porque a seção os pinta juntos, mas respondem a perguntas
/// diferentes: um decide se a guia ATRAI, o outro se ela pode ser MEXIDA (e se a faixa
/// aparece). Colapsá-los faria esconder a régua desligar o ímã, que é o oposto do desejado.
pub fn set_current_guides(snap: bool, rulers: bool) {
    CURRENT_SNAP_GUIDES.with(|c| c.set(snap));
    CURRENT_RULERS.with(|c| c.set(rulers));
}

/// Whether snapping to document guides is on this frame.
pub(crate) fn current_snap_guides() -> bool {
    CURRENT_SNAP_GUIDES.with(|c| c.get())
}

/// Whether the canvas rulers are on screen this frame.
pub(crate) fn current_rulers() -> bool {
    CURRENT_RULERS.with(|c| c.get())
}
