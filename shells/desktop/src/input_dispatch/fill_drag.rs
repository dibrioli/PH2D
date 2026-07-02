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

use crate::App;

thread_local! {
    /// `Some((start_x, start_y, on_canvas))` between a Fill-button press and its release. `on_canvas`
    /// flips true once the drag lands inside the sprite footprint (the ColorDrop began).
    static FILL_DRAG: Cell<Option<(f32, f32, bool)>> = const { Cell::new(None) };
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

    /// Primary Up: if the drag reached the canvas, complete the ColorDrop (fill + arm the threshold
    /// adjust); otherwise it was a plain click on the button → open the paint-colour picker.
    pub(crate) fn fill_drag_up(&mut self) {
        let Some((_sx, _sy, on_canvas)) = FILL_DRAG.with(Cell::get) else {
            return;
        };
        FILL_DRAG.with(|c| c.set(None));
        if on_canvas {
            let (px, py) = self.last_pointer;
            self.deliver_canvas_pointer(px, py, 1.0, PointerPhase::Up);
        } else {
            self.open_paint_color_picker();
        }
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
