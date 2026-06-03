//! Vector Pencil — pointer (drag) + keyboard input dispatch.
//!
//! Shell-only handlers that reach `VectorPencilTool` via
//! `gfx.tools.active_mut()` + downcast (ADR-0040 §3 documented exception,
//! mirror of `vector_pen_input.rs` / `painter_input.rs`).
//!
//! Unlike the Pen (discrete clicks), the Pencil is a **drag** gesture:
//! - Primary **Down** → `begin_stroke` at the world coord.
//! - Pointer **Move while pressed** → `extend_stroke` (records a sample).
//! - Primary **Up** → `finish_stroke` (decimate → Hobby smooth → commit).
//! - **Esc** → cancel the in-progress stroke, else clear the committed
//!   scene (shared with the Pen — in-memory only; AssetDb persistence is a
//!   later W2 Coord task).
//!
//! ## ⚠ Central wiring required (Coord) — see `docs/HANDOFF_vector_w2_pencil_coord.md`
//!
//! This module is not yet declared/called anywhere central. To activate:
//! 1. `shells/desktop/Cargo.toml`: add `ph2d-tool-vector-pencil` dep.
//! 2. `input_dispatch.rs`: `mod vector_pencil_input;` + route Primary
//!    Down/Move/Up + Esc to the methods below (drag state lives in the
//!    shell pointer FSM, alongside the Pen's click routing).
//! 3. The committed-scene list reused here is the Pen's existing
//!    `App::committed_vector_pen_paths` (a unified vector-scene list; a
//!    rename to `committed_vector_paths` is optional Coord cleanup).

use crate::App;
use crate::forwarding::cursor_over_hero_panel;
use ph2d_core::Vec2;
use ph2d_editor::toast::Toast;
use ph2d_tool_vector_pencil::{
    DEFAULT_MIN_SAMPLE_DISTANCE_PX, PencilStrokeOutcome, StrokeSample, VectorPencilTool,
};

impl App {
    /// `true` iff the Pencil tool is the active tool.
    fn vector_pencil_is_active(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.tools.active())
            .map(|t| t.id() == ph2d_editor::ToolId::new("vector_pencil"))
            .unwrap_or(false)
    }

    /// Convert a screen-pixel cursor position to world coords + reject
    /// chrome-owned pixels (a docked panel or any hit-indexed widget owns
    /// its area — same guard as the Pen). Returns `None` if not on the
    /// canvas / Pencil not active / no gfx.
    fn vector_pencil_world(&mut self, px: f32, py: f32) -> Option<Vec2> {
        if cursor_over_hero_panel(self.gfx.as_ref(), px, py) {
            return None;
        }
        if let Some(gfx) = self.gfx.as_ref()
            && let Some(hero) = gfx.hero_screen.as_ref()
            && hero.hit_index.hit(px, py).is_some()
        {
            return None;
        }
        if !self.vector_pencil_is_active() {
            return None;
        }
        let gfx = self.gfx.as_ref()?;
        let window_size = gfx.surface.size();
        let world = gfx.camera.screen_to_world((px, py), window_size);
        Some(Vec2::new(world[0], world[1]))
    }

    /// Primary Down — start recording a stroke. Returns `true` (consumes)
    /// iff the Pencil is active on the canvas.
    pub(crate) fn try_vector_pencil_pointer_down(&mut self, px: f32, py: f32) -> bool {
        let Some(world) = self.vector_pencil_world(px, py) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        // The near-duplicate reject radius is authored in screen pixels but
        // applied against WORLD chord lengths (extend_stroke / decimate), so
        // convert px→world at the current zoom (mirror of the Pen + Direct
        // tolerance fix). Without this the raw 1.5px reads as 1.5 world units
        // (~120 screen px at the default zoom) and a normal freehand stroke
        // is filtered down to <2 knots → "too short" / a crude straight line.
        let scale = (gfx.surface.size().height as f32) / gfx.camera.height_world.max(f32::EPSILON);
        if let Some(pencil) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorPencilTool>())
        {
            pencil.min_sample_distance_px = DEFAULT_MIN_SAMPLE_DISTANCE_PX / scale;
            pencil.begin_stroke(StrokeSample::point(world));
            return true;
        }
        false
    }

    /// Pointer Move while the primary button is held — record a sample.
    /// Returns `true` iff a stroke is being recorded (so the drag is the
    /// Pencil's, not a camera pan / marquee).
    pub(crate) fn try_vector_pencil_pointer_drag(&mut self, px: f32, py: f32) -> bool {
        let Some(world) = self.vector_pencil_world(px, py) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        if let Some(pencil) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorPencilTool>())
            && pencil.is_recording()
        {
            pencil.extend_stroke(StrokeSample::point(world));
            return true;
        }
        false
    }

    /// Primary Up — finish + commit the smoothed stroke. Returns `true`
    /// iff a stroke was being recorded.
    pub(crate) fn try_vector_pencil_pointer_up(&mut self) -> bool {
        if !self.vector_pencil_is_active() {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let outcome = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorPencilTool>())
            .filter(|p| p.is_recording())
            .map(VectorPencilTool::finish_stroke);
        let committed = matches!(outcome, Some(PencilStrokeOutcome::Committed));
        let result = match outcome {
            Some(PencilStrokeOutcome::Committed) => true,
            Some(PencilStrokeOutcome::TooShort) => {
                gfx.toasts
                    .push(Toast::info("Pencil stroke too short — draw a longer line"));
                true
            }
            _ => false,
        };
        // Snapshot the pre-commit scene (the bridge drains the stroke next
        // frame) so Ctrl+Z removes it.
        if committed {
            self.vector_checkpoint();
        }
        result
    }

    /// Escape while the Pencil is active: cancel the in-progress stroke;
    /// if none, clear the committed scene (shared with the Pen). Returns
    /// `true` only when it actually cancelled or cleared something.
    pub(crate) fn try_vector_pencil_escape(&mut self) -> bool {
        if !self.vector_pencil_is_active() {
            return false;
        }
        let cancelled = {
            let Some(gfx) = self.gfx.as_mut() else {
                return false;
            };
            match gfx
                .tools
                .active_mut()
                .and_then(|t| t.as_any_mut().downcast_mut::<VectorPencilTool>())
            {
                Some(pencil) if pencil.has_in_progress_stroke() => {
                    pencil.cancel_stroke();
                    true
                }
                _ => false,
            }
        };
        if cancelled {
            if let Some(gfx) = self.gfx.as_mut() {
                gfx.toasts.push(Toast::info("Pencil stroke cancelled"));
            }
            return true;
        }
        if !self.committed_vector_pen_paths.is_empty() {
            self.vector_checkpoint();
            self.committed_vector_pen_paths.clear();
            if let Some(gfx) = self.gfx.as_mut() {
                gfx.toasts.push(Toast::info("Vector scene cleared"));
            }
            return true;
        }
        false
    }

    /// When the Pencil is active the whole viewport is its canvas — guard
    /// the canvas-pick / rubber-band fall-through (sibling of
    /// `vector_pen_active_consume_canvas_click`).
    pub(crate) fn vector_pencil_active_consume_canvas_click(&self) -> bool {
        self.vector_pencil_is_active()
    }
}

#[cfg(test)]
mod tests {
    // `App` orchestration isn't unit-testable from the shell (winit / wgpu
    // surface). `Camera2d::screen_to_world` is unit-tested in
    // `ph2d-render::camera`; the Pencil tool's record/decimate/smooth/
    // commit logic is covered in `ph2d-tool-vector-pencil`. End-to-end is
    // the manual W2 Day-4 smoke: Pencil pill → drag a curve → release →
    // smoothed stroke persists; Esc cancels.
}
