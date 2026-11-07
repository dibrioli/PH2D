//! O que o cartão da Sprite pergunta ao mundo e o snapshot do Inspector não responde
//! ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md)).

use crate::image_choice::ImageFacts;
use crate::image_choice::library_facts;
use crate::image_use::{ImageUse, cells, display_scale, image_use, proposed_grid, shown_id};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SpriteAnimations, SpriteCutNotice, SpriteGrid, World};
use ph2d_editor_core::ImageCutNotice;
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

/// **Try with the sample hero**: o herói de fábrica entra no lugar do vazio.
pub fn try_sample_hero(world: &mut World, e: Entity) -> bool {
    sem_imagem(world, e) && ph2d_render::veste_o_heroi(world, e).is_ok()
}

/// **O aviso de imagem trocada** de `e`: só em Animated, com animações, e enquanto a grelha é a que ficou.
#[must_use]
pub fn cut_notice(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<ImageCutNotice> {
    let aviso = world.get::<SpriteCutNotice>(e)?;
    let g = world.get::<SpriteGrid>(e)?;
    let n = world.get::<SpriteAnimations>(e)?.iter().count();
    if n == 0
        || aviso.grid != [g.hframes, g.vframes]
        || image_use(world, e) != Some(ImageUse::Animated)
    {
        return None;
    }
    let px = library_facts(world, shown_id(world, e, atlas)?, db, atlas, ppm)?.px;
    Some(ImageCutNotice {
        grid: aviso.grid,
        cell_px: [px[0] / g.hframes.max(1), px[1] / g.vframes.max(1)],
        animations: u32::try_from(n).unwrap_or(u32::MAX),
    })
}

/// **Cut again**: a grade que a transparência da imagem nova propõe. Sem proposta, o corte à mão: o editor abre.
pub fn cut_again(
    world: &mut World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    facts: Option<ImageFacts>,
) -> bool {
    if world.get::<SpriteCutNotice>(e).is_none() {
        return false;
    }
    world.entity_mut(e).remove::<SpriteCutNotice>();
    let proposta = shown_id(world, e, atlas)
        .and_then(|id| db.get(&id))
        .and_then(|a| {
            let (w, h, px) = a.image_rgba8()?;
            proposed_grid(&px, w, h)
        });
    match (proposta, facts) {
        (Some(grid), Some(f)) => {
            // A grelha ainda é a velha: a escala lê-se antes de `show` a trocar.
            let k = display_scale(world, e, f);
            crate::image_use::show(world, e, f, grid, k);
            crate::image_use::recut_users(world, e);
        }
        _ => world
            .get_resource_or_init::<crate::editor::AnimEditorRequests>()
            .0
            .push(e.to_bits()),
    }
    true
}

/// **Keep the cut**: o corte fica, o aviso sai.
pub fn keep_cut(world: &mut World, e: Entity) -> bool {
    let tinha = world.get::<SpriteCutNotice>(e).is_some();
    world.entity_mut(e).remove::<SpriteCutNotice>();
    tinha
}
