//! Color Equalization panel ⟷ tool bridge — **live ECS bake** mode.
//!
//! Wave 11 follow-up (Enio 2026-05-26): the old `vector_scene` overlay
//! was replaced with direct ECS-sprite bakes per slider tick. The user
//! sees the modified imagem real in real time; no overlay layer.
//!
//! Run once per frame BEFORE `paint_hero_screen`. Does, in order:
//!
//! 1. Drives the panel's visibility (shown iff `color_equalization`
//!    is the active tool, keyed `"color_equalization"`).
//! 2. Publishes the per-frame `ColorEqualizationUiSnapshot` / histogram
//!    the panel paints next frame.
//! 3. Drains the panel-staged dropdown close + reset requests.
//! 4. Maintains a **shell-local cache of original sprite pixels** keyed
//!    by entity (`SOURCE_CACHE` thread_local). The cache is populated on
//!    first sight of each newly-selected entity (read once from
//!    `texture_edit::read_sprite_source`), so subsequent bakes re-feed
//!    the *original* — they never compound over previously-baked output.
//! 5. On `params_dirty` (and NOT mid-drag), bakes every selected entity
//!    from its cached original and writes the result back into the ECS
//!    sprite via `texture_edit::commit_edited_texture`. No overlay paint.
//! 6. On Apply, reverts every cached entity to its original, clears the
//!    cache, and returns the selection to the caller — caller's
//!    `drain_color_equalization` then re-bakes from the clean source and
//!    creates the official `ImageEditSnapshot` (undo entry intact).
//! 7. On Cancel / tool deactivate / selection shrink, reverts the
//!    relevant cached entities so the sprite snaps back to its original
//!    look (mirrors the legacy "preview disappears when you leave the
//!    tool" UX).
//!
//! ### Trade-offs
//!
//! Cost per slider tick is roughly Apply-grade (full-res pipeline +
//! texture upload). The `dragging_ceq` gate still throttles this to a
//! single bake on slider release — same heuristic that protected the
//! previous overlay rebuild.
//!
//! Undo: each Apply creates one entry (revert + caller's drain → undo).
//! Live-bake ticks that never reach Apply do *not* create entries; the
//! `SOURCE_CACHE` is the shadow source of truth and Cancel / deactivate
//! restores from it.

use crate::app_state::ColorEqualizationPreview;
use crate::hero_intents::texture_edit;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::{HeroScreen, Toast, ToastQueue, ToolRegistry};
use ph2d_host::WindowSize;
use ph2d_i18n::tr_with;
use ph2d_render::{AlphaMode, Camera2d, SpriteImage, SpriteRenderer};
use ph2d_vector::VectorScene;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// Captured original of a selected sprite at the moment it entered a
/// CEQ live-bake session. Pixels are stored in the sprite's **native
/// AlphaMode** (Atlas → Straight; Individual → premul flag derived) so a
/// revert is byte-exact. The CE pipeline always works on straight input,
/// so a `into_straight()` runs on demand per live-bake.
struct CachedOriginal {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    size_world: [f32; 2],
    alpha: AlphaMode,
    /// A folha lida (a visível, dono 08/11): a prévia e o Cancel gravam na mesma.
    sheet: u8,
    /// O recurso que a folha extra mostra por obra da prévia (no início, o original): só com ele lá se grava de volta
    /// (`shown_edit::preview_still_owns`, auditoria 08/11 F).
    asset: Option<AssetId>,
}

thread_local! {
    static SOURCE_CACHE: RefCell<BTreeMap<u64, CachedOriginal>> =
        const { RefCell::new(BTreeMap::new()) };
}

/// Tuple used by the active-tool drain path (5 fields — no AlphaMode).
type DrainTuple = (u64, Vec<u8>, u32, u32, [f32; 2], u8);

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
    toasts: &mut ToastQueue,
) -> Option<Vec<u64>> {
    // Overlay-era parameters retained for API stability — live-bake mode
    // never paints overlays, so these are intentionally unused.
    let _ = camera;
    let _ = window_size;
    let _ = vector_scene;

    let active = tools
        .active()
        .map(|t| t.id() == ph2d_editor_core::ToolId::new("color_equalization"))
        .unwrap_or(false);
    hero.panel_visibility.insert("color_equalization", active);

    // ⭐⭐⭐ **A FERRAMENTA ACOMPANHA O INSPECTOR, NÃO O SUBSTITUI** (report do Enio, 2026-09-08:
    //    *«algumas ferramentas ou painéis não criam abas»*).
    //
    // ⛔⛔ Aqui vivia um interruptor de FLANCO que escondia o Inspector enquanto a ferramenta
    //    estivesse activa — o modelo de *takeover* que antecede as abas: treze painéis publicam o
    //    MESMO rect do dock direito e não colidiam por CONVENÇÃO, porque só um estava visível de
    //    cada vez. Com as abas essa convenção deixou de ser necessária, e mantê-la custava duas
    //    coisas ao artista: a fileira não aparecia (a ferramenta não criava aba nenhuma) e o
    //    Inspector **desaparecia** sem a coluna se recolher — que é a segunda metade do report.
    //
    // ⇒ os dois passam a ser OCUPANTES do mesmo encaixe, logo a fileira nasce sozinha. Quem fica à
    //    frente também: o `slot_tabs::reconcile_z` promove o painel que ACABOU de ficar visível, e
    //    ao desactivar a ferramenta o `retain_panel_z` poda-o e o Inspector volta à frente.
    //    ⚠️ Nenhuma linha nova decide isso — a lei já existia e o *takeover* é que a contradizia.
    // ⚠️ O flanco FICA — ele tem um segundo leitor três linhas abaixo (reverter os sprites em
    //    cache ao desactivar), e só a metade do Inspector é que saiu.
    static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
    let was_active = LAST_ACTIVE.swap(active, Ordering::Relaxed);

    // Edge: deactivated this frame → revert every cached sprite. Mirrors
    // the legacy "overlay disappears on tool exit" semantic.
    if was_active && !active {
        revert_all_and_clear(sim, renderer, asset_db, toasts);
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

    // Legacy overlay cache is unused in live-bake mode; keep it empty.
    color_equalization_previews.clear();
    *last_pushed_entity = None;

    let dragging_ceq = matches!(hero.store.active_id(), Some(id) if is_ceq_drag_widget(id));
    let selected: Vec<u64> = hero.gizmo.iter_selected().collect();

    // Entities that left the selection get their pixels restored before
    // we drop them from the cache — the user shouldn't keep a stranded
    // baked sprite behind after shift-clicking out of it.
    prune_and_revert_unselected(&selected, sim, renderer, asset_db, toasts);

    // Newly-selected entities → seed cache from their *current* sprite
    // source (Atlas → asset bytes, Individual → GPU readback). Subsequent
    // bakes always read this snapshot, never the live ECS state.
    for &entity_bits in &selected {
        ensure_cached(
            entity_bits,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            toasts,
        );
    }

    // Drain panel-staged state in a tight tool borrow.
    let mut apply_bits: Vec<u64> = Vec::new();
    let mut close_dropdown: Option<u8> = None;
    let mut needs_panel_reset = false;
    let mut dirty_should_rebake = false;
    if let Some(tool) = tools.active_mut()
        && let Some(ceq) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_color_equalization::ColorEqualizationTool>()
    {
        apply_bits = ph2d_tool_runtime::drive_pending_commit(ceq, selected.iter().copied());
        close_dropdown = ceq.take_pending_close_lut_dropdown();
        needs_panel_reset = ceq.take_pending_panel_reset();
        // Apply takes priority over a live-bake tick; we only need to
        // rebake when there's no Apply firing this frame.
        if !dragging_ceq && apply_bits.is_empty() {
            dirty_should_rebake = ceq.take_params_dirty();
        }
        #[cfg(feature = "panel-color-equalization")]
        ph2d_panel_color_equalization::set_current_snapshot(Some(ceq.ui_snapshot()));
        #[cfg(feature = "panel-color-equalization")]
        ph2d_panel_color_equalization::set_current_histogram(Some(ceq.preview_histogram()));
    }

    if needs_panel_reset {
        ph2d_editor_core::panel::with_registry_opt(|reg| {
            if let Some(idx) = reg.find_by_panel_node_id(ph2d_editor_core::ids::CEQ_PANEL) {
                reg.panels_mut()[idx].populate(&mut hero.store);
            }
        });
    }

    // Apply path: restore each cached entity to its original before the
    // caller's drain runs. The drain re-reads the (now-original) sprite
    // source and writes the canonical undo entry from clean inputs.
    if !apply_bits.is_empty() {
        revert_all_and_clear(sim, renderer, asset_db, toasts);
        if let Some(slot) = close_dropdown {
            apply_dropdown_close(slot, hero);
        }
        return Some(apply_bits);
    }

    // Live-bake path: pipeline each cached source against the current
    // params; commit the result back into the ECS sprite.
    if dirty_should_rebake && !selected.is_empty() {
        let bakes = collect_live_bakes(tools, &selected);
        for (entity_bits, pixels, w, h, size_world, sheet) in bakes {
            let entity = Entity::from_bits(entity_bits);
            let image =
                SpriteImage::from_bytes(w, h, pixels, AlphaMode::Straight).into_premultiplied();
            if let Err(e) = texture_edit::commit_edited_texture(
                entity, sheet, sim, renderer, asset_db, &image, size_world, toasts,
            ) {
                toasts.push(Toast::error(tr_with(
                    "shell.color_equalization_bridge.color_equalization_gpu",
                    &[("e", &e)],
                )));
            }
            let agora = ph2d_anim_sheets::shown_edit::sheet_asset(sim.world(), entity, sheet);
            SOURCE_CACHE.with(|c| {
                if let Some(v) = c.borrow_mut().get_mut(&entity_bits) {
                    v.asset = agora;
                }
            });
        }
        // `collect_live_bakes` called `set_source` on the tool to feed
        // the pipeline, and that helper re-armed `params_dirty` (see
        // `ColorEqualizationTool::set_source_snapshot`). Without
        // draining it again here, the NEXT frame would see dirty=true
        // and re-bake (with identical params) every frame — was FPS=1
        // when a heavy stage was armed. Drain once to break the loop.
        if let Some(tool) = tools.active_mut()
            && let Some(ceq) = tool
                .as_any_mut()
                .downcast_mut::<ph2d_tool_color_equalization::ColorEqualizationTool>()
        {
            let _ = ceq.take_params_dirty();
        }
    }

    if let Some(slot) = close_dropdown {
        apply_dropdown_close(slot, hero);
    }

    None
}

/// Read the sprite's current source pixels and cache them keyed by
/// `entity_bits` — no-op if already cached. Skips on read failure
/// (missing atlas key / readback fault).
fn ensure_cached(
    entity_bits: u64,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
) {
    let entity = Entity::from_bits(entity_bits);
    let shown = ph2d_ecs::shown_sheet(sim.world(), entity);
    // A folha à vista mudou com a ferramenta aberta: a antiga volta ao original e a nova entra na cache.
    let velha = SOURCE_CACHE.with(|c| {
        let mut c = c.borrow_mut();
        match c.get(&entity_bits) {
            Some(v) if v.sheet != shown => c.remove(&entity_bits),
            _ => None,
        }
    });
    if let Some(v) = velha {
        revert_one(entity_bits, v, sim, renderer, asset_db, toasts);
    } else if SOURCE_CACHE.with(|c| c.borrow().contains_key(&entity_bits)) {
        return;
    }
    let Some(src) =
        texture_edit::read_shown_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        return;
    };
    let (size_world, sheet) = (src.old_size_world, src.sheet);
    let asset = ph2d_anim_sheets::shown_edit::sheet_asset(sim.world(), entity, sheet);
    let SpriteImage {
        pixels,
        width,
        height,
        alpha,
    } = src.image;
    SOURCE_CACHE.with(|c| {
        c.borrow_mut().insert(
            entity_bits,
            CachedOriginal {
                pixels,
                width,
                height,
                size_world,
                alpha,
                sheet,
                asset,
            },
        );
    });
}

/// Pipeline each cached selected entity in a single tool borrow via the
/// generic `RasterEditTool::run_full` path — that variant tries the
/// GPU compute chain first (when `--features gpu` is on, which it is in
/// shells/desktop) and falls back to CPU only when the GPU isn't
/// available. The writeback happens outside this function so the tool
/// borrow ends before `sim`/`renderer` are mutated.
fn collect_live_bakes(tools: &mut ToolRegistry, selected: &[u64]) -> Vec<DrainTuple> {
    use ph2d_editor_core::tool::RasterEditTool;
    let Some(tool) = tools.active_mut() else {
        return Vec::new();
    };
    let Some(raster) = tool.as_raster_edit_mut() else {
        return Vec::new();
    };
    let mut bakes = Vec::with_capacity(selected.len());
    for &entity_bits in selected {
        let cached = SOURCE_CACHE.with(|c| {
            c.borrow().get(&entity_bits).map(|cs| {
                (
                    cs.pixels.clone(),
                    cs.width,
                    cs.height,
                    cs.size_world,
                    cs.alpha,
                    cs.sheet,
                )
            })
        });
        let Some((pixels, w, h, size_world, alpha, sheet)) = cached else {
            continue;
        };
        // CE pipeline always works on straight RGBA; convert on demand
        // (no-op for already-straight, decode for premul).
        let straight = SpriteImage::from_bytes(w, h, pixels, alpha).into_straight();
        RasterEditTool::set_source(raster, straight.pixels, w, h);
        let (out, ow, oh) = RasterEditTool::run_full(raster);
        bakes.push((entity_bits, out, ow, oh, size_world, sheet));
    }
    bakes
}

/// Restore every cached sprite to its **original** pixels + AlphaMode
/// (byte-exact — Atlas-sourced sprites stay Straight, previously-baked
/// Individual+Premul sprites stay Premul) and drop all cache entries.
/// Re-encoding the cached bytes through a different alpha mode would
/// destroy the alpha=0 RGB bleed (`unpremultiply_rgba8` zeros it; a
/// fresh `into_premultiplied` zeros it too) — the source of the halo
/// Enio reported when Cancel was firing with a different alpha mode.
fn revert_all_and_clear(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    toasts: &mut ToastQueue,
) {
    let entries = SOURCE_CACHE.with(|c| std::mem::take(&mut *c.borrow_mut()));
    for (entity_bits, v) in entries {
        revert_one(entity_bits, v, sim, renderer, asset_db, toasts);
    }
}

/// Um original de volta à folha dele, só se a prévia ainda é dona dela (senão larga-se calado).
fn revert_one(
    entity_bits: u64,
    v: CachedOriginal,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    toasts: &mut ToastQueue,
) {
    let entity = Entity::from_bits(entity_bits);
    if !ph2d_anim_sheets::shown_edit::preview_still_owns(sim.world(), entity, v.sheet, v.asset) {
        return;
    }
    let image = SpriteImage::from_bytes(v.width, v.height, v.pixels, v.alpha);
    if let Err(e) = texture_edit::commit_edited_texture(
        entity,
        v.sheet,
        sim,
        renderer,
        asset_db,
        &image,
        v.size_world,
        toasts,
    ) {
        toasts.push(Toast::error(tr_with(
            "shell.color_equalization_bridge.color_equalization_gpu_3",
            &[("e", &e)],
        )));
    }
}

/// Drop cache entries for entities no longer in the selection, after
/// restoring each of them byte-exact to its original pixels + AlphaMode.
fn prune_and_revert_unselected(
    selected: &[u64],
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    toasts: &mut ToastQueue,
) {
    let live: std::collections::BTreeSet<u64> = selected.iter().copied().collect();
    let stale: Vec<(u64, CachedOriginal)> = SOURCE_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        let fora: Vec<u64> = cache
            .keys()
            .filter(|k| !live.contains(k))
            .copied()
            .collect();
        fora.into_iter()
            .filter_map(|k| cache.remove(&k).map(|v| (k, v)))
            .collect()
    });
    for (entity_bits, v) in stale {
        revert_one(entity_bits, v, sim, renderer, asset_db, toasts);
    }
}

fn apply_dropdown_close(slot: u8, hero: &mut HeroScreen) {
    let chip_id = match slot {
        1 => ph2d_tool_color_equalization::ids::CEQ_LUT_1_DROPDOWN,
        2 => ph2d_tool_color_equalization::ids::CEQ_LUT_2_DROPDOWN,
        3 => ph2d_tool_color_equalization::ids::CEQ_POSTERIZE_DROPDOWN,
        4 => ph2d_tool_color_equalization::ids::CEQ_QUANTIZE_DROPDOWN,
        _ => return,
    };
    if let Some(ph2d_editor_core::InteractiveState::Dropdown { open, .. }) =
        hero.store.get_mut(chip_id)
    {
        *open = false;
    }
}

/// `true` iff `id` is a CEQ slider/chip whose drag should pause
/// pipeline rebuilds (Quantize / Posterize are heavy enough that
/// running them per frame stalls the slider visibly).
fn is_ceq_drag_widget(id: ph2d_editor_core::NodeId) -> bool {
    use ph2d_tool_color_equalization::ids as cids;
    let sliders_and_chips: [ph2d_editor_core::NodeId; 26] = [
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
        cids::CEQ_LUT_INTENSITY,
        cids::CEQ_LUT_INTENSITY_NUM,
        cids::CEQ_LUT_MIX,
        cids::CEQ_LUT_MIX_NUM,
    ];
    sliders_and_chips.contains(&id)
}
