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

    // ── (Generic) Source push when selection drifts ───────────────────────
    // Push the selected sprite's pixels into the tool's working canvas so the
    // layer stack reflects the live sprite when the selection changes.
    if painter_is_active
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
        }

        // ── Brush cursor ring (UI hint) ──────────────────────────────────
        // The brush radius (image px) scaled to screen at the cursor, while a
        // sprite is selected and the cursor is over the canvas (not a panel).
        // Uses the same footprint-AABB mapping as the paint delivery, so the
        // ring matches where dabs land. Drawn into the overlay scene (composited
        // over the canvas this frame, like the rubber-band / bgremoval ring).
        if let Some(bits) = hero.gizmo.selection {
            let (cx, cy) = cursor;
            if hero.store.panel_at(cx, cy).is_none() {
                let size_px = painter.brush_settings().size_px;
                let (iw, _ih) = painter.canvas_size();
                let entity = ph2d_ecs::Entity::from_bits(bits);
                if iw > 0
                    && let (Some(tr), Some(sprite)) = (
                        sim.world().get::<crate::Transform>(entity),
                        sim.world().get::<ph2d_render::Sprite>(entity),
                    )
                {
                    let (tx, ty) = (tr.translation.x, tr.translation.y);
                    let (sw, sh) = (sprite.size[0], sprite.size[1]);
                    let (x0, _) =
                        camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
                    let (x1, _) =
                        camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
                    let scale = (x1 - x0).abs() / iw as f32;
                    let r_screen = (size_px * scale).max(1.0);
                    use ph2d_vector::{Affine, Brush, Circle, Color, Stroke};
                    // Light-grey ring (baked inline, like the rubber-band overlay's
                    // colour — a follow-up can swap to a theme token / 2-tone).
                    let color = Color::new([0.78, 0.78, 0.78, 0.85]); // LITERAL-COLOR-OK: overlay cursor
                    vector_scene.inner_mut().stroke(
                        &Stroke::new(1.5),
                        Affine::IDENTITY,
                        &Brush::Solid(color),
                        None,
                        &Circle::new((f64::from(cx), f64::from(cy)), f64::from(r_screen)),
                    );
                }
            }
        }

        // ── Curve editor overlay (control dots + the auto-smoothed spine) ──────
        // Drawn while a Curve session is being EDITED, regardless of the cursor /
        // panels — it's the editing chrome, not a hover hint. Maps image px →
        // screen via the SAME sprite-footprint AABB as the paint delivery, so the
        // dots sit exactly on the painted curve.
        if let Some(bits) = hero.gizmo.selection
            && let Some(overlay) = painter.curve_overlay()
        {
            let (iw, ih) = painter.canvas_size();
            let entity = ph2d_ecs::Entity::from_bits(bits);
            if iw > 0
                && ih > 0
                && let (Some(tr), Some(sprite)) = (
                    sim.world().get::<crate::Transform>(entity),
                    sim.world().get::<ph2d_render::Sprite>(entity),
                )
            {
                let (tx, ty) = (tr.translation.x, tr.translation.y);
                let (sw, sh) = (sprite.size[0], sprite.size[1]);
                let (sx0, sy0) =
                    camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
                let (sx1, sy1) =
                    camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
                use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
                let map = |p: [f32; 2]| {
                    Point::new(
                        f64::from(sx0 + p[0] / iw as f32 * (sx1 - sx0)),
                        f64::from(sy0 + p[1] / ih as f32 * (sy1 - sy0)),
                    )
                };
                let scene = vector_scene.inner_mut();
                // Spine guide — the auto-smoothed curve through the control points.
                if overlay.spine.len() >= 2 {
                    let mut path = BezPath::new();
                    path.move_to(map(overlay.spine[0]));
                    for &p in &overlay.spine[1..] {
                        path.line_to(map(p));
                    }
                    let guide = Color::new([0.55, 0.72, 1.0, 0.85]); // LITERAL-COLOR-OK: curve guide
                    scene.stroke(
                        &Stroke::new(1.5),
                        Affine::IDENTITY,
                        &Brush::Solid(guide),
                        None,
                        &path,
                    );
                }
                // Control dots — the selected one larger + accented.
                let dot = Color::new([0.95, 0.95, 0.97, 0.95]); // LITERAL-COLOR-OK: curve point
                let sel = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: selected curve point
                for (i, &p) in overlay.points.iter().enumerate() {
                    let is_sel = overlay.selected == Some(i);
                    let r = if is_sel { 6.0 } else { 4.0 };
                    let c = if is_sel { sel } else { dot };
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(c),
                        None,
                        &Circle::new(map(p), r),
                    );
                }
            }
        }

        // ── Circle editor overlay (ellipse outline + 4 axis handles + rotate + centre) ──
        // Same footprint mapping as the curve overlay; the handle indices match `CircleOverlay`:
        // 0 right, 1 top, 2 left, 3 bottom, 4 rotate, 5 centre.
        if let Some(bits) = hero.gizmo.selection
            && let Some(overlay) = painter.circle_overlay()
        {
            let (iw, ih) = painter.canvas_size();
            let entity = ph2d_ecs::Entity::from_bits(bits);
            if iw > 0
                && ih > 0
                && let (Some(tr), Some(sprite)) = (
                    sim.world().get::<crate::Transform>(entity),
                    sim.world().get::<ph2d_render::Sprite>(entity),
                )
            {
                let (tx, ty) = (tr.translation.x, tr.translation.y);
                let (sw, sh) = (sprite.size[0], sprite.size[1]);
                let (sx0, sy0) =
                    camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
                let (sx1, sy1) =
                    camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
                use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
                let map = |p: [f32; 2]| {
                    Point::new(
                        f64::from(sx0 + p[0] / iw as f32 * (sx1 - sx0)),
                        f64::from(sy0 + p[1] / ih as f32 * (sy1 - sy0)),
                    )
                };
                let scene = vector_scene.inner_mut();
                let guide = Color::new([0.55, 0.72, 1.0, 0.85]); // LITERAL-COLOR-OK: ellipse guide
                // Outline.
                if overlay.perimeter.len() >= 2 {
                    let mut path = BezPath::new();
                    path.move_to(map(overlay.perimeter[0]));
                    for &p in &overlay.perimeter[1..] {
                        path.line_to(map(p));
                    }
                    path.close_path();
                    scene.stroke(
                        &Stroke::new(1.5),
                        Affine::IDENTITY,
                        &Brush::Solid(guide),
                        None,
                        &path,
                    );
                }
                // Connector from the centre to the rotation handle.
                let mut stem = BezPath::new();
                stem.move_to(map(overlay.handles[5]));
                stem.line_to(map(overlay.handles[4]));
                scene.stroke(
                    &Stroke::new(1.0),
                    Affine::IDENTITY,
                    &Brush::Solid(guide),
                    None,
                    &stem,
                );
                // Handles: axis (white), rotate (green), centre (grey), grabbed (orange).
                let axis = Color::new([0.95, 0.95, 0.97, 0.95]); // LITERAL-COLOR-OK: axis handle
                let rotate = Color::new([0.45, 0.85, 0.50, 1.0]); // LITERAL-COLOR-OK: rotation handle
                let center = Color::new([0.75, 0.78, 0.82, 0.95]); // LITERAL-COLOR-OK: centre handle
                let grab = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: grabbed handle
                for (i, &h) in overlay.handles.iter().enumerate() {
                    let grabbed = overlay.grabbed == Some(i as u8);
                    let base = match i {
                        4 => rotate,
                        5 => center,
                        _ => axis,
                    };
                    let c = if grabbed { grab } else { base };
                    let r = if grabbed { 6.0 } else { 4.0 };
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(c),
                        None,
                        &Circle::new(map(h), r),
                    );
                }
            }
        }

        // ── Polygon editor overlay (N-gon outline + 4 axis + rotate + sides + centre) ──
        // Handle indices match `PolygonOverlay`: 0 right, 1 top, 2 left, 3 bottom, 4 rotate,
        // 5 sides (changes the side count), 6 centre.
        if let Some(bits) = hero.gizmo.selection
            && let Some(overlay) = painter.polygon_overlay()
        {
            let (iw, ih) = painter.canvas_size();
            let entity = ph2d_ecs::Entity::from_bits(bits);
            if iw > 0
                && ih > 0
                && let (Some(tr), Some(sprite)) = (
                    sim.world().get::<crate::Transform>(entity),
                    sim.world().get::<ph2d_render::Sprite>(entity),
                )
            {
                let (tx, ty) = (tr.translation.x, tr.translation.y);
                let (sw, sh) = (sprite.size[0], sprite.size[1]);
                let (sx0, sy0) =
                    camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
                let (sx1, sy1) =
                    camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
                use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
                let map = |p: [f32; 2]| {
                    Point::new(
                        f64::from(sx0 + p[0] / iw as f32 * (sx1 - sx0)),
                        f64::from(sy0 + p[1] / ih as f32 * (sy1 - sy0)),
                    )
                };
                let scene = vector_scene.inner_mut();
                let guide = Color::new([0.55, 0.72, 1.0, 0.85]); // LITERAL-COLOR-OK: polygon guide
                // Closed outline through the vertices.
                if overlay.perimeter.len() >= 2 {
                    let mut path = BezPath::new();
                    path.move_to(map(overlay.perimeter[0]));
                    for &p in &overlay.perimeter[1..] {
                        path.line_to(map(p));
                    }
                    path.close_path();
                    scene.stroke(
                        &Stroke::new(1.5),
                        Affine::IDENTITY,
                        &Brush::Solid(guide),
                        None,
                        &path,
                    );
                }
                // Connectors from the centre to the rotation + sides handles.
                for h in [overlay.handles[4], overlay.handles[5]] {
                    let mut stem = BezPath::new();
                    stem.move_to(map(overlay.handles[6]));
                    stem.line_to(map(h));
                    scene.stroke(
                        &Stroke::new(1.0),
                        Affine::IDENTITY,
                        &Brush::Solid(guide),
                        None,
                        &stem,
                    );
                }
                let axis = Color::new([0.95, 0.95, 0.97, 0.95]); // LITERAL-COLOR-OK: axis handle
                let rotate = Color::new([0.45, 0.85, 0.50, 1.0]); // LITERAL-COLOR-OK: rotation handle
                let sides = Color::new([0.40, 0.78, 0.95, 1.0]); // LITERAL-COLOR-OK: sides handle
                let center = Color::new([0.75, 0.78, 0.82, 0.95]); // LITERAL-COLOR-OK: centre handle
                let grab = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: grabbed handle
                for (i, &h) in overlay.handles.iter().enumerate() {
                    let grabbed = overlay.grabbed == Some(i as u8);
                    let base = match i {
                        4 => rotate,
                        5 => sides,
                        6 => center,
                        _ => axis,
                    };
                    let c = if grabbed { grab } else { base };
                    let r = if grabbed { 6.0 } else { 4.0 };
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(c),
                        None,
                        &Circle::new(map(h), r),
                    );
                }
            }
        }

        // ── Stencil texture overlay (rect outline + drag handles of the image-space mask) ──
        // The stencil is positioned/sized via its handles (corners = resize, centre = move) or the
        // Texture section's Offset/Size sliders; Angle rotates it. The outline shows where the mask
        // lets paint through.
        if let Some(bits) = hero.gizmo.selection
            && let Some(overlay) = painter.stencil_overlay()
        {
            let (iw, ih) = painter.canvas_size();
            let entity = ph2d_ecs::Entity::from_bits(bits);
            if iw > 0
                && ih > 0
                && let (Some(tr), Some(sprite)) = (
                    sim.world().get::<crate::Transform>(entity),
                    sim.world().get::<ph2d_render::Sprite>(entity),
                )
            {
                let (tx, ty) = (tr.translation.x, tr.translation.y);
                let (sw, sh) = (sprite.size[0], sprite.size[1]);
                let (sx0, sy0) =
                    camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
                let (sx1, sy1) =
                    camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
                use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
                let map = |p: [f32; 2]| {
                    Point::new(
                        f64::from(sx0 + p[0] / iw as f32 * (sx1 - sx0)),
                        f64::from(sy0 + p[1] / ih as f32 * (sy1 - sy0)),
                    )
                };
                let scene = vector_scene.inner_mut();
                let guide = Color::new([1.0, 0.62, 0.20, 0.9]); // LITERAL-COLOR-OK: stencil outline
                let mut path = BezPath::new();
                path.move_to(map(overlay.corners[0]));
                for &p in &overlay.corners[1..] {
                    path.line_to(map(p));
                }
                path.close_path();
                scene.stroke(
                    &Stroke::new(1.5),
                    Affine::IDENTITY,
                    &Brush::Solid(guide),
                    None,
                    &path,
                );
                // Handles: 4 corners (resize) + centre (move); the grabbed one is larger + orange.
                let handle = Color::new([0.95, 0.95, 0.97, 0.95]); // LITERAL-COLOR-OK: stencil handle
                let grab = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: grabbed handle
                for (i, &p) in overlay
                    .corners
                    .iter()
                    .enumerate()
                    .chain(std::iter::once((4usize, &overlay.center)))
                {
                    let grabbed = overlay.grabbed == Some(i as u8);
                    let c = if grabbed { grab } else { handle };
                    let r = if grabbed { 6.0 } else { 4.0 };
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(c),
                        None,
                        &Circle::new(map(p), r),
                    );
                }
            }
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
