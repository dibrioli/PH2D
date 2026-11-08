//! **Os pixels das folhas de uma sprite, descodificados UMA vez** — para as miniaturas do cartão
//! «Animation», a grelha do editor e a grelha proposta (plano GameEngineFuncional 07). Uma entrada por folha
//! (plano GEF 26 §3.4): a `0` é a do `Sprite`, a `k ≥ 1` a `SpriteSheets[k-1]`.
//!
//! Duas fontes, as mesmas que o retrato do cartão de asset resolve (`ph2d_render::texture_of`):
//! a arte de FÁBRICA (`ph2d_render::atlas::fabrica`, rasterizada do SVG e já em cache estática) e a
//! imagem importada (`SpritePixels` ou a célula do átlas → `AssetId` → `AssetDb::image_rgba8`). Uma
//! textura `Individual` sem `SpritePixels` não tem pixels em memória: fica sem folha (e sem miniatura).

use std::collections::BTreeMap;
use std::sync::Arc;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, Resource, SpritePixels, SpriteSheets, World};
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
    /// O nome do ficheiro (sem extensão), quando a base de assets o conhece.
    pub name: Option<String>,
}

/// A cache, como recurso do mundo (nunca gravado: o mundo grava componentes).
#[derive(Resource, Default)]
pub struct SheetCache {
    entries: Vec<(SheetKey, SheetEntry)>,
    /// A folha `k` de cada entidade que alguém olhou neste quadro.
    by_entity: BTreeMap<(u64, u8), SheetKey>,
}

impl SheetCache {
    /// A folha `0` da entidade, se [`refresh`] a resolveu.
    #[must_use]
    pub fn of_entity(&self, bits: u64) -> Option<&SheetEntry> {
        self.of_sheet(bits, 0)
    }

    /// A folha `k` da entidade, se [`refresh`] a resolveu.
    #[must_use]
    pub fn of_sheet(&self, bits: u64, k: u8) -> Option<&SheetEntry> {
        let key = self.by_entity.get(&(bits, k))?;
        self.entries.iter().find(|(c, _)| c == key).map(|(_, e)| e)
    }

    fn entry(&mut self, key: SheetKey, decode: impl FnOnce() -> Option<SheetEntry>) -> bool {
        if let Some(i) = self.entries.iter().position(|(k, _)| *k == key) {
            // O mais recente fica no fim: quem sai é o que ninguém olha há mais tempo.
            let e = self.entries.remove(i);
            self.entries.push(e);
            return true;
        }
        let Some(entry) = decode() else {
            return false;
        };
        if self.entries.len() >= SHEETS_KEPT {
            self.entries.remove(0);
        }
        self.entries.push((key, entry));
        true
    }
}

/// A fonte da folha `0` de uma entidade (sem descodificar nada).
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

/// As fontes de TODAS as folhas da entidade, pela ordem: `[0]` = [`sheet_key`], `[k]` = a folha extra `k`.
#[must_use]
pub fn sheet_keys(
    world: &World,
    e: Entity,
    atlas: &BTreeMap<u32, AssetId>,
) -> Vec<Option<SheetKey>> {
    let extra = world.get::<SpriteSheets>(e).map_or(&[][..], |s| &s.0[..]);
    std::iter::once(sheet_key(world, e, atlas))
        .chain(extra.iter().map(|x| Some(SheetKey::Asset(x.asset))))
        .collect()
}

fn decode(key: SheetKey, db: &AssetDb) -> Option<SheetEntry> {
    let (sheet, name) = match key {
        SheetKey::Factory(k) => {
            let art = ph2d_render::FACTORY_ART.iter().find(|a| a.key == k)?;
            let px = ph2d_render::factory_image(art);
            let [w, h] = art.image_px();
            let name = ph2d_i18n::tr(art.name).to_owned();
            (SheetImage::new(Arc::new(px.to_vec()), w, h)?, Some(name))
        }
        SheetKey::Asset(id) => {
            let asset = db.get(&id)?;
            let (w, h, px) = asset.image_rgba8()?;
            let name = db
                .tracked_paths()
                .into_iter()
                .find(|p| db.id_for_path(p) == Some(id))
                .and_then(|p| p.file_stem().map(|n| n.to_string_lossy().into_owned()));
            (SheetImage::new(Arc::new(px.into_owned()), w, h)?, name)
        }
    };
    let alpha: Vec<u8> = sheet.rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
    let guess = ph2d_sprite_sheet::guess_grid(&alpha, sheet.w, sheet.h);
    Some(SheetEntry { sheet, guess, name })
}

/// Resolve (e descodifica, se ainda não está) as folhas de cada entidade de `entities`. Chamado uma
/// vez por quadro, ANTES dos snapshots: o cartão e o editor leem o resultado.
pub fn refresh(world: &mut World, entities: &[u64], db: &AssetDb, atlas: &BTreeMap<u32, AssetId>) {
    let keys: Vec<(u64, Vec<Option<SheetKey>>)> = entities
        .iter()
        .map(|&b| (b, sheet_keys(world, Entity::from_bits(b), atlas)))
        .collect();
    let mut cache = world.get_resource_or_init::<SheetCache>();
    cache.by_entity.clear();
    for (bits, keys) in keys {
        for (k, key) in keys.into_iter().enumerate() {
            if let (Some(key), Ok(k)) = (key, u8::try_from(k))
                && cache.entry(key, || decode(key, db))
            {
                cache.by_entity.insert((bits, k), key);
            }
        }
    }
}
