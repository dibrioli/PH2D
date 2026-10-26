//! ⭐ **A arte de fábrica é imagem da biblioteca** ([plano GEF 12](../../../docs/GameEngineFuncional/12_imagem_como_asset_plano.md)):
//! os pixels de cada `FACTORY_ART` entram no `AssetDb` e a célula reservada entra no mapa
//! `chave → AssetId` do projecto, como uma imagem importada — o índice, o seletor, o *Select users* e as
//! ferramentas de imagem passam a vê-la pela mesma porta ([`crate::asset_index_build::texture_of`]).
//!
//! ⚠️ O programa traz estes pixels: o save NÃO os grava ([`is_factory_key`]) e a célula nunca se devolve
//! ao atlas.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_render::{FACTORY_ART, FactoryArt};
use std::collections::BTreeMap;

/// Mete a arte de fábrica no `db` e no mapa do atlas. Idempotente (o id é o blake3 dos pixels).
pub fn register(db: &AssetDb, atlas: &mut BTreeMap<u32, AssetId>) {
    for art in &FACTORY_ART {
        // A IMAGEM (um azulejo é um azulejo); as cópias à volta dele são só do atlas (plano GEF 13 D1).
        let [w, h] = art.image_px();
        let id = db.insert_image_rgba8(w, h, ph2d_render::factory_image(art).to_vec());
        atlas.insert(art.key, id);
    }
}

/// A arte de fábrica desta célula do atlas.
#[must_use]
pub fn art_of_key(key: u32) -> Option<&'static FactoryArt> {
    FACTORY_ART.iter().find(|a| a.key == key)
}

/// A célula é de fábrica (os pixels vêm do programa, não do ficheiro).
#[must_use]
pub fn is_factory_key(key: u32) -> bool {
    art_of_key(key).is_some()
}

/// ⭐ **Ficheiro antigo** (plano GEF 13 D2): até à onda 13 um objecto dizia «o azulejo» com a grelha das cópias
/// (`SpriteGrid` = `art.grid`, a célula do meio); hoje a região da chave JÁ é a cópia do meio. Tira essa grelha
/// de toda sprite de azulejo de fábrica e devolve quantas mudaram. Idempotente; as folhas animadas ficam.
pub fn forget_extrusion_grids(world: &mut ph2d_ecs::World) -> usize {
    let mut q = world.query::<(
        ph2d_ecs::Entity,
        &ph2d_render::Sprite,
        &ph2d_ecs::SpriteGrid,
    )>();
    let antigas: Vec<ph2d_ecs::Entity> = q
        .iter(world)
        .filter(|(_, s, g)| match s.source {
            ph2d_render::SpriteSource::Atlas { key } => art_of_key(key).is_some_and(|a| {
                a.tile && [g.hframes, g.vframes] == a.grid && g.frame == a.middle_frame()
            }),
            _ => false,
        })
        .map(|(e, ..)| e)
        .collect();
    for e in &antigas {
        world.entity_mut(*e).remove::<ph2d_ecs::SpriteGrid>();
    }
    antigas.len()
}

/// ⭐ As células cujos píxeis o ficheiro do projecto grava: as que o projecto trouxe, nunca a fábrica.
pub fn saved_by_project(atlas: &BTreeMap<u32, AssetId>) -> impl Iterator<Item = (u32, &AssetId)> {
    atlas
        .iter()
        .filter(|(k, _)| !is_factory_key(**k))
        .map(|(k, id)| (*k, id))
}

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
        super::register(&db, &mut atlas);
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
