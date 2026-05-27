//! BgRemoval protection brush — freehand "keep" mask painting + erasing
//! on the canvas.
//!
//! All NEW input handling lives in the SHELL (like
//! `input_dispatch::eyedropper`), reaching the active `BgRemovalTool`
//! via `gfx.tools.active_mut()` + downcast. The brush paints a
//! forced-foreground mask the pipeline honours in both modes (GrabCut
//! `FgHard` seed + compose force-keep). Left-button drag paints; right-
//! button drag erases.
//!
//! Drag state (`is_protect_painting` / erase mode) lives on the tool
//! rather than on `App` — the shell's `App` struct is outside this
//! feature's edit surface, and the tool is the natural per-tool home.
//!
//! Performance: a dab only mutates the mask (cheap); it does NOT re-run
//! the segmentation pipeline. The on-canvas tint reads the mask live, so
//! the matte only re-segments once the stroke ends (`end_protect_paint`
//! drops the cached preview). This is what fixed the per-dab GrabCut
//! freeze.

use std::cell::Cell;

use crate::{App, Transform};
use ph2d_editor::tool::RasterEditTool;

thread_local! {
    /// Last cursor screen position while the protection brush is armed —
    /// published by the input hooks, read by the on-canvas overlay to
    /// draw the brush-size ring. `None` when the brush isn't armed.
    static BRUSH_CURSOR: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

/// Publish the brush ring cursor (screen px), or clear it.
pub(crate) fn set_brush_cursor(p: Option<(f32, f32)>) {
    BRUSH_CURSOR.with(|c| c.set(p));
}

/// Read the brush ring cursor (screen px) for the overlay gizmo.
pub(crate) fn brush_cursor() -> Option<(f32, f32)> {
    BRUSH_CURSOR.with(|c| c.get())
}

impl App {
    /// Primary-button protection dab at `(px, py)` — paint. Returns `true`
    /// (consumes the event) when the brush is armed, so the click doesn't
    /// move/deselect the sprite; `false` when not armed / no selection.
    pub(crate) fn try_protect_paint(&mut self, px: f32, py: f32) -> bool {
        self.protect_dab(px, py, false)
    }

    /// Secondary-button protection dab at `(px, py)` — erase. Same
    /// consume semantics as [`Self::try_protect_paint`].
    pub(crate) fn try_protect_erase(&mut self, px: f32, py: f32) -> bool {
        self.protect_dab(px, py, true)
    }

    /// Continue a protection-brush drag from a cursor-move event. Paints
    /// or erases (per the drag's mode) only when a drag is already in
    /// progress; returns `true` when it consumed the move.
    pub(crate) fn protect_drag_move(&mut self, px: f32, py: f32) -> bool {
        match self.protect_drag_mode() {
            Some(erase) => {
                self.protect_dab(px, py, erase);
                true
            }
            None => false,
        }
    }

    /// End any protection-brush drag (pointer-up): clear the drag flags
    /// and drop the cached preview ONCE so the matte re-segments with the
    /// final mask. No-op when no drag was in progress.
    pub(crate) fn end_protect_paint(&mut self) {
        let mut was_painting = false;
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(tool) = gfx.tools.active_mut()
            && let Some(bg) = tool
                .as_any_mut()
                .downcast_mut::<ph2d_tool_bgremoval::BgRemovalTool>()
        {
            was_painting = bg.is_protect_painting();
            bg.set_protect_painting(false);
            bg.set_protect_erasing(false);
        }
        // Re-segment the matte once, now that the stroke is complete.
        if was_painting {
            self.bgremoval_preview = None;
        }
    }

    /// Publish the brush-ring cursor for the overlay gizmo: the cursor
    /// position while the protection brush is armed, else `None`.
    /// Called every cursor-move (one cheap downcast). The "Add area"
    /// selector is single-click (no drag, no ring) — its cursor never
    /// shows a brush ring, so it's excluded from this check.
    pub(crate) fn update_protect_brush_cursor(&mut self, px: f32, py: f32) {
        let armed = self
            .gfx
            .as_mut()
            .and_then(|g| g.tools.active_mut())
            .and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_bgremoval::BgRemovalTool>()
            })
            .map(|bg| bg.is_protect_armed())
            .unwrap_or(false);
        set_brush_cursor(if armed { Some((px, py)) } else { None });
    }

    /// Drag state: `Some(erase)` while a protection drag is in progress,
    /// `None` otherwise.
    fn protect_drag_mode(&mut self) -> Option<bool> {
        let gfx = self.gfx.as_mut()?;
        let bg = gfx
            .tools
            .active_mut()?
            .as_any_mut()
            .downcast_mut::<ph2d_tool_bgremoval::BgRemovalTool>()?;
        if bg.is_protect_painting() {
            Some(bg.is_protect_erasing())
        } else {
            None
        }
    }

    /// Core dab: footprint hit-test + paint/erase + arm the drag. Returns
    /// `true` when the brush is armed (so the click is consumed).
    fn protect_dab(&mut self, px: f32, py: f32, erase: bool) -> bool {
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
            .downcast_mut::<ph2d_tool_bgremoval::BgRemovalTool>()
        else {
            return false;
        };
        if !bg.is_protect_armed() {
            return false;
        }
        // Inside the footprint? Compute UV + paint/erase.
        if hi_x > lo_x && hi_y > lo_y && px >= lo_x && px <= hi_x && py >= lo_y && py <= hi_y {
            let u = (px - lo_x) / (hi_x - lo_x);
            let v = (py - lo_y) / (hi_y - lo_y);
            // The brush radius is stored in SOURCE px (the unit the dab
            // expects) and driven by the panel Size slider.
            let radius_px = bg.brush_radius_px();
            if erase {
                bg.erase_protect_at_uv(u, v, radius_px);
            } else {
                bg.paint_protect_at_uv(u, v, radius_px);
            }
            // NOTE: no `self.bgremoval_preview = None` here — the tint
            // overlay reads the mask live, and re-segmenting per dab is
            // what froze the GrabCut path. The matte re-runs once on
            // pointer-up (`end_protect_paint`).
        }
        // Mark the drag in progress + its mode; consume regardless of
        // in/out so the click doesn't move or deselect the sprite.
        bg.set_protect_painting(true);
        bg.set_protect_erasing(erase);
        true
    }

    // ── "Add area" automatic selector (Enio 2026-05-26) ──
    // Symmetric to the eyedropper: single click on canvas → flood-fill
    // from that source pixel into the force-remove mask. No drag, no
    // brush, no per-pixel painting — the algorithm picks the connected
    // same-colour region and writes it. Mirror of `try_eyedropper_sample`
    // in [`eyedropper.rs`] structurally; differs only in WHICH tool API
    // is called once the click is mapped to a source UV.

    /// If the BgRemoval tool is active AND the "Add area" selector is
    /// armed AND `(px, py)` falls inside the selected sprite's
    /// on-screen footprint, run a flood-fill from that source pixel
    /// into the force-remove mask + drop the cached preview so the
    /// next bridge tick re-segments with the new mask. Returns `true`
    /// when a click was consumed (so the caller skips the normal
    /// canvas / gizmo / selection logic) — including the case where
    /// the click landed outside the footprint, mirroring the
    /// eyedropper's "consume regardless of in/out" rule so a stray
    /// click can't move / deselect the sprite while the selector is
    /// armed.
    pub(crate) fn try_add_area_click(&mut self, px: f32, py: f32) -> bool {
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
        // Bail when the click lands on ANY published panel rect — the
        // selector consumes the event "regardless of in/out" of the
        // sprite to keep canvas/gizmo clicks from leaking, but that
        // also swallowed Down events headed for sliders inside the
        // BgRemoval panel (mirror of the eyedropper's panel-guard,
        // same Enio 2026-05-26 cause).
        if hero.store.panel_at(px, py).is_some() {
            return false;
        }
        let Some(bits) = hero.gizmo.selection else {
            return false;
        };
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
            .downcast_mut::<ph2d_tool_bgremoval::BgRemovalTool>()
        else {
            return false;
        };
        if !bg.is_add_area_armed() {
            return false;
        }
        // Inside the footprint? Compute source UV + flood-fill.
        if hi_x > lo_x && hi_y > lo_y && px >= lo_x && px <= hi_x && py >= lo_y && py <= hi_y {
            let u = (px - lo_x) / (hi_x - lo_x);
            let v = (py - lo_y) / (hi_y - lo_y);
            bg.flood_fill_remove_at_uv(u, v);
            // Drain the freshly-regenerated preview RIGHT HERE
            // (Enio 2026-05-27 "o efeito só apareceu depois que
            // desmarquei add area"). Without this, the Lens-F GPU
            // lifecycle in `bgremoval_preview::dispatch` runs against
            // a `bgremoval_preview = None` if the canvas-click frame's
            // `drive_preview_cache` happens to race with another
            // canvas-click effect, releasing the GPU texture before
            // the new preview is uploaded; the next frame's
            // `sim_extract` then renders the original sprite instead
            // of the live preview, and the effect only becomes
            // visible once a SECOND `params_dirty` event (e.g. un-
            // arming the toggle) forces `drive_preview_cache` to run
            // again. Eagerly draining here populates the shell cache
            // synchronously inside the input handler, so the next
            // dispatch's GPU lifecycle always sees `Some(preview)` and
            // uploads the up-to-date texture.
            if let Some((pixels, w, h)) = bg.current_preview() {
                self.bgremoval_preview = Some(ph2d_tool_runtime::PreviewCache {
                    entity_bits: bits,
                    rgba: std::sync::Arc::new(pixels.to_vec()),
                    width: w,
                    height: h,
                });
            } else {
                self.bgremoval_preview = None;
            }
        }
        // Consumed regardless of in/out so the click doesn't move or
        // deselect the sprite while the selector is armed.
        true
    }
}
