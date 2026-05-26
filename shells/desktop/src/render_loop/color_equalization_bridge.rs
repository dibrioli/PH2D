//! Color Equalization panel ⟷ tool bridge + on-canvas live preview.
//!
//! Run once per frame BEFORE `paint_hero_screen`. Does, in order:
//!
//! 1. Drives the panel's visibility (shown iff `color_equalization`
//!    is the active tool, keyed `"color_equalization"`).
//! 2. Publishes the per-frame `ColorEqualizationUiSnapshot` the panel
//!    paints next frame.
//! 3. Drains the panel-staged dropdown close requests.
//! 4. Multi-sprite preview rebuild: when params have changed AND the
//!    user is NOT currently dragging a CEQ slider/chip, regenerate the
//!    full-resolution-pipeline preview RGBA for EVERY selected sprite
//!    and cache the bitmaps in `app_state.color_equalization_previews`.
//!    Skipped while dragging so the user gets latency-free knob travel
//!    and the heavy pipeline (Quantize / Bilateral) only runs once
//!    when the slider is released.
//! 5. Returns the current multi-selection iff Apply fired this frame
//!    — the caller runs the per-sprite bake via
//!    `drain_color_equalization`.
//!
//! ### Wave 10 / Etapa 3 status (audit fix [A3])
//!
//! Fully migrated to the generic runtime channel:
//! - `drive_pending_commit` for multi-sprite Apply (Etapa 2).
//! - `drive_multi_preview_cache` for the per-sprite preview rebuild loop
//!   (Etapa 3 — replaced the hand-written downcast loop). The
//!   `BTreeMap<u64, ColorEqualizationPreview>` is now a
//!   `BTreeMap<u64, PreviewCache>` (type alias) driven by the runtime
//!   helper; `ColorEqualizationPreview` is a re-export of
//!   `ph2d_tool_runtime::PreviewCache`.
//!
//! Residual downcast (1): panel snapshot publish via
//! `set_current_snapshot(Some(ceq.ui_snapshot()))` + dropdown-close drain.
//! Both are CEQ-specific affordances per ADR-0040 §3 (documented
//! exception). Allowlisted in
//! `tests/architecture_no_downcast_to_concrete_tool_in_shell.rs`.

use crate::app_state::ColorEqualizationPreview;
use ph2d_asset::AssetDb;
use ph2d_asset::AssetId;
use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite, SpriteRenderer};
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

/// Returns `Some(entity_bits_list)` iff Apply fired this frame.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    last_pushed_entity: &mut Option<u64>,
    color_equalization_previews: &mut BTreeMap<u64, ColorEqualizationPreview>,
) -> Option<Vec<u64>> {
    let active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("color_equalization"))
        .unwrap_or(false);
    hero.panel_visibility.insert("color_equalization", active);
    // Image tools dock into the Inspector slot — hide the Inspector
    // while the tool is active so the slot isn't double-claimed (the
    // typed registry would paint both and Inspector wins the
    // z-order). On deactivate, restore the Inspector to visible so
    // the user goes back to the inspector-by-default canvas state.
    //
    // Edge-triggered (Wave 10 Etapa 4 smoke fix — see
    // bgremoval_preview.rs for rationale).
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(active, Ordering::Relaxed);
        if was != active {
            hero.panel_visibility.insert("inspector", !active);
        }
    }

    if !active {
        *last_pushed_entity = None;
        color_equalization_previews.clear();
        #[cfg(feature = "panel-color-equalization")]
        {
            ph2d_panel_color_equalization::set_current_snapshot(None);
            ph2d_panel_color_equalization::set_current_histogram(None);
        }
        return None;
    }

    // ── Detect "user is actively dragging a CEQ slider/chip" ────────
    // Pipeline reruns are the expensive part (Quantize alone can hit
    // 50-150 ms / preview at 512²). Skip rebuilds while a CEQ slider
    // or chip is the active widget — the existing preview overlays
    // stay on screen so the knob travel feels instantaneous. On
    // release (`active_id` flips back to `None`) the dirty flag
    // triggers a full rebuild for every selected sprite at full
    // quality.
    let dragging_ceq = matches!(hero.store.active_id(), Some(id) if is_ceq_drag_widget(id));

    // Snapshot the live multi-selection before we borrow `tools.active_mut()`.
    let primary = hero.gizmo.selection;
    let selected: Vec<u64> = hero.gizmo.iter_selected().collect();

    // ── Drain panel-staged state from the tool ──────────────────────
    // Three things drained in one borrow:
    //   - `pending_apply`  → returned to the caller as `apply`
    //   - `pending_close_lut_dropdown` → applied to the store BELOW
    //     (after the `ceq` borrow ends, since `hero.store` aliases)
    //   - `params_dirty`   → consumed only when NOT dragging; while
    //     dragging the flag stays set so the release-frame rebuild
    //     fires
    let mut apply: Option<Vec<u64>> = None;
    let mut close_dropdown: Option<u8> = None;
    let mut needs_preview_rebuild = false;
    let mut needs_panel_reset = false;
    if let Some(tool) = tools.active_mut()
        && let Some(ceq) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_color_equalization::ColorEqualizationTool>()
    {
        // (Generic) Multi-sprite Apply capture via runtime helper —
        // ADR-0041 contract. CEQ tool now implements RasterEditTool, so
        // `take_pending_commit` (= take_pending_apply) goes through the
        // generic channel.
        let bits = ph2d_tool_runtime::drive_pending_commit(ceq, selected.iter().copied());
        if !bits.is_empty() {
            apply = Some(bits);
        }
        close_dropdown = ceq.take_pending_close_lut_dropdown();
        needs_panel_reset = ceq.take_pending_panel_reset();
        if !dragging_ceq {
            needs_preview_rebuild = ceq.take_params_dirty();
        }
        #[cfg(feature = "panel-color-equalization")]
        ph2d_panel_color_equalization::set_current_snapshot(Some(ceq.ui_snapshot()));
        // Publish histogram do preview pra "visor" do painel (faltava,
        // por isso aparecia preto — Enio 2026-05-26 "O visor não
        // funciona, está com fundo preto").
        #[cfg(feature = "panel-color-equalization")]
        ph2d_panel_color_equalization::set_current_histogram(Some(ceq.preview_histogram()));
    }

    // Reset just fired (Reset button or fresh activation) — re-populate
    // the panel's `WidgetStore` entries so every slider knob + chip
    // text snaps back to the default-constructed track value. Without
    // this, `params` resets but the slider visuals stay where the user
    // dragged them (the panel paints `store.slider(id)` value, not the
    // snapshot). We call `Panel::populate` via the registry lookup —
    // it's the same function `register_all_panels` runs at boot, so
    // it knows every CEQ widget's default state in one place.
    if needs_panel_reset {
        ph2d_editor::panel::with_registry_opt(|reg| {
            if let Some(idx) =
                reg.find_by_panel_node_id(ph2d_tool_color_equalization::ids::CEQ_PANEL)
            {
                reg.panels_mut()[idx].populate(&mut hero.store);
            }
        });
    }

    // Selection-change forces a rebuild too: if the cached set differs
    // from the live selection (added/removed sprites), regenerate.
    let cache_keys: std::collections::BTreeSet<u64> =
        color_equalization_previews.keys().copied().collect();
    let live_keys: std::collections::BTreeSet<u64> = selected.iter().copied().collect();
    let selection_changed = cache_keys != live_keys;
    if selection_changed {
        // Drop entries for sprites that left the selection so we don't
        // keep painting stale overlays after the user shift-clicks
        // them out.
        color_equalization_previews.retain(|k, _| live_keys.contains(k));
        // Don't rebuild MID-DRAG just because selection shifted — but
        // for a true selection change while idle, force the rebuild so
        // newly-selected sprites get a preview immediately.
        if !dragging_ceq {
            needs_preview_rebuild = true;
        }
    }

    // ── Multi-sprite preview rebuild ───────────────────────────────
    // Wave 10 / Etapa 3 (audit etapa-2.md M1 follow-up): the hand-written
    // loop + per-iteration downcast was retired in favor of
    // `drive_multi_preview_cache`, which:
    //   - drops cache entries no longer in selection
    //   - cycles set_source → current_preview per entity
    //   - returns count of frames produced this call (ignored here)
    // The closure carries the shell-specific pixel readback. The tool's
    // single-source nature is honored by the helper (it cycles per
    // entity). Tool state ends up holding whichever entity came last —
    // `last_pushed_entity` tracks that so an immediate Apply doesn't
    // double-push for the primary.
    if needs_preview_rebuild
        && !selected.is_empty()
        && let Some(tool) = tools.active_mut()
        && let Some(raster) = tool.as_raster_edit_mut()
    {
        ph2d_tool_runtime::drive_multi_preview_cache(
            raster,
            selected.iter().copied(),
            color_equalization_previews,
            |entity| {
                let src = crate::hero_intents::texture_edit::read_sprite_source(
                    entity,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                )?;
                let straight = src.image.into_straight();
                Some(ph2d_tool_runtime::RasterSource {
                    pixels: straight.pixels,
                    width: straight.width,
                    height: straight.height,
                })
            },
        );
        *last_pushed_entity = selected.last().copied();
    }

    // ── Dropdown close (post-borrow store mutation) ────────────────
    if let Some(slot) = close_dropdown {
        let chip_id = match slot {
            1 => ph2d_tool_color_equalization::ids::CEQ_LUT_1_DROPDOWN,
            2 => ph2d_tool_color_equalization::ids::CEQ_LUT_2_DROPDOWN,
            3 => ph2d_tool_color_equalization::ids::CEQ_POSTERIZE_DROPDOWN,
            _ => ph2d_tool_color_equalization::ids::CEQ_QUANTIZE_DROPDOWN,
        };
        if let Some(ph2d_editor::InteractiveState::Dropdown { open, .. }) =
            hero.store.get_mut(chip_id)
        {
            *open = false;
        }
    }

    // ── Apply clears the cache + ends the live overlay session ─────
    // (the per-sprite bake replaces every selected sprite's texture;
    // continuing to paint the stale RGBA over them would read as a
    // ghost until tool deactivation.)
    if apply.is_some() {
        color_equalization_previews.clear();
    }

    // ── On-canvas overlay (per cached preview) ──────────────────────
    // Each entry is painted over its OWN sprite's footprint — multi-
    // select shows the edited look on EVERY selected sprite. The
    // sprite stays underneath the overlay (Color EQ preserves alpha +
    // dims, so the overlay covers it pixel-perfect with the new RGB
    // values).
    let quality = ph2d_editor::image_quality_for(hero.project.image_filter);
    for preview in color_equalization_previews.values() {
        let entity = ph2d_ecs::Entity::from_bits(preview.entity_bits);
        let (Some(tr), Some(sprite)) = (
            sim.world().get::<ph2d_ecs::Transform>(entity),
            sim.world().get::<Sprite>(entity),
        ) else {
            continue;
        };
        let cx = tr.translation.x + sprite.anchor[0];
        let cy = tr.translation.y + sprite.anchor[1];
        let (sw, sh) = (sprite.size[0], sprite.size[1]);
        let (x0, y0) = camera.world_to_screen([cx - sw * 0.5, cy + sh * 0.5], window_size);
        let (x1, y1) = camera.world_to_screen([cx + sw * 0.5, cy - sh * 0.5], window_size);
        vector_scene.draw_image_rgba(
            &preview.rgba,
            preview.width,
            preview.height,
            (x0 as f64, y0 as f64, x1 as f64, y1 as f64),
            quality,
        );
    }
    let _ = primary; // primary is read above for the snapshot path only.

    apply
}

/// `true` iff `id` is a CEQ slider/chip whose drag should pause
/// preview rebuilds (Quantize / Bilateral / Posterize are heavy
/// enough that running them per frame stalls the slider visibly).
/// Buttons (Apply, Auto-*) aren't included — their interaction is a
/// single tap, no multi-frame drag.
fn is_ceq_drag_widget(id: ph2d_editor::NodeId) -> bool {
    use ph2d_tool_color_equalization::ids as cids;
    let sliders_and_chips: [ph2d_editor::NodeId; 28] = [
        cids::CEQ_CLIP_LIMIT,
        cids::CEQ_CLIP_LIMIT_NUM,
        cids::CEQ_TILE_GRID,
        cids::CEQ_TILE_GRID_NUM,
        cids::CEQ_EXPOSURE,
        cids::CEQ_EXPOSURE_NUM,
        cids::CEQ_TEMPERATURE,
        cids::CEQ_TEMPERATURE_NUM,
        cids::CEQ_TINT,
        cids::CEQ_TINT_NUM,
        cids::CEQ_BRIGHTNESS,
        cids::CEQ_BRIGHTNESS_NUM,
        cids::CEQ_CONTRAST,
        cids::CEQ_CONTRAST_NUM,
        cids::CEQ_VIBRANCE,
        cids::CEQ_VIBRANCE_NUM,
        cids::CEQ_SATURATION,
        cids::CEQ_SATURATION_NUM,
        cids::CEQ_SHARPEN_AMOUNT,
        cids::CEQ_SHARPEN_AMOUNT_NUM,
        cids::CEQ_SHARPEN_RADIUS,
        cids::CEQ_SHARPEN_RADIUS_NUM,
        cids::CEQ_DENOISE_STRENGTH,
        cids::CEQ_DENOISE_STRENGTH_NUM,
        cids::CEQ_LUT_INTENSITY,
        cids::CEQ_LUT_INTENSITY_NUM,
        cids::CEQ_LUT_MIX,
        cids::CEQ_LUT_MIX_NUM,
    ];
    sliders_and_chips.contains(&id)
}
