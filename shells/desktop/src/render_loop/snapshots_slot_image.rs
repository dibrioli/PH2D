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
    let vazio = alvo.bits.is_some_and(|b| {
        ph2d_sprite_edit::vazio::sem_imagem(sim.world(), ph2d_ecs::Entity::from_bits(b))
    });
    ph2d_panel_inspector::slot_image::set_recentes(if vazio {
        ph2d_app_components::library_names::names_of(
            sim,
            asset_db,
            atlas_asset_map,
            &ph2d_sprite_edit::vazio::recentes(),
        )
    } else {
        Vec::new()
    });
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
}
