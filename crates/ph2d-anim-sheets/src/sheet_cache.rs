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
    /// O nome da arte de FÁBRICA; o de uma imagem da biblioteca é o de `ph2d_ecs::SheetNames`.
    pub name: Option<String>,
}

/// A cache, como recurso do mundo (nunca gravado: o mundo grava componentes).
#[derive(Resource, Default)]
pub struct SheetCache {
    entries: Vec<(SheetKey, SheetEntry)>,
    /// A folha `k` de cada entidade que alguém olhou neste quadro.
    by_entity: BTreeMap<(u64, u8), SheetKey>,
    /// A folha que o Painter pinta, com a tinta por assar: `(objeto, folha, revisão, imagem)`.
    live: Option<(u64, u8, (u64, u64), SheetEntry)>,
    /// A revisão vista no quadro anterior: só se lê a tinta quando ela PAROU (o traço acabou).
    seen: Option<(u64, (u64, u64))>,
    /// Há quantos quadros seguidos a revisão vista não muda ([`QUADROS_PARADOS`]).
    parado: u32,
}

/// ⭐ **Quantos quadros a tinta tem de ficar PARADA antes de se ler** ([`follow_live`]): meio segundo a 60 Hz. A água do
/// Wet re-revisa a tinta muito depois do traço, a ~24 Hz (log do dono 09/11: `TAXA DA AGUA 24.4 Hz`), e com UM quadro
/// parado a leitura (a tela inteira composta e acesa pelo relevo do papel: 61 ms por quadro a 1024² Textured Paper)
/// corria a cada passo dela — o dono viu o raw cair de 170 para 14 ao pintar. Parada meio segundo, a água assentou.
pub const QUADROS_PARADOS: u32 = 30;

impl SheetCache {
    /// A folha `0` da entidade, se [`refresh`] a resolveu.
    #[must_use]
    pub fn of_entity(&self, bits: u64) -> Option<&SheetEntry> {
        self.of_sheet(bits, 0)
    }

    /// A folha `k` da entidade, se [`refresh`] a resolveu.
    #[must_use]
    pub fn of_sheet(&self, bits: u64, k: u8) -> Option<&SheetEntry> {
        if let Some((b, s, _, e)) = &self.live
            && (*b, *s) == (bits, k)
        {
            return Some(e);
        }
        let key = self.by_entity.get(&(bits, k))?;
        self.entries.iter().find(|(c, _)| c == key).map(|(_, e)| e)
    }

    /// A fonte da folha `k` da entidade, se [`refresh`] a resolveu.
    #[must_use]
    pub fn key_of(&self, bits: u64, k: u8) -> Option<SheetKey> {
        self.by_entity.get(&(bits, k)).copied()
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
            // O nome é o da biblioteca (`ph2d_ecs::SheetNames`), o MESMO do cartão Image: não se lê aqui.
            (SheetImage::new(Arc::new(px.into_owned()), w, h)?, None)
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

/// ⭐ **A folha que o Painter pinta, AO VIVO** (dono 08/11: o editor de animações só se atualizava depois de assar).
/// `vivo` = o objeto que o Painter pinta com tinta por assar e a revisão do conteúdo dele (`None` = nada por assar: a
/// imagem gravada volta a ser a verdade). Quando a revisão fica [`QUADROS_PARADOS`] quadros sem andar (o traço acabou e
/// a água assentou), `read` dá os píxeis retos da tinta, e a folha visível desse objeto passa a mostrá-los — no editor,
/// no cartão e nas miniaturas. Uma leitura por traço, nunca por quadro nem por passo da água.
pub fn follow_live(
    world: &mut World,
    vivo: Option<(u64, (u64, u64))>,
    read: impl FnOnce() -> Option<(Vec<u8>, u32, u32)>,
) {
    let k = vivo.map(|(b, _)| ph2d_ecs::shown_sheet(world, Entity::from_bits(b)));
    let mut cache = world.get_resource_or_init::<SheetCache>();
    let (Some((bits, rev)), Some(k)) = (vivo, k) else {
        (cache.live, cache.seen, cache.parado) = (None, None, 0);
        return;
    };
    if cache.seen.replace((bits, rev)) != Some((bits, rev)) {
        cache.parado = 0;
        return; // ainda a mudar
    }
    cache.parado = cache.parado.saturating_add(1);
    if cache.parado < QUADROS_PARADOS {
        return; // parada há pouco: a água do Wet ainda pode andar
    }
    if cache
        .live
        .as_ref()
        .is_some_and(|l| (l.0, l.1, l.2) == (bits, k, rev))
    {
        return;
    }
    let guess = cache.of_sheet(bits, k).and_then(|e| e.guess);
    cache.live = read()
        .and_then(|(px, w, h)| SheetImage::new(Arc::new(px), w, h))
        .map(|sheet| {
            let entry = SheetEntry {
                sheet,
                guess,
                name: None,
            };
            (bits, k, rev, entry)
        });
}

#[cfg(test)]
#[path = "sheet_cache_tests.rs"]
mod tests;
