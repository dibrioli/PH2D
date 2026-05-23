//! BgRemoval eyedropper — canvas colour sampling + swatch delete.
//!
//! All NEW input handling for the eyedropper feature lives in the SHELL
//! (deliberately — the architecture keeps core interaction dispatch
//! untouched). These helpers reach the active `BgRemovalTool` via
//! `gfx.tools.active_mut()` + downcast, the same pattern as
//! `render_loop::bgremoval_preview`. Extracted from `input_dispatch.rs`
//! to keep that file under the HR-18 LOC cap.

use crate::{App, Transform};

impl App {
    /// If the BgRemoval tool is active AND its eyedropper is armed AND
    /// `(px, py)` falls inside the selected sprite's on-screen
    /// footprint, sample the source colour there and append it to the
    /// tool's extra-colour list. Returns `true` when a sample was
    /// attempted (so the caller early-returns and skips the normal
    /// canvas pick / gizmo / selection logic — we must not move or
    /// deselect the sprite while sampling).
    ///
    /// On a successful sample we drop `self.bgremoval_preview` so the
    /// per-frame dispatch recomputes the on-canvas overlay next frame.
    /// `add_extra_color` flips the tool's `params_dirty` flag so the
    /// canvas-preview cache rebuilds on the same frame the swatch
    /// appears (ADR-0040 TG-B; previously the overlay went stale until
    /// an unrelated panel edit nudged it).
    pub(crate) fn try_eyedropper_sample(&mut self, px: f32, py: f32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let bgremoval_active = gfx
            .tools
            .active()
            .map(|t| t.id() == ph2d_editor::ToolId::new("bgremoval"))
            .unwrap_or(false);
        if !bgremoval_active {
            return false;
        }
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return false;
        };
        let Some(bits) = hero.gizmo.selection else {
            return false;
        };
        // Sprite on-screen footprint (mirrors bgremoval_preview.rs).
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let (Some(tr), Some(sprite)) = (
            gfx.sim.world().get::<Transform>(entity),
            gfx.sim.world().get::<ph2d_render::Sprite>(entity),
        ) else {
            return false;
        };
        let (tx, ty) = (tr.translation.x, tr.translation.y);
        let (sw, sh) = (sprite.size[0], sprite.size[1]);
        let window_size = gfx.surface.size();
        let (x0, y0) = gfx
            .camera
            .world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
        let (x1, y1) = gfx
            .camera
            .world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
        let (lo_x, hi_x) = (x0.min(x1), x0.max(x1));
        let (lo_y, hi_y) = (y0.min(y1), y0.max(y1));
        // Now check the tool is actually armed; if so the click is
        // ours (consume it) whether or not it lands on the sprite.
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(bg) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_bgremoval::BgRemovalTool>()
        else {
            return false;
        };
        if !bg.is_eyedropper_armed() {
            return false;
        }
        // Inside the footprint? Compute UV + sample.
        if hi_x > lo_x && hi_y > lo_y && px >= lo_x && px <= hi_x && py >= lo_y && py <= hi_y {
            let u = (px - lo_x) / (hi_x - lo_x);
            let v = (py - lo_y) / (hi_y - lo_y);
            if let Some(rgb) = bg.sample_source_at_uv(u, v) {
                bg.add_extra_color(rgb);
                // Force the on-canvas overlay to recompute next frame.
                self.bgremoval_preview = None;
            }
        }
        // Consumed regardless of in/out so the click doesn't move or
        // deselect the sprite while the eyedropper is armed.
        true
    }

    /// If the BgRemoval tool is active and `(px, py)` hits an
    /// extra-colour swatch in the panel, remove that colour and
    /// consume the secondary click (so it doesn't open a context
    /// menu). Returns `true` when consumed.
    pub(crate) fn try_eyedropper_delete(&mut self, px: f32, py: f32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let bgremoval_active = gfx
            .tools
            .active()
            .map(|t| t.id() == ph2d_editor::ToolId::new("bgremoval"))
            .unwrap_or(false);
        if !bgremoval_active {
            return false;
        }
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return false;
        };
        let Some(hit_id) = hero.hit_index.hit(px, py) else {
            return false;
        };
        let Some(idx) = ph2d_editor::ids::bgr_swatch_index(hit_id) else {
            return false;
        };
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        if let Some(bg) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_bgremoval::BgRemovalTool>()
        {
            bg.remove_extra_color(idx);
            self.bgremoval_preview = None;
            return true;
        }
        false
    }
}
