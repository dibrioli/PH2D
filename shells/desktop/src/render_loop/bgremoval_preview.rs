//! Background-Removal panel ⟷ tool bridge + on-canvas live preview.
//!
//! Extracted from `render_loop::mod.rs` (HR-18 LOC cap) as a free
//! function, run once per frame BEFORE `paint_hero_screen`. Behavior-
//! preserving lift. Does, in order:
//!
//! 1. Pushes the active sprite's RGBA into the `BgRemovalTool` snapshot
//!    when the selection drifts (so the tool segments the live pixels).
//! 2. Drives the panel's visibility (shown iff bgremoval is active),
//!    drains the panel's `BgremovalUiEdit`s into the tool, fires the
//!    full-res commit on Apply, and publishes the normalized snapshot
//!    the panel paints next frame.
//! 3. (Re)computes the full-res straight-alpha preview and blits it on
//!    top of the sprite's on-canvas footprint (the sprite itself is
//!    suppressed from the sprite pass while previewing — see
//!    `sim_extract`).

use crate::app_state::BgremovalPreview;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite, SpriteRenderer};
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::{Affine, Brush, Circle, Color, Stroke, VectorScene};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Returns `true` iff an Apply committed this frame (the caller then
/// tears the tool down — deactivate + restore Inspector — so the
/// on-canvas preview overlay stops re-rendering on top of the freshly
/// baked sprite, which otherwise reads as a ghost edge outline while the
/// tool stays selected).
#[allow(clippy::too_many_arguments)]
#[must_use]
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    sim: &SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    mut bgremoval_ui_edits: Vec<ph2d_editor::tools::bgremoval::BgRemovalUiEdit>,
    last_bgremoval_pushed_entity: &mut Option<u64>,
    bgremoval_preview: &mut Option<BgremovalPreview>,
) -> bool {
    let bgremoval_is_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("bgremoval"))
        .unwrap_or(false);
    // Snapshot push for the active BgRemovalTool — pushed once per
    // (tool-active + new selection) tuple.
    if bgremoval_is_active
        && let Some(bits) = hero.gizmo.selection
        && *last_bgremoval_pushed_entity != Some(bits)
    {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        // Single readback chokepoint: pixels arrive carrying their alpha
        // mode (a prior premultiplied BG-Removal bake included). The
        // segmentation snapshot reasons about true colours, so normalize
        // to straight once.
        if let Some(src) = crate::hero_intents::texture_edit::read_sprite_source(
            entity,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
        ) {
            let straight = src.image.into_straight();
            if let Some(tool) = tools.active_mut()
                && let Some(bg) = tool
                    .as_any_mut()
                    .downcast_mut::<ph2d_editor::tools::bgremoval::BgRemovalTool>()
            {
                bg.set_source_snapshot(straight.pixels, straight.width, straight.height);
                *last_bgremoval_pushed_entity = Some(bits);
            }
        }
    }
    // Visibility: shown iff bgremoval is the active tool (keyed
    // "bgremoval" to match `BgRemovalPanel::ID`).
    hero.panel_visibility
        .insert("bgremoval", bgremoval_is_active);
    let params_changed = !bgremoval_ui_edits.is_empty();
    let mut apply_selection: Option<u64> = None;
    // Captured for the protection-mask overlay tint (built while the tool
    // is borrowed below, drawn in the on-canvas overlay block).
    let theme = hero.theme;
    let mut protect_tint: Option<(Arc<Vec<u8>>, u32, u32)> = None;
    // Brush-size ring gizmo: (brush radius in source px, source width).
    // Set while the protection brush is armed; drawn at the cursor.
    let mut brush_ring: Option<(f32, u32)> = None;
    if let Some(tool) = tools.active_mut()
        && let Some(bg) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_editor::tools::bgremoval::BgRemovalTool>()
    {
        for edit in bgremoval_ui_edits.drain(..) {
            bg.apply_ui_edit(edit);
        }
        if bg.take_pending_apply() {
            apply_selection = hero.gizmo.selection;
        }
        #[cfg(feature = "panel-bgremoval")]
        ph2d_panel_bgremoval::set_current_bgremoval_snapshot(if bgremoval_is_active {
            Some(bg.ui_snapshot())
        } else {
            None
        });
        if bgremoval_is_active
            && bg.has_source()
            && let Some(bits) = hero.gizmo.selection
        {
            let stale = match &*bgremoval_preview {
                Some(p) => params_changed || p.entity_bits != bits,
                None => true,
            };
            if stale {
                let mut out = Vec::new();
                // Capped-resolution preview (PREVIEW_MAX_DIM) — NOT the
                // full-res bake. Running the full pipeline at source
                // resolution on every slider tick froze the UI in the
                // GrabCut backend (a ~1M-node max-flow per drag event).
                // The overlay is drawn scaled to the footprint anyway;
                // Apply still bakes full-res via the Bgremoval action.
                let (w, h) = bg.run_canvas_preview(&mut out);
                *bgremoval_preview = Some(BgremovalPreview {
                    entity_bits: bits,
                    rgba: Arc::new(out),
                    width: w,
                    height: h,
                });
            }
        } else {
            *bgremoval_preview = None;
        }
        // Build the protection-mask overlay tint (drawn over the sprite
        // footprint so the user sees their painted "keep" region). Gated
        // on the Show-Mask toggle.
        if bgremoval_is_active && bg.show_mask() && bg.has_protect_mask() {
            let (mask, mw, mh) = bg.protect_mask_source();
            let accent = ColorToken::Accent.resolve(theme);
            if let Some((tw, th, buf)) =
                build_protect_tint(mask, mw, mh, [accent.r, accent.g, accent.b])
            {
                protect_tint = Some((Arc::new(buf), tw, th));
            }
        }
        // Brush-size ring: capture the radius (source px) + source width
        // while the brush is armed; the overlay draws it at the cursor.
        if bgremoval_is_active && bg.is_protect_armed() {
            brush_ring = Some((bg.brush_radius_px(), bg.source_size().0));
        }
    }
    if !bgremoval_is_active {
        *bgremoval_preview = None;
    }
    if let Some(bits) = apply_selection {
        hero.bus
            .push(ph2d_editor::action_bus::EditorAction::Bgremoval { entity_bits: bits });
        // Committed result becomes the new sprite texture; drop the
        // preview so the overlay stops painting the pre-commit copy.
        // The caller deactivates the tool on the returned flag so the
        // overlay never re-runs against the freshly baked sprite.
        *bgremoval_preview = None;
    }
    // On-canvas preview overlay (straight-alpha, on top of the
    // suppressed sprite's footprint).
    if let Some(preview) = &*bgremoval_preview {
        let entity = ph2d_ecs::Entity::from_bits(preview.entity_bits);
        if let (Some(tr), Some(sprite)) = (
            sim.world().get::<ph2d_ecs::Transform>(entity),
            sim.world().get::<Sprite>(entity),
        ) {
            let (tx, ty) = (tr.translation.x, tr.translation.y);
            let (sw, sh) = (sprite.size[0], sprite.size[1]);
            let (x0, y0) = camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
            let (x1, y1) = camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
            // Single source of truth: the preview's sampling quality is
            // derived from the SAME app-wide ImageFilterMode that drives
            // the wgpu sprite sampler. PixelArt → nearest, Smooth →
            // bicubic — so the on-canvas preview matches the sprite the
            // Apply will bake.
            let quality = ph2d_editor::image_quality_for(hero.project.image_filter);
            vector_scene.draw_image_rgba(
                &preview.rgba,
                preview.width,
                preview.height,
                (x0 as f64, y0 as f64, x1 as f64, y1 as f64),
                quality,
            );
            // Protection-mask tint on top — a semitransparent accent wash
            // over the painted "keep" pixels. Same footprint rect, so it
            // tracks the sprite transform like the preview.
            if let Some((tint, tw, th)) = &protect_tint {
                vector_scene.draw_image_rgba(
                    tint,
                    *tw,
                    *th,
                    (x0 as f64, y0 as f64, x1 as f64, y1 as f64),
                    quality,
                );
            }
            // Brush-size ring at the cursor — the source-px radius mapped
            // to screen via the footprint scale (footprint_w / source_w).
            if let (Some((r_src, src_w)), Some((cur_x, cur_y))) = (
                brush_ring,
                crate::input_dispatch::protect_brush::brush_cursor(),
            ) && src_w > 0
            {
                let footprint_w = (x1 - x0).abs();
                let r_screen = r_src * footprint_w / src_w as f32;
                let accent = ColorToken::Accent.resolve(theme);
                let color = Color::from_rgba8(accent.r, accent.g, accent.b, 255);
                vector_scene.inner_mut().stroke(
                    &Stroke::new(StrokeToken::Default.px() as f64),
                    Affine::IDENTITY,
                    &Brush::Solid(color),
                    None,
                    &Circle::new((cur_x as f64, cur_y as f64), r_screen as f64),
                );
            }
        }
    }
    apply_selection.is_some()
}

/// Build a capped-resolution RGBA tint image from a source-resolution
/// protection mask: protected pixels get `rgb` at [`TINT_ALPHA`], the
/// rest are fully transparent. Downsampled (nearest) to at most
/// [`TINT_CAP`] on the long axis — it's a coarse visual hint, drawn
/// scaled to the footprint anyway. Returns `None` when nothing is
/// painted (so the overlay draws nothing).
fn build_protect_tint(mask: &[u8], mw: u32, mh: u32, rgb: [u8; 3]) -> Option<(u32, u32, Vec<u8>)> {
    /// Long-axis cap for the tint buffer (visual hint, not crisp art).
    const TINT_CAP: u32 = 256;
    /// Tint opacity (~38%). // LITERAL-OK: overlay hint alpha budget
    const TINT_ALPHA: u8 = 96;
    /// Mask byte threshold (`>=` ⇒ protected).
    const PROTECT_THRESHOLD: u8 = 128;

    if mask.is_empty() || mw == 0 || mh == 0 {
        return None;
    }
    let (tw, th) = if mw <= TINT_CAP && mh <= TINT_CAP {
        (mw, mh)
    } else if mw >= mh {
        (
            TINT_CAP,
            ((mh as u64 * TINT_CAP as u64 / mw as u64).max(1)) as u32,
        )
    } else {
        (
            ((mw as u64 * TINT_CAP as u64 / mh as u64).max(1)) as u32,
            TINT_CAP,
        )
    };
    let mut out = vec![0u8; (tw as usize) * (th as usize) * 4];
    let mut any = false;
    for y in 0..th as usize {
        let sy = ((y as u64) * (mh as u64) / (th as u64)).min(mh as u64 - 1) as usize;
        for x in 0..tw as usize {
            let sx = ((x as u64) * (mw as u64) / (tw as u64)).min(mw as u64 - 1) as usize;
            if mask[sy * mw as usize + sx] >= PROTECT_THRESHOLD {
                let b = (y * tw as usize + x) * 4;
                out[b] = rgb[0];
                out[b + 1] = rgb[1];
                out[b + 2] = rgb[2];
                out[b + 3] = TINT_ALPHA;
                any = true;
            }
        }
    }
    if any { Some((tw, th, out)) } else { None }
}
