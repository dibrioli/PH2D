//! Fill (Bucket) ColorDrop drag — the shell half of the Procreate "drag the colour from the well onto
//! the canvas" gesture. A Primary Down on the Fill rail button (`PAINTER_RAIL_FILL`) arms this drag and
//! activates Fill; dragging onto the sprite delivers a canvas Down→Move to the painter's `fill_pointer`
//! (via [`App::deliver_canvas_pointer`]); the release completes the ColorDrop, or — if the press never
//! reached the canvas (a plain click) — opens the colour picker for the paint colour.
//!
//! Split from `painter_canvas_input.rs` for the HR-18 file-LOC cap. Downcast-free: the mode switch goes
//! through the `Tool` trait, delivery through the shared canvas path, the picker through `WidgetStore`.

use std::cell::Cell;

use ph2d_editor::ids;
use ph2d_editor::tool::{PanelEvent, PointerPhase, Tool};
use ph2d_tool_painter::PainterTool;

use crate::App;

/// Hold-still time (ms) over the target before the dwell fires the in-place fill and the drag flips to
/// live threshold adjust (the modal-free second gesture).
const FILL_DWELL_MS: f32 = 1000.0;
/// Pointer travel (px) that resets the dwell clock — jitter below this still counts as "held still".
const FILL_DWELL_EPS: f32 = 3.0;
/// Live-adjust gain: threshold units (`0..1`) per screen pixel from the dwell anchor. ~300 px spans full.
const FILL_LIVE_GAIN: f32 = 1.0 / 300.0;

/// The armed Fill ColorDrop drag. Beyond the original `(start, on_canvas)` it carries the dwell clock and,
/// once the dwell fires, the live-adjust anchor — so a *held* drag can fill in place and tune the
/// threshold by pointer delta, with no modal (the modal path stays for a plain drop→release).
#[derive(Clone, Copy)]
struct FillDrag {
    /// The drag has landed inside the sprite footprint — the ColorDrop began (a Down was delivered).
    on_canvas: bool,
    /// Stillness accumulated (ms) since the last move past [`FILL_DWELL_EPS`]. Advances only on-canvas.
    dwell_ms: f32,
    /// Anchor the stillness is measured from — reset whenever the pointer really moves.
    still: (f32, f32),
    /// The dwell fired: the fill has flooded and each move now live-adjusts the threshold until release.
    live: bool,
    /// Pointer position + threshold captured at the moment of firing — the origin of the live delta.
    anchor: (f32, f32),
    anchor_threshold: f32,
}

thread_local! {
    /// The armed Fill-button drag (button-press → release), or `None` when idle. See [`FillDrag`].
    static FILL_DRAG: Cell<Option<FillDrag>> = const { Cell::new(None) };

    /// `Some((last_x, last_y))` while the "Fill adjust" modal's title band is being dragged — the last
    /// cursor position, so each move offsets the card by the raw cursor delta (no dead-zone). `None` idle.
    static FILL_MODAL_DRAG: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

/// `true` while a Fill ColorDrop drag is armed (between the Fill-button press and its release). The
/// render pass reads this to draw the paint-colour cursor at the pointer until the drop lands.
pub(crate) fn fill_drag_armed() -> bool {
    FILL_DRAG.with(|c| c.get().is_some())
}

/// Map a live-adjust pointer position to a Fill threshold: the pointer delta from the dwell `anchor`,
/// where **right / up = larger** fill and **left / down = smaller** (screen y grows downward, so "up" is
/// a negative dy → the `−dy` term), scaled by [`FILL_LIVE_GAIN`], added to the anchor threshold, clamped
/// to `0..1`.
fn live_threshold(anchor: (f32, f32), anchor_threshold: f32, px: f32, py: f32) -> f32 {
    let delta = (px - anchor.0) - (py - anchor.1);
    (anchor_threshold + delta * FILL_LIVE_GAIN).clamp(0.0, 1.0)
}

/// Per-frame heartbeat, called from the render loop next to [`Tool::on_tick`]: advance the dwell clock
/// while a Fill drop is held still on-canvas, and once it crosses [`FILL_DWELL_MS`] fire the fill in
/// place and flip the drag to live threshold-adjust. `tool` is the active tool (only a Painter with a
/// pending fill reacts); `last_pointer` is the current cursor — the anchor the live delta measures from.
/// A free fn (not an `App` method) so the render loop can call it while `self.gfx.tools` is borrowed.
pub(crate) fn fill_drag_tick(tool: &mut dyn Tool, last_pointer: (f32, f32), dt_ms: f32) {
    let Some(mut st) = FILL_DRAG.with(Cell::get) else {
        return;
    };
    if st.live || !st.on_canvas {
        return;
    }
    let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
        return;
    };
    st.dwell_ms += dt_ms;
    if st.dwell_ms >= FILL_DWELL_MS {
        if painter.has_active_fill() {
            // Flood at the current seed + threshold NOW (re-fill), then anchor the live delta here.
            let t = painter.fill_threshold();
            painter.set_fill_threshold(t);
            st.live = true;
            st.anchor = last_pointer;
            st.anchor_threshold = t;
        } else {
            st.dwell_ms = 0.0; // no active fill (seed lost) — keep the plain drop drag alive.
        }
    }
    FILL_DRAG.with(|c| c.set(Some(st)));
}

impl App {
    /// A Primary Down over the **C&F** (Colour & Fill) rail button arms the ColorDrop drag. It does NOT
    /// activate Fill — a plain click must only open the colour picker (Enio 2026-07-02); Fill activates the
    /// moment the drag first reaches the canvas (see [`Self::fill_drag_move`]). Call on every Primary Down
    /// (it self-gates on the hit id); no-op otherwise.
    pub(crate) fn arm_fill_drag_if_on_button(&mut self, px: f32, py: f32) {
        let on_button = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.hit_index.hit(px, py))
            == Some(ids::PAINTER_RAIL_FILL);
        if !on_button {
            return;
        }
        FILL_DRAG.with(|c| {
            c.set(Some(FillDrag {
                on_canvas: false,
                dwell_ms: 0.0,
                still: (px, py),
                live: false,
                anchor: (0.0, 0.0),
                anchor_threshold: 0.0,
            }))
        });
    }

    /// CursorMoved while a Fill-button drag is armed. Two regimes:
    /// * **live** (dwell already fired): the seed is frozen; map the pointer delta from the anchor onto
    ///   the threshold (right/up = larger fill, left/down = smaller) and re-flood immediately.
    /// * **dropping** (not yet fired): deliver the ColorDrop, and reset the dwell clock on real movement.
    ///
    /// Returns `true` (consume the move) while armed, so it never pans / drives a gizmo.
    pub(crate) fn fill_drag_move(&mut self, px: f32, py: f32) -> bool {
        let Some(mut st) = FILL_DRAG.with(Cell::get) else {
            return false;
        };
        if st.live {
            self.painter_set_fill_threshold(live_threshold(st.anchor, st.anchor_threshold, px, py));
            return true; // no ColorDrop delivery — the seed stays at the dwell point.
        }
        // First sample inside the sprite footprint begins the drop (Down); later ones track it (Move).
        // `deliver_canvas_pointer` returns false for a Down outside the footprint, so the drop starts
        // only once the cursor is over the sprite.
        let phase = if st.on_canvas {
            PointerPhase::Move
        } else {
            // Dragging the colour onto the canvas is what ACTIVATES Fill (a plain click never does — it
            // only opens the picker). Switch the mode DIRECTLY (not via the once-per-frame rail bus) right
            // before the first Down so `on_canvas_pointer` sees Fill mode and floods on this very sample.
            if let Some(tool) = self.gfx.as_mut().and_then(|g| g.tools.active_mut()) {
                tool.handle_panel_event(PanelEvent::SelectOption(
                    ids::PAINTER_PAINT_MODE,
                    "fill".to_string(),
                ));
            }
            PointerPhase::Down
        };
        let delivered = self.deliver_canvas_pointer(px, py, 1.0, phase);
        if !st.on_canvas && delivered {
            st.on_canvas = true;
            st.still = (px, py);
            st.dwell_ms = 0.0;
        }
        // Any real move restarts the dwell timer; sub-epsilon jitter is treated as "held still".
        if (px - st.still.0).hypot(py - st.still.1) > FILL_DWELL_EPS {
            st.still = (px, py);
            st.dwell_ms = 0.0;
        }
        FILL_DRAG.with(|c| c.set(Some(st)));
        true
    }

    /// Primary Up: if the drag reached the canvas, complete the ColorDrop (fill + open the threshold
    /// adjust modal); otherwise it was a plain click on the button → open the paint-colour picker.
    pub(crate) fn fill_drag_up(&mut self) {
        let Some(st) = FILL_DRAG.with(Cell::get) else {
            return;
        };
        FILL_DRAG.with(|c| c.set(None));
        if st.live {
            // The dwell gesture already flooded + live-adjusted the fill; release just commits it (the
            // final threshold is whatever the last live move applied). No modal — that's the point.
            self.painter_commit_fill();
        } else if st.on_canvas {
            let (px, py) = self.last_pointer;
            self.deliver_canvas_pointer(px, py, 1.0, PointerPhase::Up);
            // The drop flooded a region that now awaits its live threshold adjust — open the floating
            // "Fill adjust" modal at the release point, seeded with the tool's current threshold.
            if let Some(threshold) = self.painter_active_fill_threshold()
                && let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut())
            {
                hero.store.open_fill_modal(px, py, threshold);
            }
        } else {
            self.open_paint_color_picker();
        }
    }

    /// The active Painter's current Fill threshold (`0..1`) iff a ColorDrop is pending its modal adjust
    /// ([`PainterTool::has_active_fill`]); `None` when the Painter isn't active or no fill is pending.
    /// Short-name downcast (ADR-0040 §3; gate-safe — the arch gate only matches fully-qualified
    /// `ph2d_tool_…` paths).
    fn painter_active_fill_threshold(&mut self) -> Option<f32> {
        let painter = self
            .gfx
            .as_mut()?
            .tools
            .active_mut()?
            .as_any_mut()
            .downcast_mut::<PainterTool>()?;
        painter.has_active_fill().then(|| painter.fill_threshold())
    }

    /// Set the active Painter's Fill threshold (re-floods live). No-op unless the Painter is active with a
    /// pending fill. Same short-name downcast as [`Self::painter_active_fill_threshold`] (gate-safe).
    fn painter_set_fill_threshold(&mut self, threshold: f32) {
        if let Some(painter) = self
            .gfx
            .as_mut()
            .and_then(|g| g.tools.active_mut())
            .and_then(|tool| tool.as_any_mut().downcast_mut::<PainterTool>())
        {
            painter.set_fill_threshold(threshold);
        }
    }

    /// Commit the active Painter's pending Fill (push the undo entry, clear the transient drop state),
    /// ending the dwell live-adjust gesture. Same short-name downcast (gate-safe).
    fn painter_commit_fill(&mut self) {
        if let Some(painter) = self
            .gfx
            .as_mut()
            .and_then(|g| g.tools.active_mut())
            .and_then(|tool| tool.as_any_mut().downcast_mut::<PainterTool>())
        {
            painter.fill_commit();
        }
    }

    /// A Primary Down over the Fill modal's title band arms a modal-move drag. Returns `true` (consume
    /// the Down) when it hits the handle, so the card moves instead of the Down doing anything else (and
    /// the modal never closes while dragging).
    pub(crate) fn arm_fill_modal_drag_if_on_handle(&mut self, px: f32, py: f32) -> bool {
        let on_handle = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.hit_index.hit(px, py))
            == Some(ids::PAINTER_FILL_MODAL_HANDLE);
        if !on_handle {
            return false;
        }
        FILL_MODAL_DRAG.with(|c| c.set(Some((px, py))));
        true
    }

    /// CursorMoved while the Fill modal is being dragged: offset the card by the cursor delta. Returns
    /// `true` (consume the move) while dragging, so it doesn't pan / drive a gizmo.
    pub(crate) fn fill_modal_drag_move(&mut self, px: f32, py: f32) -> bool {
        let Some((lx, ly)) = FILL_MODAL_DRAG.with(Cell::get) else {
            return false;
        };
        FILL_MODAL_DRAG.with(|c| c.set(Some((px, py))));
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.move_fill_modal(px - lx, py - ly);
        }
        true
    }

    /// Primary Up: end a Fill modal title-band drag. No-op when not dragging.
    pub(crate) fn fill_modal_drag_up(&mut self) {
        FILL_MODAL_DRAG.with(|c| c.set(None));
    }

    /// Open the rich Blender colour picker, seeded with + targeting the paint colour.
    fn open_paint_color_picker(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return;
        };
        let store = &mut hero.store;
        let seed = store
            .widget_color(ids::PAINTER_COLOR_THUMB)
            .unwrap_or([0x88, 0x88, 0x88, 0xFF]); // LITERAL-COLOR-OK: neutral default before a colour is set
        store.set_widget_color(ids::PAINTER_COLOR_THUMB, seed);
        store.set_picker_target(Some(ids::PAINTER_COLOR_THUMB));
        store.set_blender_value(
            ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{FILL_LIVE_GAIN, live_threshold};

    #[test]
    fn live_threshold_grows_right_up_shrinks_left_down_and_clamps() {
        let anchor = (100.0, 100.0);
        let base = 0.5;
        // Right (+x) and up (−y in screen space) each ENLARGE the fill.
        assert!(
            live_threshold(anchor, base, 160.0, 100.0) > base,
            "drag right = larger"
        );
        assert!(
            live_threshold(anchor, base, 100.0, 40.0) > base,
            "drag up = larger"
        );
        // Left (−x) and down (+y) each SHRINK it.
        assert!(
            live_threshold(anchor, base, 40.0, 100.0) < base,
            "drag left = smaller"
        );
        assert!(
            live_threshold(anchor, base, 100.0, 160.0) < base,
            "drag down = smaller"
        );
        // Holding still on the anchor holds the threshold.
        assert!((live_threshold(anchor, base, 100.0, 100.0) - base).abs() < 1e-6);
        // The gain is symmetric right↔up and left↔down.
        let right = live_threshold(anchor, base, 100.0 + 30.0, 100.0);
        let up = live_threshold(anchor, base, 100.0, 100.0 - 30.0);
        assert!(
            (right - up).abs() < 1e-6,
            "right and up move the threshold equally"
        );
        assert!(
            (right - (base + 30.0 * FILL_LIVE_GAIN)).abs() < 1e-6,
            "gain applied once"
        );
        // Clamped to 0..1 at the extremes.
        assert_eq!(live_threshold(anchor, 0.9, 1.0e6, 100.0), 1.0);
        assert_eq!(live_threshold(anchor, 0.1, -1.0e6, 100.0), 0.0);
    }
}
