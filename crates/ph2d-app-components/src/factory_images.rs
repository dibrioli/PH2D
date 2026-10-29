//! O nome traduzido de cada imagem de fábrica — a lei da fábrica no atlas mora na folha
//! ([`ph2d_render::factory_images`]); o nome é texto da interface (`ph2d-i18n`) e fica com o índice.

use ph2d_asset::AssetId;
use ph2d_render::factory_images::art_of_key;
use std::collections::BTreeMap;

/// O nome traduzido de cada imagem de fábrica presente no mapa.
#[must_use]
pub fn names(atlas: &BTreeMap<u32, AssetId>) -> BTreeMap<AssetId, String> {
    atlas
        .iter()
        .filter_map(|(k, id)| Some((*id, ph2d_i18n::tr(art_of_key(*k)?.name).to_owned())))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::asset_card_art::CardArt;
    use crate::asset_index_build::{TextureLibrary, build};
    use ph2d_asset::AssetDb;
    use ph2d_asset_index::{AssetKind, AssetRef};
    use ph2d_ecs::SimWorld;
    use std::collections::BTreeMap;

    /// ⭐ Plano GEF 12: num projecto VAZIO (ninguém usa nada) a biblioteca oferece as 8 imagens de fábrica, cada
    /// uma pelo nome dela; e uma importada que ninguém usa também lá está.
    #[test]
    fn a_biblioteca_oferece_a_fabrica_pelo_nome_mesmo_sem_uso() {
        let db = AssetDb::new();
        let mut atlas = BTreeMap::new();
        ph2d_render::factory_images::register(&db, &mut atlas);
        let solta = db.insert_image_rgba8(2, 2, vec![9; 16]);
        atlas.insert(ph2d_render::FIRST_IMPORT_KEY, solta);
        let mut sim = SimWorld::new();
        let index = build(
            &mut sim,
            &db,
            &atlas,
            &ph2d_asset_index::CatalogTree::new(),
            &mut CardArt::new(),
            &mut TextureLibrary::default(),
            100.0,
        );
        assert_eq!(
            index.counts().get(&AssetKind::Texture),
            Some(&(ph2d_render::FACTORY_ART.len() + 1))
        );
        let hills = atlas[&ph2d_render::HILLS_TILE_KEY];
        let nome = index
            .get(&AssetRef::Texture {
                asset: *hills.as_bytes(),
            })
            .map(|e| e.name.clone());
        assert_eq!(nome.as_deref(), Some("Hills"));
        // CONTROLO: sem a fábrica no mapa a biblioteca de um projecto vazio só tem a importada.
        let index = build(
            &mut sim,
            &db,
            &BTreeMap::from([(ph2d_render::FIRST_IMPORT_KEY, solta)]),
            &ph2d_asset_index::CatalogTree::new(),
            &mut CardArt::new(),
            &mut TextureLibrary::default(),
            100.0,
        );
        assert_eq!(index.counts().get(&AssetKind::Texture), Some(&1));
    }
}
