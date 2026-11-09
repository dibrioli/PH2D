//! ⭐ **O NORMAL MAP é da IMAGEM** ([P14 §11](../../../docs/PixelArt/P14_plano.md), «Sprite: a linha Normal Map»):
//! `imagem de cor → imagem normal`, por conteúdo (blake3) — como a `CanvasTexture` da Godot e a *secondary texture*
//! da Unity. Todo sprite que mostra aquela imagem (textura própria ou célula do átlas) acende pela MESMA normal.
//!
//! A verdade é UMA: o mapa deste módulo, que viaja no `LibraryDoc` do `ProjectState` (undo e ficheiro, irmão das
//! lápides). O `SpriteRenderer` é só a cópia na placa: [`sincronizar`] a refaz a partir daqui.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SpritePixels, World};
use ph2d_render::{Sprite, SpriteRenderer, SpriteSource};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Onde a placa guarda uma normal: a textura própria de um sprite ou a célula do átlas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Alvo {
    Celula(u32),
    Textura(u32),
}

/// O que a placa sabe fazer com uma normal (o `SpriteRenderer`; os gates trazem a sua).
pub trait PlacaDeNormais {
    /// Regista (`Some((w, h, rgba))`) ou tira (`None`) a normal de `alvo`.
    ///
    /// # Errors
    /// A placa recusa (outro tamanho, alvo desconhecido).
    fn definir(&mut self, alvo: Alvo, normal: Option<(u32, u32, &[u8])>) -> Result<(), String>;
    /// `alvo` tem normal na placa? (uma textura reciclada perde-a sem avisar.)
    fn tem(&self, alvo: Alvo) -> bool;
}

impl PlacaDeNormais for SpriteRenderer {
    fn definir(&mut self, alvo: Alvo, normal: Option<(u32, u32, &[u8])>) -> Result<(), String> {
        match alvo {
            Alvo::Celula(key) => self.definir_normal_da_celula(key, normal),
            Alvo::Textura(id) => self.definir_normal(id, normal),
        }
    }
    fn tem(&self, alvo: Alvo) -> bool {
        match alvo {
            Alvo::Celula(key) => self.celula_tem_normal(key),
            Alvo::Textura(id) => self.tem_normal(id),
        }
    }
}

#[derive(Default)]
struct Estado {
    /// A verdade: imagem de cor → imagem normal.
    mapa: BTreeMap<AssetId, AssetId>,
    /// O que a placa tem agora (para [`sincronizar`] só mexer no que mudou).
    na_placa: BTreeMap<Alvo, AssetId>,
}

thread_local! {
    static NORMAIS: RefCell<Estado> = RefCell::new(Estado::default());
}

/// Os pares `(cor, normal)`, ordenados — o que o `LibraryDoc` grava (bytes iguais para o mesmo estado).
#[must_use]
pub fn normais() -> Vec<([u8; 32], [u8; 32])> {
    NORMAIS.with(|n| {
        n.borrow()
            .mapa
            .iter()
            .map(|(c, m)| (*c.as_bytes(), *m.as_bytes()))
            .collect()
    })
}

/// A porta de volta (undo, abrir projeto): o conjunto inteiro.
pub fn set_normais(pares: &[([u8; 32], [u8; 32])]) {
    NORMAIS.with(|n| {
        n.borrow_mut().mapa = pares
            .iter()
            .map(|(c, m)| (AssetId::from_digest(*c), AssetId::from_digest(*m)))
            .collect();
    });
}

/// A normal da imagem `cor`, se tem.
#[must_use]
pub fn normal_de(cor: AssetId) -> Option<AssetId> {
    NORMAIS.with(|n| n.borrow().mapa.get(&cor).copied())
}

thread_local! {
    static NOME: RefCell<Option<(AssetId, String)>> = const { RefCell::new(None) };
}

/// A normal da imagem que o sprite `e` mostra.
#[must_use]
pub fn normal_do_sprite(
    world: &World,
    atlas: &BTreeMap<u32, AssetId>,
    e: Entity,
) -> Option<AssetId> {
    normal_de(imagem_de(world, e, atlas)?)
}

/// O nome da normal `n`, para a linha **Normal Map**; `nomear` (a biblioteca inteira a ordenar) só corre quando a
/// normal muda.
pub fn nome_da_normal(n: AssetId, nomear: impl FnOnce() -> String) -> String {
    NOME.with(|c| {
        if let Some((id, nome)) = c.borrow().as_ref()
            && *id == n
        {
            return nome.clone();
        }
        let nome = nomear();
        *c.borrow_mut() = Some((n, nome.clone()));
        nome
    })
}

/// A imagem de cor que o sprite `e` mostra.
#[must_use]
pub fn imagem_de(world: &World, e: Entity, atlas: &BTreeMap<u32, AssetId>) -> Option<AssetId> {
    ph2d_render::texture_of(world.get::<SpritePixels>(e), world.get::<Sprite>(e), atlas)
}

/// ⭐ **A imagem que o sprite `e` mostra passa a ter a normal `normal`** (`None` = tira).
///
/// # Errors
/// O texto para o artista (chave i18n): `e` sem imagem, `normal` não é imagem, ou outro tamanho que a cor.
pub fn escolher_normal(
    world: &World,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    e: Entity,
    normal: Option<AssetId>,
) -> Result<(), String> {
    let erro = |k: &str| ph2d_i18n::tr(k).to_owned();
    let cor = imagem_de(world, e, atlas)
        .ok_or_else(|| erro("app.sprite_anim.image_choice.not_a_sprite"))?;
    if let Some(m) = normal {
        let medida = |id: AssetId| db.get(&id).and_then(|a| a.image_dimensions());
        let nm = medida(m).ok_or_else(|| erro("app.sprite_anim.image_choice.not_an_image"))?;
        if Some(nm) != medida(cor) {
            return Err(erro("app.sprite_anim.normal_map.other_size"));
        }
    }
    NORMAIS.with(|n| {
        let mapa = &mut n.borrow_mut().mapa;
        match normal {
            Some(m) => mapa.insert(cor, m),
            None => mapa.remove(&cor),
        }
    });
    Ok(())
}

/// ⭐ **A porta das edições da linha Normal Map** (`ImageUseEdit`). `None` = não é dela; `Some(Ok(true))` = abra o
/// seletor (**Normal Map** clicada); `Some(Ok(false))` = feito; `Some(Err(texto))` = o aviso ao artista.
#[must_use]
pub fn aplica_edicao(
    world: &World,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    entity_bits: u64,
    edit: ph2d_editor_core::ImageUseEdit,
) -> Option<Result<bool, String>> {
    let e = Entity::from_bits(entity_bits);
    Some(match edit {
        ph2d_editor_core::ImageUseEdit::LoadNormalMap => imagem_de(world, e, atlas)
            .map(|_| true)
            .ok_or_else(|| ph2d_i18n::tr("app.sprite_anim.image_choice.not_a_sprite").to_owned()),
        ph2d_editor_core::ImageUseEdit::SetNormalMap(n) => {
            escolher_normal(world, db, atlas, e, n.map(AssetId::from_digest)).map(|()| false)
        }
        _ => return None,
    })
}

/// Os alvos que a verdade pede: cada célula do átlas e cada textura própria de sprite cuja imagem tem normal.
#[must_use]
pub fn alvos(world: &mut World, atlas: &BTreeMap<u32, AssetId>) -> BTreeMap<Alvo, AssetId> {
    NORMAIS.with(|n| {
        let mapa = &n.borrow().mapa;
        let mut out = BTreeMap::new();
        if mapa.is_empty() {
            return out;
        }
        for (key, cor) in atlas {
            if let Some(m) = mapa.get(cor) {
                out.insert(Alvo::Celula(*key), *m);
            }
        }
        let mut q = world.query::<(&Sprite, &SpritePixels)>();
        for (s, p) in q.iter(world) {
            if let (SpriteSource::Individual { texture_id }, Some(m)) = (s.source, mapa.get(&p.0)) {
                out.insert(Alvo::Textura(texture_id), *m);
            }
        }
        out
    })
}

/// ⭐ **A placa segue a verdade** — uma vez por quadro; só mexe no que mudou (nada a fazer = sem custo).
pub fn sincronizar(
    world: &mut World,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    placa: &mut dyn PlacaDeNormais,
) {
    let quer = alvos(world, atlas);
    let tem = NORMAIS.with(|n| n.borrow().na_placa.clone());
    if quer.is_empty() && tem.is_empty() {
        return;
    }
    for alvo in tem.keys().filter(|a| !quer.contains_key(a)) {
        let _ = placa.definir(*alvo, None);
    }
    for (alvo, m) in &quer {
        if tem.get(alvo) == Some(m) && placa.tem(*alvo) {
            continue;
        }
        let Some(asset) = db.get(m) else { continue };
        if let Some((w, h, px)) = asset.image_rgba8()
            && let Err(e) = placa.definir(*alvo, Some((w, h, &px)))
        {
            eprintln!("[normal map] {alvo:?}: {e}");
        }
    }
    NORMAIS.with(|n| n.borrow_mut().na_placa = quer);
}

#[cfg(test)]
#[path = "normal_map_tests.rs"]
mod tests;
