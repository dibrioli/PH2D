//! Painter — pointer input dispatch.
//!
//! Mesmo shape do `protect_brush`: shell-only handlers que reaches o
//! `PainterTool` via `gfx.tools.active_mut()` + downcast. Primary Down
//! abre um stroke + carimba o primeiro stamp; CursorMoved enquanto
//! `is_stroke_active()` carimba ao longo do path; Primary Up encerra o
//! stroke.
//!
//! Coordenadas: o cursor chega em screen pixels; converte-se para
//! canvas-local pixels (espaço do `canvas_rgba`, dimensões = sprite
//! source size) via o mesmo footprint world→screen do protect brush.
//!
//! Apply / commit: NOT triggered by pointer-up in T1.5 MVP. O painter
//! permanece ativo entre strokes; o user precisa chamar Apply
//! explicitamente (sidebar/topbar button) ou trocar de tool — o bridge
//! detecta `take_pending_commit()` e baka o canvas via `run_full`.
//! Day-7 smoke valida só "primeira pintura visível", commit fica pro
//! refinement de sidebar W2.

use crate::{App, Transform};
use ph2d_painter_brush::PointerSample;
use ph2d_tool_painter::PainterTool;

impl App {
    /// Primary Down — inicia stroke + carimba primeiro stamp. Returns
    /// `true` (consome o event) iff Painter ativo + sprite selected + o
    /// pixel está dentro do footprint do sprite.
    pub(crate) fn try_painter_paint_down(&mut self, px: f32, py: f32) -> bool {
        let Some((u, v, src_w, src_h)) = self.painter_pointer_uv(px, py) else {
            return false;
        };
        // Capture entity_bits BEFORE the mutable borrow of self.gfx —
        // derive_seed needs it (audit T1.5 round 1 A-H4 / B-M3) and
        // re-borrowing self.gfx mid-function would conflict.
        //
        // **R4-LH-9 fix:** `painter_pointer_uv` already validated
        // selection exists (returns None otherwise), so propagating via
        // `?` is honest. Prior `.unwrap_or(0)` collapsed "no selection"
        // into a fixed sentinel that could collide if the gate moved.
        let entity_bits = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.gizmo.selection);
        let Some(entity_bits) = entity_bits else {
            return false;
        };
        let sample = uv_to_sample(u, v, src_w, src_h);
        let seed = PainterTool::derive_seed(
            sample.position[0],
            sample.position[1],
            src_w,
            src_h,
            entity_bits,
        );
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
            return false;
        };
        painter.begin_stroke(seed);
        painter.queue_pointer(sample);
        true
    }

    /// Continuação de drag pós-Down. Carimba stamp adicional se stroke
    /// ativo. Returns `true` quando consume (stroke ativo).
    ///
    /// **R3-LE-1 fix:** quando cursor SAI do footprint do sprite mid-stroke,
    /// chama `break_stroke_segment()` na PainterTool — o próximo
    /// re-entrada não interpola stamps no gap (sem smear).
    pub(crate) fn painter_drag_move(&mut self, px: f32, py: f32) -> bool {
        // First: try to compute UV. If outside footprint OR no selection,
        // we still need to inform the active Painter (if any) that the
        // brush "lifted" mid-stroke so the NEXT pointer inside the
        // footprint starts a fresh segment rather than interpolating.
        let uv = self.painter_pointer_uv(px, py);
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
            return false;
        };
        if !painter.is_stroke_active() {
            return false;
        }
        match uv {
            Some((u, v, src_w, src_h)) => {
                painter.queue_pointer(uv_to_sample(u, v, src_w, src_h));
                true
            }
            None => {
                // Cursor exited footprint mid-drag. Break the segment so a
                // re-entry doesn't smear stamps across the gap. We DO NOT
                // consume the event (return false): the cursor may be
                // pertinent to other handlers (e.g., panel hover) while
                // the stroke is paused outside the sprite.
                painter.break_stroke_segment();
                false
            }
        }
    }

    /// Primary Up — encerra stroke ativo (no-op se nenhum). Idempotente.
    pub(crate) fn end_painter_paint(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(tool) = gfx.tools.active_mut() else {
            return;
        };
        let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
            return;
        };
        painter.end_stroke();
    }

    /// **R3-LE-2 fix:** when Painter is the active tool, Primary Down
    /// that falls OUTSIDE the sprite footprint (or with no selection)
    /// must NOT fall through to gizmo / rubber-band / canvas-pick logic.
    /// The painting mode owns the canvas; off-target clicks are no-ops.
    /// Returns `true` iff Painter is active (event consumed silently).
    pub(crate) fn painter_active_consume_canvas_click(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.tools.active())
            .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
            .unwrap_or(false)
    }

    /// Pure-fn helper: screen px → (u, v) ∈ [0, 1) iff `(px, py)` is
    /// inside the AABB `[lo_x, hi_x) × [lo_y, hi_y)`. None caso contrário.
    /// Extracted from `painter_pointer_uv` so the bounds + B-C1 anchor +
    /// B-H3 exclusive-high-bound logic is unit-testable without an
    /// `AppState` orchestration (audit T1.5 round 2 F2 MISSING-GATE-
    /// PAINTER-E2E mitigation).
    #[must_use]
    #[doc(hidden)]
    pub(crate) fn uv_from_bounds(
        px: f32,
        py: f32,
        lo_x: f32,
        hi_x: f32,
        lo_y: f32,
        hi_y: f32,
    ) -> Option<(f32, f32)> {
        if hi_x <= lo_x || hi_y <= lo_y {
            return None;
        }
        if px < lo_x || px >= hi_x || py < lo_y || py >= hi_y {
            return None;
        }
        let u = (px - lo_x) / (hi_x - lo_x);
        let v = (py - lo_y) / (hi_y - lo_y);
        Some((u, v))
    }

    /// Resolve cursor screen-px → (u, v normalizado em [0,1) do sprite,
    /// source_w, source_h) iff Painter ativo + selection + dentro do
    /// footprint. None caso contrário.
    ///
    /// **Convenções (audit T1.5 round 1):**
    /// - Footprint corner uses `tr.translation + sprite.anchor` — mesma
    ///   convenção do `painter_bridge` overlay (B-C1). Sem anchor,
    ///   click e overlay desalinham quando `sprite.anchor != [0, 0]`.
    /// - Bounds são **exclusive no high side** (`px < hi_x`): `u = 1.0`
    ///   mapearia para `src_w` (1 pixel além do último válido). Audit
    ///   B-H3.
    /// - **T1.5 MVP requer sprite axis-aligned** (Transform.rotation = 0,
    ///   scale = 1). Sprites rotacionados/escalonados produzem
    ///   AABB-fit que NÃO bate com o quad visível; clicks dentro do AABB
    ///   mas fora do quad rotado caem num UV inválido. Audit B-H1 —
    ///   suporte completo de rotação é W2 quando o sidebar Painter
    ///   adicionar gesto rotate-aware.
    fn painter_pointer_uv(&mut self, px: f32, py: f32) -> Option<(f32, f32, u32, u32)> {
        // Don't paint THROUGH docked chrome. The Painter sidebar is a
        // right-dock takeover that overlaps the sprite footprint, so a
        // Primary Down (or a drag) over the panel would otherwise deposit
        // stamps "behind" the panel. Returning None here gates both the
        // stroke-start (`try_painter_paint_down`) and the mid-stroke stamp
        // (`painter_drag_move`, whose None branch breaks the segment so a
        // drag re-entering the canvas doesn't smear across the panel). The
        // click then falls through to the panel's own UI dispatch (slider
        // etc.); the silent-consume arm is gated on the same predicate in
        // `input_dispatch.rs` so the panel actually receives it.
        if crate::forwarding::cursor_over_hero_panel(self.gfx.as_ref(), px, py) {
            return None;
        }
        let gfx = self.gfx.as_mut()?;
        let painter_active = gfx
            .tools
            .active()
            .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
            .unwrap_or(false);
        if !painter_active {
            return None;
        }
        let hero = gfx.hero_screen.as_ref()?;
        let bits = hero.gizmo.selection?;
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let tr = gfx.sim.world().get::<Transform>(entity)?;
        let sprite = gfx.sim.world().get::<ph2d_render::Sprite>(entity)?;
        // Audit B-C1: usar `translation + anchor` (paridade com
        // `painter_bridge.rs:149-151` + `bgremoval_preview.rs:247-248`).
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
        // Audit B-H3 + round-2 F2: bounds + UV via pure-fn helper
        // (exclusive high side; testable in isolation).
        let (u, v) = Self::uv_from_bounds(px, py, lo_x, hi_x, lo_y, hi_y)?;
        // Read source dim from PainterTool's canvas (já set_source-pushed
        // pelo bridge). Fallback p/ sprite.size se ainda não populado.
        let (src_w, src_h) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
            .map(|p| {
                let (w, h) = (sw as u32, sh as u32);
                let (cw, ch) = p.canvas_size();
                if cw > 0 && ch > 0 { (cw, ch) } else { (w, h) }
            })
            .unwrap_or((sw as u32, sh as u32));
        Some((u, v, src_w, src_h))
    }
}

/// Converte (u, v, src_w, src_h) em [`PointerSample`] no espaço do
/// `canvas_rgba` do PainterTool. Pressure constante 1.0 — mouse sem
/// suporte (Pencil/tablet chegam quando T-input materializar).
fn uv_to_sample(u: f32, v: f32, src_w: u32, src_h: u32) -> PointerSample {
    PointerSample {
        position: [u * src_w as f32, v * src_h as f32],
        pressure: 1.0,
        tilt: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ──────────────────────────────────────────────────────────────────────
    // Audit T1.5 round 2 F2 (MISSING-GATE-PAINTER-E2E) mitigation: the
    // hit-test + UV math is testable through `App::uv_from_bounds` —
    // a pure fn that takes precomputed AABB bounds. The wrapping
    // `painter_pointer_uv` integrates `world_to_screen` + `gizmo.
    // selection` + downcast — those are end-to-end (App), but the
    // mathematical core is gated here.
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn uv_from_bounds_interior_point() {
        let r = App::uv_from_bounds(5.0, 10.0, 0.0, 10.0, 0.0, 20.0).unwrap();
        assert!((r.0 - 0.5).abs() < 1e-6);
        assert!((r.1 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn uv_from_bounds_lo_edge_inclusive() {
        // px == lo_x is allowed → u = 0.0.
        let r = App::uv_from_bounds(0.0, 0.0, 0.0, 10.0, 0.0, 20.0).unwrap();
        assert_eq!(r.0, 0.0);
        assert_eq!(r.1, 0.0);
    }

    #[test]
    fn uv_from_bounds_hi_edge_exclusive() {
        // Audit B-H3 regression — px == hi_x rejects (no UV=1.0 stamp).
        assert!(App::uv_from_bounds(10.0, 5.0, 0.0, 10.0, 0.0, 20.0).is_none());
        assert!(App::uv_from_bounds(5.0, 20.0, 0.0, 10.0, 0.0, 20.0).is_none());
    }

    #[test]
    fn uv_from_bounds_outside_rejects() {
        assert!(App::uv_from_bounds(-1.0, 5.0, 0.0, 10.0, 0.0, 20.0).is_none());
        assert!(App::uv_from_bounds(5.0, -1.0, 0.0, 10.0, 0.0, 20.0).is_none());
        assert!(App::uv_from_bounds(15.0, 5.0, 0.0, 10.0, 0.0, 20.0).is_none());
    }

    #[test]
    fn uv_from_bounds_degenerate_rect_rejects() {
        // Zero-width or zero-height AABB → no valid UV.
        assert!(App::uv_from_bounds(0.0, 0.0, 5.0, 5.0, 0.0, 10.0).is_none());
        assert!(App::uv_from_bounds(0.0, 0.0, 0.0, 10.0, 5.0, 5.0).is_none());
        // Inverted (hi < lo) — caught by `hi_x <= lo_x` guard.
        assert!(App::uv_from_bounds(0.0, 0.0, 10.0, 5.0, 0.0, 20.0).is_none());
    }

    #[test]
    fn uv_to_sample_maps_canvas_pixels() {
        // Verify UV → canvas-pixel mapping (u·src_w, v·src_h).
        let s = uv_to_sample(0.5, 0.25, 100, 200);
        assert_eq!(s.position, [50.0, 50.0]);
        assert_eq!(s.pressure, 1.0);
        assert_eq!(s.tilt, 0.0);
    }
}
