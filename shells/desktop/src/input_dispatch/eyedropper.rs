//! BgRemoval eyedropper — canvas colour sampling + swatch delete.
//!
//! All NEW input handling for the eyedropper feature lives in the SHELL
//! (deliberately — the architecture keeps core interaction dispatch
//! untouched). These helpers reach the active `BgRemovalTool` via
//! `gfx.tools.active_mut()` + downcast, the same pattern as
//! `render_loop::bgremoval_preview`. Extracted from `input_dispatch.rs`
//! to keep that file under the HR-18 LOC cap.

use crate::App;

impl App {
    /// If the BgRemoval tool is active AND its eyedropper is armed AND
    /// `(px, py)` falls inside the selected sprite's on-screen
    /// footprint, sample the source colour there and append it to the
    /// tool's extra-colour list. Returns `true` when a sample was
    /// attempted (so the caller early-returns and skips the normal
    /// canvas pick / gizmo / selection logic — we must not move or
    /// deselect the sprite while sampling).
    ///
    /// On a successful sample we drop `self.bgremoval.preview` so the
    /// per-frame dispatch recomputes the on-canvas overlay next frame.
    /// `add_extra_color` flips the tool's `params_dirty` flag so the
    /// canvas-preview cache rebuilds on the same frame the swatch
    /// appears (ADR-0040 TG-B; previously the overlay went stale until
    /// an unrelated panel edit nudged it).
    pub(crate) fn try_eyedropper_sample(&mut self, px: f32, py: f32) -> bool {
        // The CHROME first — the eyedropper consumes the event "regardless of in/out" of the sprite to
        // keep the canvas from moving/deselecting, so an unguarded arm swallows Down events headed for
        // the UI's own widgets: sliders inside the BgRemoval panel went clickable-but-un-draggable
        // (Enio 2026-05-26), and the left rail / top bars leaked the same way until 2026-07-16 because
        // this asked `panel_at` alone — half the question. One door, asked before anything else.
        if crate::chrome_hit::pointer_over_chrome(self.gfx.as_ref(), px, py) {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let bgremoval_active = gfx
            .tools
            .active()
            .map(|t| t.id() == ph2d_editor_core::ToolId::new("bgremoval"))
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
        // ⭐⭐⭐ **A UV de origem vem da PORTA** ([`ph2d_sprite_screen::uv_sob_o_ponteiro`], 2026-09-15) — até aqui
        // era uma caixa alinhada aos eixos tirada da pose LOCAL, cega à rotação, ao pai e à MALHA.
        let window_size = gfx.scene_window();
        let uv = ph2d_sprite_screen::uv_sob_o_ponteiro(
            &gfx.sim,
            gfx.present.world_mut(),
            &gfx.camera,
            window_size,
            bits,
            px,
            py,
        );
        if matches!(uv, ph2d_sprite_screen::UvSobOPonteiro::SemSujeito) {
            return false; // a selecção não é uma sprite desenhada — o clique não é nosso
        }
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
        // Inside the footprint? Sample.
        if let ph2d_sprite_screen::UvSobOPonteiro::Uv(u, v) = uv
            && (0.0..=1.0).contains(&u)
            && (0.0..=1.0).contains(&v)
            && let Some(rgb) = bg.sample_source_at_uv(u, v)
        {
            bg.add_extra_color(rgb);
            // `add_extra_color` already flips `params_dirty=true`,
            // so the bridge's `drive_preview_cache` re-runs the
            // pipeline next frame. We deliberately do NOT drop
            // `self.bgremoval.preview` here (was: `= None`) — the
            // segmentation is slow when the user has several picks
            // (Enio 2026-05-26: was "imagem desaparece" because
            // ~18 frames rendered with no overlay while the
            // pipeline rebuilt). Keeping the last good cache lets
            // the canvas keep painting the previous matte until
            // the new one is ready, so the user sees a smooth
            // transition instead of a black flash.
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
            .map(|t| t.id() == ph2d_editor_core::ToolId::new("bgremoval"))
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
        let Some(idx) = ph2d_editor_core::ids::bgr_swatch_index(hit_id) else {
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
            self.bgremoval.preview = None;
            return true;
        }
        false
    }
}
