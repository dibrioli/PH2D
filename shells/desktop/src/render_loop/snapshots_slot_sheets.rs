//! ⭐ **As folhas de um Animated como o cartão Image as mostra** (plano GEF 26 §3.3). Filho de `snapshots_slot_image`.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SimWorld, SpriteGrid, SpriteSheets};
use ph2d_panel_inspector::slot_image::SlotImage;
use ph2d_panel_inspector::slot_sheets::{SheetCard, SheetFace};
use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    /// Os nomes da biblioteca de cada imagem: ordenar a biblioteca só quando as imagens mudam.
    static NOMES: RefCell<(Vec<AssetId>, Vec<String>)> = const { RefCell::new((Vec::new(), Vec::new())) };
}

/// As folhas do objeto `b` (a `0` primeiro); `None` fora de um Animated com imagem.
pub(super) fn sheet_card(
    sim: &mut SimWorld,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
    b: u64,
) -> Option<SheetCard> {
    use ph2d_app_sprite_anim::image_use::{ImageUse, image_use};
    let (w, e) = (sim.world(), Entity::from_bits(b));
    if image_use(w, e) != Some(ImageUse::Animated) || ph2d_sprite_edit::vazio::sem_imagem(w, e) {
        return None;
    }
    let g = w
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let first = ph2d_render::texture_of(w.get(e), w.get(e), atlas)?;
    let extra = w
        .get::<SpriteSheets>(e)
        .map(|s| s.0.to_vec())
        .unwrap_or_default();
    let (ids, grids): (Vec<AssetId>, Vec<[u32; 2]>) =
        std::iter::once((first, [g.hframes, g.vframes]))
            .chain(extra.iter().map(|x| (x.asset, [x.hframes, x.vframes])))
            .unzip();
    let shown = ph2d_ecs::shown_sheet(w, e);
    let nomes = NOMES.with(|c| {
        let mut c = c.borrow_mut();
        if c.0 != ids {
            let lib = ph2d_app_components::library_names::session_library(sim, db, atlas);
            let nomes = ids
                .iter()
                .map(|id| lib.get(id).cloned().unwrap_or_default())
                .collect();
            *c = (ids.clone(), nomes);
        }
        c.1.clone()
    });
    let sheets = ids
        .into_iter()
        .zip(grids)
        .zip(nomes)
        .map(|((id, grid), name)| {
            let t = ph2d_app_components::asset_index_build::image_thumb(db, id)?;
            let px = ph2d_app_sprite_anim::image_choice::image_facts(id, db, atlas, ppm)?.px;
            Some(SheetFace {
                image: SlotImage {
                    entity_bits: b,
                    name,
                    rgba: t.rgba,
                    w: t.w,
                    h: t.h,
                },
                px: (px[0], px[1]),
                grid,
            })
        })
        .collect::<Option<_>>()?;
    Some(SheetCard {
        entity_bits: b,
        shown,
        sheets,
    })
}

#[cfg(test)]
#[path = "snapshots_slot_sheets_tests.rs"]
mod tests;
