//! BgRemoval protection brush — freehand "keep" mask painting on the
//! canvas.
//!
//! All NEW input handling lives in the SHELL (like
//! `input_dispatch::eyedropper`), reaching the active `BgRemovalTool`
//! via `gfx.tools.active_mut()` + downcast. The brush paints a
//! forced-foreground mask the pipeline honours in both modes (GrabCut
//! `FgHard` seed + compose force-keep).
//!
//! Drag handling mirrors the eyedropper, but the "painting in progress"
//! flag lives on the tool (`is_protect_painting` / `set_protect_painting`)
//! rather than on `App` — the shell's `App` struct is outside this
//! feature's edit surface, and the tool is the natural per-tool home
//! for the transient state.

use crate::{App, Transform};

/// On-screen brush radius (px). Mapped into source pixels per the
/// sprite's on-screen footprint scale before painting, so the brush
/// feels the same size regardless of zoom / source resolution.
/// // LITERAL-PX-OK: input brush metric
const BRUSH_SCREEN_PX: f32 = 22.0;

impl App {
    /// Begin / continue a protection-brush dab at `(px, py)`. When the
    /// BgRemoval tool is active AND its protection brush is armed, paints
    /// a dab into the mask (if the point is inside the selected sprite
    /// footprint), arms the drag, and returns `true` so the caller
    /// consumes the event (we must not move/deselect the sprite while
    /// painting) — armed in or out of bounds. Returns `false` when the
    /// brush is not armed / no sprite is selected.
    pub(crate) fn try_protect_paint(&mut self, px: f32, py: f32) -> bool {
        self.protect_dab(px, py)
    }

    /// Continue a protection-brush drag from a cursor-move event. Paints
    /// only when a drag is already in progress (set on pointer-down);
    /// returns `true` when it consumed the move.
    pub(crate) fn protect_drag_move(&mut self, px: f32, py: f32) -> bool {
        if !self.is_protect_painting() {
            return false;
        }
        self.protect_dab(px, py);
        true
    }

    /// End any protection-brush drag (pointer-up). No-op when not armed.
    pub(crate) fn end_protect_paint(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        if let Some(tool) = gfx.tools.active_mut()
            && let Some(bg) = tool
                .as_any_mut()
                .downcast_mut::<ph2d_editor::tools::bgremoval::BgRemovalTool>()
        {
            bg.set_protect_painting(false);
        }
    }

    /// Whether a protection-brush drag is currently in progress.
    fn is_protect_painting(&mut self) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        gfx.tools
            .active_mut()
            .and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_editor::tools::bgremoval::BgRemovalTool>()
            })
            .map(|bg| bg.is_protect_painting())
            .unwrap_or(false)
    }

    /// Core dab: footprint hit-test + paint + arm the drag. Returns
    /// `true` when the brush is armed (so the click is consumed).
    fn protect_dab(&mut self, px: f32, py: f32) -> bool {
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
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(bg) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_editor::tools::bgremoval::BgRemovalTool>()
        else {
            return false;
        };
        if !bg.is_protect_armed() {
            return false;
        }
        // Inside the footprint? Compute UV + paint.
        if hi_x > lo_x && hi_y > lo_y && px >= lo_x && px <= hi_x && py >= lo_y && py <= hi_y {
            let u = (px - lo_x) / (hi_x - lo_x);
            let v = (py - lo_y) / (hi_y - lo_y);
            // Map the on-screen brush radius into SOURCE pixels — the
            // unit `paint_protect_at_uv` expects.
            let (src_w, _src_h) = bg.source_size();
            let footprint_w = hi_x - lo_x;
            let radius_px = if footprint_w > 0.0 && src_w > 0 {
                BRUSH_SCREEN_PX * (src_w as f32) / footprint_w
            } else {
                BRUSH_SCREEN_PX
            };
            bg.paint_protect_at_uv(u, v, radius_px);
            // Force the on-canvas overlay to recompute next frame
            // (painting pushes no `BgremovalUiEdit`). `bgremoval_preview`
            // is a disjoint `App` field from `self.gfx`, so this write is
            // legal while `bg` (borrowed via `gfx`) is live.
            self.bgremoval_preview = None;
        }
        // Mark the drag in progress + consume regardless of in/out so the
        // click doesn't move or deselect the sprite while painting.
        bg.set_protect_painting(true);
        true
    }
}
