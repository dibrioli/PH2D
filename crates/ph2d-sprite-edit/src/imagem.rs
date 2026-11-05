//! ⭐ **A imagem editada volta a todo objecto que a usa** ([ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)).
//! A imagem é endereçada pelo conteúdo: editar dá um id novo, e quem usava o antigo passa ao novo.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SpritePixels, World};
use ph2d_render::{Sprite, SpriteSource, texture_of};
use std::cell::Cell;
use std::collections::BTreeMap;

thread_local! {
    /// A imagem aberta no Painter pelo botão: `(objecto, imagem)`. A próxima pintura dele vale para quem a usa.
    static EDICAO: Cell<Option<(Entity, AssetId)>> = const { Cell::new(None) };
}

/// A imagem da biblioteca que `e` mostra.
fn shown_id(world: &World, e: Entity, atlas: &BTreeMap<u32, AssetId>) -> Option<AssetId> {
    texture_of(world.get::<SpritePixels>(e), world.get::<Sprite>(e), atlas)
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
        .filter(|(e, s, px)| Some(*e) != fora && texture_of(*px, Some(*s), atlas) == Some(id))
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
    let Some((quem, antigo)) = EDICAO.with(Cell::get).filter(|(q, _)| *q == e) else {
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
#[path = "imagem_tests.rs"]
mod tests;
