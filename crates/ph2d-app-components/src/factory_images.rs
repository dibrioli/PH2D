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
        let px = ph2d_render::factory_pixels(art);
        let id = db.insert_image_rgba8(art.px[0], art.px[1], px.to_vec());
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

/// O nome traduzido de cada imagem de fábrica presente no mapa.
#[must_use]
pub fn names(atlas: &BTreeMap<u32, AssetId>) -> BTreeMap<AssetId, String> {
    atlas
        .iter()
        .filter_map(|(k, id)| Some((*id, ph2d_i18n::tr(art_of_key(*k)?.name).to_owned())))
        .collect()
}
