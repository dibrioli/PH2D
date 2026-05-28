//! Vector Pen — pointer input dispatch.
//!
//! Shell-only handlers that reach `VectorPenTool` via
//! `gfx.tools.active_mut()` + downcast (ADR-0040 §3 documented
//! exception, mirror of `painter_input.rs`).
//!
//! ## Coordinate space (R4 redesign — sem sprite gating)
//!
//! Primary Down on canvas → convert screen px → **world-space coords**
//! via `camera.screen_to_world` → call
//! [`VectorPenTool::on_canvas_click`]. The Pen tool stores the world
//! coord directly as the vertex position (the vector network IS the
//! spatial asset; no parent sprite involved — per ADR-0056 §1.1).
//!
//! Primary Down anywhere with Pen active and chrome NOT consuming →
//! still goes to the Pen (no off-canvas concept; entire viewport is
//! the Pen's "canvas" while active).

use crate::App;
use crate::forwarding::cursor_over_hero_panel;
use ph2d_core::Vec2;
use ph2d_tool_vector_pen::VectorPenTool;

impl App {
    /// Primary Down — adds a vertex / extends path / close-paths the
    /// in-progress network at the world-space coordinate derived from
    /// the cursor's screen-pixel position.
    ///
    /// Returns `true` (consumes the event) iff Pen tool is active.
    /// Returns `false` when Pen isn't active so the event falls
    /// through to the standard gizmo / rubber-band / canvas-pick
    /// cascade.
    pub(crate) fn try_vector_pen_click(&mut self, px: f32, py: f32) -> bool {
        // R5 fix (audit Lens-T HIGH-T-2): if cursor is over any docked
        // panel (Inspector / Hierarchy / Widget Gallery / Painter
        // sidebar / etc.), DON'T consume the click as a Pen vertex —
        // let `forward_to_hero` deliver it to the panel widget. Without
        // this gate, dragging an Inspector slider while Pen is active
        // would silently add a vertex behind the panel + the slider
        // would never respond. Mirror precedent: painter's
        // `try_painter_paint_down` gates via sprite footprint hit-test
        // (which has the same effect — panels are outside the sprite
        // footprint).
        if cursor_over_hero_panel(self.gfx.as_ref(), px, py) {
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
        // Screen px → world coords via camera. No sprite footprint
        // gate; the entire canvas region (minus panels) is the Pen's
        // drawing surface.
        let window_size = gfx.surface.size();
        let world = gfx.camera.screen_to_world((px, py), window_size);
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(pen) = tool.as_any_mut().downcast_mut::<VectorPenTool>() else {
            return false;
        };
        // Outcome is intentionally discarded — bridge picks up
        // `pending_committed` next frame; Toast feedback for
        // Rejected / NoOpNearExistingVertex is W2 UX work.
        let _outcome = pen.on_canvas_click(Vec2::new(world[0], world[1]));
        true
    }

    /// When Pen tool is active, the entire viewport is its canvas —
    /// no "off-canvas" concept exists. This consume-helper stays as
    /// a sibling of `painter_active_consume_canvas_click` for parity,
    /// but in practice `try_vector_pen_click` will have already
    /// consumed any Primary Down by the time this is checked. Kept
    /// for the canvas-pick / rubber-band fall-through logic so an
    /// edge case (Pen active + some path that bypasses
    /// `try_vector_pen_click`) doesn't fall through to selection.
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
    // screen_to_world` is unit-tested in `ph2d-render::camera`.
    // The end-to-end smoke is the Day-7 manual check: clica Pen
    // pill → 3 cliques no canvas → triângulo Vello → 4º clique
    // close-path → asset salva.
}
