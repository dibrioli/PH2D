//! Vector Shape — pointer (drag) + keyboard input dispatch.
//!
//! Shell-only handlers reaching `VectorShapeTool` via
//! `gfx.tools.active_mut()` + downcast (ADR-0040 §3 exception, mirror of
//! `vector_pencil_input.rs`). Drag gesture:
//! - Primary **Down** → `begin_shape`.
//! - Pointer **Move while pressed** → `update_shape`.
//! - Primary **Up** → `finish_shape` (commit the configured primitive).
//! - **Esc** → cancel the in-progress drag, else clear the committed scene.
//!
//! ## ⚠ Central wiring required (Coord) — see `docs/HANDOFF_vector_w2_shape_coord.md`
//!
//! Identical wiring to the Pencil (`69b3788`): `Cargo.toml` dep,
//! `input_dispatch.rs` `mod` + Down/Move/Up routing, `keyboard.rs` Esc.
//! Reuses the shared `App::committed_vector_pen_paths` scene list.

use crate::App;
use crate::forwarding::cursor_over_hero_panel;
use ph2d_core::Vec2;
use ph2d_editor::toast::Toast;
use ph2d_tool_vector_shape::{ShapeKind, ShapeOutcome, VectorShapeTool};

impl App {
    fn vector_shape_is_active(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.tools.active())
            .map(|t| t.id() == ph2d_editor::ToolId::new("vector_shape"))
            .unwrap_or(false)
    }

    /// Screen px → world, rejecting chrome-owned pixels. `None` if not on
    /// the canvas / Shape not active.
    fn vector_shape_world(&mut self, px: f32, py: f32) -> Option<Vec2> {
        if cursor_over_hero_panel(self.gfx.as_ref(), px, py) {
            return None;
        }
        if let Some(gfx) = self.gfx.as_ref()
            && let Some(hero) = gfx.hero_screen.as_ref()
            && hero.hit_index.hit(px, py).is_some()
        {
            return None;
        }
        if !self.vector_shape_is_active() {
            return None;
        }
        let gfx = self.gfx.as_ref()?;
        let window_size = gfx.surface.size();
        let world = gfx.camera.screen_to_world((px, py), window_size);
        Some(Vec2::new(world[0], world[1]))
    }

    /// Primary Down — start the drag. Returns `true` iff Shape is active on
    /// the canvas.
    pub(crate) fn try_vector_shape_pointer_down(&mut self, px: f32, py: f32) -> bool {
        let Some(world) = self.vector_shape_world(px, py) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        if let Some(shape) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorShapeTool>())
        {
            shape.begin_shape(world);
            return true;
        }
        false
    }

    /// Pointer Move while pressed — update the drag end-point. Returns
    /// `true` iff a drag is in progress.
    pub(crate) fn try_vector_shape_pointer_drag(&mut self, px: f32, py: f32) -> bool {
        let Some(world) = self.vector_shape_world(px, py) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        if let Some(shape) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorShapeTool>())
            && shape.has_in_progress_shape()
        {
            shape.update_shape(world);
            return true;
        }
        false
    }

    /// Primary Up — commit the shape. Returns `true` iff a drag was in
    /// progress.
    pub(crate) fn try_vector_shape_pointer_up(&mut self) -> bool {
        if !self.vector_shape_is_active() {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let outcome = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorShapeTool>())
            .filter(|s| s.has_in_progress_shape())
            .map(VectorShapeTool::finish_shape);
        match outcome {
            Some(ShapeOutcome::Committed) => true,
            Some(ShapeOutcome::TooSmall) => {
                gfx.toasts
                    .push(Toast::info("Shape too small — drag to size it"));
                true
            }
            _ => false,
        }
    }

    /// Escape while Shape is active: cancel the drag; if none, clear the
    /// committed scene (shared with Pen/Pencil).
    pub(crate) fn try_vector_shape_escape(&mut self) -> bool {
        if !self.vector_shape_is_active() {
            return false;
        }
        let cancelled = {
            let Some(gfx) = self.gfx.as_mut() else {
                return false;
            };
            match gfx
                .tools
                .active_mut()
                .and_then(|t| t.as_any_mut().downcast_mut::<VectorShapeTool>())
            {
                Some(shape) if shape.has_in_progress_shape() => {
                    shape.cancel_shape();
                    true
                }
                _ => false,
            }
        };
        if cancelled {
            if let Some(gfx) = self.gfx.as_mut() {
                gfx.toasts.push(Toast::info("Shape cancelled"));
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

    /// Pick the active sub-mode by [`ShapeKind::ALL`] index (0-4) — driven by
    /// the number-key hotkeys 1-5 while Shape is active. Returns `true`
    /// (consumes the key) iff Shape is active and `index` is valid. This is
    /// the functional path to all five sub-modes; the on-screen picker is the
    /// end-of-implementation chrome polish (the retired floating panel is not
    /// revived — ADR/decision 2026-05-17).
    pub(crate) fn try_vector_shape_set_kind(&mut self, index: usize) -> bool {
        if !self.vector_shape_is_active() {
            return false;
        }
        let Some(&kind) = ShapeKind::ALL.get(index) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        if let Some(shape) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorShapeTool>())
        {
            shape.set_kind(kind);
            gfx.toasts
                .push(Toast::info(format!("Shape: {}", kind.value())));
            return true;
        }
        false
    }

    /// Whole viewport is the Shape canvas — guard the canvas-pick / rubber-
    /// band fall-through (sibling of `vector_pencil_active_consume_canvas_click`).
    pub(crate) fn vector_shape_active_consume_canvas_click(&self) -> bool {
        self.vector_shape_is_active()
    }
}

#[cfg(test)]
mod tests {
    // `App` orchestration isn't unit-testable from the shell. The Shape
    // tool's drag/generate/commit logic is covered in
    // `ph2d-tool-vector-shape`; the generators in `ph2d-vector-doc`. E2E is
    // the W2 Day-8 smoke: Shape pill → pick sub-mode → drag → commit.
}
