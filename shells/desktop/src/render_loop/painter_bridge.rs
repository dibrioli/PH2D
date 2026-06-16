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
//! 5. (Painter-specific) GPU preview lifecycle — uploads the composite
//!    into an `IndividualTextureStore` slot for the next frame's
//!    `PreviewOverride` (sprite suppression — see below), replacing the
//!    old Vello on-canvas overlay.
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
//! ## Sprite suppression (W3 — replaces the Vello overlay)
//!
//! The live preview is the LAYER COMPOSITE (base layer = the sprite image
//! itself + strokes). It no longer paints as a Vello overlay ON TOP of the
//! still-rendered sprite (that duplicated the image: lowering the base
//! layer's opacity faded only the overlay, revealing the full-opacity sprite
//! underneath — Enio smoke 2026-06-01). Instead, mirroring BgRemoval, the
//! bridge uploads the premultiplied composite into an `IndividualTextureStore`
//! slot ([`PainterPreviewGpu`]); the next frame's `sim_extract` emits a
//! `PreviewOverride` that SUPPRESSES the source sprite and samples this
//! texture in its place. So the composite (incl. base-layer opacity) IS the
//! sprite, in-place, through the same sprite shader as Apply.

use super::painter_gpu_preview::{self, PainterGpuPreview};
use crate::app_state::{PainterPreview, PainterPreviewGpu};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_editor::toast::{Toast, ToastQueue};
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, SpriteRenderer, premultiply_rgba8};
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;
use std::sync::Arc;

// `painter_has_unflushed_strokes` + `apply_layer_reparent` (tool-concrete downcast
// queries) moved to `painter_bridge_queries.rs` (HR-18 file-LOC cap).

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
    // Reserved (the on-canvas preview now goes through the sprite pipeline via
    // a PreviewOverride, not a Vello overlay): kept for future UI hints / the
    // GPU LayerCompositor upload path. See module header "Sprite suppression".
    _camera: &Camera2d,
    _window_size: WindowSize,
    _vector_scene: &mut VectorScene,
    last_painter_pushed_entity: &mut Option<u64>,
    painter_preview: &mut Option<PainterPreview>,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    // GPU live-preview session (compositor + premul blit). `None` until the
    // first GPU-representable frame lazily builds it (ADR-0045 Phase 3 step 2).
    painter_gpu_preview: &mut Option<PainterGpuPreview>,
    commit_requested: &mut bool,
    undo_requested: &mut bool,
    redo_requested: &mut bool,
    toasts: &mut ToastQueue,
) -> bool {
    let painter_is_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
        .unwrap_or(false);

    // W2.T2.5: consume the Cmd/Ctrl+Enter commit flag (set in
    // `handle_editor_key`). Taken unconditionally so it can't leak to a
    // later painter activation; only acted on in the downcast block below.
    let commit_requested = std::mem::take(commit_requested);
    // W2.T2.2: same unconditional-take discipline for the stroke
    // undo/redo flags (Cmd+Z / Cmd+Shift+Z while Painter is active).
    let undo_requested = std::mem::take(undo_requested);
    let redo_requested = std::mem::take(redo_requested);

    // ── (W2.T2.1 + W3.T3.4) Dock visibility (mode C toggle) ───────────────
    // Espelha o padrão BgRemoval/Padding bridge: o slot Inspector vira
    // "takeover" do Painter quando ativo. Mode C (Enio): o slot compartilhado
    // alterna entre a brush sidebar e o layers panel via
    // `PainterTool::dock_shows_layers` (flipado por qualquer um dos toggles de
    // header). Lê o flag via downcast (o estado vive no tool, não num painel —
    // evita dep panel→panel). Edge-triggered inspector hide pra não stompar
    // toggle manual do rail (Wave 10 Etapa 4 fix).
    // W5: the shared dock slot now has THREE states — brush sidebar / layers /
    // Brush Studio. The Studio takes priority when open; else the layers toggle
    // decides sidebar-vs-layers (mode C). One downcast reads both flags.
    let (painter_shows_layers, painter_shows_studio) = if painter_is_active {
        tools
            .active_mut()
            .and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
            })
            .map(|p| (p.dock_shows_layers(), p.show_brush_studio()))
            .unwrap_or((false, false))
    } else {
        (false, false)
    };
    hero.panel_visibility.insert(
        "painter_brush_studio",
        painter_is_active && painter_shows_studio,
    );
    hero.panel_visibility.insert(
        "painter_layers",
        painter_is_active && painter_shows_layers && !painter_shows_studio,
    );
    hero.panel_visibility.insert(
        "painter_sidebar",
        painter_is_active && !painter_shows_layers && !painter_shows_studio,
    );
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(painter_is_active, Ordering::Relaxed);
        if was != painter_is_active {
            hero.panel_visibility
                .insert("inspector", !painter_is_active);
            // W3.T3.4: bring the layers panel into the paint z-order the
            // moment Painter opens (canonical `bump_panel_z` "panel first
            // opened" path). Unlike `painter_sidebar`, `PAINTER_LAYERS_PANEL`
            // is NOT in the `z_order` fallback list in editor-core
            // `screens/hero.rs`, so without this it stays out of the paint
            // walk and never renders even when its visibility flips true
            // (mode C toggle → sidebar hides, layers shows nothing).
            // FOLLOW-UP (Coord): also add `ids::PAINTER_LAYERS_PANEL` to that
            // fallback list (mirror `PAINTER_SIDEBAR_PANEL`) as defense-in-depth.
            if painter_is_active {
                hero.store
                    .bump_panel_z(ph2d_editor::ids::PAINTER_LAYERS_PANEL);
                // W5: same rationale for the Brush Studio — it shares the dock
                // slot but isn't in the editor-core z_order fallback list, so it
                // must enter the paint walk explicitly or it never renders when
                // its visibility flips true (mode toggle → sidebar hides).
                hero.store
                    .bump_panel_z(ph2d_editor::ids::PAINTER_BRUSH_STUDIO_PANEL);
            }
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
    // B.1: carries the partial dirty bbox from the preview drain to the GPU
    // upload below (frame-local — same function scope, so no `PreviewCache`
    // field change needed). `Some` = upload only this sub-rect; `None` = full.
    let mut painter_dirty_bbox: Option<(u32, u32, u32, u32)> = None;
    // True when the GPU producer owns the preview slot this frame (representable
    // stack) — gates the CPU lifecycle block off so the two never fight the slot.
    let mut gpu_owns_preview = false;
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
        // W2.T2.2: stroke undo/redo. Both methods mark the preview
        // dirty, so the `take_preview_arc` drain below re-blits the
        // restored canvas this same frame. Mutually exclusive by
        // construction (one keypress sets one flag); guarded anyway so a
        // stray double-set can't undo-then-redo into a no-op.
        if undo_requested {
            painter.undo_last_stroke();
        } else if redo_requested {
            painter.redo_last_stroke();
        }
        // Selection-drift invalidation (mirror of drive_preview_cache).
        if let (Some(existing), Some(sel)) = (painter_preview.as_ref(), hero.gizmo.selection)
            && existing.entity_bits != sel
        {
            *painter_preview = None;
        }
        // GPU-vs-CPU preview decision (ADR-0045 Phase 3 step 2): representable
        // stack → GPU composite (fast slider drags), else CPU `take_preview_arc`
        // below. Both end in `painter_preview_gpu`. See `try_drive`.
        if !gpu_owns_preview {
            gpu_owns_preview = painter_gpu_preview::try_drive(
                painter_gpu_preview,
                renderer,
                painter,
                hero.gizmo.selection,
                painter_preview_gpu,
                toasts,
            );
        }
        if gpu_owns_preview {
            // CPU cache unused while the GPU owns the slot — clear it so the
            // inactive/apply release + the gated CPU block below see `None`.
            *painter_preview = None;
        } else if let (Some(sel), Some((rgba, w, h))) =
            (hero.gizmo.selection, painter.take_preview_arc())
        {
            // B.1: the bbox the drain recomposed (Some = partial fast lane).
            painter_dirty_bbox = painter.take_preview_upload_bbox();
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

        // (W5) Brush Studio snapshot publish — the panel paints the full brush
        // surface off this clone each frame (its own uncapped snapshot, separate
        // from `ui_snapshot`). Published whenever Painter is active; the panel
        // only renders when `show_brush_studio` drives its visibility.
        #[cfg(feature = "panel-brush-studio")]
        ph2d_panel_brush_studio::set_current_brush_studio_snapshot(if painter_is_active {
            Some(painter.brush_studio_snapshot())
        } else {
            None
        });

        // (W3.T3.4 + B.5 perf) Layers snapshot publish pro docked layers panel.
        // The panel paints a row per layer off this clone. **Gated on
        // `layers_revision()`:** the `LayerStack` is metadata-only, but the clone
        // (N rows × name `String`) ran EVERY frame Painter was active — including
        // every mouse-move during a layer drag, which made the reparent feel
        // sluggish (Enio 2026-06-02 "muito lenta"). `layers_revision` bumps only
        // on structural/metadata edits (`invalidate_composite` + `set_source`),
        // NOT strokes and NOT cursor moves. So during an in-flight drag the
        // structure is stable → we skip the clone entirely; the panel keeps its
        // last published snapshot and reads the live `painter_layer_drag()` cursor
        // for the overlay (panel re-paints every frame regardless). First
        // activation always publishes (sentinel `u64::MAX` ≠ any real revision);
        // the single persistent `PainterTool` instance keeps `layers_revision`
        // monotonic for the app lifetime, so an unchanged revision genuinely means
        // an unchanged stack (never a stale skip).
        #[cfg(feature = "panel-painter-layers")]
        {
            use std::sync::atomic::{AtomicU64, Ordering};
            static LAST_LAYERS_REV: AtomicU64 = AtomicU64::new(u64::MAX);
            let rev = painter.layers_revision();
            if LAST_LAYERS_REV.swap(rev, Ordering::Relaxed) != rev {
                ph2d_panel_painter_layers::set_current_layers(Some(painter.layers().clone()));
            }
            // (W3 multi-select) Publish the selection set every frame — a tiny
            // BTreeSet (≤ HARD_CAP_LAYERS u64s). NOT gated on `layers_revision`:
            // a plain re-click that collapses a multi-selection onto the
            // already-active layer changes the selection WITHOUT a structural
            // edit, so a revision gate would miss it. The panel reads this for
            // the multi-row highlight (active = strong outline, others = wash).
            ph2d_panel_painter_layers::set_current_selection(painter.selection());
        }

        // ── (W2.T2.3) Color thumb ⟷ Blender picker round-trip ──────────
        //
        // The thumb is a floating swatch in the canvas top-right that
        // opens the shared `INSP_BLENDER_PICKER` (pointer.rs Down) seeded
        // with the Painter's live color. Two directions, mutually
        // exclusive by the picker_target test to avoid a feedback loop:
        //
        //   • target == thumb  → picker is DRIVING. Push the picked sRGB
        //     into the Painter via SetColorSrgb. Do NOT write widget_color
        //     here — hero.rs's generic picker read-back
        //     (`set_widget_color(target, value.rgba)`) already mirrors the
        //     live picker value into the thumb every frame while open.
        //   • target != thumb → publish the Painter's live color into the
        //     thumb so the swatch reflects color set by other means
        //     (eyedropper, future shortcuts).
        //
        // Change-detection tracks the LAST sRGB we pushed (picker→picker
        // comparison) rather than `active_color_srgb8()`. The sRGB8↔OKLCH
        // round-trip is only stable within ±1 LSB (see `color::tests::
        // srgb_black_and_white_round_trip_within_lsb`), so comparing the
        // picker's exact bytes against the round-tripped painter color
        // would re-fire `apply_ui_edit` (→ preview-dirty) every frame on a
        // 1-LSB mismatch. Comparing picker-bytes to the last picker-bytes
        // we applied is exact and idempotent. Reset on close so re-opening
        // the same color still seeds the painter once.
        use std::sync::atomic::{AtomicU32, Ordering};
        static LAST_PUSHED_SRGB: AtomicU32 = AtomicU32::new(u32::MAX);
        if hero.store.picker_target() == Some(ph2d_editor::ids::PAINTER_COLOR_THUMB) {
            if let Some((value, _, _, _)) = hero
                .store
                .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
            {
                let packed = u32::from_le_bytes(value.rgba);
                if LAST_PUSHED_SRGB.swap(packed, Ordering::Relaxed) != packed {
                    painter
                        .apply_ui_edit(ph2d_tool_painter::PainterUiEdit::SetColorSrgb(value.rgba));
                }
            }
        } else {
            // Picker not driving the thumb: forget the applied value (so a
            // later re-open re-seeds) and mirror the live painter color
            // into the thumb swatch.
            LAST_PUSHED_SRGB.store(u32::MAX, Ordering::Relaxed);
            hero.store.set_widget_color(
                ph2d_editor::ids::PAINTER_COLOR_THUMB,
                painter.ui_snapshot().active_color_srgb8(),
            );
        }
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
        // Apply baked the strokes — release the preview slot explicitly (the GPU
        // producer gated the CPU `None => release` off) and hand bookkeeping back.
        release_preview_texture(renderer, painter_preview_gpu);
        gpu_owns_preview = false;
    }
    // ── GPU lifecycle for the live-preview texture (W3 sprite-suppression) ──
    // Mirror of `bgremoval_preview`: upload the premultiplied composite into a
    // transient `IndividualTextureStore` slot; NEXT frame's `sim_extract` reads
    // `painter_preview_gpu` to emit a `PreviewOverride` that SUPPRESSES the
    // source sprite and samples THIS texture in its place. The composite is
    // STRAIGHT sRGB8 (the canvas / `take_preview_arc`); byte-space
    // `premultiply_rgba8` matches EXACTLY what Apply's
    // `SpriteImage::into_premultiplied` produces, so the live preview is
    // byte-for-byte identical to the committed result on the same
    // `Rgba8UnormSrgb` + premul-blend sprite shader (no Vello gamma/blend
    // divergence, no image duplication). 1-frame lag is imperceptible.
    //
    // On a GPU-owned frame the GPU producer fills the slot; hide the CPU cache
    // from this block so it neither re-uploads nor releases that slot.
    let cpu_preview = painter_preview.as_ref().filter(|_| !gpu_owns_preview);
    match cpu_preview {
        Some(preview) => {
            let cache_token = Arc::as_ptr(&preview.rgba) as usize;
            let needs_upload = match *painter_preview_gpu {
                None => true,
                Some(gpu) => {
                    gpu.arc_token != cache_token
                        || gpu.entity_bits != preview.entity_bits
                        || gpu.width != preview.width
                        || gpu.height != preview.height
                }
            };
            if needs_upload {
                // B.1: partial sub-rect upload when the drain reported a tracked
                // dirty bbox AND a matching GPU texture already holds the prior
                // full frame (same entity + dims). The fast lane only fires after
                // a full upload synced the texture (the composite cache is `Some`
                // only post-full-recompose, which uploads `bbox == None`), and any
                // structural / metadata / dims / entity change forces a full
                // upload — so the un-touched GPU pixels are always current. The
                // `bx+bw<=w && by+bh<=h` guard keeps `extract_region` in bounds
                // (defensive — a bad bbox falls back to full, never panics the
                // render loop). Everything else → full upload.
                let partial =
                    painter_dirty_bbox.and_then(|(bx, by, bw, bh)| match *painter_preview_gpu {
                        Some(gpu)
                            if gpu.entity_bits == preview.entity_bits
                                && gpu.width == preview.width
                                && gpu.height == preview.height
                                && bw > 0
                                && bh > 0
                                && bx + bw <= preview.width
                                && by + bh <= preview.height =>
                        {
                            Some((gpu.texture_id, bx, by, bw, bh))
                        }
                        _ => None,
                    });
                let upload_result: Result<u32, _> = match partial {
                    Some((texture_id, bx, by, bw, bh)) => {
                        // Gather + premultiply ONLY the bbox sub-rect (tightly
                        // packed bw*bh*4) and upload it over the existing texture.
                        let mut region =
                            extract_region(&preview.rgba, preview.width, bx, by, bw, bh);
                        premultiply_rgba8(&mut region);
                        renderer
                            .replace_individual_pixels_region(texture_id, bx, by, bw, bh, &region)
                            .map(|()| texture_id)
                    }
                    None => {
                        let mut premul_bytes = (*preview.rgba).clone();
                        premultiply_rgba8(&mut premul_bytes);
                        match *painter_preview_gpu {
                            Some(gpu) => renderer
                                .replace_individual_pixels(
                                    gpu.texture_id,
                                    preview.width,
                                    preview.height,
                                    &premul_bytes,
                                )
                                .map(|()| gpu.texture_id),
                            None => renderer.acquire_individual(
                                preview.width,
                                preview.height,
                                &premul_bytes,
                            ),
                        }
                    }
                };
                match upload_result {
                    Ok(texture_id) => {
                        *painter_preview_gpu = Some(PainterPreviewGpu {
                            texture_id,
                            width: preview.width,
                            height: preview.height,
                            arc_token: cache_token,
                            entity_bits: preview.entity_bits,
                        });
                    }
                    Err(e) => {
                        toasts.push(Toast::error(format!(
                            "Painter: upload da preview pra GPU falhou ({e}). \
                             Tentando novamente no próximo frame."
                        )));
                        release_preview_texture(renderer, painter_preview_gpu);
                    }
                }
            }
        }
        None => {
            // Release only when the CPU path owns the slot; on a GPU-owned frame
            // the GPU producer owns it — leave it intact for next frame.
            if !gpu_owns_preview {
                release_preview_texture(renderer, painter_preview_gpu);
            }
        }
    }
    !apply_selection.is_empty()
}

/// Gather a tightly-packed `w*h*4` RGBA8 sub-rect at `(x, y)` out of a
/// canvas-sized straight buffer (row stride `stride_px*4`) — the inverse of the
/// compositor's `blit_region`, for the B.1 partial GPU upload. The caller's
/// guard (`x+w <= stride_px`, `y+h <= height`) keeps every row copy in bounds.
fn extract_region(full: &[u8], stride_px: u32, x: u32, y: u32, w: u32, h: u32) -> Vec<u8> {
    let row_bytes = (w * 4) as usize;
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for ry in 0..h {
        let src_off = (((y + ry) * stride_px + x) * 4) as usize;
        let dst_off = (ry * w * 4) as usize;
        out[dst_off..dst_off + row_bytes].copy_from_slice(&full[src_off..src_off + row_bytes]);
    }
    out
}

/// Release the Painter live-preview's `IndividualTextureStore` slot (if any)
/// and zero the GPU cache. Called when the preview cache turns `None` (tool
/// deactivated, Apply committed, no source) and on upload error — next frame
/// re-acquires from scratch.
fn release_preview_texture(
    renderer: &mut SpriteRenderer,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
) {
    if let Some(gpu) = painter_preview_gpu.take() {
        renderer.individual_mut().release(gpu.texture_id);
    }
}
