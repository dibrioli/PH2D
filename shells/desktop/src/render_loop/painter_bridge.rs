//! Painter panel ⟷ tool bridge + on-canvas live preview (W1 T1.5).
//!
//! Modeled after `bgremoval_preview.rs` but **without** protect-mask
//! tint / brush ring / panel snapshot publish (those land in W2 with
//! the Procreate-style sidebar). What stays:
//!
//! 1. (Generic) Pushes the active sprite's RGBA into `PainterTool`
//!    via [`ph2d_tool_runtime::drive_source_push`] over the
//!    [`RasterEditTool`] upcast (so the canvas reflects the live
//!    sprite pixels at stroke start). Gated on
//!    `!is_stroke_active()` so mid-stroke selection drift doesn't
//!    silently wipe the working canvas (R3-LF-2).
//! 2. (Painter-specific, R4-LG-1 fast path) Zero-copy preview drain
//!    via downcast + [`ph2d_tool_painter::PainterTool::take_preview_arc`]
//!    — bypasses [`ph2d_tool_runtime::drive_preview_cache`] (which
//!    would `to_vec` the buffer, costing ~960 MB/s of allocator churn
//!    at 60 fps painting). Per-stroke painting now touches the
//!    allocator only on `Arc::make_mut` cycles (~1 clone per
//!    preview-drain frame).
//! 3. (Generic) Captures multi-sprite Apply selection via
//!    [`ph2d_tool_runtime::drive_pending_commit`] — `request_commit`
//!    sets the flag; bridge converts to `EditorAction::OneShotImageOp`.
//! 4. (Painter-specific) Inactive-path cache clear (mirror of
//!    BgRemoval's safety pattern after the Wave 10 C1 audit fix).
//! 5. (Painter-specific) On-canvas overlay paints the cached canvas
//!    RGBA on top of the sprite footprint (sprite suppression is W2 —
//!    T1.5 MVP relies on canvas alpha covering source).
//!
//! ## Cleanup semantics
//!
//! Apply (`pending_commit` true) returns the multi-sprite selection;
//! the caller drops the preview cache and pushes
//! `EditorAction::OneShotImageOp { tool_id: "painter", entity_bits }`
//! per entity. `Painter`'s `run_full` returns the canvas RGBA which the
//! shell's image_edit dispatch writes back into the sprite texture
//! (same path as bgremoval / CEQ / upscale).
//!
//! ## Sprite suppression
//!
//! T1.5 MVP intentionally does NOT suppress the underlying sprite in
//! sim_extract while a stroke is in flight — the canvas overlay paints
//! on top with full alpha so the user sees the painted state directly.
//! W2 may add suppression once the sidebar gains an "isolate stroke"
//! affordance.

use crate::app_state::PainterPreview;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite, SpriteRenderer};
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

/// Whether the active Painter tool has unflushed strokes since its last
/// source push — painting that would be LOST on a mode toggle.
///
/// `PainterTool` is a stroke/vector tool with no `RasterEditTool` impl,
/// so this query needs the concrete-type downcast. It lives in this
/// (allowlisted) bridge file rather than inline in `render_loop/mod.rs`
/// — the central dispatch must stay free of tool-concrete downcasts per
/// the `architecture_no_downcast_to_concrete_tool_in_shell` gate.
#[must_use]
pub(crate) fn painter_has_unflushed_strokes(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| {
            t.as_any_mut()
                .downcast_mut::<ph2d_tool_painter::PainterTool>()
                .map(|p| p.has_painted_since_source())
        })
        .unwrap_or(false)
}

/// Returns `true` iff an Apply committed this frame (caller tears the
/// tool down — deactivate + restore Inspector — so the on-canvas overlay
/// stops re-rendering on top of the freshly baked sprite).
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
    last_painter_pushed_entity: &mut Option<u64>,
    painter_preview: &mut Option<PainterPreview>,
    commit_requested: &mut bool,
) -> bool {
    let painter_is_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
        .unwrap_or(false);

    // W2.T2.5: consume the Cmd/Ctrl+Enter commit flag (set in
    // `handle_editor_key`). Taken unconditionally so it can't leak to a
    // later painter activation; only acted on in the downcast block below.
    let commit_requested = std::mem::take(commit_requested);

    // ── (W2.T2.1) Sidebar visibility + Inspector takeover toggle ──────────
    // Espelha o padrão BgRemoval/Padding bridge: panel sidebar é "takeover"
    // do slot Inspector quando Painter ativo. Edge-triggered inspector hide
    // pra não stompar toggle manual do rail (Wave 10 Etapa 4 fix).
    hero.panel_visibility
        .insert("painter_sidebar", painter_is_active);
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(painter_is_active, Ordering::Relaxed);
        if was != painter_is_active {
            hero.panel_visibility
                .insert("inspector", !painter_is_active);
        }
    }

    // ── (Generic) Source push when selection drifts ───────────────────────
    //
    // **R3-LF-2 fix:** skip source-push WHILE a stroke is active. The
    // bridge's job is to prepare the canvas for fresh painting; if the
    // user is mid-drag and `gizmo.selection` happens to drift (e.g.,
    // programmatic re-select via Hierarchy ctrl-click), pushing a new
    // source mid-stroke wholesale REPLACES `canvas_rgba` and silently
    // discards every stamp deposited in the current stroke. The defensive
    // `end_stroke()` inside `set_source` only stops FUTURE stamps — it
    // doesn't recover the lost ones. Gating here keeps the in-flight
    // canvas alive; selection drift will be honoured on the next stroke
    // (Up → next Down).
    let painter_in_stroke = if painter_is_active {
        tools
            .active_mut()
            .and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
            })
            .is_some_and(|p| p.is_stroke_active())
    } else {
        false
    };
    if painter_is_active
        && !painter_in_stroke
        && let Some(tool) = tools.active_mut()
        && let Some(raster) = tool.as_raster_edit_mut()
    {
        let _pushed = ph2d_tool_runtime::drive_source_push(
            raster,
            hero.gizmo.selection,
            last_painter_pushed_entity,
            |entity| {
                let src = crate::hero_intents::texture_edit::read_sprite_source(
                    entity,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                )?;
                // Painter's canvas storage is RGBA8 straight (matches
                // bgremoval's into_straight() discipline). Subsequent
                // CPU stamp render reads/writes straight; alpha-over
                // arithmetic in cpu_render does the premul dance per
                // pixel. T-perf W5+ migration to GPU pipeline will
                // straight→premul at upload boundary (cpu_render.rs §).
                let straight = src.image.into_straight();
                // Audit T1.5 round 1 B-M4: reject degenerate sources
                // (zero-sized) at the boundary; PainterTool::set_source
                // would silently accept a 0×0 canvas and `queue_pointer`
                // would early-out invisibly.
                if straight.width == 0 || straight.height == 0 {
                    return None;
                }
                Some(ph2d_tool_runtime::RasterSource {
                    pixels: straight.pixels,
                    width: straight.width,
                    height: straight.height,
                })
            },
        );
    }

    // Audit T1.5 round 1 B-H2: NO ghost `panel_visibility` insert. Painter
    // has no docked panel in T1.5 (sidebar lands W2 via
    // `ph2d-panel-painter`); inserting into the BTreeMap every frame just
    // to flip an unread bit risks colliding with the W2 sidebar's own
    // panel_visibility key. The Painter pill's pressed state is computed
    // directly off `tools.active().id()` in the topbar paint pass.

    let mut apply_selection: Vec<u64> = Vec::new();

    // ── Drain current_preview (FAST PATH) + capture Apply ─────────────────
    //
    // **R4-LG-1 fix:** bypass `drive_preview_cache` (which does a 16 MB
    // `pixels.to_vec()` per dirty drain — at 60 fps painting that's ~960
    // MB/s of allocator churn). Painter-specific downcast lets us pull
    // the canvas as a 1-atomic-inc `Arc<Vec<u8>>` clone via
    // `take_preview_arc()`. Net: per-stroke painting touches the
    // allocator ONCE per `Arc::make_mut` cycle (i.e., once per preview-
    // drain frame), not every pointer event.
    //
    // `drive_pending_commit` stays on the generic `&mut dyn RasterEditTool`
    // path — it's only called once per Apply, not per frame.
    if let Some(tool) = tools.active_mut()
        && let Some(painter) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_painter::PainterTool>()
    {
        // W2.T2.5: Cmd/Ctrl+Enter commit-without-switching — set
        // pending_commit so the `drive_pending_commit` drain below bakes
        // the stroke into the sprite this same frame.
        if commit_requested {
            painter.request_commit();
        }
        // Selection-drift invalidation (mirror of drive_preview_cache).
        if let (Some(existing), Some(sel)) = (painter_preview.as_ref(), hero.gizmo.selection)
            && existing.entity_bits != sel
        {
            *painter_preview = None;
        }
        // Zero-copy preview drain — populates cache iff a new frame
        // arrived AND a sprite is selected to tag it with.
        if let (Some(sel), Some((rgba, w, h))) = (hero.gizmo.selection, painter.take_preview_arc())
        {
            *painter_preview = Some(ph2d_tool_runtime::PreviewCache {
                entity_bits: sel,
                rgba,
                width: w,
                height: h,
            });
        }
        // Apply / commit capture — same trait path as bgremoval.
        apply_selection = ph2d_tool_runtime::drive_pending_commit(
            painter as &mut dyn ph2d_editor::tool::RasterEditTool,
            hero.gizmo.iter_selected(),
        );

        // (W2.T2.1) Snapshot publish pro docked sidebar — `painter_sidebar`
        // panel paint() lê via thread_local current_snapshot.
        #[cfg(feature = "panel-painter-sidebar")]
        ph2d_panel_painter_sidebar::set_current_painter_snapshot(if painter_is_active {
            Some(painter.ui_snapshot())
        } else {
            None
        });
    }

    // ── Inactive path — clear LOCAL bridge state only (NOT the tool's,
    // which `on_deactivate` already cleared via `ToolRegistry::set_active`).
    // Mirror of bgremoval C1 audit fix.
    if !painter_is_active {
        *painter_preview = None;
        *last_painter_pushed_entity = None;
    }
    if !apply_selection.is_empty() {
        for bits in &apply_selection {
            hero.bus
                .push(ph2d_editor::action_bus::EditorAction::OneShotImageOp {
                    tool_id: "painter",
                    entity_bits: *bits,
                });
        }
        *painter_preview = None;
    }
    // On-canvas preview overlay (straight-alpha, on top of the sprite's
    // footprint). T1.5 MVP: the underlying sprite is NOT suppressed —
    // canvas alpha covers it; user sees the painted result directly.
    if let Some(preview) = &*painter_preview {
        let entity = ph2d_ecs::Entity::from_bits(preview.entity_bits);
        if let (Some(tr), Some(sprite)) = (
            sim.world().get::<ph2d_ecs::Transform>(entity),
            sim.world().get::<Sprite>(entity),
        ) {
            let cx = tr.translation.x + sprite.anchor[0];
            let cy = tr.translation.y + sprite.anchor[1];
            let (sw, sh) = (sprite.size[0], sprite.size[1]);
            let (x0, y0) = camera.world_to_screen([cx - sw * 0.5, cy + sh * 0.5], window_size);
            let (x1, y1) = camera.world_to_screen([cx + sw * 0.5, cy - sh * 0.5], window_size);
            let quality = ph2d_editor::image_quality_for(hero.project.image_filter);
            vector_scene.draw_image_rgba(
                &preview.rgba,
                preview.width,
                preview.height,
                (x0 as f64, y0 as f64, x1 as f64, y1 as f64),
                quality,
            );
        }
    }
    !apply_selection.is_empty()
}
