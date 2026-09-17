//! Merge Sprites — composite ≥ 2 selected sprites' images into a
//! single new Individual-texture sprite at the union bounding box,
//! then despawn the originals. Triggered by the Hierarchy row's
//! right-click → "Merge Sprites" entry (Enio 2026-05-27).
//!
//! Pipeline (single one-shot CPU pass):
//!   1. For each selected entity, read source RGBA via the
//!      `texture_edit::read_sprite_source` chokepoint (carries the
//!      sprite's `AlphaMode`; we normalise to straight here so the
//!      compositor is uniform). Cache the transform parameters +
//!      pre-compute a tight axis-aligned world-bbox of the rotated
//!      / scaled / anchored quad.
//!   2. Union the per-source world bboxes → output rect in world
//!      meters. Quantise to `project.pixels_per_meter` for the
//!      output image dims (consistent with import-time density).
//!   3. Backward warp: for each output pixel, project the pixel
//!      centre to world coords; for each source (in selection
//!      order), skip when outside its world bbox, else inverse-
//!      transform to source image px, bilinear-sample (straight),
//!      premultiply, alpha-over into the output (premultiplied
//!      accumulator). The output buffer ends up premultiplied —
//!      matches the Apply path bake (`Sprite.premultiplied = true`).
//!   4. Upload the output as a new `IndividualTextureStore` slot,
//!      spawn one fresh sprite at the union-bbox centre (parenting
//!      under the right-clicked row's parent so the Hierarchy slot
//!      stays in place), and despawn every source entity.
//!
//! Undo: deferred. Multi-sprite snapshot (full Transform + Sprite +
//! Name + ChildOf + source-pixels of N entities) is heavier than the
//! existing per-sprite `ImageEditTransaction` shape; revisit when
//! the user asks (Enio explicitly skipped undo for v1).

use ph2d_i18n::{tr, tr_with};
use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor_core::{Toast, ToastQueue};
use ph2d_render::{Sprite, SpriteRenderer};

use crate::EPS_PIXELS_PER_METER;
use crate::hero_intents::sprite_merge_result::{
    MergeResult, MergedLayers, last_merge_result_set, last_merged_layers_set,
};
use crate::hero_intents::texture_edit;

/// Per-source record gathered in the read pass and reused twice (for
/// bbox union + per-pixel inverse transform during the warp).
struct SrcRecord {
    /// Entity_bits — recorded so the despawn pass at the end targets
    /// ONLY entities that actually contributed pixels (audit A1: the
    /// previous blanket despawn killed entities that failed to read,
    /// silently losing their data).
    bits: u64,
    rgba: Vec<u8>,
    w: u32,
    h: u32,
    // Forward-transform parameters (snapshotted once; world borrow
    // released before the despawn pass below):
    tx: f32,
    ty: f32,
    rot: f32,
    // Cached `rot.cos()` / `rot.sin()` — the per-pixel inner loop
    // calls `world_to_image` for every (out_x, out_y, src), which
    // would otherwise pay one cos+sin per call (≈ 60-cycle
    // transcendentals × N pixels × M sources ≈ tens of millions of
    // ops on a worst-case 4k² merge). Hoisting cuts that to one
    // pair per source — measured ~2× speedup on the dense loop.
    cos_t: f32,
    sin_t: f32,
    scale_x: f32,
    scale_y: f32,
    /// `1.0 / scale_x` — same hoisting reasoning as `cos_t`; lets
    /// `world_to_image` use a multiply where it used to divide.
    inv_scale_x: f32,
    inv_scale_y: f32,
    anchor_x: f32,
    anchor_y: f32,
    size_w: f32,
    size_h: f32,
    /// `1.0 / size_w` — hoisted reciprocal (perf, see `cos_t`).
    inv_size_w: f32,
    inv_size_h: f32,
    // Pre-computed world AABB of the rotated quad — lets the per-
    // pixel inner loop skip sources whose footprint can't possibly
    // cover the current output pixel.
    world_min_x: f32,
    world_max_x: f32,
    world_min_y: f32,
    world_max_y: f32,
}

/// Drain `EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::MergeSprites)`. Returns `true` if the
/// caller should set `title_dirty` (a toast was pushed).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_merge_sprites(
    entity_bits_list: Vec<u64>,
    primary_bits: u64,
    // **Guardar cada fonte como uma CAMADA** (plano `docs/Sprite_projeto/18` W10, Enio
    // 2026-08-21). A geometria é a mesma — mesma união, mesmo warp, mesmo resultado no ecrã —, e o
    // que muda é que cada fonte fica também no seu próprio buffer, para o documento do Painter os
    // receber como camadas. ⚠️ Custa N buffers do tamanho da saída em vez de um: é por isso que é
    // um MODO e não o comportamento de sempre.
    to_layers: bool,
    project_pixels_per_meter: f32,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
) -> bool {
    if entity_bits_list.len() < 2 {
        toasts.push(Toast::warning(tr(
            "shell.sprite_merge.merge_sprites_select_2",
        )));
        return true;
    }
    let project_pm = project_pixels_per_meter.max(EPS_PIXELS_PER_METER);

    let (ordered_bits, total_requested) =
        order_selection(entity_bits_list, primary_bits, sim, renderer, toasts);

    let srcs = read_sources(ordered_bits, sim, renderer, asset_db, atlas_asset_map);

    if srcs.len() < 2 {
        toasts.push(Toast::error(tr("shell.sprite_merge.merge_sprites_could")));
        return true;
    }

    let Some(warp::MergeGrid {
        union_min_x,
        union_max_x,
        union_min_y,
        union_max_y,
        union_w_m,
        union_h_m,
        out_pm,
        out_w,
        out_h,
    }) = warp::merge_grid(&srcs, project_pm, renderer, toasts)
    else {
        return true;
    };

    let (out_rgba, layer_rgba) = warp::composite(
        &srcs,
        out_w,
        out_h,
        union_min_x,
        union_max_y,
        out_pm,
        to_layers,
    );

    // Step 4a — upload to a fresh Individual slot BEFORE despawning the
    // originals so a failed acquire bails without losing data.
    let texture_id = match renderer.acquire_individual(out_w, out_h, &out_rgba) {
        Ok(id) => id,
        Err(e) => {
            toasts.push(Toast::error(tr_with(
                "shell.sprite_merge.merge_sprites_gpu",
                &[("e", &e)],
            )));
            return true;
        }
    };

    // Step 4b — gather the primary's parent (Hierarchy anchor) before
    // despawn invalidates the entity. The dedup at Step 0 guarantees
    // `primary_bits` is in `srcs[0]` whenever it was in the original
    // selection AND read succeeded; otherwise fall back to None
    // (root-level merge).
    let parent_opt = sim
        .world()
        .get::<ph2d_ecs::ChildOf>(Entity::from_bits(primary_bits))
        .map(|c| c.parent());

    // Step 4c — despawn originals. ONLY the entities whose pixels
    // actually contributed to the output (audit A1 + B-H1: blindly
    // despawning `entity_bits_list` would have destroyed sprites that
    // failed readback AND non-sprite entities in the selection like a
    // camera the user happened to multi-select). `ChildOf` cascade
    // despawns descendants too — same convention as `HierDelete`.
    let n_sources = srcs.len();
    let n_requested = total_requested;
    // ⚠️ **Os nomes ANTES do despawn.** Uma camada chamada `sprite_3f00000001` não diz nada a
    // ninguém; o nome que o artista deu diz tudo — e daqui a três linhas ele já não existe.
    let layer_names: Vec<String> = if to_layers {
        srcs.iter()
            .map(|src| {
                sim.world()
                    .get::<ph2d_ecs::Name>(Entity::from_bits(src.bits))
                    .map(|n| n.as_str().to_string())
                    .unwrap_or_else(|| tr("shell.sprite_merge.layer").to_string())
            })
            .collect()
    } else {
        Vec::new()
    };
    for src in &srcs {
        sim.world_mut().despawn(Entity::from_bits(src.bits));
    }

    // Step 4d — spawn the merged sprite at the union-bbox centre.
    let world_center_x = (union_min_x + union_max_x) * 0.5;
    let world_center_y = (union_min_y + union_max_y) * 0.5;
    let mut transform = Transform::default();
    transform.translation.x = world_center_x;
    transform.translation.y = world_center_y;
    let mut merged_sprite =
        Sprite::individual(texture_id, [union_w_m, union_h_m], [1.0, 1.0, 1.0, 1.0]);
    merged_sprite.premultiplied = true;
    // ⚠️ **O CARIMBO DURÁVEL DOS PIXELS.** Irmão do que faltava nas ilhas do BG-Removal
    // (2026-08-20): o `texture_id` é uma alocação de GPU e morre com o processo, e o
    // `save_sprite_pixels` recolhe pelos carimbos — uma sprite `Individual` sem ele **não é
    // gravada**, e reabrir o projeto devolvia o merge invisível.
    //
    // ⚠️ Aqui o desaparecimento é PIOR que nas ilhas: o merge **despawna os originais**, por isso
    // não havia de onde refazer. *O gate irmão existe para que a terceira vez não aconteça.*
    let merged_pixels_id = asset_db.insert_image_rgba8(out_w, out_h, out_rgba);

    // Uniqueness: a 2nd merge would otherwise produce another "Merged"
    // — collision risk per the 2026-05-27 same-name bug. Bump with the
    // shared scheme (` (1)`, ` (2)`, ...).
    let merged_name = ph2d_unique_name::unique_name(sim, tr("shell.sprite_merge.merged"));
    let new_entity = match parent_opt {
        Some(parent) => sim
            .world_mut()
            .spawn((
                transform,
                merged_sprite,
                ph2d_ecs::Name::new(merged_name),
                ph2d_ecs::SpritePixels(merged_pixels_id),
                ph2d_ecs::ChildOf(parent),
            ))
            .id(),
        None => sim
            .world_mut()
            .spawn((
                transform,
                merged_sprite,
                ph2d_ecs::Name::new(merged_name),
                ph2d_ecs::SpritePixels(merged_pixels_id),
            ))
            .id(),
    };

    // The new merged entity_bits is communicated upward through a
    // panic-free side channel: the caller in `render_loop/mod.rs`
    // reads it via the returned struct (audit B-H2 — Photoshop /
    // Figma promote the merged layer to the selection so the user's
    // next action operates on it). See `MergeResult` below.
    last_merge_result_set(MergeResult {
        new_entity_bits: new_entity.to_bits(),
    });
    // **O documento em camadas**, quando foi ele que se pediu (plano `docs/Sprite_projeto/18` W10).
    //
    // ⚠️ Canal PRÓPRIO, e não mais um campo no `MergeResult`: aquele é `Copy` e vive num `Cell`,
    // e um `Vec` lá dentro obrigaria a converter o canal inteiro por causa de um modo. *Um dado
    // com outro tempo de vida merece o seu canal, não uma emenda no do vizinho.*
    if to_layers {
        last_merged_layers_set(MergedLayers {
            entity_bits: new_entity.to_bits(),
            width: out_w,
            height: out_h,
            layers: layer_names.into_iter().zip(layer_rgba).collect(),
        });
    }

    // Audit B-H3: surface skipped entities in the toast so the user
    // notices when a non-sprite / readback-failed entity was in the
    // selection and got SKIPPED (not merged, but also not destroyed).
    let skipped = n_requested.saturating_sub(n_sources);
    if skipped > 0 {
        toasts.push(Toast::success(tr_with(
            "shell.sprite_merge.merged_sprites_skipped",
            &[("n_sources", &n_sources), ("skipped", &skipped)],
        )));
    } else {
        toasts.push(Toast::success(tr_with(
            "shell.sprite_merge.merged_sprites",
            &[("n_sources", &n_sources)],
        )));
    }
    true
}

/// A selecção deduplicada com a linha clicada à frente, e o aviso da precisão que a fusão custa;
/// devolve `(a ordem, quantas se pediram)`.
fn order_selection(
    entity_bits_list: Vec<u64>,
    primary_bits: u64,
    sim: &SimWorld,
    renderer: &SpriteRenderer,
    toasts: &mut ToastQueue,
) -> (Vec<u64>, usize) {
    // Dedup + reorder so the right-clicked entity (`primary_bits`)
    // lands at index 0. Audit B2: the grid-snap heuristic at Step 2.5
    // uses `srcs[0]` as the "primary" anchor, and the Hierarchy parent
    // is read from `primary_bits` — those two must agree, otherwise
    // the lossless-snap targets a different sprite than the user
    // right-clicked. Audit A2: dedup defends against accidental
    // duplicates in the selection iter that would over-composite
    // (premul-over isn't idempotent).
    let mut ordered_bits: Vec<u64> = Vec::with_capacity(entity_bits_list.len());
    if entity_bits_list.contains(&primary_bits) {
        ordered_bits.push(primary_bits);
    }
    for &bits in &entity_bits_list {
        if bits != primary_bits && !ordered_bits.contains(&bits) {
            ordered_bits.push(bits);
        }
    }
    let total_requested = ordered_bits.len();

    // **A PRECISÃO QUE A FUSÃO CUSTA** (plano `docs/Sprite_projeto/18` W7).
    //
    // ⚠️ O acumulador é de 8 bits e a composição «over» é **aritmética sobre a cor** — pela pergunta
    // única da auditoria (`docs/Sprite_projeto/19` §1), converter aqui é **correcto**: preservar
    // exigiria um compositor de 16 bits, que é código novo que ninguém pediu.
    //
    // ⚠️ **O que estava errado era o silêncio**, e aqui ele custa mais que numa ferramenta: a fusão
    // **despawna os originais**. Uma ferramenta rebaixa uma sprite que se pode desfazer olhando
    // para ela; esta apaga as fontes, e o artista só descobre a perda quando for exportar.
    let downgraded = ordered_bits
        .iter()
        .filter(|&&bits| texture_edit::holds_sixteen_bit(Entity::from_bits(bits), sim, renderer))
        .count();
    if downgraded > 0 {
        toasts.push(Toast::info(tr_with(
            "shell.sprite_merge.converted_sprite_s_to",
            &[("downgraded", &downgraded)],
        )));
    }
    (ordered_bits, total_requested)
}

/// O passo 1: lê cada fonte e fotografa a pose dela — quem não contribui salta, e fica fora do
/// despawn.
fn read_sources(
    ordered_bits: Vec<u64>,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
) -> Vec<SrcRecord> {
    // Step 1 — read each source + snapshot its transform.
    let mut srcs: Vec<SrcRecord> = Vec::with_capacity(ordered_bits.len());
    for &bits in &ordered_bits {
        let entity = Entity::from_bits(bits);
        let Some(read) =
            texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
        else {
            // Skip — non-sprite entity OR atlas miss / readback fail.
            // Audit A1 + B-H1: the entity is also EXCLUDED from the
            // despawn pass below, so a partial read never destroys
            // the user's sprite without its pixels making it into
            // the merged output.
            continue;
        };
        // Compositing in PREMULTIPLIED space — both for the sampler
        // (avoids the dark-fringe straight-bilinear produces at
        // partial-alpha edges: a half-pixel between opaque red and
        // transparent reads as `R/2, A/2` which composes to
        // half-brightness; premul makes it `R/2, A/2` which composes
        // to full-brightness, half-coverage — the GPU's behaviour for
        // bake-time `Rgba8UnormSrgb` textures) AND for the accumulator
        // (premul "over" is `dst = src + dst*(1-src.a)` with no
        // divide). Atlas sprites are stored straight on disk → premul
        // at read time; Individual-source sprites are already premul
        // after BG-Removal / Trim / etc. bakes → `into_premultiplied`
        // no-ops them.
        let premul = read.image.into_premultiplied();
        let world = sim.world();
        let Some(tr) = world.get::<Transform>(entity) else {
            continue;
        };
        let Some(sprite) = world.get::<Sprite>(entity) else {
            continue;
        };
        let tx = tr.translation.x;
        let ty = tr.translation.y;
        let rot = tr.rotation;
        let scale_x = tr.scale.x;
        let scale_y = tr.scale.y;
        let anchor_x = sprite.anchor[0];
        let anchor_y = sprite.anchor[1];
        let size_w = sprite.size[0];
        let size_h = sprite.size[1];
        // Audit A4 + A5: skip sources whose world footprint OR image
        // dims are degenerate. They contribute zero pixels but the
        // previous code grew the union bbox + ran the inner loop per
        // pixel only to reject every sample → wasted CPU AND output
        // area. `scale` near zero is also caught here — `world_to_image`
        // would divide by zero downstream.
        if size_w.abs() < 1e-6
            || size_h.abs() < 1e-6
            || scale_x.abs() < 1e-6
            || scale_y.abs() < 1e-6
            || premul.width == 0
            || premul.height == 0
        {
            continue;
        }
        // World AABB of the rotated quad — walk the 4 image corners
        // through the same forward chain compose uses
        // (`T * R * Ta * S * P_local`).
        let cos_t = rot.cos();
        let sin_t = rot.sin();
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for &(cx_u, cy_u) in &[(-0.5_f32, -0.5_f32), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)] {
            let lx = cx_u * size_w;
            let ly = cy_u * size_h;
            let sx = lx * scale_x;
            let sy = ly * scale_y;
            let ax = sx + anchor_x;
            let ay = sy + anchor_y;
            let rx = ax * cos_t - ay * sin_t;
            let ry = ax * sin_t + ay * cos_t;
            let wx = rx + tx;
            let wy = ry + ty;
            if wx < min_x {
                min_x = wx;
            }
            if wx > max_x {
                max_x = wx;
            }
            if wy < min_y {
                min_y = wy;
            }
            if wy > max_y {
                max_y = wy;
            }
        }
        srcs.push(SrcRecord {
            bits,
            rgba: premul.pixels,
            w: premul.width,
            h: premul.height,
            tx,
            ty,
            rot,
            cos_t,
            sin_t,
            scale_x,
            scale_y,
            inv_scale_x: 1.0 / scale_x,
            inv_scale_y: 1.0 / scale_y,
            anchor_x,
            anchor_y,
            size_w,
            size_h,
            inv_size_w: 1.0 / size_w,
            inv_size_h: 1.0 / size_h,
            world_min_x: min_x,
            world_max_x: max_x,
            world_min_y: min_y,
            world_max_y: max_y,
        });
    }
    srcs
}

/// World→source-pixel resample math (inverse mapping + premultiplied
/// bilinear sampler) lives in a sibling `#[path]` child module so this
/// file stays under the HR-18 LOC cap; `super::SrcRecord` + its private
/// fields stay reachable from there. CPU-unit-tested in that file.
#[path = "sprite_merge_resample.rs"]
mod resample;
use resample::{bilinear_sample_premul, world_to_image};
/// Os passos 2 e 3 da fusão (a grelha de saída e o warp) — filho por assunto, como o `resample`.
#[path = "sprite_merge_warp.rs"]
mod warp;
