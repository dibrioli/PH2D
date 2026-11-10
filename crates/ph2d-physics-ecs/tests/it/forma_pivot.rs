//! **A forma cresce em volta do PIVÔ dela** (GEF02 onda 8, dono 10/11: «cresce com o pivot da shape como centro se a
//! shape sofre a transformação»). O pivô é o `ShapeLook::pivot`, em fração da meia-extensão; o solver lê a forma pela
//! porta única `forma_resolvida`, então o cartão, a curva e o tique a tique dão o mesmo lugar.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, ShapeLook};

/// Uma parede estática (para nada cair) com a forma FILHA `1 × 1` centrada em `(0, 2)`.
fn parede() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let p = sim
        .world_mut()
        .spawn((
            Name::new("Wall"),
            Transform::from_translation(Vec2::new(0.0, 2.0)),
        ))
        .id();
    ph2d_physics_ecs::torna_corpo(sim.world_mut(), p, BodyKind::Static, [0.5, 0.5], false);
    let forma = ph2d_physics_ecs::main_shape(sim.world(), p).expect("a forma filha");
    (sim, p, forma)
}

fn meia(sim: &mut SimWorld, forma: Entity, h: [f32; 2]) {
    sim.world_mut()
        .get_mut::<Collider>(forma)
        .expect("forma")
        .shape = ColliderShape::Cuboid {
        half_x: h[0],
        half_y: h[1],
    };
}

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4
}

/// ⭐ O pivô no canto de baixo à esquerda: a forma é arrastada para que o canto caia na origem e, quando cresce de
/// `1 × 1` para `3 × 3`, o canto fica onde estava. Controlo: sem `ShapeLook` cresce em volta do centro.
#[test]
fn o_pivo_no_canto_fica_parado_quando_a_forma_cresce() {
    let corre = |pivo: Option<[f32; 2]>| {
        let (mut sim, p, forma) = parede();
        if let Some(pivot) = pivo {
            sim.world_mut().entity_mut(forma).insert(ShapeLook {
                pivot,
                ..ShapeLook::default()
            });
        }
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, true, 1);
        let antes = bridge.body_aabb(p).expect("parede");
        meia(&mut sim, forma, [1.5, 1.5]);
        bridge.dispatch(&mut sim, true, 2);
        (antes, bridge.body_aabb(p).expect("parede"))
    };
    let ((mn0, mx0), (mn1, mx1)) = corre(Some([-1.0, -1.0]));
    assert!(perto(mn0, [0.0, 2.0]), "o canto na origem: {mn0:?}");
    assert!(perto(mx0, [1.0, 3.0]), "{mx0:?}");
    assert!(perto(mn1, mn0), "o canto ficou: {mn0:?} → {mn1:?}");
    assert!(
        perto(mx1, [3.0, 5.0]),
        "cresceu para cima e à direita: {mx1:?}"
    );
    let ((c0, _), (c1, _)) = corre(None);
    assert!(perto(c0, [-0.5, 1.5]), "controlo centrado: {c0:?}");
    assert!(
        perto(c1, [-1.5, 0.5]),
        "controlo cresce em volta do centro: {c1:?}"
    );
}

/// A PEÇA (2.ª forma) com o pivô no meio da borda de cima: a perna cresce para baixo, o topo fica.
#[test]
fn a_peca_cresce_em_volta_do_pivo_dela() {
    let (mut sim, p, _) = parede();
    let perna = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, -3.0)),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.1,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            ShapeLook {
                pivot: [0.0, 1.0],
                ..ShapeLook::default()
            },
            ph2d_ecs::ChildOf(p),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 1);
    let (mn0, _) = bridge.body_aabb(p).expect("parede");
    assert!(
        perto(mn0, [-0.5, -2.0]),
        "a perna pende do pivô em y = −1: {mn0:?}"
    );
    meia(&mut sim, perna, [0.1, 1.5]);
    bridge.dispatch(&mut sim, true, 2);
    let (mn1, _) = bridge.body_aabb(p).expect("parede");
    assert!(
        perto(mn1, [-0.5, -4.0]),
        "o topo ficou e a ponta desceu: {mn1:?}"
    );
}

/// A largura pela curva, um valor por TIQUE (o `TimelineScene` do app): em cada tique a borda esquerda, o pivô,
/// continua na origem.
struct LarguraAnimada(Entity);

impl ph2d_physics_ecs::SceneAtTick for LarguraAnimada {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        let w = 0.5 + tick as f32 * 0.05;
        meia(sim, self.0, [w, 0.5]);
        true
    }
}

#[test]
fn a_largura_animada_cresce_em_volta_do_pivo_a_cada_tique() {
    let (mut sim, p, forma) = parede();
    sim.world_mut().entity_mut(forma).insert(ShapeLook {
        pivot: [-1.0, 0.0],
        ..ShapeLook::default()
    });
    let mut bridge = PhysicsBridge::new();
    for t in 1..=40 {
        bridge.dispatch_with_scene(&mut sim, true, t, &mut LarguraAnimada(forma));
        let (mn, mx) = bridge.body_aabb(p).expect("parede");
        let w = 0.5 + t as f32 * 0.05;
        assert!(
            perto(mn, [0.0, 1.5]),
            "tique {t}: a borda esquerda no pivô: {mn:?}"
        );
        assert!(perto(mx, [2.0 * w, 2.5]), "tique {t}: {mx:?}");
    }
}
