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
use ph2d_editor::tool::{PanelEvent, PointerPhase};
use ph2d_tool_painter::PainterTool;

use crate::App;

thread_local! {
    /// `Some((start_x, start_y, on_canvas))` between a Fill-button press and its release. `on_canvas`
    /// flips true once the drag lands inside the sprite footprint (the ColorDrop began).
    static FILL_DRAG: Cell<Option<(f32, f32, bool)>> = const { Cell::new(None) };

    /// `Some((last_x, last_y))` while the "Fill adjust" modal's title band is being dragged — the last
    /// cursor position, so each move offsets the card by the raw cursor delta (no dead-zone). `None` idle.
    static FILL_MODAL_DRAG: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

/// `true` while a Fill ColorDrop drag is armed (between the Fill-button press and its release). The
/// render pass reads this to draw the paint-colour cursor at the pointer until the drop lands.
pub(crate) fn fill_drag_armed() -> bool {
    FILL_DRAG.with(|c| c.get().is_some())
}

impl App {
    /// A Primary Down over the Fill rail button arms the ColorDrop drag + activates Fill. Call on every
    /// Primary Down (it self-gates on the hit id); no-op otherwise.
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
        // Activate Fill directly — the rail bus drains once per frame, so the first canvas sample of a
        // fast drag could arrive before a bus-routed mode switch lands.
        if let Some(tool) = self.gfx.as_mut().and_then(|g| g.tools.active_mut()) {
            tool.handle_panel_event(PanelEvent::SelectOption(
                ids::PAINTER_PAINT_MODE,
                "fill".to_string(),
            ));
        }
        FILL_DRAG.with(|c| c.set(Some((px, py, false))));
    }

    /// CursorMoved while a Fill-button drag is armed: deliver the ColorDrop to the painter. Returns
    /// `true` (consume the move) while armed, so it never pans / drives a gizmo.
    pub(crate) fn fill_drag_move(&mut self, px: f32, py: f32) -> bool {
        let Some((sx, sy, on_canvas)) = FILL_DRAG.with(Cell::get) else {
            return false;
        };
        // First sample inside the sprite footprint begins the drop (Down); later ones track it (Move).
        // `deliver_canvas_pointer` returns false for a Down outside the footprint, so the drop starts
        // only once the cursor is over the sprite.
        let phase = if on_canvas {
            PointerPhase::Move
        } else {
            PointerPhase::Down
        };
        let delivered = self.deliver_canvas_pointer(px, py, 1.0, phase);
        if !on_canvas && delivered {
            FILL_DRAG.with(|c| c.set(Some((sx, sy, true))));
        }
        true
    }

    /// Primary Up: if the drag reached the canvas, complete the ColorDrop (fill + open the threshold
    /// adjust modal); otherwise it was a plain click on the button → open the paint-colour picker.
    pub(crate) fn fill_drag_up(&mut self) {
        let Some((_sx, _sy, on_canvas)) = FILL_DRAG.with(Cell::get) else {
            return;
        };
        FILL_DRAG.with(|c| c.set(None));
        if on_canvas {
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
