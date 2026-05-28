//! Vector Pen — pointer + keyboard input dispatch.
//!
//! Shell-only handlers that reach `VectorPenTool` via
//! `gfx.tools.active_mut()` + downcast (ADR-0040 §3 documented
//! exception, mirror of `painter_input.rs`).
//!
//! Primary Down on the canvas → `screen px → camera.screen_to_world →
//! world coords` → [`VectorPenTool::on_canvas_click`]. The world coord is
//! stored directly as the vertex position (the vector network IS the
//! spatial asset; no parent sprite — ADR-0056 §1.1). With Pen active the
//! whole viewport (minus chrome/panels) is the drawing surface.

use crate::App;
use crate::forwarding::cursor_over_hero_panel;
use ph2d_core::Vec2;
use ph2d_editor::toast::Toast;
use ph2d_tool_vector_pen::{PenClickOutcome, VectorPenTool};

impl App {
    /// Primary Down — adds a vertex / extends path / close-paths the
    /// in-progress network at the world-space coordinate derived from
    /// the cursor's screen-pixel position.
    ///
    /// Returns `true` (consumes the event) iff the Pen tool is active.
    /// Returns `false` when Pen isn't active so the event falls through
    /// to the standard gizmo / rubber-band / canvas-pick cascade.
    pub(crate) fn try_vector_pen_click(&mut self, px: f32, py: f32) -> bool {
        // Don't consume over chrome: a docked panel (slider drag) or any
        // hit-indexed chrome widget (TopBar pill, LeftRail button) owns
        // its pixels — otherwise a click there would silently drop a
        // vertex behind the widget AND fail to reach the chrome handler
        // (e.g. the PEN pill's own toggle-off).
        if cursor_over_hero_panel(self.gfx.as_ref(), px, py) {
            return false;
        }
        if let Some(gfx) = self.gfx.as_ref()
            && let Some(hero) = gfx.hero_screen.as_ref()
            && hero.hit_index.hit(px, py).is_some()
        {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let pen_active = gfx
            .tools
            .active()
            .map(|t| t.id() == ph2d_editor::ToolId::new("vector_pen"))
            .unwrap_or(false);
        if !pen_active {
            return false;
        }
        let window_size = gfx.surface.size();
        let world = gfx.camera.screen_to_world((px, py), window_size);
        let outcome = match gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorPenTool>())
        {
            Some(pen) => pen.on_canvas_click(Vec2::new(world[0], world[1])),
            None => return false,
        };
        // Surface a hint when the click did nothing actionable (vertex
        // cap hit or position out of bounds). NoOp-near-vertex is silent
        // snap behavior — no toast.
        if matches!(outcome, PenClickOutcome::Rejected) {
            gfx.toasts.push(Toast::warning(
                "Vector click rejected — vertex limit or position out of bounds",
            ));
        }
        true
    }

    /// Escape while the Pen tool is active: cancel the in-progress path
    /// first; if none is in progress, clear the committed scene. Returns
    /// `true` (consumes Escape) only when it actually cancelled or
    /// cleared something, so an idle Escape still falls through to the
    /// hero pipeline (widget blur). W1 in-memory scene model — no
    /// persistence until W2 AssetDb.
    pub(crate) fn try_vector_pen_escape(&mut self) -> bool {
        let cancelled_in_progress = {
            let Some(gfx) = self.gfx.as_mut() else {
                return false;
            };
            let pen_active = gfx
                .tools
                .active()
                .map(|t| t.id() == ph2d_editor::ToolId::new("vector_pen"))
                .unwrap_or(false);
            if !pen_active {
                return false;
            }
            match gfx
                .tools
                .active_mut()
                .and_then(|t| t.as_any_mut().downcast_mut::<VectorPenTool>())
            {
                Some(pen) if pen.has_in_progress_path() => {
                    pen.reset_path();
                    true
                }
                _ => false,
            }
        };
        if cancelled_in_progress {
            if let Some(gfx) = self.gfx.as_mut() {
                gfx.toasts.push(Toast::info("Vector path cancelled"));
            }
            return true;
        }
        if !self.committed_vector_pen_paths.is_empty() {
            self.committed_vector_pen_paths.clear();
            if let Some(gfx) = self.gfx.as_mut() {
                gfx.toasts.push(Toast::info("Vector scene cleared"));
            }
            return true;
        }
        false
    }

    /// When the Pen tool is active the entire viewport is its canvas —
    /// no "off-canvas" concept exists. Sibling of
    /// `painter_active_consume_canvas_click` for parity; guards the
    /// canvas-pick / rubber-band fall-through so a Primary Down that
    /// bypassed `try_vector_pen_click` doesn't reach selection.
    pub(crate) fn vector_pen_active_consume_canvas_click(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.tools.active())
            .map(|t| t.id() == ph2d_editor::ToolId::new("vector_pen"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    // The `App` orchestration isn't unit-testable from the shell
    // (depends on winit / wgpu surface). The pure-fn `Camera2d::
    // screen_to_world` is unit-tested in `ph2d-render::camera`; the Pen
    // tool's click/cancel logic is covered in `ph2d-tool-vector-pen`.
    // End-to-end smoke is the manual Day-7 check: Pen pill → 3 clicks →
    // triangle → 4th click close-path → triangle persists; Esc cancels.
}
