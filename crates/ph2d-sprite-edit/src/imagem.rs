//! ⭐ **A imagem editada volta a todo objecto que a usa** ([ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)).
//! A imagem é endereçada pelo conteúdo: editar dá um id novo, e quem usava o antigo passa ao novo.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SpritePixels, World};
use ph2d_render::{Sprite, SpriteSource, texture_of};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

/// Sobe RGBA8 à placa e devolve a textura (o `SpriteRenderer` no app).
pub type Sobe<'a> = dyn FnMut(u32, u32, &[u8]) -> Result<u32, String> + 'a;

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

/// Dois RGBA8 retos dão o mesmo desenho?
fn mesmos_pixeis(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len()
        && a.as_chunks::<4>()
            .0
            .iter()
            .zip(b.as_chunks::<4>().0)
            .all(|(p, q)| p == q || (p[3] == 0 && q[3] == 0))
}

/// ⭐ **Os píxeis da imagem que `origem` mostra mudaram**: `rgba` (RGBA8 reto) vale para TODO objecto que usa a mesma
/// imagem. A imagem é endereçada pelo conteúdo, e por isso editar dá um id novo. Devolve quantos objectos mudaram.
pub fn troca_os_pixeis(
    world: &mut World,
    acquire: &mut Sobe<'_>,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    origem: Entity,
    (w, h, rgba): (u32, u32, Vec<u8>),
) -> Result<usize, String> {
    let Some(antigo) = shown_id(world, origem, atlas) else {
        return Err(ph2d_i18n::tr("object_add.not_born").to_owned());
    };
    // Abrir e sair sem pintar não é editar: um píxel transparente não tem cor, e o caminho pode tê-la zerado.
    let igual = db.get(&antigo).is_some_and(|a| {
        a.image_rgba8()
            .is_some_and(|(aw, ah, px)| (aw, ah) == (w, h) && mesmos_pixeis(&px, &rgba))
    });
    if igual {
        return Ok(0);
    }
    let texture_id = acquire(w, h, &rgba)?;
    let novo = db.insert_image_rgba8(w, h, rgba);
    veio_de(novo, antigo);
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

/// O artista saiu do Draw: a pintura seguinte, por outro caminho, volta a ser só do objecto.
pub fn fim_da_edicao() {
    EDICAO.with(|c| c.set(None));
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
    veio_de(novo, antigo);
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
