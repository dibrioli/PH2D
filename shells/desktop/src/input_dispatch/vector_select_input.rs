//! Vector Select — pointer (click + marquee drag) + Escape input dispatch.
//!
//! Shell-only handlers reaching `VectorSelectTool` via `gfx.tools.active_mut()`
//! plus downcast (ADR-0040 §3 documented exception, mirror of
//! `vector_pencil_input.rs`).
//!
//! Unlike the create-tools (Pen/Pencil/Shape, which APPEND), Select EDITS the
//! shared [`VectorSelection`] over the committed scene and never mutates
//! geometry:
//! - Primary **Down** → anchor a marquee at the world coord.
//! - Pointer **Move while pressed** → grow the marquee.
//! - Primary **Up** → a sub-threshold marquee resolves to a CLICK (point-select
//!   the topmost network); otherwise a crossing-marquee select. Shift = add.
//! - **Esc** → cancel the marquee + clear the selection.
//!
//! The tool lives in `gfx.tools`; the committed scene + selection are separate
//! `App` fields, so the methods that need all three (`click` / `finish_marquee`)
//! borrow them simultaneously as disjoint fields.

use crate::App;
use crate::forwarding::cursor_over_hero_panel;
use ph2d_core::Vec2;
use ph2d_tool_vector_select::VectorSelectTool;

/// Below this screen-space marquee diagonal an Up is a click (point-select the
/// topmost network) rather than a crossing-marquee select.
const VECTOR_SELECT_CLICK_PX: f32 = 3.0;

impl App {
    fn vector_select_is_active(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.tools.active())
            .map(|t| t.id() == ph2d_editor::ToolId::new("vector_select"))
            .unwrap_or(false)
    }

    /// Screen px → world, rejecting chrome-owned pixels (same guard as Pencil).
    fn vector_select_world(&self, px: f32, py: f32) -> Option<Vec2> {
        if cursor_over_hero_panel(self.gfx.as_ref(), px, py) {
            return None;
        }
        if let Some(gfx) = self.gfx.as_ref()
            && let Some(hero) = gfx.hero_screen.as_ref()
            && hero.hit_index.hit(px, py).is_some()
        {
            return None;
        }
        if !self.vector_select_is_active() {
            return None;
        }
        let gfx = self.gfx.as_ref()?;
        let world = gfx.camera.screen_to_world((px, py), gfx.surface.size());
        Some(Vec2::new(world[0], world[1]))
    }

    /// Pixels-per-world-unit (mirror of the overlay bridge's `k`) for converting
    /// the screen-space click threshold to a world-space marquee diagonal.
    fn vector_select_camera_scale(&self) -> f32 {
        self.gfx
            .as_ref()
            .map(|g| (g.surface.size().height as f32) / g.camera.height_world.max(f32::EPSILON))
            .unwrap_or(1.0)
    }

    /// Primary Down — anchor a marquee. Consumes iff Select is active on canvas.
    pub(crate) fn try_vector_select_pointer_down(&mut self, px: f32, py: f32) -> bool {
        let Some(world) = self.vector_select_world(px, py) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        if let Some(sel) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorSelectTool>())
        {
            sel.begin_marquee(world);
            return true;
        }
        false
    }

    /// Pointer Move while pressed — grow the marquee. `true` iff marqueeing.
    pub(crate) fn try_vector_select_pointer_drag(&mut self, px: f32, py: f32) -> bool {
        let Some(world) = self.vector_select_world(px, py) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        if let Some(sel) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorSelectTool>())
            && sel.is_marqueeing()
        {
            sel.update_marquee(world);
            return true;
        }
        false
    }

    /// Primary Up — resolve the gesture. A sub-threshold marquee is a click
    /// (point-select the topmost network at the anchor); otherwise a crossing-
    /// marquee select. Shift extends. `true` iff a marquee was open.
    pub(crate) fn try_vector_select_pointer_up(&mut self) -> bool {
        if !self.vector_select_is_active() {
            return false;
        }
        let additive = self.modifiers.shift_key();
        let k = self.vector_select_camera_scale();
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(sel) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorSelectTool>())
        else {
            return false;
        };
        let Some((min, max)) = sel.marquee_rect() else {
            return false;
        };
        // `sel` borrows `self.gfx.tools`. Cancel the tool-side marquee; we resolve
        // BOTH the click and the marquee PLACEMENT-AWARE below (the tool's own
        // hit-tests are rest-pose, so they'd select a gizmo-moved shape at its old
        // position). `sel`'s borrow ends here.
        let is_click = (max - min).length() * k < VECTOR_SELECT_CLICK_PX;
        sel.cancel_marquee();
        let Some(gfx) = self.gfx.as_ref() else {
            return true;
        };
        if is_click {
            // A click point-selects the topmost MOVED shape under the cursor (or
            // clears on empty). Shift toggles.
            match crate::render_loop::vector_scene::pick_index(
                &gfx.sim,
                &self.vector_scene_entities,
                &self.committed_vector_pen_paths,
                [min.x, min.y],
            ) {
                Some(idx) if additive => {
                    if self.vector_selection.contains_network(idx) {
                        self.vector_selection.networks.retain(|&n| n != idx);
                    } else {
                        self.vector_selection.networks.push(idx);
                    }
                }
                Some(idx) => self.vector_selection.select_only_network(idx),
                None if !additive => self.vector_selection.clear(),
                None => {}
            }
        } else {
            // Crossing marquee over the shapes' MOVED world AABBs. Shift extends.
            let hits = crate::render_loop::vector_scene::marquee_hits(
                &gfx.sim,
                &self.vector_scene_entities,
                &self.committed_vector_pen_paths,
                [min.x, min.y],
                [max.x, max.y],
            );
            if additive {
                for h in hits {
                    if !self.vector_selection.networks.contains(&h) {
                        self.vector_selection.networks.push(h);
                    }
                }
            } else {
                self.vector_selection.networks = hits;
                self.vector_selection.vertices.clear();
            }
        }
        true
    }

    /// Escape — cancel an in-progress marquee + clear the selection. `true` iff
    /// something was cancelled/cleared.
    pub(crate) fn try_vector_select_escape(&mut self) -> bool {
        if !self.vector_select_is_active() {
            return false;
        }
        let mut acted = false;
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(sel) = gfx
                .tools
                .active_mut()
                .and_then(|t| t.as_any_mut().downcast_mut::<VectorSelectTool>())
            && sel.is_marqueeing()
        {
            sel.cancel_marquee();
            acted = true;
        }
        if !self.vector_selection.is_empty() {
            self.vector_selection.clear();
            acted = true;
        }
        acted
    }

    /// The whole viewport is Select's canvas — guard the canvas-pick / sprite
    /// rubber-band fall-through (sibling of `vector_pencil_active_consume_…`).
    pub(crate) fn vector_select_active_consume_canvas_click(&self) -> bool {
        self.vector_select_is_active()
    }
}

#[cfg(test)]
mod tests {
    // `App` orchestration isn't unit-testable from the shell (winit / wgpu
    // surface). `Camera2d::screen_to_world` is unit-tested in
    // `ph2d-render::camera`; the Select tool's click / marquee logic is covered
    // in `ph2d-tool-vector-select`. End-to-end is the W2 Day-11 manual smoke:
    // Select pill → click a network selects it → marquee selects crossing →
    // Esc clears.
}
