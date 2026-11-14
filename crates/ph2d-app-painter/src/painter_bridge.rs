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
//! ## Sprite suppression
//!
//! The live preview is the LAYER COMPOSITE, uploaded into an `IndividualTextureStore` slot
//! ([`PainterPreviewGpu`]); the next frame's `PreviewOverride` SUPPRESSES the source sprite and
//! samples this texture in its place, through the same sprite shader as Apply.
//! (história: docs/Painter/historia_comentarios_app_painter_2026-11-10.md)

use crate::painter_bridge_assets::{load_brush_shape_image, load_brush_texture_image};
use crate::painter_gpu_preview::{self, PainterGpuPreview};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::ToolRegistry;
use ph2d_editor_core::toast::ToastQueue;
use ph2d_host::WindowSize;
use ph2d_preview_slot::PreviewGpu as PainterPreviewGpu;
use ph2d_render::{Camera2d, SpriteRenderer};
use ph2d_tool_runtime::PreviewCache as PainterPreview;
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

/// Frame cap for the `PH2D_PREVIEW_DUMP` diagnostic trap (BUGS_painter.md #11).
const PREVIEW_DUMP_MAX_FRAMES: u32 = 240;

/// Returns `true` iff an Apply committed this frame (caller tears the
/// tool down — deactivate + restore Inspector — so the on-canvas overlay
/// stops re-rendering on top of the freshly baked sprite).
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    sim: &SimWorld,
    // The PRESENTATION world (read-only): where skeleton-bound posed art lives; the canvas chrome
    // and the handle grab both read it.
    present: &ph2d_ecs::World,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    // The composite goes through the sprite pipeline; these drive the on-canvas brush cursor ring.
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    // Text system for on-canvas labels (the Line dimension overlay's px / angle numbers).
    text_system: &mut ph2d_text::TextSystem,
    // Last cursor position in screen pixels (for the brush cursor ring).
    cursor: (f32, f32),
    last_painter_pushed_entity: &mut Option<u64>,
    painter_preview: &mut Option<PainterPreview>,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    // GPU live-preview session (compositor + premul blit), built lazily.
    painter_gpu_preview: &mut Option<PainterGpuPreview>,
    commit_requested: &mut bool,
    undo_requested: &mut bool,
    redo_requested: &mut bool,
    toasts: &mut ToastQueue,
    // `true` while a pointer button is held (gesture in flight) — see `set_shape_draft_hold`.
    pointer_held: bool,
    // `false` once the pointer left the window or it lost focus: the cursor-following chrome hides.
    pointer_inside: bool,
    // The bucket colour-drag state, resolved by the shell (input layer).
    fill_drag_armed: bool,
    // The sprite pixel reader from the shell: STRAIGHT alpha `Vec<u8>` + dims.
    read_source: impl FnOnce(
        ph2d_ecs::Entity,
        &SimWorld,
        &mut SpriteRenderer,
        &AssetDb,
        &BTreeMap<u32, AssetId>,
    ) -> Option<(Vec<u8>, u32, u32)>,
    // The frame profiler's counter from the shell (`render_loop::note_preview_px`).
    note_preview_px: &dyn Fn(u64),
) -> bool {
    // `PH2D_PAINT_PERF=1`: per-frame producer, drain path, phase timings and canvas dims.
    // Zero cost when unset.
    let perf_t0 = crate::paint_perf::on().then(std::time::Instant::now);
    let mut dbg_trivial = false;
    let mut dbg_gray = false;
    let mut dbg_active_is_mask = false;
    let mut dbg_dims = (0u32, 0u32);
    // Which arm the last drain took + WHY a trivial stack fell to the full arm (impasto / mask scratch).
    let mut dbg_branch = ph2d_tool_painter::DrainBranch::Idle;
    let mut dbg_impasto = false;
    let mut dbg_mask_scratch = false;

    let painter_is_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor_core::ToolId::new("painter"))
        .unwrap_or(false);

    // Commit/undo/redo flags are taken unconditionally so they can't leak to a later activation.
    let commit_requested = std::mem::take(commit_requested);
    let undo_requested = std::mem::take(undo_requested);
    let redo_requested = std::mem::take(redo_requested);

    // ── Dock visibility ───────────────────────────────────────────────────
    crate::painter_bridge_phases::dock_visibility(hero, painter_is_active);

    // ── C&F colour picker → brush colour (single source of truth, all modes) ──
    crate::painter_bridge_phases::forward_picker_colour(hero, tools);

    // ── Source push when the painter has no document for the selection → bind it ──
    crate::painter_bridge_phases::bind_document(
        crate::painter_bridge_phases::BindCtx {
            hero,
            tools,
            renderer,
            last_painter_pushed_entity,
            painter_preview_gpu,
            painter_gpu_preview,
            toasts,
        },
        painter_is_active,
        // The read trio is captured here, not passed on.
        |entity, renderer| read_source(entity, sim, renderer, asset_db, atlas_asset_map),
    );
    #[cfg(feature = "panel-painter-layers")]
    if painter_is_active
        && let Some(p) = tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut())
    {
        crate::wetpaint_fps::conduz(p);
    }

    let mut apply_selection: Vec<u64> = Vec::new();

    // ── Drain current_preview (FAST PATH) + capture Apply ─────────────────
    // The downcast pulls the canvas as an `Arc` clone (`take_preview_arc`), bypassing the
    // per-drain `to_vec` of `drive_preview_cache`. Dirty bbox: `Some` = sub-rect, `None` = full.
    let mut painter_dirty_bbox: Option<(u32, u32, u32, u32)> = None;
    let mut painter_dirty_caixas: Vec<(u32, u32, u32, u32)> = Vec::new();
    // The tool's canvas CONTENT version: the upload keys on it, not on the `Arc` pointer, so the
    // shell never holds a clone of the live canvas (which made every stamp copy the plane).
    let mut cache_version: u64 = 0;
    // True when the GPU producer owns the preview slot this frame (representable
    // stack) — gates the CPU lifecycle block off so the two never fight the slot.
    let mut gpu_owns_preview = false;
    // PH2D_PAINT_PERF sub-phase accumulators (ms) — which part of dispatch a slow frame spends its
    // time in. `elapsed_ms` reads the mark only when the env is on (the `Instant` is `None` otherwise).
    let (mut ph_preview, mut ph_panel, mut ph_overlay) = (0f32, 0f32, 0f32);
    // …and the OVERLAY phase split by call.
    let (mut ph_ov_tol, mut ph_ov_selection, mut ph_ov_chrome) = (0f32, 0f32, 0f32);
    // …and the same treatment for PANEL (by step) and CHROME (by overlay call).
    let mut ph_panel_sub = [0f32; crate::paint_perf::PANEL_SUB];
    let mut ph_chrome_sub = [0f32; crate::paint_perf::CHROME_SUB];
    let elapsed_ms =
        |m: Option<std::time::Instant>| m.map_or(0.0, |t| t.elapsed().as_secs_f64() as f32 * 1e3);
    let m_preview = perf_t0.map(|_| std::time::Instant::now());
    crate::painter_bridge_eyedropper_cursor::esquece_a_mira(); // só a desenha quem a redesenhar
    if let Some(tool) = tools.active_mut()
        && let Some(painter) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_painter::PainterTool>()
    {
        // Knob-drag RELEASE: no event arrives on release, so the hold drops here, every frame.
        painter.set_shape_draft_hold(pointer_held);
        crate::composite_smoke::rabisca(painter);
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
        // Image texture kind picked → native file picker, install the luminance (cancel reverts).
        if painter.take_brush_texture_image_request() {
            load_brush_texture_image(painter, asset_db, toasts);
        }
        // Same for the brush Shape slot (cancel leaves the falloff).
        if painter.take_brush_shape_image_request() {
            load_brush_shape_image(painter, asset_db, toasts);
        }
        // A ponta de N imagens (L19): «Add image» pede uma imagem a mais para a Shape.
        if painter.take_brush_shape_hose_request() {
            crate::painter_bridge_assets::load_brush_shape_hose_image(painter, asset_db, toasts);
        }
        // O dual brush (L6): «Dual image» pede a imagem da 2.ª ponta.
        if painter.take_brush_dual_image_request() {
            crate::painter_bridge_assets::load_brush_dual_image(painter, asset_db, toasts);
        }
        // Presets of the four media: the panel request is served here (the disk is the bridge's).
        #[cfg(feature = "panel-painter-layers")]
        crate::painter_bridge_presets::drive(painter, toasts);
        // O preview VIVO do pincel (A15: em todos os meios): repinta-se numa thread a cada mudança.
        #[cfg(feature = "panel-painter-layers")]
        crate::painter_bridge_preview::drive(painter);
        // Selection-drift invalidation (mirror of drive_preview_cache).
        if let (Some(existing), Some(sel)) = (painter_preview.as_ref(), hero.gizmo.selection)
            && existing.entity_bits != sel
        {
            *painter_preview = None;
        }
        // GPU-vs-CPU preview decision: representable stack → GPU, else CPU below. See `try_drive`.
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
        } else if let (Some(sel), Some((drained, w, h))) =
            (hero.gizmo.selection, painter.take_preview_arc())
        {
            // B.1: the bbox the drain recomposed (Some = partial fast lane).
            painter_dirty_caixas = painter.preview_upload_caixas().collect();
            painter_dirty_bbox = painter.take_preview_upload_bbox();
            // Published area (`painter-dispatch` divisor). ⚠️ bbox is `(x, y, w, h)`, not corners.
            note_preview_px(painter_dirty_bbox.map_or_else(
                || u64::from(w) * u64::from(h),
                |r| {
                    crate::painter_bridge_upload::janelas_do_upload(r, &painter_dirty_caixas)
                        .iter()
                        .map(|&(_, _, bw, bh)| u64::from(bw) * u64::from(bh))
                        .sum()
                },
            ));
            // `PH2D_PREVIEW_DUMP=<dir>`: dump each frame's CPU composite (pre-overlay) as PNG,
            // capped (BUGS_painter.md #11). Zero cost when unset.
            static DUMP: std::sync::OnceLock<Option<std::ffi::OsString>> =
                std::sync::OnceLock::new();
            if let Some(dir) = DUMP.get_or_init(|| std::env::var_os("PH2D_PREVIEW_DUMP")) {
                use std::sync::atomic::{AtomicU32, Ordering};
                static N: AtomicU32 = AtomicU32::new(0);
                let n = N.fetch_add(1, Ordering::Relaxed);
                if n < PREVIEW_DUMP_MAX_FRAMES {
                    let path = std::path::Path::new(&dir).join(format!("preview_{n:04}.png"));
                    let _ = image::save_buffer(&path, &drained[..], w, h, image::ColorType::Rgba8);
                }
            }
            // Patch the dirty region into the shell's OWN buffer and drop `drained`: the tool
            // stays the sole owner of its canvas, so its next `Arc::make_mut` writes in place.
            crate::painter_bridge_repeat::larga(); // B20.10: sole owner again ⇒ patch in place
            let mirror = crate::painter_bridge_upload::own_preview_buffer(
                painter_preview.take(),
                sel,
                w,
                h,
                &drained,
                painter_dirty_bbox,
                &painter_dirty_caixas,
            );
            *painter_preview = Some(ph2d_tool_runtime::PreviewCache {
                entity_bits: sel,
                rgba: mirror,
                width: w,
                height: h,
            });
        }
        // Content version for the upload plan (unchanged on an idle frame → the plan Skips).
        cache_version = painter.canvas_version();

        // PH2D_PAINT_PERF: close the PREVIEW phase (try_drive + drain), open the PANEL phase.
        ph_preview = elapsed_ms(m_preview);
        let m_panel = perf_t0.map(|_| std::time::Instant::now());

        // PH2D_PAINT_PERF diagnostic context (cheap reads; only used when the var is set).
        if perf_t0.is_some() {
            dbg_trivial = painter.preview_is_trivial_stack();
            dbg_gray = painter.mask_view_grayscale().is_some();
            dbg_active_is_mask = painter.active_is_mask();
            dbg_dims = painter.source_size();
            let (branch, impasto, mask_scratch) = painter.preview_drain_diag();
            dbg_branch = branch;
            dbg_impasto = impasto;
            dbg_mask_scratch = mask_scratch;
        }
        // `PH2D_PREVIEW_DIAG=1`: logs the preview-slot producer + CPU bbox, on CHANGE only
        // (BUGS_painter.md #11).
        static PREVIEW_DIAG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if *PREVIEW_DIAG.get_or_init(|| std::env::var_os("PH2D_PREVIEW_DIAG").is_some()) {
            let now = (gpu_owns_preview, painter_dirty_bbox);
            if crate::paint_perf::preview_diag_changed(now) {
                eprintln!(
                    "[preview-diag] gpu_owns={gpu_owns_preview} cpu_dirty_bbox={painter_dirty_bbox:?}"
                );
            }
        }
        // Apply / commit capture — same trait path as bgremoval.
        // Two declarations, not `allow(unused_mut)`: only the layers block reassigns it.
        #[cfg(feature = "panel-painter-layers")]
        let mut m_p = perf_t0.map(|_| std::time::Instant::now());
        #[cfg(not(feature = "panel-painter-layers"))]
        let m_p = perf_t0.map(|_| std::time::Instant::now());
        apply_selection = ph2d_tool_runtime::drive_pending_commit(
            painter as &mut dyn ph2d_editor_core::tool::RasterEditTool,
            hero.gizmo.iter_selected(),
        );
        ph_panel_sub[0] = elapsed_ms(m_p);

        // Layers snapshot publish, gated on `layers_revision()` (structural/metadata edits,
        // not strokes nor cursor moves). The `m_p` reassignment must live inside the `cfg`:
        // without the feature nobody reads it.
        #[cfg(feature = "panel-painter-layers")]
        {
            m_p = perf_t0.map(|_| std::time::Instant::now());
            use std::sync::atomic::{AtomicU64, Ordering};
            static LAST_LAYERS_REV: AtomicU64 = AtomicU64::new(u64::MAX);
            let rev = painter.layers_revision();
            if LAST_LAYERS_REV.swap(rev, Ordering::Relaxed) != rev {
                ph2d_panel_painter_layers::set_current_layers(Some(painter.layers().clone()));
            }
            ph_panel_sub[1] = elapsed_ms(m_p);
            m_p = perf_t0.map(|_| std::time::Instant::now());
            // Selection set every frame, NOT revision-gated: a re-click collapsing a
            // multi-selection changes it without a structural edit.
            ph2d_panel_painter_layers::set_current_selection(painter.selection());
            // (Mask view) Publish which mask's grayscale-view eye is open so its row draws it open.
            ph2d_panel_painter_layers::set_current_mask_grayscale_view(
                painter.mask_view_grayscale(),
            );
            // Active brush snapshot every frame (not revision-gated; a few floats).
            let brush_snapshot = painter.brush_settings();
            ph_panel_sub[2] = elapsed_ms(m_p);
            m_p = perf_t0.map(|_| std::time::Instant::now());
            let stroke_method_u8 = brush_snapshot.stroke_method;
            ph2d_panel_painter_layers::set_current_brush(Some(brush_snapshot));
            // Wet Tuning side panel: same snapshot; visibility mirrors the tool's Tuning
            // checkbox; z bump on open.
            ph2d_panel_wet_tuning::set_current_brush(Some(brush_snapshot));
            let tuning_open =
                painter_is_active && brush_snapshot.wetpaint && brush_snapshot.wet_tuning_open;
            hero.panel_visibility.insert("wet_tuning", tuning_open);
            {
                use std::sync::atomic::{AtomicBool, Ordering};
                static TUNING_WAS_OPEN: AtomicBool = AtomicBool::new(false);
                if !TUNING_WAS_OPEN.swap(tuning_open, Ordering::Relaxed) && tuning_open {
                    hero.store
                        .bump_panel_z(ph2d_editor_core::ids::WET_TUNING_PANEL);
                }
            }
            // The rail radio FOLLOWS the painter's mode (the Impasto tool list changes it too);
            // the medium decides which tools the rail offers.
            hero.store.set_painter_rail_wet(brush_snapshot.wetpaint);
            ph2d_editor_core::screens::hero::chrome::sync_painter_rail_to_mode(
                &mut hero.store,
                painter.active_paint_mode_id(),
            );
            // Eyedropper is momentary: on pick completion (armed → not) the rail returns to Brush.
            {
                use std::sync::atomic::{AtomicBool, Ordering};
                static PREV_EYEDROPPER_ARMED: AtomicBool = AtomicBool::new(false);
                let armed = painter.eyedropper_armed();
                if PREV_EYEDROPPER_ARMED.swap(armed, Ordering::Relaxed) && !armed {
                    ph2d_editor_core::screens::hero::chrome::reset_painter_rail_to_brush(
                        &mut hero.store,
                    );
                }
            }
            // The Shapes rail follows the stroke method (edge-detected).
            {
                use std::sync::atomic::{AtomicU8, Ordering};
                static PREV_STROKE_METHOD: AtomicU8 = AtomicU8::new(u8::MAX);
                if PREV_STROKE_METHOD.swap(stroke_method_u8, Ordering::Relaxed) != stroke_method_u8
                {
                    ph2d_editor_core::screens::hero::chrome::sync_painter_rail_to_stroke_method(
                        &mut hero.store,
                        stroke_method_u8,
                    );
                }
            }
            // (Brush UI) Publish the dock view-mode so the panel renders either
            // the Layers/Effects body or the Brush-properties body (header toggle).
            ph2d_panel_painter_layers::set_current_dock_shows_layers(painter.dock_shows_layers());
            // (Previews) Grain · Shape · Paper · Flow, cada um só quando a versão muda.
            crate::painter_bridge_previews::publica_previews(painter);
            // Re-capture the multi-layer Shape when its source sprite changed (before the preview).
            ph_panel_sub[3] = elapsed_ms(m_p);
            m_p = perf_t0.map(|_| std::time::Instant::now());
            painter.refresh_shape_source_if_changed();
            // Coloured Shape preview: the tool re-bakes only when the Shape appearance changes.
            if painter.refresh_shape_color_preview() {
                ph2d_panel_painter_layers::set_current_brush_shape_color_preview(
                    painter.shape_color_preview(),
                );
            }
            ph_panel_sub[4] = elapsed_ms(m_p);
        }

        // PH2D_PAINT_PERF: close the PANEL phase (snapshot publish + shape re-bake), open OVERLAY.
        ph_panel = elapsed_ms(m_panel);
        let m_overlay = perf_t0.map(|_| std::time::Instant::now());

        // Shape-editor grab tolerance follows the live camera every frame.
        crate::painter_bridge_overlays::refresh_shape_grab_tol(
            painter,
            hero,
            sim,
            camera,
            window_size,
        );
        ph_ov_tol = elapsed_ms(m_overlay);
        let m_ov_sel = perf_t0.map(|_| std::time::Instant::now());
        // Repeat Image FIRST: the 3×3 tile preview is canvas CONTENT, so it sits UNDER all chrome.
        crate::painter_bridge_overlays::draw_repeat_image(
            painter,
            hero,
            sim,
            camera,
            window_size,
            vector_scene,
            painter_preview.as_ref(),
        );
        // Selection overlay next (under the editor handles + brush ring): marching ants + hatching + the
        // crosshair cursor.
        crate::painter_bridge_selection_overlay::draw_selection_overlay(
            painter,
            hero,
            sim,
            camera,
            window_size,
            vector_scene,
        );
        ph_ov_selection = elapsed_ms(m_ov_sel);
        let m_ov_chrome = perf_t0.map(|_| std::time::Instant::now());
        crate::painter_bridge_overlays::draw_overlays(
            painter,
            hero,
            sim,
            present,
            camera,
            window_size,
            vector_scene,
            text_system,
            cursor,
            pointer_inside,
            &mut ph_chrome_sub,
            perf_t0.is_some(),
            fill_drag_armed,
        );
        // PH2D_PAINT_PERF: close the OVERLAY phase (and its last sub-call).
        ph_ov_chrome = elapsed_ms(m_ov_chrome);
        ph_overlay = elapsed_ms(m_overlay);
    }

    // ── O caminho INACTIVO e o APPLY ──────────────────────────────────────
    if crate::painter_bridge_phases::settle_inactive_and_apply(
        hero,
        renderer,
        painter_is_active,
        &apply_selection,
        painter_preview,
        last_painter_pushed_entity,
        painter_preview_gpu,
    ) {
        gpu_owns_preview = false;
    }
    // ── GPU lifecycle for the live-preview texture (W3 sprite-suppression) ──
    // Upload the premultiplied composite into a transient slot; next frame's `PreviewOverride`
    // samples it in place of the sprite, byte-identical to Apply. On a GPU-owned frame the CPU
    // cache is hidden from this block.
    let m_upload = perf_t0.map(|_| std::time::Instant::now());
    crate::painter_bridge_upload::upload_cpu_preview(
        renderer,
        painter_preview.as_ref().filter(|_| !gpu_owns_preview),
        painter_dirty_bbox,
        &painter_dirty_caixas,
        cache_version,
        gpu_owns_preview,
        painter_preview_gpu,
        toasts,
    );

    if let Some(t0) = perf_t0
        && painter_is_active
    {
        // Record this frame's dispatch; the frame timer pairs it and prints one summary per window.
        crate::paint_perf::record_dispatch(crate::paint_perf::FrameInfo {
            gpu: gpu_owns_preview,
            dispatch_ms: t0.elapsed().as_secs_f64() as f32 * 1e3,
            preview_ms: ph_preview,
            panel_ms: ph_panel,
            overlay_ms: ph_overlay,
            ov_tol_ms: ph_ov_tol,
            ov_selection_ms: ph_ov_selection,
            ov_chrome_ms: ph_ov_chrome,
            panel_sub: ph_panel_sub,
            chrome_sub: ph_chrome_sub,
            upload_ms: elapsed_ms(m_upload),
            // Filled by `record_dispatch` (the fold is noted inside `try_drive`).
            fold_ms: 0.0,
            fold_full: false,
            w: dbg_dims.0,
            h: dbg_dims.1,
            gray: dbg_gray,
            active_is_mask: dbg_active_is_mask,
            lane_partial: painter_dirty_bbox.is_some(),
            trivial: dbg_trivial,
            branch: dbg_branch,
            impasto: dbg_impasto,
            mask_scratch: dbg_mask_scratch,
        });
    }
    !apply_selection.is_empty()
}
