//! Timeline dope-sheet dispatch channel on [`WidgetStore`] (W2.E5b).
//!
//! Editor-core knows no timeline semantics: the pointer/wheel dispatch stashes
//! typed [`TimelineGesture`]s / [`TimelineWheel`]s here and the timeline panel
//! drains + interprets them each frame (which diamond was hit, drag-to-move,
//! clear-on-empty, anchored zoom + pan of the time axis). Lean mirror of the
//! graph-surface channel (`graph_ops`); keyboard (Delete/undo) is handled
//! shell-side against the panel selection.

use super::*;
use crate::interaction::types::{TimelineGesture, TimelineHitKind, TimelineWheel};

impl WidgetStore {
    /// ⭐ **Quem sabe o estado de um menu é o painel que o abriu**: a timeline publica, a cada
    /// quadro em que o menu `kind` está aberto, as linhas que são o estado corrente (a
    /// interpolação da chave, a extrapolação da faixa). O pintor do menu acende-as.
    ///
    /// ⚠️ Presas ao `kind` (que traz a chave/faixa): as de um menu já fechado nunca acendem outro.
    /// `titulo` é o cabeçalho que o menu mostra (o cartão da chave: «Key at frame 12 · …»); `""`
    /// = sem cabeçalho.
    pub fn set_menu_marks(
        &mut self,
        kind: crate::interaction::ContextMenuKind,
        ids: &[NodeId],
        titulo: &str,
    ) {
        self.menu_marks.0 = Some(kind);
        self.menu_marks.1.clear();
        self.menu_marks.1.extend_from_slice(ids);
        self.menu_marks.2.clear();
        self.menu_marks.2.push_str(titulo);
    }

    /// O cabeçalho publicado para o menu `kind` (vazio se não houver).
    #[must_use]
    pub fn menu_title(&self, kind: crate::interaction::ContextMenuKind) -> &str {
        if self.menu_marks.0 == Some(kind) {
            &self.menu_marks.2
        } else {
            ""
        }
    }

    /// A linha `id` do menu `kind` foi publicada como acesa por [`Self::set_menu_marks`].
    #[must_use]
    pub fn menu_row_is_marked(
        &self,
        kind: crate::interaction::ContextMenuKind,
        id: NodeId,
    ) -> bool {
        self.menu_marks.0 == Some(kind) && self.menu_marks.1.contains(&id)
    }

    /// Stash one timeline pointer gesture (dispatch → panel).
    pub fn push_timeline_gesture(&mut self, gesture: TimelineGesture) {
        self.timeline_gestures.push(gesture);
    }

    /// Drain this frame's timeline gestures. `collect()` retains the store's Vec
    /// capacity (the drain keeps the allocation), so a steady drag reuses it.
    pub fn drain_timeline_gestures(&mut self) -> std::vec::Drain<'_, TimelineGesture> {
        self.timeline_gestures.drain(..)
    }

    /// If `id`'s state is an [`InteractiveState::TimelineSurface`], return its
    /// `(surface parent, hit kind)`. Editor-core copies both out and never
    /// dereferences the opaque handles inside `kind`.
    pub fn timeline_surface_at_id(&self, id: NodeId) -> Option<(NodeId, TimelineHitKind)> {
        match self.get(id) {
            Some(InteractiveState::TimelineSurface { parent, kind, .. }) => Some((*parent, *kind)),
            _ => None,
        }
    }

    /// A alça pinta-se fora do repouso: publica a cor de REPOUSO dela para o id quente/apertado
    /// (o mesmo que [`Self::active_id`]/[`Self::hot_id`] e o clique), para o cursor se tingir.
    pub fn publish_timeline_cursor_tint(&mut self, rgba: [u8; 4]) {
        if let Some(id) = self.active_id().or_else(|| self.hot_id()) {
            self.timeline_tinta_do_cursor = Some((id, rgba));
        }
    }

    /// A cor publicada para `id`; `None` quando o alvo é neutro (corpo, faixa vazia, outra alça).
    #[must_use]
    pub fn timeline_cursor_tint(&self, id: NodeId) -> Option<[u8; 4]> {
        self.timeline_tinta_do_cursor
            .filter(|(i, _)| *i == id)
            .map(|(_, c)| c)
    }

    /// Arm a fresh timeline capture at the Down point: not yet a drag, and this
    /// is the origin the slop is measured from. `double` is whether this Down was
    /// the second of a double-click (from [`WidgetStore::record_pointer_down`]);
    /// the Up reads it back to choose `DoubleClick` over `Click` (markers only —
    /// see `pointer_up`).
    pub fn begin_timeline_press(&mut self, x: f32, y: f32, double: bool) {
        self.timeline_slop.begin(x, y);
        self.timeline_double = double;
    }

    /// Promote the capture to a DRAG once the pointer has travelled past
    /// [`crate::interaction::PRESS_SLOP_PX`] from the Down point. Called on every move.
    ///
    /// Without the slop, one pixel of hand tremor between press and release makes
    /// Up choose `End` over `Click`, and every tap target that only answers to a
    /// click (the row twirl) becomes a coin flip.
    pub fn note_timeline_pointer(&mut self, x: f32, y: f32) {
        self.timeline_slop.note(x, y);
    }

    /// Is the live timeline capture already a DRAG (past the slop)? Read-only — the panel's
    /// handles paint the dragging state from it; [`Self::take_timeline_moved`] still consumes it.
    #[must_use]
    pub fn timeline_dragging(&self) -> bool {
        self.timeline_slop.moved()
    }

    /// Read + reset the "timeline capture moved" flag (Up chooses End vs Click).
    pub fn take_timeline_moved(&mut self) -> bool {
        self.timeline_slop.take_moved()
    }

    /// Read + reset the "this capture began as a double-click" flag (Up chooses
    /// `DoubleClick` over `Click` for a marker — e.g. to open its rename field).
    pub fn take_timeline_double(&mut self) -> bool {
        std::mem::take(&mut self.timeline_double)
    }

    // ── Wheel: anchored zoom + pan + scroll ──────────────────────────────
    /// Accumulate a frame's wheel for `surface` across its three axes (zoom /
    /// horizontal pan / vertical scroll). The anchor follows the latest cursor.
    /// The panel drains + applies it.
    pub fn add_timeline_wheel(
        &mut self,
        surface: NodeId,
        zoom: f32,
        pan: f32,
        scroll: f32,
        anchor_x: f32,
    ) {
        let w = self.timeline_wheel.entry(surface).or_default();
        w.zoom_delta += zoom;
        w.pan_delta += pan;
        w.scroll_delta += scroll;
        w.anchor_x = anchor_x;
    }

    /// Drain the accumulated wheel for `surface` (removes the entry).
    pub fn take_timeline_wheel(&mut self, surface: NodeId) -> Option<TimelineWheel> {
        self.timeline_wheel.remove(&surface)
    }

    // ── Time-axis rect registry (wheel hit-test) ─────────────────────────
    /// Republish `surface`'s time-axis rect (panel, each frame) so
    /// `dispatch_wheel` can tell when the cursor is over the dope-sheet.
    pub fn set_timeline_canvas(&mut self, surface: NodeId, rect: Rect) {
        self.timeline_canvas.insert(surface, rect);
    }

    /// Forget every registered timeline canvas (the panel calls this while
    /// hidden, so a closed timeline leaves no stale wheel-zoom rect).
    pub fn clear_timeline_canvas(&mut self) {
        self.timeline_canvas.clear();
    }

    /// The timeline surface whose time-axis rect contains `(x, y)`, if any.
    pub fn timeline_surface_at(&self, x: f32, y: f32) -> Option<NodeId> {
        self.timeline_canvas
            .iter()
            .find(|(_, r)| r.contains(x, y))
            .map(|(id, _)| *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tap_that_trembles_within_the_slop_is_still_a_click() {
        // Without this, a twirl (which only answers to Click) is a coin flip: the
        // dispatch used to call the capture "moved" on the first pixel of travel.
        let mut store = WidgetStore::with_capacity(0);
        store.begin_timeline_press(100.0, 50.0, false);
        store.note_timeline_pointer(101.0, 52.0);
        store.note_timeline_pointer(100.0, 47.0);
        assert!(!store.take_timeline_moved(), "still a tap");
    }

    #[test]
    fn travelling_past_the_slop_makes_it_a_drag_for_good() {
        let mut store = WidgetStore::with_capacity(0);
        store.begin_timeline_press(100.0, 50.0, false);
        store.note_timeline_pointer(100.0 + crate::interaction::PRESS_SLOP_PX + 0.1, 50.0);
        // Coming back inside the slop does NOT demote it — a drag stays a drag.
        store.note_timeline_pointer(100.0, 50.0);
        assert!(store.take_timeline_moved());
        assert!(!store.take_timeline_moved(), "the flag resets on read");
    }

    #[test]
    fn each_press_rearms_the_slop_from_its_own_origin() {
        let mut store = WidgetStore::with_capacity(0);
        store.begin_timeline_press(0.0, 0.0, false);
        store.note_timeline_pointer(500.0, 0.0);
        assert!(store.take_timeline_moved());
        store.begin_timeline_press(500.0, 0.0, false);
        store.note_timeline_pointer(501.0, 0.0);
        assert!(!store.take_timeline_moved(), "measured from the new Down");
    }

    #[test]
    fn the_double_click_flag_round_trips_and_resets_on_read() {
        let mut store = WidgetStore::with_capacity(0);
        store.begin_timeline_press(10.0, 10.0, true);
        assert!(store.take_timeline_double(), "the double flag is readable");
        assert!(!store.take_timeline_double(), "and resets on read");
        // A fresh single-click press clears it.
        store.begin_timeline_press(10.0, 10.0, true);
        store.begin_timeline_press(10.0, 10.0, false);
        assert!(!store.take_timeline_double(), "re-armed by the latest Down");
    }
}
