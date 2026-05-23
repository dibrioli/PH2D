//! Image-edit + import drain phase.
//!
//! Wave 3.2 stage A — extracted from `render_loop::mod.rs` as a free
//! function. Drains the 4 image-edit intents (trim / make_square /
//! bgremoval / undo) and the file-picker import, returning whether
//! any of them mutated visible state (caller ORs into `title_dirty`).
//!
//! Behavior-preserving lift. Each drain helper still lives in
//! `crate::hero_intents::*`; this orchestrator just decides whether
//! to call them based on the consolidated-drain locals from mod.rs.
//!
//! Bgremoval keeps its own filter-and-replace on the bus here
//! (deliberately NOT consolidated up top) so its `bgremoval_active`
//! gate runs AFTER any same-frame `ActivateTool { tool_id: "bgremoval" }`
//! fires — the
//! 1-frame-no-defer contract from the pre-Wave-2.5 `pending_bgremoval`
//! field.

use crate::{ImageEditSnapshot, hero_intents, image_import::import_image_at_camera};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::{Toast, ToastQueue, ToolRegistry};
use ph2d_render::{Camera2d, SpriteRenderer};
use std::collections::BTreeMap;

/// Dispatches the 4 image-edit drains + file picker import. Returns
/// `true` iff any drain pushed a toast (caller sets `title_dirty`).
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    trim_entities: Vec<u64>,
    make_square_entities: Vec<u64>,
    real_size_entities: Vec<u64>,
    rasterize_entities: Vec<u64>,
    padding_apply: Option<(u64, ph2d_tool_padding::PaddingSpec, bool)>,
    color_equalization_apply: Option<Vec<u64>>,
    equalize_sizes_apply: Option<Vec<u64>>,
    upscale_apply: Option<Vec<u64>>,
    undo_image_edit: bool,
    hero: &mut HeroScreen,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    image_edit_undo: &mut Option<ImageEditSnapshot>,
    tools: &mut ToolRegistry,
    camera: &Camera2d,
    next_import_cell: &mut u32,
    last_bgremoval_pushed_entity: &mut Option<u64>,
) -> bool {
    let mut title_dirty = false;

    // ImageToolsV1: drain Trim Transparency request — read the
    // sprite's atlas-source RGBA pixels, run the trim algorithm,
    // and (if any transparent border was found) re-source the
    // sprite to a fresh `IndividualTextureStore` entry at the
    // trimmed dimensions. Atlas-shared sprites cannot be edited
    // in-place (would corrupt every sibling sharing the same
    // key); we materialise the trim result as a NEW individual
    // texture and repoint only this entity. Individual-source
    // sprites would need a GPU readback to fetch their current
    // pixels — unsupported in V1; surface a toast and bail.
    //
    // World-position preservation: after the crop, the entity's
    // `Transform.translation` is shifted by
    // `ph2d_editor::image_edit::recenter_after_crop` so the *visual*
    // center of the surviving opaque content stays put even when it
    // lived off-center inside the original frame. The shift
    // happens in pure-CPU pixel math (Y-flip handled inside
    // `recenter_after_crop`); HR-5-deterministic.
    for entity_bits in trim_entities {
        if hero_intents::drain_trim_transparency(
            entity_bits,
            hero.project.pixels_per_meter,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            toasts,
            image_edit_undo,
        ) {
            title_dirty = true;
        }
    }
    // Make Square drain — parallel to Trim Transparency. Pads
    // the source image with transparent pixels on the shorter
    // axis so the result is square (width == height), then
    // repoints the sprite to a fresh IndividualTextureStore
    // entry and updates Sprite::size at the current px/m.
    //
    // Audit fixes applied: M1 (cap output dim against
    // device.max_texture_dimension_2d BEFORE acquire — was
    // deferred device-loss); M2 (sub-pixel recenter via
    // recenter_after_pad for odd-diff parity with Trim — was
    // accumulating 0.5 px drift across Trim↔Square cycles);
    // C1 (release OLD individual texture id after a successful
    // re-acquire — was leaking GPU memory on repeated edits).
    for entity_bits in make_square_entities {
        if hero_intents::drain_make_square(
            entity_bits,
            hero.project.pixels_per_meter,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            toasts,
            image_edit_undo,
        ) {
            title_dirty = true;
        }
    }
    // Rasterize drain — bake Transform.scale + .rotation into the
    // source pixel buffer (Mitchell-Netravali resample + rotation) and
    // reset Transform to identity. Per-sprite broadcast in the
    // `OneShotImageOp` arm above; mirror of Trim / Make Square.
    for entity_bits in rasterize_entities {
        if hero_intents::drain_rasterize(
            entity_bits,
            hero.project.pixels_per_meter,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            toasts,
            image_edit_undo,
        ) {
            title_dirty = true;
        }
    }
    // Real Size drain — reset the selected sprite's Transform scale to
    // 1:1 (preserving flip sign). Unlike Trim / Make Square this is a
    // pure ECS Transform write — no pixel readback, no texture rebind,
    // no pivot reproject (scale changes around the existing pivot). The
    // ±1 reset itself is the single-source-of-truth pure helper in
    // `ph2d-tool-real-size`.
    for entity_bits in real_size_entities {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
            let new = ph2d_tool_real_size::real_size_scale([t.scale.x, t.scale.y]);
            if t.scale.x != new[0] || t.scale.y != new[1] {
                t.scale.x = new[0];
                t.scale.y = new[1];
                // Project mutated → dirty the title (no toast: the visible
                // scale snap is the feedback).
                title_dirty = true;
            }
        }
    }
    // Padding Apply drain — resize the selected sprite's source by the
    // tool's signed per-edge spec (`add_padding`), swap to a fresh
    // Individual texture, and reproject the pivot so the original
    // content's world position holds. Parallel to Make Square; the spec
    // was captured from the active `PaddingTool` by `padding_bridge`.
    if let Some((entity_bits, spec, recenter_pivot)) = padding_apply
        && hero_intents::drain_padding(
            entity_bits,
            spec,
            recenter_pivot,
            hero.project.pixels_per_meter,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            toasts,
            image_edit_undo,
        )
    {
        title_dirty = true;
    }
    // Color Equalization drain — multi-sprite Apply. The bridge
    // returns the entire `iter_selected()` snapshot when the user
    // pressed Apply, so we iterate and bake each sprite via the
    // currently-active ColorEqualizationTool (its source snapshot is
    // re-pushed per-entity here to avoid stale RGBA between bakes).
    if let Some(bits_list) = color_equalization_apply {
        let ceq_id = ph2d_editor::ToolId::new("color_equalization");
        let ceq_active = tools.active().map(|t| t.id() == ceq_id).unwrap_or(false);
        if ceq_active {
            for entity_bits in bits_list {
                let mut ran = false;
                if let Some(tool) = tools.active_mut()
                    && let Some(ceq) =
                        tool.as_any_mut()
                            .downcast_mut::<ph2d_tool_color_equalization::ColorEqualizationTool>()
                {
                    ran = hero_intents::drain_color_equalization(
                        entity_bits,
                        sim,
                        renderer,
                        asset_db,
                        atlas_asset_map,
                        toasts,
                        image_edit_undo,
                        ceq,
                    );
                }
                if ran {
                    title_dirty = true;
                }
            }
        }
    }
    // Equalize Sizes drain — multi-sprite Apply. Cross-sprite: Max
    // mode needs the global max over the selection, so the bake runs
    // ONCE over the whole bits_list (not once per sprite like CEQ).
    // The bridge returns the full `iter_selected()` snapshot on
    // Apply; we downcast the active tool and forward to
    // `drain_equalize_sizes`.
    if let Some(bits_list) = equalize_sizes_apply {
        let eqs_id = ph2d_editor::ToolId::new("equalize_sizes");
        let eqs_active = tools.active().map(|t| t.id() == eqs_id).unwrap_or(false);
        if eqs_active
            && let Some(tool) = tools.active_mut()
            && let Some(eqs) = tool
                .as_any_mut()
                .downcast_mut::<ph2d_tool_equalize_sizes::EqualizeSizesTool>()
        {
            // Capture Square-grid origin before borrowing `tools` for
            // the drain — the snap math (align-to-grid) anchors to it.
            let grid_origin = hero.grid.snap_state.square_cfg.origin;
            if hero_intents::drain_equalize_sizes(
                &bits_list,
                hero.project.pixels_per_meter,
                grid_origin,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                toasts,
                image_edit_undo,
                eqs,
            ) {
                title_dirty = true;
            }
        }
    }
    // Upscale Apply drain — sabor 3 (mirror of Color Equalization),
    // per-sprite. The bridge returns the full multi-selection on
    // Apply; we downcast the active tool and bake each sprite via
    // `drain_upscale` (which re-pushes the source per-entity to avoid
    // stale RGBA between bakes).
    if let Some(bits_list) = upscale_apply {
        let ups_id = ph2d_editor::ToolId::new("upscale");
        let ups_active = tools.active().map(|t| t.id() == ups_id).unwrap_or(false);
        if ups_active {
            for entity_bits in bits_list {
                let mut ran = false;
                if let Some(tool) = tools.active_mut()
                    && let Some(ups) = tool
                        .as_any_mut()
                        .downcast_mut::<ph2d_tool_upscale::UpscaleTool>()
                {
                    ran = hero_intents::drain_upscale(
                        entity_bits,
                        hero.project.pixels_per_meter,
                        sim,
                        renderer,
                        asset_db,
                        atlas_asset_map,
                        toasts,
                        image_edit_undo,
                        ups,
                    );
                }
                if ran {
                    title_dirty = true;
                }
            }
        }
    }
    // Bg Removal drain — parallel to Trim Transparency, but
    // dimensions are preserved (the algorithm only mutates
    // alpha + optionally despills RGB, never crops). No pivot
    // reproject for that reason. Reads source RGBA via the
    // same Atlas / Individual branch the Trim drain uses,
    // calls `BgRemovalTool::run_full_resolution` at the
    // sprite's native size, and swaps to a fresh Individual
    // texture so the on-canvas sprite picks up the new alpha.
    //
    // Gate on BgRemoval being the active tool — the user is
    // expected to apply WHILE the tool is active (the Apply
    // Toggle lives in its panel). If they switch tools in
    // the same frame, leave the `Bgremoval` variant on the
    // bus so the next activation can complete the round-trip
    // (the filter predicate below only matches when
    // `bgremoval_active`, so the variant is pushed back as
    // a leftover otherwise).
    //
    // The consolidated drain at the top intentionally pushed
    // every `Bgremoval` variant BACK onto the bus
    // (`bgremoval_leftover`). We pick it up here, where the
    // `bgremoval_active` gate runs AFTER any same-frame
    // `ActivateTool { tool_id: "bgremoval" }` has already fired —
    // preserving the
    // 1-frame-no-defer contract from the pre-Wave-2.5
    // `pending_bgremoval` field.
    let bgremoval_id = ph2d_editor::ToolId::new("bgremoval");
    let bgremoval_active = tools
        .active()
        .map(|t| t.id() == bgremoval_id)
        .unwrap_or(false);
    let bgremoval_entity = {
        let mut found: Option<u64> = None;
        let leftovers: Vec<ph2d_editor::action_bus::EditorAction> = hero
            .bus
            .drain()
            .filter_map(|a| match a {
                ph2d_editor::action_bus::EditorAction::OneShotImageOp {
                    tool_id: "bgremoval",
                    entity_bits,
                } if bgremoval_active && found.is_none() => {
                    found = Some(entity_bits);
                    None
                }
                other => Some(other),
            })
            .collect();
        for a in leftovers {
            hero.bus.push(a);
        }
        found
    };
    if let Some(entity_bits) = bgremoval_entity {
        let bg = tools
            .active_mut()
            .and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_bgremoval::BgRemovalTool>()
            })
            .expect("bgremoval_active gate guarantees a BgRemovalTool");
        if hero_intents::drain_bgremoval(
            entity_bits,
            hero.project.pixels_per_meter,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            toasts,
            image_edit_undo,
            bg,
            last_bgremoval_pushed_entity,
        ) {
            title_dirty = true;
        }
    }
    // Image-edit undo drain. Cmd+Z (or TOOL_UNDO click)
    // pushes `EditorAction::UndoImageEdit` onto the bus; the
    // shell owns the snapshot. Single-level: each new edit
    // overwrites the slot (releasing the previous pre-source),
    // so undo restores at most the MOST RECENT Trim / Make
    // Square / Bg Removal.
    if undo_image_edit
        && hero_intents::drain_undo_image_edit(image_edit_undo, sim, renderer, toasts)
    {
        title_dirty = true;
    }
    // Publish whether a snapshot is currently stored so the UI
    // can dim the TOOL_UNDO chip when there's nothing to undo.
    hero.image_edit.has_undoable = image_edit_undo.is_some();
    // M14.4c: drain pending import request → open native
    // file picker, import every selected image (PNG/WEBP/
    // JPEG), spawn a sprite per image at the camera center.
    if hero.import_requested {
        hero.import_requested = false;
        let picked = rfd::FileDialog::new()
            .add_filter("Image (PNG / WEBP / JPEG)", &["png", "webp", "jpg", "jpeg"])
            .pick_files();
        let pixels_per_meter = hero.project.pixels_per_meter;
        if let Some(paths) = picked {
            for path in paths {
                match import_image_at_camera(
                    sim,
                    &mut *renderer,
                    asset_db,
                    camera,
                    *next_import_cell,
                    &path,
                    pixels_per_meter,
                    atlas_asset_map,
                ) {
                    Ok(spawned_label) => {
                        // Monotonic increment — the Skyline atlas
                        // grows up to 4096²; no slot reuse cycle
                        // (the old `% 8 + 8` math was for the
                        // M5 grid placeholder).
                        *next_import_cell = next_import_cell.saturating_add(1);
                        toasts.push(Toast::success(format!("Imported {spawned_label}")));
                        title_dirty = true;
                    }
                    Err(e) => {
                        eprintln!("M14.4c import failed: {e}");
                        toasts.push(Toast::error(format!("Import failed: {e}")));
                        title_dirty = true;
                    }
                }
            }
        }
    }
    title_dirty
}
