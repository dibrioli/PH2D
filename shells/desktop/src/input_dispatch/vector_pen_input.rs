//! Vector Pen — pointer input dispatch.
//!
//! Shell-only handlers that reach `VectorPenTool` via
//! `gfx.tools.active_mut()` + downcast (ADR-0040 §3 documented
//! exception, mirror of `painter_input.rs`).
//!
//! Primary Down on canvas inside sprite footprint → translates screen
//! pixel → network-local pixel coordinate → calls
//! [`VectorPenTool::on_canvas_click`]. Primary Down outside the
//! footprint while Pen is active → silent consume (no fall-through to
//! gizmo / rubber-band — same UX rule as the Painter case).

use crate::{App, Transform};
use ph2d_core::Vec2;
use ph2d_tool_vector_pen::VectorPenTool;

impl App {
    /// Primary Down — adds a vertex / extends path / close-paths.
    ///
    /// Returns `true` (consumes the event) iff Pen tool is active AND
    /// a sprite is selected AND the click landed inside the sprite
    /// footprint. Otherwise returns `false` (caller may fall through
    /// to other handlers).
    pub(crate) fn try_vector_pen_click(&mut self, px: f32, py: f32) -> bool {
        let Some((nx, ny)) = self.vector_pen_pointer_xy(px, py) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(pen) = tool.as_any_mut().downcast_mut::<VectorPenTool>() else {
            return false;
        };
        // Outcome is intentionally discarded here — bridge picks up
        // `pending_committed` next frame, and Toast feedback for
        // Rejected / NoOpNearExistingVertex is W2 UX work.
        let _outcome = pen.on_canvas_click(Vec2::new(nx, ny));
        true
    }

    /// When Pen tool is active and Primary Down lands OFF-canvas (no
    /// selection or outside footprint), consume the event silently to
    /// match the Painter rule (don't fall through to selection /
    /// gizmo drag while a canvas-authoring tool owns the canvas).
    pub(crate) fn vector_pen_active_consume_canvas_click(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.tools.active())
            .map(|t| t.id() == ph2d_editor::ToolId::new("vector_pen"))
            .unwrap_or(false)
    }

    /// Resolve cursor screen-px → network-local pixel coordinates iff
    /// Pen is active + selection + inside footprint. Returns
    /// `(net_x, net_y)` in sprite-source-pixel space (matches the
    /// space Pen tool's `on_canvas_click` expects, per T1.5 R1 doc
    /// contract).
    ///
    /// Mirror of `painter_pointer_uv` minus the (u, v, sw, sh) tuple —
    /// the Pen tool wants explicit network coords, not normalized UV.
    fn vector_pen_pointer_xy(&self, px: f32, py: f32) -> Option<(f32, f32)> {
        let gfx = self.gfx.as_ref()?;
        let pen_active = gfx
            .tools
            .active()
            .map(|t| t.id() == ph2d_editor::ToolId::new("vector_pen"))
            .unwrap_or(false);
        if !pen_active {
            return None;
        }
        let hero = gfx.hero_screen.as_ref()?;
        let bits = hero.gizmo.selection?;
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let tr = gfx.sim.world().get::<Transform>(entity)?;
        let sprite = gfx.sim.world().get::<ph2d_render::Sprite>(entity)?;
        // Same footprint anchor convention as painter_input.rs:198-201
        // (Lens C / B-C1 audit precedent).
        let cx = tr.translation.x + sprite.anchor[0];
        let cy = tr.translation.y + sprite.anchor[1];
        let (sw, sh) = (sprite.size[0], sprite.size[1]);
        let window_size = gfx.surface.size();
        let (x0, y0) = gfx
            .camera
            .world_to_screen([cx - sw * 0.5, cy + sh * 0.5], window_size);
        let (x1, y1) = gfx
            .camera
            .world_to_screen([cx + sw * 0.5, cy - sh * 0.5], window_size);
        let (lo_x, hi_x) = (x0.min(x1), x0.max(x1));
        let (lo_y, hi_y) = (y0.min(y1), y0.max(y1));
        let (u, v) = Self::uv_from_bounds(px, py, lo_x, hi_x, lo_y, hi_y)?;
        // u/v ∈ [0, 1) over the sprite footprint → network-local px.
        Some((u * sw, v * sh))
    }
}

#[cfg(test)]
mod tests {
    // The `App` orchestration isn't unit-testable from the shell
    // (depends on winit / wgpu surface). The pure-fn `uv_from_bounds`
    // helper is already tested in `painter_input::tests`. The end-to-
    // end smoke is the Day-7 manual check: clica Pen pill → 3 cliques
    // no canvas → triângulo Vello → 4º clique close-path → asset
    // salva.
}
