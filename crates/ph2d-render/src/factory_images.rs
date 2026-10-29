//! ⭐ **A arte de fábrica é imagem da biblioteca** ([plano GEF 12](../../../docs/GameEngineFuncional/12_imagem_como_asset_plano.md)):
//! os pixels de cada `FACTORY_ART` entram no `AssetDb` e a célula reservada entra no mapa
//! `chave → AssetId` do projecto, como uma imagem importada — o índice, o seletor, o *Select users* e as
//! ferramentas de imagem passam a vê-la pela mesma porta ([`crate::texture_of`]).
//!
//! Desceu da `ph2d-app-components` (`factory_images.rs`) em 2026-10-28: a família da animação de sprite
//! lê a MESMA lei, e uma família não chama outra (gate `architecture_no_dependency_climbs_a_layer`). Os
//! nomes traduzidos (`ph2d_app_components::factory_images::names`) ficaram com o índice que os mostra.
//!
//! ⚠️ O programa traz estes pixels: o save NÃO os grava ([`is_factory_key`]) e a célula nunca se devolve
//! ao atlas.

use crate::{FACTORY_ART, FactoryArt};
use ph2d_asset::{AssetDb, AssetId};
use std::collections::BTreeMap;

/// Mete a arte de fábrica no `db` e no mapa do atlas. Idempotente (o id é o blake3 dos pixels).
pub fn register(db: &AssetDb, atlas: &mut BTreeMap<u32, AssetId>) {
    for art in &FACTORY_ART {
        // A IMAGEM (um azulejo é um azulejo); as cópias à volta dele são só do atlas (plano GEF 13 D1).
        let [w, h] = art.image_px();
        let id = db.insert_image_rgba8(w, h, crate::factory_image(art).to_vec());
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
    let mut q = world.query::<(ph2d_ecs::Entity, &crate::Sprite, &ph2d_ecs::SpriteGrid)>();
    let antigas: Vec<ph2d_ecs::Entity> = q
        .iter(world)
        .filter(|(_, s, g)| match s.source {
            crate::SpriteSource::Atlas { key } => art_of_key(key).is_some_and(|a| {
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
