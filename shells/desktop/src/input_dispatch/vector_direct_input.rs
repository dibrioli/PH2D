//! Vector Direct-Select — vertex/handle grab + drag + Escape input dispatch.
//!
//! Shell-only handlers reaching `VectorDirectTool` via `gfx.tools.active_mut()`
//! plus downcast (ADR-0040 §3 documented exception, mirror of
//! `vector_pencil_input.rs`).
//!
//! Direct-Select EDITS committed network geometry in place (it never appends):
//! - Primary **Down** → grab the nearest vertex / tangent within tolerance
//!   (records a Move op in that asset's `edit_log` → undo-able).
//! - Pointer **Move while pressed** → drag the grabbed handle. Alt breaks the
//!   tangent (`VertexKind::Free`, independent handles — Illustrator).
//! - Primary **Up** → end the grab (the logged op stays for undo).
//! - **Esc** → cancel an in-progress grab + clear the selection.
//!
//! `begin_drag` needs `&committed` + `&mut selection`, `drag_to` needs
//! `&mut committed`: the tool lives in `gfx.tools`, the scene + selection are
//! separate `App` fields, so they borrow simultaneously as disjoint fields.

use crate::App;
use crate::forwarding::cursor_over_hero_panel;
use ph2d_core::Vec2;
use ph2d_tool_vector_direct::{DEFAULT_GRAB_TOLERANCE_PX, DirectOutcome, VectorDirectTool};

impl App {
    fn vector_direct_is_active(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.tools.active())
            .map(|t| t.id() == ph2d_editor::ToolId::new("vector_direct"))
            .unwrap_or(false)
    }

    /// Screen px → world, rejecting chrome-owned pixels (same guard as Pencil).
    fn vector_direct_world(&self, px: f32, py: f32) -> Option<Vec2> {
        if cursor_over_hero_panel(self.gfx.as_ref(), px, py) {
            return None;
        }
        if let Some(gfx) = self.gfx.as_ref()
            && let Some(hero) = gfx.hero_screen.as_ref()
            && hero.hit_index.hit(px, py).is_some()
        {
            return None;
        }
        if !self.vector_direct_is_active() {
            return None;
        }
        let gfx = self.gfx.as_ref()?;
        let world = gfx.camera.screen_to_world((px, py), gfx.surface.size());
        Some(Vec2::new(world[0], world[1]))
    }

    /// Pixels-per-world-unit (mirror of the overlay bridge's `k`) — converts the
    /// screen-space grab tolerance to world units at the current zoom.
    fn vector_direct_camera_scale(&self) -> f32 {
        self.gfx
            .as_ref()
            .map(|g| (g.surface.size().height as f32) / g.camera.height_world.max(f32::EPSILON))
            .unwrap_or(1.0)
    }

    /// Primary Down — grab the nearest vertex/tangent within tolerance. Consumes
    /// iff Direct is active on canvas (a miss still consumes — Direct owns the
    /// canvas click so it never falls through to a sprite rubber-band).
    pub(crate) fn try_vector_direct_pointer_down(&mut self, px: f32, py: f32) -> bool {
        let Some(world) = self.vector_direct_world(px, py) else {
            return false;
        };
        let tol = DEFAULT_GRAB_TOLERANCE_PX / self.vector_direct_camera_scale();
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        // ADR-0076: per-shape inverse placements so the grab hit-tests each shape's
        // MOVED visual (gizmo-transformed), not its rest pose.
        let inv = crate::render_loop::vector_scene::inverse_placements(
            &gfx.sim,
            &self.vector_scene_entities,
        );
        let Some(direct) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorDirectTool>())
        else {
            return false;
        };
        // `direct` borrows `self.gfx.tools`; `committed` / `selection` are
        // disjoint `App` fields → simultaneous borrow is sound.
        let grabbed = matches!(
            direct.begin_drag(
                &self.committed_vector_pen_paths,
                &mut self.vector_selection,
                world,
                tol,
                &inv,
            ),
            DirectOutcome::Grabbed(_)
        );
        // A grab precedes an in-place vertex/tangent edit (drag_to next frames)
        // → snapshot the pre-edit scene so Ctrl+Z reverts it. Direct consumes
        // the canvas click regardless (it owns the canvas while active).
        if grabbed {
            self.vector_checkpoint();
        }
        true
    }

    /// Pointer Move while pressed — drag the grabbed handle. Alt breaks the
    /// tangent. `true` iff a grab is live.
    pub(crate) fn try_vector_direct_pointer_drag(&mut self, px: f32, py: f32) -> bool {
        let alt = self.modifiers.alt_key();
        let Some(world) = self.vector_direct_world(px, py) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let inv = crate::render_loop::vector_scene::inverse_placements(
            &gfx.sim,
            &self.vector_scene_entities,
        );
        if let Some(direct) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorDirectTool>())
            && direct.is_dragging()
        {
            direct.drag_to(&mut self.committed_vector_pen_paths, world, alt, &inv);
            return true;
        }
        false
    }

    /// Primary Up — end the grab (the Move op stays logged for undo). `true` iff
    /// a grab was live.
    pub(crate) fn try_vector_direct_pointer_up(&mut self) -> bool {
        if !self.vector_direct_is_active() {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        if let Some(direct) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorDirectTool>())
            && direct.is_dragging()
        {
            direct.end_drag();
            return true;
        }
        false
    }

    /// Escape — cancel an in-progress grab + clear the selection. `true` iff
    /// something was cancelled/cleared.
    pub(crate) fn try_vector_direct_escape(&mut self) -> bool {
        if !self.vector_direct_is_active() {
            return false;
        }
        let mut acted = false;
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(direct) = gfx
                .tools
                .active_mut()
                .and_then(|t| t.as_any_mut().downcast_mut::<VectorDirectTool>())
            && direct.is_dragging()
        {
            direct.cancel_drag();
            acted = true;
        }
        if !self.vector_selection.is_empty() {
            self.vector_selection.clear();
            acted = true;
        }
        acted
    }

    /// The whole viewport is Direct's canvas — guard the canvas-pick / sprite
    /// rubber-band fall-through (sibling of `vector_pencil_active_consume_…`).
    pub(crate) fn vector_direct_active_consume_canvas_click(&self) -> bool {
        self.vector_direct_is_active()
    }
}

#[cfg(test)]
mod tests {
    // `App` orchestration isn't unit-testable from the shell (winit / wgpu
    // surface). `Camera2d::screen_to_world` is unit-tested in
    // `ph2d-render::camera`; the Direct tool's grab / drag / Move-op logging is
    // covered in `ph2d-tool-vector-direct`. End-to-end is the W2 Day-11 manual
    // smoke: Direct pill → drag a vertex → geometry moves → Alt-drag breaks the
    // tangent → Esc clears.
}
