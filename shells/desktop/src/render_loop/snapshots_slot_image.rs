//! A linha *Image* do Inspector e os factos do cartão da Sprite. Filho de `snapshots_inspector` por `#[path]`.

use super::*;

/// ⭐ A imagem da linha *Image* do Inspector (plano GEF 12): miniatura e nome da sprite seleccionada.
pub(super) fn publish_slot_image(
    alvo: &Alvo,
    sim: &mut SimWorld,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    ppm: f32,
) {
    ph2d_panel_inspector::slot_image::set_current_sprite_facts(alvo.bits.and_then(|b| {
        let e = ph2d_ecs::Entity::from_bits(b);
        use ph2d_app_sprite_anim::sprite_card as card;
        let w = sim.world();
        let (pivot_ponto, pivot_px) =
            ph2d_sprite_edit::pivot::pivot_de(w, e, asset_db, atlas_asset_map, ppm)?;
        Some(ph2d_panel_inspector::slot_image::SpriteFacts {
            entity_bits: b,
            sem_imagem: ph2d_sprite_edit::vazio::sem_imagem(w, e),
            tem_filhos: ph2d_sprite_edit::vazio::tem_filhos(w, e),
            pivot_ponto,
            pivot_px,
            folha: card::folha_em_still(w, e, asset_db, atlas_asset_map, ppm),
            corte_velho: card::cut_notice(w, e, asset_db, atlas_asset_map, ppm),
            pivot_no_editor: ph2d_app_sprite_anim::editor::pivot_lives_in_editor(w, e),
        })
    }));
    ph2d_panel_inspector::slot_image::set_current_slot_image(alvo.bits.and_then(|b| {
        let e = ph2d_ecs::Entity::from_bits(b);
        let (name, t) =
            ph2d_app_components::asset_index_build::image_card(sim, asset_db, atlas_asset_map, e)?;
        Some(ph2d_panel_inspector::slot_image::SlotImage {
            entity_bits: b,
            name,
            rgba: t.rgba,
            w: t.w,
            h: t.h,
        })
    }));
    ph2d_panel_inspector::slot_image::set_current_normal_map(alvo.bits.and_then(|b| {
        let e = ph2d_ecs::Entity::from_bits(b);
        use ph2d_app_sprite_anim::normal_map as nm;
        let n = nm::normal_do_sprite(sim.world(), atlas_asset_map, e)?;
        let nome = nm::nome_da_normal(n, || {
            let mut lib =
                ph2d_app_components::library_names::session_library(sim, asset_db, atlas_asset_map);
            lib.remove(&n).unwrap_or_default()
        });
        Some((b, nome))
    }));
    let folhas = alvo
        .bits
        .and_then(|b| folhas::sheet_card(sim, asset_db, atlas_asset_map, ppm, b));
    ph2d_panel_inspector::slot_sheets::set_current_sheets(folhas);
}

/// As folhas de um Animated no cartão Image (plano GEF 26 §3.3).
#[path = "snapshots_slot_sheets.rs"]
mod folhas;
