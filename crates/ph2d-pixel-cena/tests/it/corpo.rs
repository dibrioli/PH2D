//! P14b-2: o corpo do Pixel Sprite na física REAL (a ponte do rapier) — pousa pela slice, e a
//! slice escondida no quadro desliga a forma.

use ph2d_ecs::{Entity, Name, PixelSprite, SimWorld, SpriteAnimator, SpriteGrid, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, FormaDesligada, LockRotation, PhysicsBridge, RigidBody,
};
use ph2d_pixel_cena::fixture::{body, heroi_p14};
use ph2d_pixel_cena::{adiciona_collider_da_slice, animador_de_nascenca, deriva, deriva_formas};
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

/// A key do quadro 3 trocada por uma caixa menor, 2 px à esquerda e com o FUNDO 8 px acima (y 0..8
/// em vez de 2..16), os pés no mesmo sítio (8, 15): meia-extensão, offset e fundo mudam todos.
const BODY_3_ALTA: Rect = Rect::new(4, 0, 4, 8);

/// `(centro − pose do corpo, meia-extensão)` da caixa do RAPIER (o corpo não roda: `LockRotation`).
fn caixa_no_rapier(ponte: &PhysicsBridge, e: Entity) -> ([f32; 2], [f32; 2]) {
    let (lo, hi) = ponte.body_aabb(e).expect("corpo no rapier");
    let (x, y, _) = ponte.body_pose(e).expect("pose");
    (
        [(lo[0] + hi[0]) * 0.5 - x, (lo[1] + hi[1]) * 0.5 - y],
        [(hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5],
    )
}

fn perto(a: ([f32; 2], [f32; 2]), b: ([f32; 2], [f32; 2])) -> bool {
    let d = |p: [f32; 2], q: [f32; 2]| (p[0] - q[0]).abs().max((p[1] - q[1]).abs());
    d(a.0, b.0) < 1e-4 && d(a.1, b.1) < 1e-4
}

/// ⭐ P14 §7 `a_forma_segue_a_key_da_slice`, contra o cuboide do RAPIER: com o Play a correr, o quadro
/// mostrado (o do animador, o editor fica no 0) passa de 2 a 3 e de volta, e o colisor do solver
/// troca de caixa no lugar (`retune_body`, sem respawn); a caixa do gizmo (`Collider::local_aabb`)
/// diz o mesmo. *Mutação: `deriva_formas` com o quadro do editor em vez do `quadro_mostrado` ⇒ a
/// caixa do 3 fica a do 0.*
#[test]
fn a_forma_segue_a_key_da_slice_no_rapier() {
    let mut s = heroi_p14();
    let id = body(&s);
    let mut lista = s.slices().to_vec();
    let b = lista.iter_mut().find(|c| c.id == id).expect("body");
    let k3 = b.keys.iter_mut().find(|k| k.frame == 3).expect("key 3");
    k3.shape.bounds = BODY_3_ALTA;
    k3.shape.pivot = Some((4, 15));
    apply(&mut s, Op::Slices(lista)).expect("key 3");
    let ed = quadro(&s, 0);
    let (mut sim, e) = cena(&s, ed);
    sim.world_mut().entity_mut(e).insert(animador_de_nascenca());
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    let f = ph2d_pixel_cena::collider_da_slice(sim.world(), e, id).expect("a filha");
    let toca = |sim: &mut SimWorld, i: u32| {
        sim.world_mut()
            .get_mut::<SpriteGrid>(e)
            .expect("grelha")
            .frame = i;
        sim.world_mut()
            .get_mut::<SpriteAnimator>(e)
            .expect("animador")
            .playing = true;
    };
    // `(offset, meia-extensão)` em metros: BODY_0 8×14 centro (8, 9); BODY_3_ALTA 4×8 centro (6, 4).
    let caixa_0 = ([0.0, 6.0 / 16.0], [0.25, 7.0 / 16.0]);
    let caixa_3 = ([-2.0 / 16.0, 11.0 / 16.0], [2.0 / 16.0, 4.0 / 16.0]);
    let mut ponte = PhysicsBridge::new();
    let mut tick = 0_u64;
    let mut corre = |sim: &mut SimWorld, ponte: &mut PhysicsBridge, n: u64| {
        for _ in 0..n {
            tick += 1;
            deriva_formas(sim.world_mut(), |_| Some((&s, ed)), M);
            ponte.dispatch(sim, true, tick);
        }
    };
    let gizmo = |sim: &SimWorld| sim.world().get::<Collider>(f).expect("c").local_aabb();
    toca(&mut sim, 2);
    corre(&mut sim, &mut ponte, 10);
    let corpos = ponte.arena_body_count();
    let r2 = caixa_no_rapier(&ponte, e);
    assert!(
        perto(r2, caixa_0),
        "quadro 2: o rapier tem {r2:?}, devia {caixa_0:?}"
    );
    assert!(perto(gizmo(&sim), caixa_0), "quadro 2: o gizmo");
    toca(&mut sim, 3);
    corre(&mut sim, &mut ponte, 1);
    let r3 = caixa_no_rapier(&ponte, e);
    assert!(
        perto(r3, caixa_3),
        "quadro 3: o rapier tem {r3:?}, devia {caixa_3:?}"
    );
    assert!(perto(gizmo(&sim), caixa_3), "quadro 3: o gizmo");
    assert_eq!(
        ponte.arena_body_count(),
        corpos,
        "trocou no lugar, sem corpo novo"
    );
    let y = altura(&sim, e);
    assert!(
        y > TOPO + 0.5,
        "o corpo ainda voa (a troca foi a meio do Play): y {y}"
    );
    toca(&mut sim, 2);
    corre(&mut sim, &mut ponte, 1);
    let r = caixa_no_rapier(&ponte, e);
    assert!(perto(r, caixa_0), "de volta ao 2: {r:?}");
}
