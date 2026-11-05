//! ⭐ **Edit in Pixel** ([ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)): a imagem abre
//! num Pixel Sprite de trabalho, por cima da sprite, que se esconde: apagar um píxel tem de mostrar o que está atrás, não
//! a imagem antiga. Sair do Draw devolve a sprite e tira o objecto de trabalho. Os píxeis são da shell e do Pixel Art.

use ph2d_ecs::{Entity, PixelSprite, Transform, Visibility, World};
use std::cell::Cell;

/// Uma imagem aberta no Pixel Art.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aberta {
    /// A sprite cuja imagem está a ser editada.
    pub origem: Entity,
    /// O Pixel Sprite de trabalho.
    pub trabalho: Entity,
    /// O documento dele.
    pub key: u32,
    /// Já se viu o Draw ligado: antes disso, «não está em Draw» é só o quadro de entrada.
    entrou: bool,
    /// O olho da sprite antes de a edição a esconder.
    visivel: Option<Visibility>,
}

thread_local! {
    static ABERTA: Cell<Option<Aberta>> = const { Cell::new(None) };
}

/// Há uma edição aberta?
#[must_use]
pub fn aberta() -> Option<Aberta> {
    ABERTA.with(Cell::get)
}

/// `trabalho` (recém-nascido) toma o lugar de `origem` com o documento `key`.
pub fn abre(world: &mut World, origem: Entity, trabalho: Entity, key: u32) {
    let posicao = world.get::<Transform>(origem).copied();
    let visivel = world.get::<Visibility>(origem).copied();
    world.entity_mut(origem).insert(Visibility::hidden());
    let mut ent = world.entity_mut(trabalho);
    ent.insert(PixelSprite(key));
    if let (Some(p), Some(mut t)) = (posicao, ent.get_mut::<Transform>()) {
        t.translation = p.translation;
        t.rotation = p.rotation;
    }
    ABERTA.with(|c| {
        c.set(Some(Aberta {
            origem,
            trabalho,
            key,
            entrou: false,
            visivel,
        }));
    });
}

/// A edição aberta terminou? `em_draw` é o objecto em Draw neste quadro. Devolve-a uma vez, já com a sprite de volta.
pub fn terminou(world: &mut World, em_draw: Option<u64>) -> Option<Aberta> {
    let mut a = aberta()?;
    if em_draw == Some(a.trabalho.to_bits()) {
        a.entrou = true;
        ABERTA.with(|c| c.set(Some(a)));
        return None;
    }
    if !a.entrou {
        return None;
    }
    encerra(world)
}

/// Fecha a edição aberta agora, a qualquer momento (antes de gravar o projecto: o objecto de trabalho não é do artista).
pub fn encerra(world: &mut World) -> Option<Aberta> {
    let a = ABERTA.with(Cell::take)?;
    if let Ok(mut ent) = world.get_entity_mut(a.origem) {
        match a.visivel {
            Some(v) => {
                ent.insert(v);
            }
            None => {
                ent.remove::<Visibility>();
            }
        }
    }
    world.despawn(a.trabalho);
    Some(a)
}

#[cfg(test)]
#[path = "no_pixel_tests.rs"]
mod tests;
