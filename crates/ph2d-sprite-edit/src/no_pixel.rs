//! ⭐ **Edit in Pixel** ([ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)): a imagem abre
//! num Pixel Sprite de trabalho, por cima da sprite, que se esconde: apagar um píxel tem de mostrar o que está atrás, não
//! a imagem antiga. Sair do Draw devolve a sprite e tira o objecto de trabalho. Os píxeis são da shell e do Pixel Art.

use ph2d_ecs::{Entity, PixelSprite, TiledDraw, Transform, Visibility, World};
use ph2d_render::Sprite;
use std::cell::Cell;

/// Uma imagem aberta no Pixel Art.
#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// A vista do canvas à entrada, `(centro, altura em m)`: o Pixel Art enquadra o documento dele, e sair devolve-a.
    pub vista: ([f32; 2], f32),
}

thread_local! {
    static ABERTA: Cell<Option<Aberta>> = const { Cell::new(None) };
}

/// Há uma edição aberta?
#[must_use]
pub fn aberta() -> Option<Aberta> {
    ABERTA.with(Cell::get)
}

/// A pose que põe a imagem de `w × h` px exactamente sobre o desenho de `origem`: o centro do quad, a escala do tamanho
/// no mundo (uma cópia, se repete) e o espelho. O Pixel Art pinta 1 px = `1/ppm` m a partir do centro.
fn pose_sobre(world: &World, origem: Entity, (w, h): (u32, u32), ppm: f32) -> Option<Transform> {
    let mundo = ph2d_ecs::world_transform(world, origem)?;
    let s = world.get::<Sprite>(origem)?;
    let lado = world.get::<TiledDraw>(origem).map_or(s.size, |t| t.tile);
    let a = s.resolve_anchor(ppm);
    let ancora = Transform {
        translation: ph2d_core::Vec2::new(a[0], a[1]),
        ..Default::default()
    };
    let mut t = mundo;
    t.translation = Transform::compose(mundo, ancora).translation;
    let sinal = |espelho: bool| if espelho { -1.0 } else { 1.0 };
    t.scale.x *= lado[0] * ppm / w.max(1) as f32 * sinal(s.flip_x);
    t.scale.y *= lado[1] * ppm / h.max(1) as f32 * sinal(s.flip_y);
    Some(t)
}

/// `trabalho` (recém-nascido) toma o lugar de `origem` com o documento `key`, de `tamanho` px.
pub fn abre(
    world: &mut World,
    origem: Entity,
    trabalho: Entity,
    key: u32,
    tamanho: (u32, u32),
    ppm: f32,
    vista: ([f32; 2], f32),
) {
    let pose = pose_sobre(world, origem, tamanho, ppm);
    let visivel = world.get::<Visibility>(origem).copied();
    world.entity_mut(origem).insert(Visibility::hidden());
    let mut ent = world.entity_mut(trabalho);
    ent.insert(PixelSprite(key));
    if let (Some(p), Some(mut t)) = (pose, ent.get_mut::<Transform>()) {
        *t = p;
    }
    ABERTA.with(|c| {
        c.set(Some(Aberta {
            origem,
            trabalho,
            key,
            entrou: false,
            visivel,
            vista,
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
