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

use crate::App;
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

    /// Resolve cursor screen-px → (u, v do sprite — NÃO clampado a [0,1),
    /// source_w, source_h) iff Painter ativo + selection. None só quando falta
    /// Painter/selection ou o cursor está sobre a chrome dockada — a sprite
    /// inteira fica paintável (dentro ou fora do quad), pra que um traço na
    /// borda / fora da viewport não derrube o segmento e estanque a simulação.
    ///
    /// **Mapeamento:** ponteiro screen-px → world (inverse camera) → UV do sprite via
    /// [`ph2d_render::sprite_world_to_uv`], que inverte o `RenderInstance.basis` (a mesma
    /// matriz que `pick_sprite_at_world` usa) → **honra translation + rotation + scale +
    /// skew + anchor**, então o pincel pinta exatamente sob o cursor independente da
    /// transform da sprite. (Antes: AABB-fit só por translation → rotacionar/escalonar
    /// quebrava a referência de posição — a antiga limitação "T1.5 axis-aligned", B-H1.)
    /// - UV **não clampado**: pode ser negativo ou `≥ 1` fora do quad. O clamp ao grid
    ///   acontece a jusante (envelope de dab + cutoff de raio do splat), então a sprite
    ///   inteira fica paintável sem que a borda do quad estanque o traço (B-H3 antigo).
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
        // Selection bits (Copy) — read + drop the hero borrow before the `&mut present` below.
        let bits = gfx.hero_screen.as_ref()?.gizmo.selection?;
        let window_size = gfx.surface.size();
        // Pointer screen px → world (inverse camera), then world → the selected sprite's
        // texture UV honouring its FULL transform (translation + rotation + scale + skew)
        // via the renderer's `RenderInstance.basis` — the SAME inversion `pick_sprite_at_world`
        // uses, so a moved/rotated/scaled sprite paints exactly under the cursor. (The old path
        // fit an axis-aligned AABB from translation only, so any rotation/scale mis-mapped —
        // the painter "lost the real position"; ex-B-H1 "T1.5 axis-aligned" limitation.) `None`
        // when the click is off the sprite quad — gates the stroke like the old bounds check.
        let world = gfx.camera.screen_to_world((px, py), window_size);
        // UNCLAMPED on purpose: the whole sprite must stay paintable even when the
        // cursor grazes / crosses the quad edge (e.g. the sprite extends past the
        // viewport). The old `[0,1)`-gated `sprite_world_to_uv` returned `None`
        // off-quad → `painter_drag_move` broke the segment → the live wet field
        // stalled ("stops simulating in the invisible areas"). Out-of-quad UV is
        // safe: the dab envelope clamps to the canvas grid and the splat's radius
        // cutoff drops a dab whose footprint misses every cell. The hero-panel gate
        // above still suppresses painting behind docked chrome.
        let (u, v) =
            ph2d_render::sprite_world_to_uv_unclamped(gfx.present.world_mut(), bits, world)?;
        // Source dim from PainterTool's canvas (set_source-pushed by the bridge); fallback to
        // the sprite size if the canvas isn't populated yet.
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let sprite_size = gfx
            .sim
            .world()
            .get::<ph2d_render::Sprite>(entity)
            .map(|s| (s.size[0] as u32, s.size[1] as u32));
        let (src_w, src_h) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
            .map(|p| {
                let (cw, ch) = p.canvas_size();
                if cw > 0 && ch > 0 {
                    (cw, ch)
                } else {
                    sprite_size.unwrap_or((0, 0))
                }
            })
            .or(sprite_size)?;
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
    // The screen→world→UV hit-test now lives in `ph2d_render::sprite_world_to_uv`
    // (honours the full sprite basis: translation + rotation + scale + skew), gated
    // by its own unit tests in `ph2d-render::picking`. The wrapping `painter_pointer_uv`
    // integrates `screen_to_world` + `gizmo.selection` + present-world lookup — end-to-end.
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn uv_to_sample_maps_canvas_pixels() {
        // Verify UV → canvas-pixel mapping (u·src_w, v·src_h).
        let s = uv_to_sample(0.5, 0.25, 100, 200);
        assert_eq!(s.position, [50.0, 50.0]);
        assert_eq!(s.pressure, 1.0);
        assert_eq!(s.tilt, 0.0);
    }
}
