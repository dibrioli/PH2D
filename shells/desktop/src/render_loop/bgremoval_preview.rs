//! Background-Removal panel ⟷ tool bridge + on-canvas live preview.
//!
//! Extracted from `render_loop::mod.rs` (HR-18 LOC cap) as a free
//! function, run once per frame BEFORE `paint_hero_screen`. Behavior-
//! preserving lift. Does, in order:
//!
//! 1. (Generic) Pushes the active sprite's RGBA into the `BgRemovalTool`
//!    snapshot when the selection drifts — via
//!    [`ph2d_tool_runtime::drive_source_push`] over the
//!    [`RasterEditTool`] upcast (so the tool segments the live pixels).
//! 2. (Generic) Drains `current_preview` via
//!    [`ph2d_tool_runtime::drive_preview_cache`] into the shell's
//!    `BgremovalPreview` cache (Arc-backed) for the on-canvas overlay.
//! 3. (Generic) Captures the multi-sprite Apply selection via
//!    [`ph2d_tool_runtime::drive_pending_commit`] (panel events that
//!    flipped `pending_apply` arrived earlier in the frame via
//!    `EditorAction::ToolPanelEvent → Tool::handle_panel_event`,
//!    ADR-0040 TG-B).
//! 4. (BgR-specific) Panel visibility (shown iff bgremoval is active,
//!    Inspector hidden while active), panel-store reset on Reset click,
//!    BgRemoval snapshot publish, protect-mask tint overlay, brush ring.
//! 5. (BgR-specific) On-canvas overlay paints the cached preview RGBA
//!    on top of the sprite's footprint (the sprite itself is suppressed
//!    from the sprite pass while previewing — see `sim_extract`).
//!
//! Wave 10 / Etapa 1.B (ADR-0041 follow-up): the parts marked
//! "(Generic)" used to be inlined ~80 LOC of per-tool boilerplate; they
//! now live in `ph2d-tool-runtime` and are composed by helper calls.
//! What stays here is BgR-specific (protect mask, brush ring, panel
//! snapshot — all documented `as_any_mut` exceptions per ADR-0040 §3).

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
    last_bgremoval_pushed_entity: &mut Option<u64>,
    bgremoval_preview: &mut Option<BgremovalPreview>,
) -> bool {
    let bgremoval_is_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("bgremoval"))
        .unwrap_or(false);

    // ── (Generic) Source push when selection drifts ───────────────────────
    // Wave 10 / Etapa 1.B: replaces the previous hand-written push block
    // (~30 LOC) with one call to `drive_source_push` over the
    // `as_raster_edit_mut()` upcast. The closure carries the
    // shell-specific pixel readback (asset_db + sprite renderer).
    if bgremoval_is_active
        && let Some(tool) = tools.active_mut()
        && let Some(raster) = tool.as_raster_edit_mut()
    {
        let _pushed = ph2d_tool_runtime::drive_source_push(
            raster,
            hero.gizmo.selection,
            last_bgremoval_pushed_entity,
            |entity| {
                let src = crate::hero_intents::texture_edit::read_sprite_source(
                    entity,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                )?;
                // Single readback chokepoint: pixels arrive carrying their
                // alpha mode (a prior premultiplied BG-Removal bake
                // included). The segmentation snapshot reasons about true
                // colours, so normalize to straight once.
                let straight = src.image.into_straight();
                Some(ph2d_tool_runtime::RasterSource {
                    pixels: straight.pixels,
                    width: straight.width,
                    height: straight.height,
                })
            },
        );
    }

    // ── (BgR-specific) Panel visibility + Inspector toggle ────────────────
    // Shown iff bgremoval is the active tool (keyed "bgremoval" to match
    // `BgRemovalPanel::ID`). Hide the Inspector while bgremoval is active
    // (image tools dock into the Inspector slot).
    //
    // Wave 10 Etapa 4 smoke fix (Enio 2026-05-24): only write
    // `inspector` visibility on the activate↔deactivate EDGE, not every
    // frame. The level-triggered version stomped on the user's rail
    // toggle whenever no image tool was active — every frame would
    // force `inspector = true`, undoing the manual hide. Edge trigger
    // preserves the rail-toggle semantics while still hiding inspector
    // at tool activation + restoring at deactivation.
    hero.panel_visibility
        .insert("bgremoval", bgremoval_is_active);
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(bgremoval_is_active, Ordering::Relaxed);
        if was != bgremoval_is_active {
            hero.panel_visibility
                .insert("inspector", !bgremoval_is_active);
        }
    }

    // Captured for the protection-mask overlay tint + brush ring (built
    // while the tool is borrowed below, drawn in the on-canvas overlay block).
    let theme = hero.theme;
    let mut protect_tint: Option<(Arc<Vec<u8>>, u32, u32)> = None;
    let mut brush_ring: Option<(f32, u32)> = None;
    let mut needs_panel_reset = false;
    let mut apply_selection: Vec<u64> = Vec::new();

    // ── (Mixed) Generic preview cache drain + BgR-specific extras ─────────
    // The (Generic) parts go through tool-runtime helpers; the BgR-specific
    // bits (panel snapshot, panel reset, protect tint, brush ring) keep
    // their downcast — ADR-0040 §3 documented exception. We acquire the
    // BgRemovalTool downcast once and do BOTH inside it so we don't
    // borrow the tool twice (the runtime helpers take &mut dyn
    // RasterEditTool; the BgR-specific bits take &mut BgRemovalTool).
    if let Some(tool) = tools.active_mut()
        && let Some(bg) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_bgremoval::BgRemovalTool>()
    {
        // (Generic) Drain current_preview into the shell cache.
        // ADR-0041: current_preview drains the tool's dirty flag, so the
        // helper just needs to call it. Selection drift is handled by the
        // helper's own invalidation pass.
        ph2d_tool_runtime::drive_preview_cache(bg, hero.gizmo.selection, bgremoval_preview);

        // (Generic) Capture the multi-sprite Apply selection.
        apply_selection = ph2d_tool_runtime::drive_pending_commit(bg, hero.gizmo.iter_selected());

        // (BgR-specific) Panel-store reset propagation.
        if bg.take_pending_panel_reset() {
            needs_panel_reset = true;
        }

        // (BgR-specific) Snapshot publish for the docked panel.
        #[cfg(feature = "panel-bgremoval")]
        ph2d_panel_bgremoval::set_current_bgremoval_snapshot(if bgremoval_is_active {
            Some(bg.ui_snapshot())
        } else {
            None
        });

        // (BgR-specific) Protection-mask overlay tint, gated on Show-Mask.
        if bgremoval_is_active && bg.show_mask() && bg.has_protect_mask() {
            let (mask, mw, mh) = bg.protect_mask_source();
            let accent = ColorToken::Accent.resolve(theme);
            if let Some((tw, th, buf)) =
                build_protect_tint(mask, mw, mh, [accent.r, accent.g, accent.b])
            {
                protect_tint = Some((Arc::new(buf), tw, th));
            }
        }

        // (BgR-specific) Brush-size ring: capture the radius (source px)
        // + source width while the brush is armed.
        if bgremoval_is_active && bg.is_protect_armed() {
            brush_ring = Some((bg.brush_radius_px(), bg.source_size().0));
        }
    }

    // ── Inactive path — clear LOCAL bridge state only ────────────────────
    // Wave 10 / Etapa 2 audit [C1 CRITICAL fix]: previous version called
    // `drive_deactivate_cleanup` on `tools.active_mut()` here, but that
    // returns the CURRENTLY-ACTIVE tool (which may be CEQ or Upscale —
    // other RasterEditTools), NOT the BgRemoval tool we're a bridge for.
    // Calling `RasterEditTool::deactivate()` on the wrong tool zeroes
    // the state of whichever raster tool is active (drains its
    // pending_apply, params_dirty, cached_canvas_preview, etc.) —
    // destroying its session.
    //
    // BgRemoval's own `Tool::on_deactivate` already fires when
    // `ToolRegistry::set_active` switches AWAY from bgremoval, and that
    // path already clears `cached_canvas_preview` + all transient flags
    // (Etapa 1.B audit fix A2). The bridge only needs to clear its own
    // shell-side cache here.
    if !bgremoval_is_active {
        *bgremoval_preview = None;
        *last_bgremoval_pushed_entity = None;
    }
    // Reset just fired — re-populate the panel's `WidgetStore` so
    // every slider knob / chip text snaps back to defaults. Without
    // this, `params` resets but the slider visuals stay where the
    // user dragged them (panel paints `store.slider(id)`, not the
    // snapshot).
    if needs_panel_reset {
        ph2d_editor::panel::with_registry_opt(|reg| {
            if let Some(idx) = reg.find_by_panel_node_id(ph2d_editor::ids::BGR_PANEL) {
                reg.panels_mut()[idx].populate(&mut hero.store);
            }
        });
    }
    if !apply_selection.is_empty() {
        for bits in &apply_selection {
            hero.bus
                .push(ph2d_editor::action_bus::EditorAction::OneShotImageOp {
                    tool_id: "bgremoval",
                    entity_bits: *bits,
                });
        }
        // Committed result becomes the new sprite texture; drop the
        // preview so the overlay stops painting the pre-commit copy.
        // The caller deactivates the tool on the returned flag so the
        // overlay never re-runs against the freshly baked sprite.
        *bgremoval_preview = None;
    }
    // On-canvas preview overlay — replaces the suppressed sprite
    // visually. Computes the FULL image-local → screen affine
    // (translation + rotation + scale + anchor + camera projection)
    // so the overlay tracks rotated / scaled sprites pixel-accurately
    // (Enio 2026-05-26: the old axis-aligned dest rect drifted off
    // the sprite once it spun).
    if let Some(preview) = &*bgremoval_preview {
        let entity = ph2d_ecs::Entity::from_bits(preview.entity_bits);
        if let (Some(tr), Some(sprite)) = (
            sim.world().get::<ph2d_ecs::Transform>(entity),
            sim.world().get::<Sprite>(entity),
        ) {
            let img_to_screen = sprite_image_to_screen_affine(
                preview.width,
                preview.height,
                &tr,
                &sprite,
                camera,
                window_size,
            );
            let quality = ph2d_editor::image_quality_for(hero.project.image_filter);
            vector_scene.draw_image_rgba_transformed(
                &preview.rgba,
                preview.width,
                preview.height,
                img_to_screen,
                quality,
            );
            // Protection-mask tint on top — same affine so it tracks
            // the sprite transform like the preview.
            if let Some((tint, tw, th)) = &protect_tint {
                let tint_to_screen = sprite_image_to_screen_affine(
                    *tw,
                    *th,
                    &tr,
                    &sprite,
                    camera,
                    window_size,
                );
                vector_scene.draw_image_rgba_transformed(
                    tint,
                    *tw,
                    *th,
                    tint_to_screen,
                    quality,
                );
            }
            // Brush-size ring at the cursor — the source-px radius
            // mapped to screen via the footprint scale (extracted
            // from the affine's per-axis magnitude).
            if let (Some((r_src, src_w)), Some((cur_x, cur_y))) = (
                brush_ring,
                crate::input_dispatch::protect_brush::brush_cursor(),
            ) && src_w > 0
            {
                // Scale factor from source pixels → screen pixels on
                // the X axis. `Affine` matrix is [a b c; d e f]; the
                // column vector `[a, d]` is image-X mapped to screen
                // — its magnitude is the per-pixel scale.
                let m = sprite_image_to_screen_affine(
                    preview.width,
                    preview.height,
                    &tr,
                    &sprite,
                    camera,
                    window_size,
                )
                .as_coeffs();
                let pixel_scale = (m[0] * m[0] + m[1] * m[1]).sqrt() as f32; // |col 0|
                let src_to_screen =
                    pixel_scale * preview.width as f32 / src_w as f32;
                let r_screen = r_src * src_to_screen;
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
    !apply_selection.is_empty()
}

/// Build the affine that maps image-local pixel coords (0..image_w,
/// 0..image_h, Y-down) to screen pixels (Y-down). Chains:
///   image-px → unit → sprite-local meters (Y-flip, size scale) →
///   transform.scale → transform.rotation → anchor offset →
///   translation (pivot) → camera projection.
///
/// Reused by every overlay layer (preview RGBA, protect tint) so they
/// share the EXACT same destination geometry — no per-layer drift.
fn sprite_image_to_screen_affine(
    image_w: u32,
    image_h: u32,
    tr: &ph2d_ecs::Transform,
    sprite: &Sprite,
    camera: &Camera2d,
    window_size: WindowSize,
) -> Affine {
    let image_w = image_w as f64;
    let image_h = image_h as f64;
    let size_w = sprite.size[0] as f64;
    let size_h = sprite.size[1] as f64;
    // image-px → centered, with Y flipped (image-Y is down, world-Y up):
    //   (px, py) ↦ ((px/w - 0.5) * size_w, (0.5 - py/h) * size_h)
    let img_to_local =
        Affine::scale_non_uniform(size_w / image_w, -size_h / image_h)
            * Affine::translate((-image_w * 0.5, -image_h * 0.5));
    // Sprite-local meters → world. Mirror of the sprite renderer's
    // composite: scale → rotate → anchor offset → translation pivot.
    let local_to_world =
        Affine::translate((tr.translation.x as f64, tr.translation.y as f64))
            * Affine::rotate(tr.rotation as f64)
            * Affine::translate((sprite.anchor[0] as f64, sprite.anchor[1] as f64))
            * Affine::scale_non_uniform(tr.scale.x as f64, tr.scale.y as f64);
    // World → screen. Y flips (world-Y up, screen-Y down). Uniform
    // scale `k = window.height / camera.height_world` (square pixels).
    let k = (window_size.height as f64) / (camera.height_world as f64).max(1e-6);
    let world_to_screen = Affine::translate((
        window_size.width as f64 * 0.5,
        window_size.height as f64 * 0.5,
    )) * Affine::scale_non_uniform(k, -k)
        * Affine::translate((-camera.center[0] as f64, -camera.center[1] as f64));
    world_to_screen * local_to_world * img_to_local
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
