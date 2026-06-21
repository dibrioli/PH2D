//! Canvas painting input (SHELL-only) — routes Primary pointer Down/Move/Up to
//! the active [`PainterTool`] as `CanvasPaintTool::on_canvas_pointer` (ADR-0040
//! Amendment 3), converting screen → image-space pixels via the selected sprite's
//! on-screen footprint.
//!
//! Mirrors the bgremoval protection-brush plumbing (`protect_brush.rs`): gate on
//! the painter being active + a selected sprite, AABB footprint hit-test
//! (`world_to_screen` of the sprite corners — matches `bgremoval_preview.rs`),
//! UV → image px via the painter's `canvas_size`. The concrete downcast to
//! [`PainterTool`] is the documented ADR-0040 §3 exception (same shape as
//! protect_brush / eyedropper), here used to read `canvas_size` while delivering.
//!
//! Limitations (Fase 0b): the footprint AABB ignores sprite rotation (parity with
//! the protect brush); desktop CursorMoved has no pen pressure, so Move/Up deliver
//! pressure 1.0 (real pressure arrives on the iPad shell). Both are follow-ups.

use std::cell::Cell;

use ph2d_editor::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_tool_painter::PainterTool;

use crate::App;
use crate::Transform;

thread_local! {
    /// `true` between a consumed painter Down and the matching Up — so CursorMoved
    /// keeps feeding the open stroke and the Up arm knows to close it.
    static STROKE_ACTIVE: Cell<bool> = const { Cell::new(false) };
}

impl App {
    /// Nudge the active Painter brush radius — `[` (`dir < 0`) shrinks, `]`
    /// (`dir >= 0`) grows (Blender/Photoshop convention). Returns `true` when
    /// consumed (the active tool IS the Painter), so the bracket key doesn't fall
    /// through to other handlers.
    pub(crate) fn painter_nudge_brush_size(&mut self, dir: i32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
            return false;
        };
        painter.nudge_brush_size(dir);
        true
    }

    /// Toggle the active Painter brush's eraser mode (`E`). Returns `true` when
    /// consumed (the active tool IS the Painter), so `E` falls through otherwise.
    pub(crate) fn painter_toggle_eraser(&mut self) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
            return false;
        };
        painter.toggle_brush_eraser();
        true
    }

    /// Borrow the active tool as the concrete [`PainterTool`] (the ADR-0040 §3
    /// downcast exception, allowlisted to this painter-input module). `None` when
    /// the Painter is not the active tool.
    fn painter_tool_mut(&mut self) -> Option<&mut PainterTool> {
        self.gfx
            .as_mut()?
            .tools
            .active_mut()?
            .as_any_mut()
            .downcast_mut::<PainterTool>()
    }

    /// Primary Down on the Falloff curve graph's empty canvas (Custom preset):
    /// add a control point where the artist clicked and select it. Returns `true`
    /// (consuming) only when a point was added — a press on an existing handle
    /// (`FalloffHit::Point`) returns `false` so the panel's drag dispatch grabs
    /// it, and an off-canvas press also falls through.
    pub(crate) fn painter_falloff_canvas_add(&mut self, px: f32, py: f32) -> bool {
        let ph2d_panel_painter_layers::FalloffHit::Canvas(nx, ny) =
            ph2d_panel_painter_layers::falloff_hit_test(px, py)
        else {
            return false;
        };
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        if let Some(id) = painter.add_brush_falloff_point_at(nx, ny) {
            ph2d_panel_painter_layers::set_selected_falloff_point(Some(id));
            return true;
        }
        false
    }

    /// Secondary Down on a Falloff curve control point (Custom preset): select it
    /// and open the handle-type menu (Vector / Auto) at the cursor. The chrome
    /// handler parks the choice in `HeroScreen.pending_falloff_point_handle`;
    /// [`Self::painter_apply_pending_falloff_handle`] drains it. Returns `true`
    /// (consuming) iff a point was hit.
    pub(crate) fn painter_falloff_open_point_menu(&mut self, px: f32, py: f32) -> bool {
        let ph2d_panel_painter_layers::FalloffHit::Point(id) =
            ph2d_panel_painter_layers::falloff_hit_test(px, py)
        else {
            return false;
        };
        // Only meaningful while the Painter owns the active tool.
        if self.painter_tool_mut().is_none() {
            return false;
        }
        ph2d_panel_painter_layers::set_selected_falloff_point(Some(id));
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            hero.store
                .open_context_menu(ph2d_editor::interaction::ContextMenuRequest {
                    x: px,
                    y: py,
                    kind: ph2d_editor::interaction::ContextMenuKind::FalloffPointHandle,
                });
        }
        true
    }

    /// Delete/Backspace: remove the selected Falloff control point. Returns `true`
    /// (consuming) iff a point was selected and dropped, so the key falls through
    /// otherwise (it must not eat Delete for other tools).
    pub(crate) fn painter_delete_selected_falloff_point(&mut self) -> bool {
        let Some(id) = ph2d_panel_painter_layers::selected_falloff_point() else {
            return false;
        };
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        painter.remove_brush_falloff_point(id);
        ph2d_panel_painter_layers::set_selected_falloff_point(None);
        true
    }

    /// Primary Down on the canvas while the Painter is active: convert to image
    /// space and deliver as [`PointerPhase::Down`]. Returns `true` (consuming the
    /// click) iff the painter started a stroke, so it doesn't also pick/move the
    /// sprite.
    pub(crate) fn painter_canvas_down(&mut self, px: f32, py: f32, pressure: f32) -> bool {
        // A press over a docked panel (Layers / Brush properties) belongs to the
        // panel's own widgets — sliders, colour picker, buttons. Don't start a
        // canvas stroke there: that painted *through* the panel and stole the
        // slider drag (the stroke's Move capture swallowed the drag). Mirror of
        // the gizmo/canvas-pick gate (`store.panel_at(..).is_none()`). A stroke
        // already open keeps painting over the panel via the Move path (it does
        // not re-enter here).
        let over_panel = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.panel_at(px, py).is_some());
        if over_panel {
            return false;
        }
        let started = self.deliver_canvas_pointer(px, py, pressure, PointerPhase::Down);
        if started {
            STROKE_ACTIVE.with(|s| s.set(true));
        }
        started
    }

    /// CursorMoved while a painter stroke is open: deliver as [`PointerPhase::Move`].
    /// Returns `true` (early-returning the move) while a stroke is active.
    pub(crate) fn painter_canvas_move(&mut self, px: f32, py: f32) -> bool {
        if !STROKE_ACTIVE.with(Cell::get) {
            return false;
        }
        let delivered = self.deliver_canvas_pointer(px, py, 1.0, PointerPhase::Move);
        if !delivered {
            // Painter no longer active / selection gone — abandon the stroke so we
            // stop swallowing moves.
            STROKE_ACTIVE.with(|s| s.set(false));
        }
        delivered
    }

    /// Primary Up: close any open painter stroke. No-op when not painting.
    pub(crate) fn painter_canvas_up(&mut self) {
        if !STROKE_ACTIVE.with(Cell::get) {
            return;
        }
        STROKE_ACTIVE.with(|s| s.set(false));
        let (px, py) = self.last_pointer;
        self.deliver_canvas_pointer(px, py, 1.0, PointerPhase::Up);
    }

    /// Gate on painter-active + a selected sprite, convert screen→image pixels via
    /// the sprite footprint, and deliver one [`CanvasPointer`]. Returns the tool's
    /// `consumed` result, or `false` when the sample was not delivered (not the
    /// painter / no selection / Down outside the footprint).
    fn deliver_canvas_pointer(
        &mut self,
        px: f32,
        py: f32,
        pressure: f32,
        phase: PointerPhase,
    ) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let painter_active = gfx
            .tools
            .active()
            .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
            .unwrap_or(false);
        if !painter_active {
            return false;
        }
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return false;
        };
        let Some(bits) = hero.gizmo.selection else {
            return false;
        };
        // Sprite on-screen footprint (mirrors bgremoval_preview.rs / protect_brush).
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
        if !(hi_x > lo_x && hi_y > lo_y) {
            return false;
        }
        // A Down only starts a stroke when inside the footprint (outside clicks
        // fall through to pan / selection); Move/Up always reach an open stroke so
        // you can paint to and past the edge.
        let inside = px >= lo_x && px <= hi_x && py >= lo_y && py <= hi_y;
        if phase == PointerPhase::Down && !inside {
            return false;
        }
        let u = (px - lo_x) / (hi_x - lo_x);
        let v = (py - lo_y) / (hi_y - lo_y);
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
            return false;
        };
        let (iw, ih) = painter.canvas_size();
        if iw == 0 || ih == 0 {
            return false;
        }
        let ev = CanvasPointer {
            pos: [u * iw as f32, v * ih as f32],
            pressure,
            tilt: [0.0, 0.0],
            phase,
        };
        painter.on_canvas_pointer(ev)
    }
}
