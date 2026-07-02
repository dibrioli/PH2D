//! Canvas painting input (SHELL-only) — routes Primary pointer Down/Move/Up to
//! the active [`PainterTool`] as `CanvasPaintTool::on_canvas_pointer` (ADR-0040
//! Amendment 3), converting screen → image-space pixels via the selected sprite's
//! on-screen footprint.
//!
//! Mirrors the bgremoval protection-brush plumbing (`protect_brush.rs`): gate on
//! the painter being active + a selected sprite, then map screen → image px with
//! the FULL sprite affine (`bgremoval_preview::sprite_image_to_screen_affine` —
//! size · scale · rotation · anchor · camera), so the brush tracks the sprite
//! under any transform. The concrete downcast to [`PainterTool`] is the documented
//! ADR-0040 §3 exception (same shape as protect_brush / eyedropper), here used to
//! read `canvas_size` + deliver the pointer.
//!
//! Limitation: desktop CursorMoved has no pen pressure, so Move/Up deliver
//! pressure 1.0 (real pressure arrives on the iPad shell) — a follow-up.

use std::cell::Cell;

use ph2d_editor::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_tool_painter::PainterTool;

use crate::App;
use crate::Transform;

/// Shape-editor (Curve/Circle) control-handle grab radius in SCREEN px — scaled to image px by the
/// sprite footprint before it's forwarded to the tool, so the hit target stays a constant on-screen
/// size at any zoom.
const SHAPE_GRAB_TOL_SCREEN_PX: f32 = 10.0;

thread_local! {
    /// `true` between a consumed painter Down and the matching Up — so CursorMoved
    /// keeps feeding the open stroke and the Up arm knows to close it.
    static STROKE_ACTIVE: Cell<bool> = const { Cell::new(false) };

    /// The Falloff control point being dragged by a shell-owned gesture (the stable
    /// id), `None` when idle. Set when a left-click on the EMPTY graph adds a point
    /// (so the same press immediately drags it off the line — a corner needs a
    /// non-collinear point); CursorMoved drags it; Primary Up clears it. Existing
    /// handles keep using the panel's `CurvePoint` drag.
    static FALLOFF_DRAG: Cell<Option<u8>> = const { Cell::new(None) };
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
    /// add a control point where clicked, select it, and GRAB it so the same
    /// gesture drags it (a corner needs a point off the line — adding on the line
    /// is collinear). Returns `true` (consuming) only when a point was added — a
    /// press on an existing handle (`FalloffHit::Point`) returns `false` so the
    /// panel's drag dispatch grabs it, and an off-canvas press also falls through.
    pub(crate) fn painter_falloff_canvas_add(&mut self, px: f32, py: f32) -> bool {
        let ph2d_panel_painter_layers::FalloffHit::Canvas(nx, ny) =
            ph2d_panel_painter_layers::falloff_hit_test(px, py)
        else {
            return false;
        };
        // While a floating overlay is open (the colour picker, or a right-click
        // context menu), a click belongs to it — not to the graph underneath.
        // Without this guard, clicking inside the floating picker dropped stray
        // points onto the curve. (Earlier the gate used `hit_index.hit`, but the
        // panel chrome registers hit rects over the graph area, so that blocked
        // ALL adds — this open-overlay gate is precise.)
        let overlay_open = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.picker_target().is_some() || h.store.context_menu().is_some());
        if overlay_open {
            return false;
        }
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        if let Some(id) = painter.add_brush_falloff_point_at(nx, ny) {
            ph2d_panel_painter_layers::set_selected_falloff_point(Some(id));
            FALLOFF_DRAG.with(|c| c.set(Some(id)));
            return true;
        }
        false
    }

    /// CursorMoved while a shell-owned Falloff add-drag is live: move the grabbed
    /// point to the cursor (clamped into the graph). Returns `true` (consuming the
    /// move) while dragging, so it doesn't also pan / drive a gizmo.
    pub(crate) fn painter_falloff_drag(&mut self, px: f32, py: f32) -> bool {
        let Some(id) = FALLOFF_DRAG.with(Cell::get) else {
            return false;
        };
        if let Some((nx, ny)) = ph2d_panel_painter_layers::falloff_canvas_norm(px, py)
            && let Some(painter) = self.painter_tool_mut()
        {
            painter.set_brush_falloff_point(id, nx, ny);
        }
        true
    }

    /// Primary Up: end any shell-owned Falloff add-drag. No-op when not dragging.
    pub(crate) fn painter_falloff_release(&mut self) {
        FALLOFF_DRAG.with(|c| c.set(None));
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

    /// Secondary Down on an on-canvas **Curve / Free Hand** editor control point: select it and open the
    /// handle-kind menu (Free / Aligned / Vector / Auto) at the cursor. The chrome handler parks the choice
    /// in `HeroScreen.pending_curve_point_handle`; [`Self::painter_apply_pending_curve_handle`] drains it.
    /// Returns `true` (consuming) iff a point was hit. Mirrors `deliver_canvas_pointer`'s screen→image
    /// geometry so the hit-test lands exactly on the painted control dots.
    pub(crate) fn painter_curve_open_point_menu(&mut self, px: f32, py: f32) -> bool {
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
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let (Some(tr), Some(sprite)) = (
            gfx.sim.world().get::<Transform>(entity),
            gfx.sim.world().get::<ph2d_render::Sprite>(entity),
        ) else {
            return false;
        };
        let window_size = gfx.surface.size();
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
        let affine = crate::render_loop::bgremoval_preview::sprite_image_to_screen_affine(
            iw,
            ih,
            tr,
            sprite,
            &gfx.camera,
            window_size,
        );
        let img = affine.inverse() * ph2d_vector::Point::new(f64::from(px), f64::from(py));
        let c = affine.as_coeffs();
        let pixel_scale = (c[0] * c[0] + c[1] * c[1]).sqrt() as f32;
        let tol = if pixel_scale > 0.0 {
            SHAPE_GRAB_TOL_SCREEN_PX / pixel_scale
        } else {
            SHAPE_GRAB_TOL_SCREEN_PX
        };
        if !painter.curve_select_point_at([img.x as f32, img.y as f32], tol) {
            return false; // off a control point — fall through (pan / other secondary-click handlers)
        }
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.store
                .open_context_menu(ph2d_editor::interaction::ContextMenuRequest {
                    x: px,
                    y: py,
                    kind: ph2d_editor::interaction::ContextMenuKind::CurvePointHandle,
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
    ///
    /// **Per-frame coalescing (perf):** for the restore + whole-shape re-stamp methods
    /// ([`PainterTool::coalesces_canvas_motion`]) every raw winit `CursorMoved` between frames used to
    /// re-stamp the ENTIRE shape — a death-spiral under high-Hz input (more events → more re-stamps →
    /// slower frame → more events), and all but the last per frame are reverted by the next move's
    /// restore, so wasted. Here those Moves are BUFFERED (latest wins) and flushed once per frame by
    /// [`Self::flush_pending_painter_move`]; incremental / Free Hand methods (which deposit dabs / capture
    /// path samples per event) deliver immediately as before. See `HANDOFF_per_layer_color_perf` §1.R.
    pub(crate) fn painter_canvas_move(&mut self, px: f32, py: f32) -> bool {
        if !STROKE_ACTIVE.with(Cell::get) {
            return false;
        }
        if self.painter_coalesces_motion() {
            // Coalesce: keep only the latest position; the frame loop delivers ONE Move per frame.
            self.pending_painter_move = Some((px, py));
            return true; // consume the move (don't pan / drive a gizmo) — delivered at the frame flush
        }
        // Non-coalescing delivery is authoritative for the latest position: drop any move buffered while
        // the method was still coalescing (a mid-stroke method flip), so the frame flush can't later
        // re-stamp at that now-stale position.
        self.pending_painter_move = None;
        // Incremental methods stamp per raw event (between frames). Time each so the HUD's "paint ms"
        // reflects the REAL per-frame painter cost, not just the once-per-frame coalesced flush.
        let t0 = std::time::Instant::now();
        let delivered = self.deliver_canvas_pointer(px, py, 1.0, PointerPhase::Move);
        self.paint_stamp_us_this_frame = self
            .paint_stamp_us_this_frame
            .saturating_add(t0.elapsed().as_micros() as u64);
        if delivered {
            self.paint_stamps_this_frame = self.paint_stamps_this_frame.saturating_add(1);
        } else {
            // Painter no longer active / selection gone — abandon the stroke so we
            // stop swallowing moves.
            STROKE_ACTIVE.with(|s| s.set(false));
        }
        delivered
    }

    /// Whether the active tool is the Painter in a stroke method that only ever shows the latest pointer
    /// position (so its Moves may be coalesced to one delivery per frame). Scoped mutable downcast — the
    /// `Tool` trait exposes only `as_any_mut`, and the read is cheap.
    fn painter_coalesces_motion(&mut self) -> bool {
        self.gfx
            .as_mut()
            .and_then(|g| g.tools.active_mut())
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
            .is_some_and(|p| p.coalesces_canvas_motion())
    }

    /// Deliver the buffered (coalesced) painter Move, if any, as one [`PointerPhase::Move`]. Called once
    /// per frame at the top of `run_render_frame` (before the encode window) and before a pen-up, so the
    /// final position is stamped before the stroke commits.
    pub(crate) fn flush_pending_painter_move(&mut self) {
        let Some((px, py)) = self.pending_painter_move.take() else {
            return; // nothing buffered — incremental stamps (if any) are timed in `painter_canvas_move`
        };
        let t0 = std::time::Instant::now();
        let delivered = self.deliver_canvas_pointer(px, py, 1.0, PointerPhase::Move);
        self.paint_stamp_us_this_frame = self
            .paint_stamp_us_this_frame
            .saturating_add(t0.elapsed().as_micros() as u64);
        if delivered {
            self.paint_stamps_this_frame = self.paint_stamps_this_frame.saturating_add(1);
        } else {
            STROKE_ACTIVE.with(|s| s.set(false));
        }
    }

    /// Primary Up: close any open painter stroke. No-op when not painting.
    pub(crate) fn painter_canvas_up(&mut self) {
        if !STROKE_ACTIVE.with(Cell::get) {
            return;
        }
        // Stamp any buffered (coalesced) final position BEFORE the commit, so the released shape matches
        // the last cursor position rather than the previous frame's flush.
        self.flush_pending_painter_move();
        STROKE_ACTIVE.with(|s| s.set(false));
        let (px, py) = self.last_pointer;
        self.deliver_canvas_pointer(px, py, 1.0, PointerPhase::Up);
    }

    /// Gate on painter-active + a selected sprite, convert screen→image pixels via
    /// the sprite footprint, and deliver one [`CanvasPointer`]. Returns the tool's
    /// `consumed` result, or `false` when the sample was not delivered (not the
    /// painter / no selection / Down outside the footprint).
    ///
    /// `pub(crate)` so the Fill (Bucket) ColorDrop drag (`input_dispatch::fill_drag`) can deliver its
    /// button-initiated drop through the same screen→image conversion + `on_canvas_pointer` path.
    pub(crate) fn deliver_canvas_pointer(
        &mut self,
        px: f32,
        py: f32,
        pressure: f32,
        phase: PointerPhase,
    ) -> bool {
        // Read Alt before the `gfx` borrow — the Line method constrains to 45° while it's held
        // (Blender Alt-drag). The frozen `CanvasPointer` carries no modifiers, so it's forwarded
        // out-of-band via `PainterTool::set_line_constrain` just before delivery.
        let alt = self.modifiers.alt_key();
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
        let window_size = gfx.surface.size();
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
        // Screen → image-px via the FULL sprite affine (size · scale · rotation · anchor · camera) —
        // the same geometry the renderer + bgremoval use, so the brush tracks the sprite under any
        // resize, AR change OR rotation. `u`/`v` is the image fraction, NOT clamped (a Repeat-Image
        // neighbour tile lands outside `[0, 1]`).
        let affine = crate::render_loop::bgremoval_preview::sprite_image_to_screen_affine(
            iw,
            ih,
            tr,
            sprite,
            &gfx.camera,
            window_size,
        );
        let img = affine.inverse() * ph2d_vector::Point::new(f64::from(px), f64::from(py));
        let (u, v) = (
            (img.x / f64::from(iw)) as f32,
            (img.y / f64::from(ih)) as f32,
        );
        // A Down only starts a stroke when inside the footprint (outside clicks fall through to pan /
        // selection); Move/Up always reach an open stroke so you can paint to and past the edge.
        // **Repeat Image + Tiling**: the 8 neighbour tiles are paintable too — extend the Down hit
        // region to the 3×3 grid on each tiled axis. The coordinate is passed UN-wrapped so a stroke
        // crossing a tile boundary stays continuous; the painter's Tiling dab-wrap maps those off-
        // canvas dabs back onto the canvas, and the preview re-tiles the painted result.
        let repeat = painter.repeat_image();
        let tiling = painter.brush_tiling();
        let in_x = if repeat && tiling[0] {
            (-1.0..2.0).contains(&u)
        } else {
            (0.0..=1.0).contains(&u)
        };
        let in_y = if repeat && tiling[1] {
            (-1.0..2.0).contains(&v)
        } else {
            (0.0..=1.0).contains(&v)
        };
        if phase == PointerPhase::Down && !(in_x && in_y) {
            return false;
        }
        let ev = CanvasPointer {
            pos: [u * iw as f32, v * ih as f32],
            pressure,
            tilt: [0.0, 0.0],
            phase,
        };
        // Control-handle grab radius for the shape editors + Stencil handles: screen tolerance ÷ the
        // affine's per-image-pixel screen scale (|linear column 0|), so it stays a constant on-screen
        // size at any zoom / rotation. Out-of-band, like `set_line_constrain`.
        let c = affine.as_coeffs();
        let pixel_scale = (c[0] * c[0] + c[1] * c[1]).sqrt() as f32;
        let grab_tol_img = if pixel_scale > 0.0 {
            SHAPE_GRAB_TOL_SCREEN_PX / pixel_scale
        } else {
            SHAPE_GRAB_TOL_SCREEN_PX
        };
        painter.set_shape_grab_tol_px(grab_tol_img);
        painter.set_line_constrain(alt);
        painter.set_uniform_scale(self.modifiers.shift_key()); // Shift = uniform Stencil scale (Sprite gizmo)
        painter.set_line_snap(self.modifiers.shift_key()); // Shift = 15° direction snap in the Line polyline editor
        // Grid snap for drawing-tool points: resolve the pointer to the nearest editor-grid node in WORLD
        // space (reuses `GridSnapState::snap_world` — all grid kinds + magnetism), map it back to image px,
        // and forward it. Only when the snap grid is ON; brush painting is unaffected (the tool consumes
        // this only in shape point-placement paths, never in gizmo / corner-handle drags).
        let grid_snap = if gfx
            .hero_screen
            .as_ref()
            .is_some_and(|h| h.grid.snap_state.snap_enabled)
        {
            let world = gfx.camera.screen_to_world((px, py), window_size);
            let snapped = gfx
                .hero_screen
                .as_mut()
                .expect("hero present")
                .grid
                .snap_state
                .snap_world(world, [0.0, 0.0]);
            let (sx, sy) = gfx.camera.world_to_screen(snapped, window_size);
            let gimg = affine.inverse() * ph2d_vector::Point::new(f64::from(sx), f64::from(sy));
            Some([gimg.x as f32, gimg.y as f32])
        } else {
            None
        };
        painter.set_grid_snap(grid_snap);
        painter.on_canvas_pointer(ev)
    }

    /// Enter: commit the in-progress shape (Curve/Circle — bake the painted stroke). Returns `true`
    /// (consuming the key) iff a shape session was open, so Enter falls through to text fields / other
    /// handlers otherwise. Mirror of the Falloff-delete shell helper (ADR-0040 §3 downcast).
    pub(crate) fn painter_shape_commit(&mut self) -> bool {
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        painter.commit_open_shape()
    }

    /// Secondary (right-click) Down while a Line polyline is still being drawn: END point-creation and
    /// enter the editing phase (Blender/CAD convention — right-click finishes the chain). Returns `true`
    /// (consuming, so no context menu opens) iff it ended an in-progress Line. No-op otherwise, so a
    /// right-click falls through to the context menu when there's no open Line.
    pub(crate) fn painter_line_finish_points(&mut self) -> bool {
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        painter.line_finish_points()
    }

    /// Esc: discard the in-progress shape (revert the preview). Returns `true` (consuming) iff a
    /// session was open.
    pub(crate) fn painter_shape_cancel(&mut self) -> bool {
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        painter.cancel_open_shape()
    }

    /// Delete/Backspace: remove the selected Curve control point. Returns `true` (consuming) iff a
    /// point was removed, so the key falls through to the Falloff-point delete / other tools when
    /// no curve point is selected.
    pub(crate) fn painter_curve_delete_selected_point(&mut self) -> bool {
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        painter.curve_delete_selected()
    }
}
