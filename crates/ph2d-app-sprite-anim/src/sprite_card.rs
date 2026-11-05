//! O que o cartão da Sprite pergunta ao mundo e o snapshot do Inspector não responde
//! ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md)).

use crate::image_choice::library_facts;
use crate::image_use::{ImageUse, cells, image_use, proposed_grid, shown_id};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, World};
use ph2d_render::Sprite;
use ph2d_sprite_edit::vazio::sem_imagem;
use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    /// A imagem é endereçada pelo conteúdo: a proposta de grelha de um id nunca muda.
    static FOLHAS: RefCell<BTreeMap<AssetId, Option<[u32; 2]>>> = const { RefCell::new(BTreeMap::new()) };
}

/// A grelha de `e` se ele mostra em Still uma imagem que é folha: o corte da própria imagem, senão o que a
/// transparência propõe. É o que o aviso de folha pergunta (o salto de 100 × 100 para 800 × 400 px).
#[must_use]
pub fn folha_em_still(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<[u32; 2]> {
    if image_use(world, e) != Some(ImageUse::Still) {
        return None;
    }
    let id = shown_id(world, e, atlas)?;
    if let Some(f) = library_facts(world, id, db, atlas, ppm).filter(|f| cells(*f) > 1) {
        return f.slicing.map(|s| s.grid);
    }
    FOLHAS.with(|m| {
        *m.borrow_mut().entry(id).or_insert_with(|| {
            let asset = db.get(&id)?;
            let (w, h, px) = asset.image_rgba8()?;
            proposed_grid(&px, w, h)
        })
    })
}

/// **Keep old size**: `e` volta ao tamanho de antes do Replace, com a imagem nova. O collider automático segue.
pub fn keep_size(world: &mut World, e: Entity, tamanho: [f32; 2]) -> bool {
    let Some(antes) = world.get::<Sprite>(e).map(|s| s.size) else {
        return false;
    };
    if antes == tamanho {
        return false;
    }
    if let Some(mut s) = world.get_mut::<Sprite>(e) {
        s.size = tamanho;
    }
    crate::tiled::follow_auto_collider(world, e, antes, tamanho);
    true
}

/// Os píxeis e o tamanho de `e` antes de um Replace; `None` se ela ainda não mostra uma imagem.
#[must_use]
pub fn antes_do_replace(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<([u32; 2], [f32; 2])> {
    if sem_imagem(world, e) {
        return None;
    }
    let f = library_facts(world, shown_id(world, e, atlas)?, db, atlas, ppm)?;
    Some((f.px, world.get::<Sprite>(e)?.size))
}

#[cfg(test)]
#[path = "sprite_card_tests.rs"]
mod tests;
