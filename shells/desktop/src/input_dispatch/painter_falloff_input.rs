//! Painter **Falloff-graph** input (SHELL-only) — the shell-owned gestures on the Brush panel's
//! Falloff curve graph (Custom preset): left-click the empty graph to ADD + drag a control point,
//! right-click a point for the handle-type menu, Delete to remove the selection. Split from
//! `painter_canvas_input` for the HR-18 file-LOC cap; shares its `painter_tool_mut` downcast
//! (the allowlisted ADR-0040 §3 exception).

use std::cell::Cell;

use crate::App;

thread_local! {
    /// The Falloff control point being dragged by a shell-owned gesture (the stable
    /// id), `None` when idle. Set when a left-click on the EMPTY graph adds a point
    /// (so the same press immediately drags it off the line — a corner needs a
    /// non-collinear point); CursorMoved drags it; Primary Up clears it. Existing
    /// handles keep using the panel's `CurvePoint` drag.
    static FALLOFF_DRAG: Cell<Option<u8>> = const { Cell::new(None) };
}

impl App {
    /// Primary Down on the Falloff curve graph's empty canvas (Custom preset):
    /// add a control point where clicked, select it, and GRAB it so the same
    /// gesture drags it (a corner needs a point off the line — adding on the line
    /// is collinear). Returns `true` (consuming) only when a point was added — a
    /// press on an existing handle (`FalloffHit::Point`) returns `false` so the
    /// panel's drag dispatch grabs it, and an off-canvas press also falls through.
    pub(crate) fn painter_falloff_canvas_add(&mut self, px: f32, py: f32) -> bool {
        let ph2d_panel_painter_layers::FalloffHit::Canvas(nx, ny) =
            ph2d_panel_painter_layers::falloff_hit_test(px, py)
        else {
            return false;
        };
        // While a floating overlay is open (the colour picker, or a right-click
        // context menu), a click belongs to it — not to the graph underneath.
        // Without this guard, clicking inside the floating picker dropped stray
        // points onto the curve. (Earlier the gate used `hit_index.hit`, but the
        // panel chrome registers hit rects over the graph area, so that blocked
        // ALL adds — this open-overlay gate is precise.)
        let overlay_open = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.picker_target().is_some() || h.store.context_menu().is_some());
        if overlay_open {
            return false;
        }
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        if let Some(id) = painter.add_brush_falloff_point_at(nx, ny) {
            ph2d_panel_painter_layers::set_selected_falloff_point(Some(id));
            FALLOFF_DRAG.with(|c| c.set(Some(id)));
            return true;
        }
        false
    }

    /// CursorMoved while a shell-owned Falloff add-drag is live: move the grabbed
    /// point to the cursor (clamped into the graph). Returns `true` (consuming the
    /// move) while dragging, so it doesn't also pan / drive a gizmo.
    pub(crate) fn painter_falloff_drag(&mut self, px: f32, py: f32) -> bool {
        let Some(id) = FALLOFF_DRAG.with(Cell::get) else {
            return false;
        };
        if let Some((nx, ny)) = ph2d_panel_painter_layers::falloff_canvas_norm(px, py)
            && let Some(painter) = self.painter_tool_mut()
        {
            painter.set_brush_falloff_point(id, nx, ny);
        }
        true
    }

    /// Primary Up: end any shell-owned Falloff add-drag. No-op when not dragging.
    pub(crate) fn painter_falloff_release(&mut self) {
        FALLOFF_DRAG.with(|c| c.set(None));
    }

    /// Secondary Down on a Falloff curve control point (Custom preset): select it
    /// and open the handle-type menu (Vector / Auto) at the cursor. The chrome
    /// handler parks the choice in `HeroScreen.pending_falloff_point_handle`;
    /// [`Self::painter_apply_pending_falloff_handle`] drains it. Returns `true`
    /// (consuming) iff a point was hit.
    pub(crate) fn painter_falloff_open_point_menu(&mut self, px: f32, py: f32) -> bool {
        let ph2d_panel_painter_layers::FalloffHit::Point(id) =
            ph2d_panel_painter_layers::falloff_hit_test(px, py)
        else {
            return false;
        };
        // Only meaningful while the Painter owns the active tool.
        if self.painter_tool_mut().is_none() {
            return false;
        }
        ph2d_panel_painter_layers::set_selected_falloff_point(Some(id));
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            hero.store
                .open_context_menu(ph2d_editor::interaction::ContextMenuRequest {
                    x: px,
                    y: py,
                    kind: ph2d_editor::interaction::ContextMenuKind::FalloffPointHandle,
                });
        }
        true
    }

    /// Delete/Backspace: remove the selected Falloff control point. Returns `true`
    /// (consuming) iff a point was selected and dropped, so the key falls through
    /// otherwise (it must not eat Delete for other tools).
    pub(crate) fn painter_delete_selected_falloff_point(&mut self) -> bool {
        let Some(id) = ph2d_panel_painter_layers::selected_falloff_point() else {
            return false;
        };
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        painter.remove_brush_falloff_point(id);
        ph2d_panel_painter_layers::set_selected_falloff_point(None);
        true
    }
}
