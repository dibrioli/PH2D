//! **A CENA 122 — A CAIXA QUE CRESCE NO CHÃO** (GEF02 onda 8 itens 2–3): a Height da forma de
//! colisão tem chaves na Timeline (1 m nos segundos pares, 2 m nos ímpares, por um minuto) e a física
//! corre — chaves e não o laço do transporte: com a física armada o relógio não volta ao início. Três casos do
//! `reshape_push`: sozinha no chão, no canto (chão + parede) e com uma caixa solta em cima. Nenhuma
//! afunda no chão nem na parede; a de cima sobe com ela. A curva chega ao solver por tique (8.3).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::canal::familia::FISICA;
use ph2d_ecs::canal_fisica as f;
use ph2d_ecs::{Entity, FieldKey, Name, Transform, World};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_physics_scene::spawn_floor;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::{PropKind, TimelineDoc};

/// O centro de uma caixa de 1 m pousada no chão de `spawn_floor` (topo em y = −0,8).
const POUSADA_Y: f32 = -0.3;
/// Quantos segundos de chaves: uma por segundo, 1 m nos pares e 2 m nos ímpares.
pub const SEGUNDOS: u32 = 60;

/// As chaves da Height `(s, m)`.
pub fn chaves() -> impl Iterator<Item = (f64, f32)> {
    (0..=SEGUNDOS).map(|s| (f64::from(s), if s % 2 == 0 { 1.0 } else { 2.0 }))
}

fn caixote(world: &mut World, nome: &str, x: f32) -> Entity {
    let e = world
        .spawn((
            Name::new(nome),
            Transform::from_translation(Vec2::new(x, POUSADA_Y)),
        ))
        .id();
    ph2d_physics_ecs::torna_corpo(world, e, BodyKind::Dynamic, [0.5, 0.5], false)
}

/// **A geometria da cena 122**: devolve os três caixotes que crescem.
pub fn build_grow_scene(world: &mut World) -> [Entity; 3] {
    spawn_floor(world);
    world.spawn((
        Name::new("Wall"),
        Transform::from_translation(Vec2::new(4.2, 1.2)),
        Sprite::atlas(WHITE_TILE_KEY, [0.4, 4.0], [0.40, 0.42, 0.48, 1.0]),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.2,
                half_y: 2.0,
            },
            ..Collider::default()
        },
    ));
    let so = caixote(world, "Crate Alone", -2.5);
    let topo = caixote(world, "Crate Under Box", 0.0);
    let canto = caixote(world, "Crate In Corner", 3.5);
    world.spawn((
        Name::new("Loose Box"),
        Transform::from_translation(Vec2::new(0.0, POUSADA_Y + 0.8)),
        Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], [0.95, 0.6, 0.2, 1.0]),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.3,
                half_y: 0.3,
            },
            ..Collider::default()
        },
    ));
    [so, topo, canto]
}

/// **As chaves da Height** de cada caixote, no dono do campo (a forma filha do corpo).
pub fn author_grow_tracks(world: &World, doc: &mut TimelineDoc, caixotes: [Entity; 3]) {
    // O trecho cobre todas as chaves: os 4 s de omissão congelavam a curva em 1 m.
    let fim = Some(f64::from(SEGUNDOS));
    doc.set_clip_length_override(doc.active_index(), fim);
    doc.set_scene_length(fim);
    let prop = PropKind::Field(FieldKey::new(FISICA, f::MEIO_Y, 0, 0));
    for c in caixotes {
        let dono = ph2d_physics_ecs::dono_do_campo(world, c.to_bits(), f::MEIO_Y);
        for (t, v) in chaves() {
            doc.insert_key(
                dono,
                prop,
                RationalTime::from_seconds(t),
                AnimValue::Float(v),
                Interp::Linear,
            );
        }
    }
}

/// **A cena 122** — corre sozinha, com o «Crate Alone» escolhido.
pub fn physics_smoke_grow(ctx: &mut crate::SceneCtx<'_>) {
    let caixotes = build_grow_scene(ctx.world);
    author_grow_tracks(ctx.world, &mut *ctx.timeline, caixotes);
    ctx.want.camera_center = Some([0.6, 0.8]);
    ctx.want.camera_height_world = Some(7.0);
    ctx.want.select = Some(caixotes[0].to_bits());
    println!(
        "[physics-smoke 122] THE CRATE THAT GROWS ON THE FLOOR. Three crates have keys on Height in \
         the Timeline (1 m on even seconds, 2 m on odd seconds, for one minute) and the physics runs. Watch the bottom edge \
         of each crate: it stays ON the floor line while it grows - it never sinks into it. The \
         corner crate never enters the wall. The orange box on the middle crate rides up with it \
         and does not jump off. Drag the Timeline back: the crates come back to the same size."
    );
}

#[cfg(test)]
#[test]
fn a_cena_122_poe_a_track_da_height_em_cada_caixote() {
    let mut w = World::new();
    let c = build_grow_scene(&mut w);
    let mut doc = TimelineDoc::new();
    author_grow_tracks(&w, &mut doc, c);
    let prop = PropKind::Field(FieldKey::new(FISICA, f::MEIO_Y, 0, 0));
    for e in c {
        let dono = ph2d_physics_ecs::dono_do_campo(&w, e.to_bits(), f::MEIO_Y);
        assert_ne!(dono, e.to_bits(), "a Height é da forma filha, não do corpo");
        assert!(
            doc.binding_for(dono, prop).is_some(),
            "a Height tem a track"
        );
    }
}
