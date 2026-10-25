//! **Os pixels da folha de uma sprite, descodificados UMA vez** — para as miniaturas do cartão
//! «Animation», a grelha do editor e a grelha proposta (plano GameEngineFuncional 07).
//!
//! Duas fontes, as mesmas que o retrato do cartão de asset resolve (`asset_index_build::texture_of`):
//! a arte de FÁBRICA (`ph2d_render::atlas::fabrica`, rasterizada do SVG e já em cache estática) e a
//! imagem importada (`SpritePixels` ou a célula do átlas → `AssetId` → `AssetDb::image_rgba8`). Uma
//! textura `Individual` sem `SpritePixels` não tem pixels em memória: fica sem folha (e sem miniatura).

use std::collections::BTreeMap;
use std::sync::Arc;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, Resource, SpritePixels, World};
use ph2d_editor_core::SheetImage;
use ph2d_sprite_sheet::GridGuess;

/// Quantas folhas ficam descodificadas: o recurso é a RAM — a do herói são 4,7 MB, e o editor e o
/// cartão só olham para a sprite escolhida e para as abas abertas.
pub const SHEETS_KEPT: usize = 16;

/// De onde uma folha vem.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SheetKey {
    Factory(u32),
    Asset(AssetId),
}

#[derive(Clone, Debug)]
pub struct SheetEntry {
    pub sheet: SheetImage,
    /// A grelha que a transparência propõe (`guess_grid`), calculada ao descodificar.
    pub guess: Option<GridGuess>,
}

/// A cache, como recurso do mundo (nunca gravado: o mundo grava componentes).
#[derive(Resource, Default)]
pub struct SheetCache {
    entries: Vec<(SheetKey, SheetEntry)>,
    /// A folha de cada entidade que alguém olhou neste quadro.
    by_entity: BTreeMap<u64, SheetKey>,
}

impl SheetCache {
    /// A folha da entidade, se [`refresh`] a resolveu.
    #[must_use]
    pub fn of_entity(&self, bits: u64) -> Option<&SheetEntry> {
        let key = self.by_entity.get(&bits)?;
        self.entries.iter().find(|(k, _)| k == key).map(|(_, e)| e)
    }

    fn entry(&mut self, key: SheetKey, decode: impl FnOnce() -> Option<SheetImage>) -> bool {
        if let Some(i) = self.entries.iter().position(|(k, _)| *k == key) {
            // O mais recente fica no fim: quem sai é o que ninguém olha há mais tempo.
            let e = self.entries.remove(i);
            self.entries.push(e);
            return true;
        }
        let Some(sheet) = decode() else {
            return false;
        };
        let alpha: Vec<u8> = sheet.rgba.chunks_exact(4).map(|p| p[3]).collect();
        let guess = ph2d_sprite_sheet::guess_grid(&alpha, sheet.w, sheet.h);
        if self.entries.len() >= SHEETS_KEPT {
            self.entries.remove(0);
        }
        self.entries.push((key, SheetEntry { sheet, guess }));
        true
    }
}

/// A fonte da folha de uma entidade (sem descodificar nada).
#[must_use]
pub fn sheet_key(world: &World, e: Entity, atlas: &BTreeMap<u32, AssetId>) -> Option<SheetKey> {
    if let Some(p) = world.get::<SpritePixels>(e) {
        return Some(SheetKey::Asset(p.0));
    }
    match world.get::<ph2d_render::Sprite>(e)?.source {
        ph2d_render::SpriteSource::Atlas { key } => {
            if ph2d_render::FACTORY_ART.iter().any(|a| a.key == key) {
                Some(SheetKey::Factory(key))
            } else {
                atlas.get(&key).copied().map(SheetKey::Asset)
            }
        }
        _ => None,
    }
}

fn decode(key: SheetKey, db: &AssetDb) -> Option<SheetImage> {
    match key {
        SheetKey::Factory(k) => {
            let art = ph2d_render::FACTORY_ART.iter().find(|a| a.key == k)?;
            let px = ph2d_render::factory_pixels(art);
            SheetImage::new(Arc::new(px.to_vec()), art.px[0], art.px[1])
        }
        SheetKey::Asset(id) => {
            let asset = db.get(&id)?;
            let (w, h, px) = asset.image_rgba8()?;
            SheetImage::new(Arc::new(px.into_owned()), w, h)
        }
    }
}

/// Resolve (e descodifica, se ainda não está) a folha de cada entidade de `entities`. Chamado uma
/// vez por quadro, ANTES dos snapshots: o cartão e o editor leem o resultado.
pub fn refresh(world: &mut World, entities: &[u64], db: &AssetDb, atlas: &BTreeMap<u32, AssetId>) {
    let keys: Vec<(u64, Option<SheetKey>)> = entities
        .iter()
        .map(|&b| (b, sheet_key(world, Entity::from_bits(b), atlas)))
        .collect();
    let mut cache = world.get_resource_or_init::<SheetCache>();
    cache.by_entity.clear();
    for (bits, key) in keys {
        if let Some(key) = key
            && cache.entry(key, || decode(key, db))
        {
            cache.by_entity.insert(bits, key);
        }
    }
}
