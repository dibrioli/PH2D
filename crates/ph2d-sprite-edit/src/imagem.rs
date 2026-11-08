//! ⭐ **A imagem editada volta a todo objecto que a usa** ([ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)).
//! A imagem é endereçada pelo conteúdo: editar dá um id novo, e quem usava o antigo passa ao novo.

use ph2d_asset::AssetId;
use ph2d_ecs::{Entity, SpritePixels, World};
use ph2d_render::{Sprite, SpriteSource, texture_of};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

thread_local! {
    /// A imagem aberta no Painter pelo botão: `(objecto, imagem)`. A próxima pintura dele vale para quem a usa.
    static EDICAO: Cell<Option<(Entity, AssetId)>> = const { Cell::new(None) };
    /// De que imagem cada edição veio: a edição é a mesma imagem para o artista, e herda-lhe o nome.
    static ORIGENS: RefCell<BTreeMap<AssetId, AssetId>> = const { RefCell::new(BTreeMap::new()) };
}

/// ⭐ **A imagem editada continua a ser a mesma para o artista** (dono 05/11): herda o nome da de onde veio e toma-lhe o
/// lugar, enquanto só ela estiver em uso. Sem nome conhecido na origem, fica o de quem a usa, como antes de editar.
/// A origem só vive nesta sessão: o `.ph2dproj` não grava nomes de imagem.
pub fn herda_o_nome(
    ids: &mut BTreeMap<AssetId, Option<String>>,
    live: &BTreeSet<AssetId>,
    ficheiro: impl Fn(&AssetId) -> Option<String>,
) {
    ORIGENS.with(|o| herda(ids, live, &ficheiro, &o.borrow()));
}

fn herda(
    ids: &mut BTreeMap<AssetId, Option<String>>,
    live: &BTreeSet<AssetId>,
    ficheiro: &impl Fn(&AssetId) -> Option<String>,
    origens: &BTreeMap<AssetId, AssetId>,
) {
    for novo in origens.keys().filter(|n| live.contains(n)) {
        let mut cadeia = vec![];
        let mut a = *novo;
        while let Some(&o) = origens.get(&a).filter(|o| !cadeia.contains(*o)) {
            cadeia.push(o);
            a = o;
        }
        let nome = cadeia
            .last()
            .and_then(|r| ficheiro(r).or_else(|| ids.get(r).cloned().flatten()));
        if let Some(nome) = nome {
            ids.insert(*novo, Some(nome));
        }
        for o in cadeia.iter().filter(|o| !live.contains(*o)) {
            ids.remove(o);
        }
    }
}

fn veio_de(novo: AssetId, antigo: AssetId) {
    if novo != antigo {
        ORIGENS.with(|o| {
            o.borrow_mut().entry(novo).or_insert(antigo);
        });
    }
}

/// A imagem da biblioteca que `e` mostra.
#[must_use]
pub fn shown_id(world: &World, e: Entity, atlas: &BTreeMap<u32, AssetId>) -> Option<AssetId> {
    texture_of(world.get::<SpritePixels>(e), world.get::<Sprite>(e), atlas)
}

/// A imagem da folha VISÍVEL de `e` (plano GEF 26 F13): a que o botão Edit in Painter abre.
#[must_use]
pub fn visible_id(world: &World, e: Entity, atlas: &BTreeMap<u32, AssetId>) -> Option<AssetId> {
    match ph2d_ecs::shown_sheet(world, e) {
        0 => shown_id(world, e, atlas),
        k => world
            .get::<ph2d_ecs::SpriteSheets>(e)?
            .sheet(k)
            .map(|s| s.asset),
    }
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

/// ⭐ As folhas extra que mostravam `antigo` passam a `novo` (plano GEF 26 F3/F13): a textura delas resolve-se pelo
/// nome no cache derivado, logo basta o nome.
fn aponta_folhas(world: &mut World, antigo: AssetId, novo: AssetId) {
    let mut q = world.query::<&mut ph2d_ecs::SpriteSheets>();
    for mut s in q.iter_mut(world) {
        if s.0.iter().any(|x| x.asset == antigo) {
            s.0.iter_mut()
                .filter(|x| x.asset == antigo)
                .for_each(|x| x.asset = novo);
        }
    }
}

/// Em que deu uma troca de píxeis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trocados {
    /// Quem passou a mostrar a imagem editada.
    pub quem: Vec<Entity>,
    /// A imagem de antes.
    pub antigo: AssetId,
    /// A imagem de agora.
    pub novo: AssetId,
}

/// O botão Edit in Painter foi carregado em `e`.
pub fn marca_edicao(world: &World, e: Entity, atlas: &BTreeMap<u32, AssetId>) {
    EDICAO.with(|c| c.set(visible_id(world, e, atlas).map(|id| (e, id))));
}

/// O artista saiu do Draw: a pintura seguinte, por outro caminho, volta a ser só do objecto.
pub fn fim_da_edicao() {
    EDICAO.with(|c| c.set(None));
}

/// Há uma edição marcada pelo botão Edit in Painter? (o relato do smoke)
#[must_use]
pub fn edicao_marcada() -> bool {
    EDICAO.with(Cell::get).is_some()
}

/// O Painter acabou de gravar a pintura em `e`, em `texture_id`: se `e` veio do botão, os outros objectos que usavam a
/// imagem passam à pintura. Devolve-os, além de `e`.
pub fn segue_a_pintura(
    world: &mut World,
    atlas: &BTreeMap<u32, AssetId>,
    e: Entity,
    texture_id: u32,
) -> Option<Trocados> {
    let (quem, antigo) = EDICAO.with(Cell::get).filter(|(q, _)| *q == e)?;
    let extra = ph2d_ecs::shown_sheet(world, e) > 0;
    let novo = if extra {
        visible_id(world, e, atlas)?
    } else {
        world.get::<SpritePixels>(e).map(|p| p.0)?
    };
    veio_de(novo, antigo);
    // A textura de uma folha extra sobe RETA (`ph2d_render::upload_sheet_image`).
    let premultiplied = !extra && world.get::<Sprite>(e).is_some_and(|s| s.premultiplied);
    let outros = usuarios(world, antigo, Some(e), atlas);
    for o in &outros {
        aponta(world, *o, texture_id, novo, premultiplied);
    }
    aponta_folhas(world, antigo, novo);
    // A pintura seguinte parte da que agora se vê.
    EDICAO.with(|c| c.set(Some((quem, novo))));
    Some(Trocados {
        quem: outros,
        antigo,
        novo,
    })
}

#[cfg(test)]
#[path = "imagem_tests.rs"]
mod tests;
