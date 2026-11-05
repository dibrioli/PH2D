//! O que o cartão da Sprite pergunta ao mundo e o snapshot do Inspector não responde
//! ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md)).

use crate::image_choice::library_facts;
use crate::image_use::{ImageUse, cells, image_use, proposed_grid, shown_id};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Children, Entity, SpritePixels, World};
use ph2d_render::{Sprite, SpriteSource, WHITE_TILE_KEY};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Quantas imagens o convite do cartão vazio oferece.
const QUANTOS_RECENTES: usize = 3;

thread_local! {
    /// A imagem é endereçada pelo conteúdo: a proposta de grelha de um id nunca muda.
    static FOLHAS: RefCell<BTreeMap<AssetId, Option<[u32; 2]>>> = const { RefCell::new(BTreeMap::new()) };
    /// A imagem aberta no Painter pelo botão: `(objecto, imagem)`. A próxima pintura dele vale para quem a usa.
    static EDICAO: std::cell::Cell<Option<(Entity, AssetId)>> = const { std::cell::Cell::new(None) };
    /// As imagens escolhidas nesta sessão, a última primeiro.
    static RECENTES: RefCell<Vec<AssetId>> = const { RefCell::new(Vec::new()) };
}

/// A imagem `id` acabou de ser escolhida para um objecto.
pub fn lembra(id: AssetId) {
    RECENTES.with(|r| {
        let mut r = r.borrow_mut();
        r.retain(|o| *o != id);
        r.insert(0, id);
        r.truncate(QUANTOS_RECENTES);
    });
}

/// As imagens recentes, a última primeiro.
#[must_use]
pub fn recentes() -> Vec<AssetId> {
    RECENTES.with(|r| r.borrow().clone())
}

/// O quadrado branco de quem ainda não escolheu imagem (`Add ▸ Sprite`, `Static Body`)?
#[must_use]
pub fn sem_imagem(world: &World, e: Entity) -> bool {
    world.get::<SpritePixels>(e).is_none()
        && world.get::<Sprite>(e).is_some_and(|s| {
            matches!(s.source, SpriteSource::Atlas { key } if key == WHITE_TILE_KEY)
        })
}

/// `e` tem filhos?
#[must_use]
pub fn tem_filhos(world: &World, e: Entity) -> bool {
    world.get::<Children>(e).is_some_and(|c| !c.is_empty())
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

/// A imagem pintada a partir do convite toma o lugar de `vazio`: a mesma posição, o mesmo nome, e o quadrado branco sai.
pub fn substitui_o_vazio(world: &mut World, vazio: Entity, nova: Entity) {
    if !sem_imagem(world, vazio) {
        return;
    }
    let posicao = world.get::<ph2d_ecs::Transform>(vazio).map(|t| t.translation);
    if let (Some(p), Some(mut t)) = (posicao, world.get_mut::<ph2d_ecs::Transform>(nova)) {
        t.translation = p;
    }
    world.despawn(vazio);
}

/// Os píxeis da imagem que `e` mostra: `(w, h, RGBA8 reto)`.
#[must_use]
pub fn imagem_de(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
) -> Option<(u32, u32, Vec<u8>)> {
    let asset = db.get(&shown_id(world, e, atlas)?)?;
    let (w, h, px) = asset.image_rgba8()?;
    Some((w, h, px.into_owned()))
}

/// Os objectos que mostram a imagem `id`, salvo `fora`.
fn usuarios(
    world: &mut World,
    id: AssetId,
    fora: Option<Entity>,
    atlas: &BTreeMap<u32, AssetId>,
) -> Vec<Entity> {
    let mut q = world.query::<(Entity, &Sprite, Option<&SpritePixels>)>();
    q.iter(world)
        .filter(|(e, s, px)| {
            Some(*e) != fora && ph2d_render::texture_of(*px, Some(*s), atlas) == Some(id)
        })
        .map(|(e, _, _)| e)
        .collect()
}

/// `e` passa a mostrar os píxeis `novo` (a textura `texture_id`), sem mudar de tamanho nem de uso.
fn aponta(world: &mut World, e: Entity, texture_id: u32, novo: AssetId, premultiplied: bool) {
    let mut ent = world.entity_mut(e);
    if let Some(mut s) = ent.get_mut::<Sprite>() {
        s.source = SpriteSource::Individual { texture_id };
        s.premultiplied = premultiplied;
    }
    ent.insert(SpritePixels(novo));
    // A folha empacotada à mão descrevia os píxeis antigos: no próximo load ela apagaria a edição.
    ent.remove::<ph2d_ecs::SpriteSheetRef>();
}

/// ⭐ **Os píxeis da imagem que `origem` mostra mudaram**: `rgba` (RGBA8 reto) vale para TODO objecto que usa a mesma
/// imagem. A imagem é endereçada pelo conteúdo, e por isso editar dá um id novo. Devolve quantos objectos mudaram.
pub fn troca_os_pixeis(
    world: &mut World,
    acquire: &mut dyn FnMut(u32, u32, &[u8]) -> Result<u32, String>,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    origem: Entity,
    (w, h, rgba): (u32, u32, Vec<u8>),
) -> Result<usize, String> {
    let Some(antigo) = shown_id(world, origem, atlas) else {
        return Err(ph2d_i18n::tr("object_add.not_born").to_owned());
    };
    let texture_id = acquire(w, h, &rgba)?;
    let novo = db.insert_image_rgba8(w, h, rgba);
    if novo == antigo {
        return Ok(0);
    }
    let todos = usuarios(world, antigo, None, atlas);
    for e in &todos {
        aponta(world, *e, texture_id, novo, false);
    }
    Ok(todos.len())
}

/// O botão Edit in Painter foi carregado em `e`.
pub fn marca_edicao(world: &World, e: Entity, atlas: &BTreeMap<u32, AssetId>) {
    EDICAO.with(|c| c.set(shown_id(world, e, atlas).map(|id| (e, id))));
}

/// O Painter acabou de gravar a pintura em `e`, em `texture_id`: se `e` veio do botão, os outros objectos que usavam a
/// imagem passam à pintura. Devolve quantos, além de `e`.
pub fn segue_a_pintura(
    world: &mut World,
    atlas: &BTreeMap<u32, AssetId>,
    e: Entity,
    texture_id: u32,
) -> usize {
    let Some((quem, antigo)) = EDICAO.with(std::cell::Cell::get).filter(|(q, _)| *q == e) else {
        return 0;
    };
    let Some(novo) = world.get::<SpritePixels>(e).map(|p| p.0) else {
        return 0;
    };
    let premultiplied = world.get::<Sprite>(e).is_some_and(|s| s.premultiplied);
    let outros = usuarios(world, antigo, Some(e), atlas);
    for o in &outros {
        aponta(world, *o, texture_id, novo, premultiplied);
    }
    // A pintura seguinte parte da que agora se vê.
    EDICAO.with(|c| c.set(Some((quem, novo))));
    outros.len()
}

#[cfg(test)]
#[path = "sprite_card_tests.rs"]
mod tests;
