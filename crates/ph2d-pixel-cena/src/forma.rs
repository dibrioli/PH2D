//! **Pivô e forma de colisão a partir das slices** ([P14 §4 C](../../../docs/PixelArt/P14_plano.md)).
//!
//! «Add Collider» numa slice é um GESTO: a filha «Collision Shape» do corpo com [`SegueSlice`]
//! (gravado). A forma e o offset do `Collider` dela são DERIVADOS da key visível da slice no
//! [`crate::quadro_mostrado`] — `registar` grava-os no ponto neutro, logo a key a mudar não faz
//! passo de undo nem muda o ficheiro; [`deriva_formas`] reescreve-os a cada quadro.

use ph2d_ecs::{ChildOf, Component, Entity, Name, PixelSprite, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, FormaDesligada, RigidBody, forma_de_retangulo, nasce_com_forma,
};
use ph2d_pixel_core::{FrameId, SliceId, SliceShape, Sprite};
use serde::{Deserialize, Serialize};

/// A forma desta «Collision Shape» segue a slice `slice` (o [`SliceId`] cru) do Pixel Sprite pai.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegueSlice {
    pub slice: u32,
}

impl ph2d_ecs::SimComponent for SegueSlice {}

/// O nome canónico no registo.
pub const SEGUE_SLICE: &str = "ph2d::pixel::SegueSlice";

/// **A origem do objecto** no quadro `frame`, em píxeis do sprite (Y para baixo): o pivô da key
/// visível da slice do pivô, senão o centro do sprite ([`ph2d_pixel_core::slice::pivot_px`]).
#[must_use]
pub fn pivo_px(sprite: &Sprite, frame: FrameId) -> (f64, f64) {
    let pos = sprite.frame_index(frame).unwrap_or(0);
    ph2d_pixel_core::slice::pivot_px(sprite, u32::try_from(pos).unwrap_or(u32::MAX))
}

/// A forma e o offset (metros, Y para cima, relativo à origem) da key `k` no quadro `frame`.
fn collider_da_key(sprite: &Sprite, frame: FrameId, k: SliceShape, m_por_px: f64) -> Collider {
    let b = k.bounds;
    let (shape, off) = forma_de_retangulo(
        [
            f64::from(b.x),
            f64::from(b.y),
            f64::from(b.w),
            f64::from(b.h),
        ],
        pivo_px(sprite, frame),
        m_por_px,
    );
    #[allow(clippy::cast_possible_truncation)]
    let offset = [off[0] as f32, off[1] as f32];
    Collider {
        shape,
        offset,
        ..Collider::default()
    }
}

/// A forma da slice `slice` no quadro `frame` (key visível), ou `None` se escondida/inexistente:
/// a regra única do `deriva_formas` e do import do Sprite Inspector (P14e).
#[must_use]
pub(crate) fn collider_visivel(
    sprite: &Sprite,
    slice: SliceId,
    frame: FrameId,
    m_por_px: f64,
) -> Option<Collider> {
    let pos = u32::try_from(sprite.frame_index(frame)?).ok()?;
    let k = sprite.slice(slice)?.visible_at(pos)?;
    Some(collider_da_key(sprite, frame, k, m_por_px))
}

fn key_em(sprite: &Sprite, slice: SliceId, frame: FrameId) -> Option<SliceShape> {
    let pos = u32::try_from(sprite.frame_index(frame)?).ok()?;
    sprite.slice(slice)?.shape_at(pos)
}

/// ⭐ **«Add Collider» na slice `slice`** do Pixel Sprite `e`: `e` vira corpo `Dynamic` (o tipo com
/// que o *Add Body* do Game Engine nasce, `physics_apply.rs`) se ainda não é; nasce a filha forma
/// (a PRIMEIRA, logo a principal) com [`SegueSlice`], o nome da slice e a forma da key em
/// `quadro`. Só a mutação: o chamador de UI faz o commit (um gesto = um passo). `None` sem a slice.
pub fn adiciona_collider_da_slice(
    world: &mut World,
    e: Entity,
    slice: SliceId,
    sprite: &Sprite,
    quadro: FrameId,
    m_por_px: f64,
) -> Option<Entity> {
    let nome = sprite.slice(slice)?.name.clone();
    let k = key_em(sprite, slice, quadro)?;
    if world.get::<RigidBody>(e).is_none() {
        if world.get::<Collider>(e).is_none() {
            world.entity_mut(e).insert(RigidBody {
                kind: BodyKind::Dynamic,
            });
        } else {
            ph2d_physics_ecs::torna_corpo(world, e, BodyKind::Dynamic, [0.5, 0.5], false);
        }
    }
    let c = collider_da_key(sprite, quadro, k, m_por_px);
    let filha = nasce_com_forma(world, e, c.shape);
    let mut f = world.entity_mut(filha);
    if let Some(mut col) = f.get_mut::<Collider>() {
        col.offset = c.offset;
    }
    f.insert((SegueSlice { slice: slice.0 }, Name::new(nome)));
    Some(filha)
}

/// A «Collision Shape» filha de `e` que segue a slice `slice`, se já existe (o «Add Collider» não
/// duplica).
#[must_use]
pub fn collider_da_slice(world: &World, e: Entity, slice: SliceId) -> Option<Entity> {
    world
        .get::<ph2d_ecs::Children>(e)?
        .iter()
        .copied()
        .find(|&f| {
            world
                .get::<SegueSlice>(f)
                .is_some_and(|s| s.slice == slice.0)
        })
}

/// **A forma por quadro**: em cada filha com [`SegueSlice`] cujo pai é um Pixel Sprite, escreve a
/// forma e o offset da key da slice no [`crate::quadro_mostrado`] (só se mudaram, para não acordar
/// a ponte). `doc_de(chave)` dá o documento e o quadro do editor. A ponte do rapier refaz o collider
/// pelo `BodyDesc` (em repouso respawn; a meio do Play `retune_body` troca a forma no lugar).
///
/// Slice ESCONDIDA no quadro (key vazia): a forma fica a da última key visível e ganha
/// [`FormaDesligada`] (derivado, não gravado) — a ponte desliga o colisor; o `Visibility` do
/// artista não se toca. A key visível de novo tira-o.
pub fn deriva_formas<'a>(
    world: &mut World,
    doc_de: impl Fn(u32) -> Option<(&'a Sprite, FrameId)>,
    m_por_px: f64,
) {
    let mut q = world.query::<(Entity, &SegueSlice, &ChildOf, &Collider)>();
    let mut fazer = Vec::new();
    let mut ligar = Vec::new();
    for (e, segue, pai, col) in q.iter(world) {
        let p = pai.parent();
        let Some(ps) = world.get::<PixelSprite>(p) else {
            continue;
        };
        let Some((sprite, editor)) = doc_de(ps.0) else {
            continue;
        };
        let quadro = crate::quadro_mostrado(world, p, sprite, editor);
        let Some(pos) = sprite
            .frame_index(quadro)
            .and_then(|i| u32::try_from(i).ok())
        else {
            continue;
        };
        let desligada = world.get::<FormaDesligada>(e).is_some();
        let Some(k) = sprite
            .slice(SliceId(segue.slice))
            .and_then(|s| s.visible_at(pos))
        else {
            if !desligada {
                ligar.push((e, false));
            }
            continue;
        };
        if desligada {
            ligar.push((e, true));
        }
        let novo = collider_da_key(sprite, quadro, k, m_por_px);
        if novo.shape != col.shape || novo.offset != col.offset {
            fazer.push((e, novo.shape, novo.offset));
        }
    }
    for (e, shape, offset) in fazer {
        if let Some(mut c) = world.get_mut::<Collider>(e) {
            c.shape = shape;
            c.offset = offset;
        }
    }
    for (e, on) in ligar {
        if on {
            world.entity_mut(e).remove::<FormaDesligada>();
        } else {
            world.entity_mut(e).insert(FormaDesligada);
        }
    }
}
