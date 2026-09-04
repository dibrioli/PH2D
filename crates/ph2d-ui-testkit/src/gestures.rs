//! ⭐⭐ **OS GESTOS SINTÉTICOS do arnês** — irmão do [`super`] por RESPONSABILIDADE.
//!
//! O pai MONTA e PINTA um painel; isto **conduz um ponteiro de verdade** por cima do que
//! ele pintou, pelo mesmo `dispatch_pointer` do produto — que é a única metade capaz de
//! apanhar um widget que pinta, regista, reencaminha e roteia, e ainda assim está **morto
//! sob o rato** porque o `populate` nunca o registou. O doc do [`MockPanelHost::click_at`]
//! traz o caso que pagou essa lição.
//!
//! ⚠️ **O corte foi obrigado pela INTEGRAÇÃO de 2026-09-04:** o arnês veio da
//! `line/components` e a `line/UIUX` acrescentou um campo ao `PaintCtx` que ele constrói à
//! mão — a linha extra levou o ficheiro a `714 / 700`. *Um tecto de LOC é a única coisa
//! deste repo que só a árvore COMBINADA acusa.*

use super::{MockPanelHost, NANOS_PER_SECOND};
use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, dispatch_pointer};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

impl MockPanelHost {
    /// **Drive a REAL pointer click at `(x, y)`** — Down then Up — through the same
    /// [`dispatch_pointer`] the shell runs, over the hit index the last [`Self::paint`] built.
    /// Returns the `WidgetEvent`s the dispatcher emitted (feed them to `apply_panel_event`).
    ///
    /// ## Why this exists (the last hole in the "green-but-dead" family)
    ///
    /// [`Self::paint`] proves a widget REGISTERS A HIT RECT and [`Self::apply_panel_event`]
    /// proves the panel FORWARDS an event it is handed. Neither proves a POINTER on that rect
    /// ever becomes that event — and it does not, unless the id also carries an
    /// `InteractiveState` in the store: `dispatch_pointer`'s Down only makes a hit `active`
    /// when it is *focusable*, and an id absent from the store is not. So a widget can paint,
    /// hit-register, forward and route — every gate green — and still be **stone dead under
    /// the mouse** because `populate` never registered it. That is not a hypothetical: it is
    /// the Impasto light rig (Enio 2026-07-12, *"nem o checkbox nem se pode selecionar outra
    /// luz"*), and before it the hierarchy companions.
    ///
    /// A widget is not done when it paints. It is done when a test CLICKS it.
    pub fn click_at(&mut self, x: f32, y: f32) -> Vec<WidgetEvent> {
        let mut out = Vec::new();
        for kind in [PointerKind::Down, PointerKind::Up] {
            // Space the clicks a full second apart: inside the double-click window the
            // dispatcher would upgrade the second Click to a DoubleClick and the assertion
            // would fail for a reason that has nothing to do with the seam under test.
            self.clock_ns += NANOS_PER_SECOND;
            let arena = Bump::new();
            let event = PointerEvent {
                x,
                y,
                pressure: 1.0,
                kind,
                source: PointerSource::Mouse,
                button: PointerButton::Primary,
                timestamp_ns: self.clock_ns,
            };
            out.extend_from_slice(dispatch_pointer(
                &mut self.store,
                &self.hit_index,
                event,
                &arena,
            ));
        }
        out
    }

    /// **Drive a REAL pointer DRAG** from `(x0, y0)` to `(x1, y1)` — Down, Move, Up — through the same
    /// [`dispatch_pointer`] the shell runs.
    ///
    /// [`Self::click_at`] proves a widget is alive under the mouse; it cannot prove anything about a
    /// control whose whole meaning is the MOTION — a `CurvePoint` handle emits nothing on a Down/Up in
    /// the same place, so a gate built from clicks would be green over a handle that never moves a value.
    ///
    /// The Move carries the same button state as the Down, which is what makes the dispatcher treat it
    /// as a drag of the active widget rather than as hover.
    pub fn drag_at(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<WidgetEvent> {
        let mut out = Vec::new();
        for (kind, x, y) in [
            (PointerKind::Down, x0, y0),
            (PointerKind::Move, x1, y1),
            (PointerKind::Up, x1, y1),
        ] {
            self.clock_ns += NANOS_PER_SECOND;
            let arena = Bump::new();
            let event = PointerEvent {
                x,
                y,
                pressure: 1.0,
                kind,
                source: PointerSource::Mouse,
                button: PointerButton::Primary,
                timestamp_ns: self.clock_ns,
            };
            out.extend_from_slice(dispatch_pointer(
                &mut self.store,
                &self.hit_index,
                event,
                &arena,
            ));
        }
        out
    }

    /// The id a pointer at `(x, y)` actually LANDS ON — the dispatcher's own resolution
    /// (topmost = last registered wins). When [`Self::click_at`] emits nothing, this says
    /// whether the widget lost the hit to something painted over it, or was never hit at all.
    pub fn hit_at(&self, x: f32, y: f32) -> Option<NodeId> {
        self.hit_index.hit(x, y)
    }
}
