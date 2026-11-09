//! P14b-2: o corpo do Pixel Sprite na física REAL (a ponte do rapier) — pousa pela slice, e a
//! slice escondida no quadro desliga a forma.

use ph2d_ecs::{Entity, Name, PixelSprite, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, FormaDesligada, LockRotation, PhysicsBridge, RigidBody,
};
use ph2d_pixel_cena::fixture::{body, heroi_p14};
use ph2d_pixel_cena::{adiciona_collider_da_slice, deriva_formas};
use ph2d_pixel_core::{FrameId, Op, Rect, SliceKey, SliceShape, Sprite, apply, slice};

/// 16 px por metro.
const M: f64 = 1.0 / 16.0;
/// O topo do chão (cuboide de meia-altura 0,1 centrado em 0).
const TOPO: f32 = 0.1;

fn quadro(s: &Sprite, i: usize) -> FrameId {
    s.frames()[i].id
}

/// Chão estático + o herói (Pixel Sprite com «Add Collider» na slice `body`) a 3 m.
fn cena(s: &Sprite, q: FrameId) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: TOPO,
            },
            ..Collider::default()
        },
        Transform::IDENTITY,
    ));
    let mut t = Transform::IDENTITY;
    t.translation.y = 3.0;
    let e = w
        .spawn((t, Name::new("Heroi"), PixelSprite(1), LockRotation))
        .id();
    adiciona_collider_da_slice(w, e, body(s), s, q, M).expect("collider");
    (sim, e)
}

/// Corre 240 passos fixos (4 s) com a forma derivada a cada passo, como a costura faz.
fn corre(s: &Sprite, q: FrameId) -> (SimWorld, Entity) {
    let (mut sim, e) = cena(s, q);
    let mut ponte = PhysicsBridge::new();
    for tick in 1..=240_u64 {
        deriva_formas(sim.world_mut(), |_| Some((s, q)), M);
        ponte.dispatch(&mut sim, true, tick);
    }
    (sim, e)
}

fn altura(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).expect("t").translation.y
}

/// A altura de repouso = topo + (fundo da key − pivô)·m_por_px (Y do sprite para baixo).
fn esperada(fundo_px: f64, pivo_y_px: f64) -> f32 {
    TOPO + ((fundo_px - pivo_y_px) * M) as f32
}

#[test]
fn o_corpo_pousa_pela_slice() {
    let s = heroi_p14();
    let tol = 0.02;
    // Quadro 0: `BODY_0` (fundo 16), pés em y 15 ⇒ pousa 1 px acima do topo.
    let (sim, e) = corre(&s, quadro(&s, 0));
    let y0 = altura(&sim, e);
    let alvo = esperada(16.0, 15.0);
    assert!(
        (y0 - alvo).abs() < tol,
        "quadro 0: pousou em {y0}, devia {alvo}"
    );
    // Quadro 3: key MENOR, o mesmo fundo (16) e os mesmos pés ⇒ a mesma altura.
    let (sim, e) = corre(&s, quadro(&s, 3));
    let y3 = altura(&sim, e);
    assert!(
        (y3 - alvo).abs() < tol,
        "quadro 3 (key menor, mesmo fundo): {y3}, devia {alvo}"
    );
    // Sem pivô (o centro, y 8): a origem sobe 8 px acima do fundo.
    let mut c = s.clone();
    let op = slice::set_pivot(&c, None).expect("muda");
    apply(&mut c, op).expect("None");
    let (sim, e) = corre(&c, quadro(&c, 0));
    let yc = altura(&sim, e);
    let alvo_c = esperada(16.0, 8.0);
    assert!(
        (yc - alvo_c).abs() < tol,
        "pivô no centro: pousou em {yc}, devia {alvo_c}"
    );
}

#[test]
fn a_slice_escondida_nao_colide() {
    let mut s = heroi_p14();
    let id = body(&s);
    let mut lista = s.slices().to_vec();
    let b = lista.iter_mut().find(|c| c.id == id).expect("body");
    b.keys.push(SliceKey {
        frame: 5,
        shape: SliceShape {
            bounds: Rect::new(0, 0, 0, 0),
            center: None,
            pivot: None,
        },
    });
    apply(&mut s, Op::Slices(lista)).expect("key vazia");
    // A forma nasce no quadro 0 (visível) e corre no 5 (escondida).
    let q5 = quadro(&s, 5);
    let (mut sim, e) = cena(&s, quadro(&s, 0));
    let mut ponte = PhysicsBridge::new();
    for tick in 1..=240_u64 {
        deriva_formas(sim.world_mut(), |_| Some((&s, q5)), M);
        ponte.dispatch(&mut sim, true, tick);
    }
    let y = altura(&sim, e);
    assert!(y < -1.0, "a slice escondida ainda colide: parou em {y}");
    // E o marcador é derivado: a key visível tira-o.
    let mut q = sim.world_mut().query::<(Entity, &FormaDesligada)>();
    assert_eq!(q.iter(sim.world()).count(), 1, "uma forma desligada");
    deriva_formas(sim.world_mut(), |_| Some((&s, quadro(&s, 0))), M);
    let mut q = sim.world_mut().query::<&FormaDesligada>();
    assert_eq!(q.iter(sim.world()).count(), 0, "visível ⇒ ligada");
}
