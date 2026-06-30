//! Painter (layers + effects) panel ⟷ tool bridge + on-canvas live preview.
//!
//! Modeled after `bgremoval_preview.rs`. What it does:
//!
//! 1. (Generic) Pushes the active sprite's RGBA into `PainterTool`
//!    via [`ph2d_tool_runtime::drive_source_push`] over the
//!    [`RasterEditTool`] upcast (so the layer stack reflects the live
//!    sprite pixels when the selection changes).
//! 2. (Painter-specific) Zero-copy preview drain via downcast +
//!    [`ph2d_tool_painter::PainterTool::take_preview_arc`] — bypasses
//!    [`ph2d_tool_runtime::drive_preview_cache`] (which would `to_vec` the
//!    buffer). Touches the allocator only on `Arc::make_mut` cycles.
//! 3. (Generic) Captures multi-sprite Apply selection via
//!    [`ph2d_tool_runtime::drive_pending_commit`] — `request_commit`
//!    sets the flag; bridge converts to `EditorAction::OneShotImageOp`.
//! 4. (Painter-specific) Inactive-path cache clear (mirror of
//!    BgRemoval's safety pattern).
//! 5. (Painter-specific) GPU preview lifecycle — uploads the composite
//!    into an `IndividualTextureStore` slot for the next frame's
//!    `PreviewOverride` (sprite suppression — see below).
//!
//! ## Cleanup semantics
//!
//! Apply (`pending_commit` true) returns the multi-sprite selection;
//! the caller drops the preview cache and pushes
//! `EditorAction::OneShotImageOp { tool_id: "painter", entity_bits }`
//! per entity. `Painter`'s `run_full` returns the composited layer stack which
//! the shell's image_edit dispatch writes back into the sprite texture
//! (same path as bgremoval / CEQ / upscale).
//!
//! ## Sprite suppression (replaces the Vello overlay)
//!
//! The live preview is the LAYER COMPOSITE (base layer = the sprite image
//! itself). It no longer paints as a Vello overlay ON TOP of the
//! still-rendered sprite (that duplicated the image: lowering the base
//! layer's opacity faded only the overlay, revealing the full-opacity sprite
//! underneath — Enio smoke 2026-06-01). Instead, mirroring BgRemoval, the
//! bridge uploads the premultiplied composite into an `IndividualTextureStore`
//! slot ([`PainterPreviewGpu`]); the next frame's `sim_extract` emits a
//! `PreviewOverride` that SUPPRESSES the source sprite and samples this
//! texture in its place. So the composite (incl. base-layer opacity) IS the
//! sprite, in-place, through the same sprite shader as Apply.

use super::painter_bridge_assets::{load_brush_shape_image, load_brush_texture_image};
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
    // The composite preview goes through the sprite pipeline (PreviewOverride),
    // not a Vello overlay — but these drive the on-canvas **brush cursor ring**
    // (a UI hint drawn into the overlay scene). See module header + the ring below.
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    // Last cursor position in screen pixels (for the brush cursor ring).
    cursor: (f32, f32),
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

    // ── Dock visibility ───────────────────────────────────────────────────
    // When the painter (layers + effects) tool is active, the shared Inspector
    // slot is taken over by the docked Layers panel. Edge-triggered inspector
    // hide so it doesn't stomp a manual rail toggle.
    hero.panel_visibility
        .insert("painter_layers", painter_is_active);
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(painter_is_active, Ordering::Relaxed);
        if was != painter_is_active {
            hero.panel_visibility
                .insert("inspector", !painter_is_active);
            // Bring the layers panel into the paint z-order the moment the tool
            // opens — `PAINTER_LAYERS_PANEL` is NOT in the editor-core `z_order`
            // fallback list, so without this it stays out of the paint walk.
            if painter_is_active {
                hero.store
                    .bump_panel_z(ph2d_editor::ids::PAINTER_LAYERS_PANEL);
            }
        }
    }

    // ── Source push when the selection drifts → bind the painter document ──
    // Push the selected sprite's pixels into the painter, but via `bind_document` (NOT the generic
    // `set_source`) so the OUTGOING sprite's multi-layer stack is stashed by id instead of flattened —
    // switching sprites preserves each sprite's layers (Enio 2026-06-26). Painter canvas storage is RGBA8
    // straight (matches bgremoval's `into_straight()`); 0×0 sources are rejected at the boundary.
    if painter_is_active
        && let Some(tool) = tools.active_mut()
        && let Some(painter) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_painter::PainterTool>()
        && let Some(bits) = hero.gizmo.selection
        && *last_painter_pushed_entity != Some(bits)
        && let Some(src) = crate::hero_intents::texture_edit::read_sprite_source(
            ph2d_ecs::Entity::from_bits(bits),
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
        )
    {
        let straight = src.image.into_straight();
        if straight.width != 0 && straight.height != 0 {
            painter.bind_document(bits, straight.pixels, straight.width, straight.height);
            *last_painter_pushed_entity = Some(bits);
        }
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
        // Cmd/Ctrl+Enter Apply — bake the layer composite into the sprite this
        // same frame (the `drive_pending_commit` drain below picks it up).
        if commit_requested {
            painter.request_commit();
        }
        // Structural undo/redo (Cmd+Z / Cmd+Shift+Z). Both mark the preview
        // dirty so the `take_preview_arc` drain below re-composites this frame.
        if undo_requested {
            painter.undo_last();
        } else if redo_requested {
            painter.redo_last();
        }
        // The user picked the Image texture kind → open a native file picker, decode + install the
        // luminance as the brush texture (the pure engine has no file I/O). Cancel/failure reverts.
        if painter.take_brush_texture_image_request() {
            load_brush_texture_image(painter, asset_db, toasts);
        }
        // Same path for the brush **Shape** (silhouette) slot when the user picks Image in the Shape
        // dropdown. Cancel/failure simply leaves the silhouette as the falloff (nothing to revert).
        if painter.take_brush_shape_image_request() {
            load_brush_shape_image(painter, asset_db, toasts);
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

        // (B.5 perf) Layers snapshot publish pro docked layers panel.
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
            // (Brush UI) Publish the active brush snapshot every frame — a tiny
            // Copy struct (size/colour/blend). The panel's Brush section reads it
            // to position the Size/RGB sliders + the blend chip. Not revision-
            // gated: brush edits don't bump `layers_revision`, and the cost is a
            // few floats.
            ph2d_panel_painter_layers::set_current_brush(Some(painter.brush_settings()));
            // (Brush UI) Publish the dock view-mode so the panel renders either
            // the Layers/Effects body or the Brush-properties body (header toggle).
            ph2d_panel_painter_layers::set_current_dock_shows_layers(painter.dock_shows_layers());
            // (Texture preview) Publish the brush Image texture (lum + dims) for the panel's Texture
            // preview — gated on the tool's image version so the heavy `Vec` is cloned only on change.
            {
                use std::sync::atomic::{AtomicU64, Ordering};
                static LAST_TEX_IMG_VER: AtomicU64 = AtomicU64::new(u64::MAX);
                let ver = painter.brush_texture_image_version();
                if LAST_TEX_IMG_VER.swap(ver, Ordering::Relaxed) != ver {
                    let img = painter
                        .brush_texture_image()
                        .map(|(lum, w, h)| (std::sync::Arc::new(lum.to_vec()), w, h));
                    ph2d_panel_painter_layers::set_current_brush_texture_image(img);
                }
            }
            // (Shape preview) Publish the brush Shape image (the silhouette tip) the same way — gated
            // on the tool's shape-image version so the heavy `Vec` is cloned only on change.
            {
                use std::sync::atomic::{AtomicU64, Ordering};
                static LAST_SHAPE_IMG_VER: AtomicU64 = AtomicU64::new(u64::MAX);
                let ver = painter.brush_shape_image_version();
                if LAST_SHAPE_IMG_VER.swap(ver, Ordering::Relaxed) != ver {
                    let img = painter
                        .brush_shape_image()
                        .map(|(lum, w, h)| (std::sync::Arc::new(lum.to_vec()), w, h));
                    ph2d_panel_painter_layers::set_current_brush_shape_image(img);
                }
            }
            // (Shape source linkage) If the multi-layer Shape was captured from the ACTIVE sprite, re-capture
            // it when that sprite changed (paint / opacity / visibility / undo) — keeping the per-layer
            // colours. Cheap revision compare per frame; re-captures only on a change. Before the preview
            // refresh so the preview reflects the re-captured Shape the same frame.
            painter.refresh_shape_source_if_changed();
            // (Shape preview, Per-Layer Color) Publish the multi-layer COLOURED composite so the Shape
            // preview shows the per-layer colours — the colours need the per-layer pixels, which only the
            // tool has. The tool re-bakes the composite ONLY when the Shape appearance changes (a cheap
            // key-compare per frame), so we publish (and pay the bake) on an edit, never per frame.
            if painter.refresh_shape_color_preview() {
                ph2d_panel_painter_layers::set_current_brush_shape_color_preview(
                    painter.shape_color_preview(),
                );
            }
        }

        super::painter_bridge_overlays::draw_overlays(
            painter,
            hero,
            sim,
            camera,
            window_size,
            vector_scene,
            cursor,
        );
        // Repeat Image: the 3×3 tile preview (the composite drawn at the 8 neighbour positions).
        super::painter_bridge_overlays::draw_repeat_image(
            painter,
            hero,
            sim,
            camera,
            window_size,
            vector_scene,
            painter_preview.as_ref(),
        );
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
                // Bisection toggle `PH2D_PAINT_FULL_UPLOAD=1`: force a FULL upload (disable the B.1 partial
                // lane) to bisect the "rectangular artifacts". See `HANDOFF_per_layer_color_perf_artifacts`.
                static FORCE_FULL_UPLOAD: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
                let force_full = *FORCE_FULL_UPLOAD
                    .get_or_init(|| std::env::var_os("PH2D_PAINT_FULL_UPLOAD").is_some());
                let partial = (!force_full)
                    .then_some(painter_dirty_bbox)
                    .flatten()
                    .and_then(|(bx, by, bw, bh)| match *painter_preview_gpu {
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

/// Drive the live preview of a sprite used as the brush **Shape** while it is NOT the selected sprite, so
/// brush opacity/blend remote-control edits show on it in real time. Mirrors the active-sprite preview but
/// into a SEPARATE [`IndividualTextureStore`] slot ([`AppState::painter_shape_source_preview_gpu`]): the
/// painter composites the stashed Shape-source document (only when dirty) and we upload it; the next
/// frame's `sim_extract` emits a second `PreviewOverride` suppressing that sprite + sampling this slot.
/// Released when the painter deactivates, the Shape source becomes the selected sprite, or it is cleared.
pub(super) fn drive_shape_source_preview(
    tools: &mut ToolRegistry,
    renderer: &mut SpriteRenderer,
    shape_source_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) {
    let painter_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
        .unwrap_or(false);
    let painter = painter_active
        .then(|| tools.active_mut())
        .flatten()
        .and_then(|t| {
            t.as_any_mut()
                .downcast_mut::<ph2d_tool_painter::PainterTool>()
        });
    let Some(painter) = painter else {
        release_preview_texture(renderer, shape_source_preview_gpu);
        return;
    };
    let Some(sprite) = painter.shape_source_preview_sprite() else {
        release_preview_texture(renderer, shape_source_preview_gpu);
        return;
    };
    // Re-acquire if the slot is stale (a different shape-source sprite).
    if shape_source_preview_gpu.is_some_and(|g| g.entity_bits != sprite) {
        release_preview_texture(renderer, shape_source_preview_gpu);
    }
    // Recomposite + upload ONLY when the source changed (dirty); otherwise the held slot is re-used every
    // frame (the override samples it), so a static shape source costs nothing.
    let Some((bytes, w, h)) = painter.take_shape_source_preview() else {
        return;
    };
    let mut premul = bytes;
    premultiply_rgba8(&mut premul);
    let result = match *shape_source_preview_gpu {
        Some(gpu) if gpu.width == w && gpu.height == h => renderer
            .replace_individual_pixels(gpu.texture_id, w, h, &premul)
            .map(|()| gpu.texture_id),
        _ => {
            release_preview_texture(renderer, shape_source_preview_gpu); // dims changed → fresh slot
            renderer.acquire_individual(w, h, &premul)
        }
    };
    match result {
        Ok(texture_id) => {
            *shape_source_preview_gpu = Some(PainterPreviewGpu {
                texture_id,
                width: w,
                height: h,
                arc_token: 0, // not Arc-cached; dirty-gated re-upload instead
                entity_bits: sprite,
            });
        }
        Err(e) => {
            toasts.push(Toast::error(format!(
                "Painter: upload da preview da imagem-shape falhou ({e})."
            )));
            release_preview_texture(renderer, shape_source_preview_gpu);
        }
    }
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
