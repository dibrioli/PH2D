//! **O readback escreve ANCESTRAL ANTES de DESCENDENTE.**
//!
//! `Transform` é LOCAL e compõe com o pai (W5), então o readback converte a pose
//! de MUNDO que o solver deu para o local que o pai vigente exige. Se o pai for
//! escrito DEPOIS do filho, o filho foi convertido contra uma pose que não existe
//! mais — e o erro é exatamente o quanto o pai andou entre as duas escritas.
//!
//! ⚠️ **Reportado pelo Enio no smoke da cena 67** (*"o reset não consegue devolver
//! o conjunto à posição original"*) e **PRÉ-EXISTENTE desde o W5**: o mapa de
//! corpos é um `BTreeMap<Entity>`, e a ordem de `Entity` **não** é a de spawn —
//! medido, um par pai/filho itera **o filho primeiro**. Durante o play isso é um
//! atraso de UM frame (pequeno, e some no ruído do movimento); num REWIND o "frame
//! anterior" é o mundo inteiro antes do salto, e o filho aterrissa a **4,91 m** —
//! a distância que o pai caiu.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, world_transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

fn body(sim: &mut SimWorld, name: &str, at: Vec2, parent: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                ..Collider::default()
            },
            Transform::from_translation(at),
        ))
        .id();
    if let Some(p) = parent {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

fn world_pos(sim: &SimWorld, e: Entity) -> Vec2 {
    world_transform(sim.world(), e)
        .expect("corpo com Transform")
        .translation
}

/// Pai em `(0,3)` e filho em local `(1,0)` — mundo `(1,3)`. Os dois dinâmicos,
/// **sem joint nenhum**: o defeito é da conversão de espaço, não da restrição.
fn pair() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let p = body(&mut sim, "P", Vec2::new(0.0, 3.0), None);
    let c = body(&mut sim, "C", Vec2::new(1.0, 0.0), Some(p));
    (sim, p, c)
}

/// **O RESET devolve o filho, não só a raiz.**
///
/// Este é o gate do report: nasceu vermelho com o filho a **4,910 m** do lugar —
/// e 4,905 m é exatamente `½·g·t²` para o segundo que ele simulou, ou seja a
/// queda do PAI aparecendo dentro do local do filho.
#[test]
fn a_parented_body_returns_to_its_authored_pose_on_reset() {
    let (mut sim, p, c) = pair();
    let p0 = world_pos(&sim, p);
    let c0 = world_pos(&sim, c);

    let mut bridge = PhysicsBridge::new();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    assert!(
        (world_pos(&sim, c) - c0).length() > 1.0,
        "premissa: o filho tem de ter caído de verdade antes do Reset"
    );

    // RESET: o relógio volta ao tique 0, parado.
    bridge.dispatch(&mut sim, false, 0);

    let dp = (world_pos(&sim, p) - p0).length();
    let dc = (world_pos(&sim, c) - c0).length();
    assert!(dp < 1e-3, "a RAIZ não voltou ({dp:.4} m) — outro defeito");
    assert!(
        dc < 1e-3,
        "o FILHO voltou a {dc:.4} m da pose autorada. O readback o converteu \
         contra o pai do frame ANTERIOR (a pose desabada), então o local dele \
         absorveu a queda inteira do pai"
    );
}

/// **E durante o PLAY o filho está onde o solver disse**, no MESMO frame.
///
/// A mesma causa, no regime em que ela é pequena: escrito contra o pai do frame
/// anterior, o filho fica um frame atrás — invisível a olho, e é por isso que o
/// defeito sobreviveu ao W5 inteiro. O oráculo é o próprio solver: a pose de
/// mundo composta tem de bater com a que o corpo tem no rapier.
#[test]
fn a_parented_body_is_where_the_solver_put_it_on_every_frame() {
    let (mut sim, p, c) = pair();
    let mut bridge = PhysicsBridge::new();
    let mut worst = 0.0f32;
    for t in 1..=30u64 {
        bridge.dispatch(&mut sim, true, t);
        // O pai e o filho caem juntos, então a distância entre eles é a única
        // grandeza que o atraso de um frame pode mexer — e ela é geometria pura.
        let d = (world_pos(&sim, c) - world_pos(&sim, p)).length();
        worst = worst.max((d - 1.0).abs());
    }
    assert!(
        worst < 1e-3,
        "a distância pai↔filho variou {worst:.4} m durante a queda; os dois caem \
         com a mesma aceleração, então ela é constante — o que varia é o filho \
         sendo convertido contra o pai de um frame atrás"
    );
}

/// **Três níveis** — o neto tem de ser escrito depois do filho, que tem de ser
/// escrito depois do pai. Ordenar só *"raízes primeiro"* consertaria um nível e
/// deixaria o seguinte com o mesmo defeito.
#[test]
fn the_order_holds_for_a_chain_three_levels_deep() {
    let mut sim = SimWorld::new();
    let a = body(&mut sim, "A", Vec2::new(0.0, 5.0), None);
    let b = body(&mut sim, "B", Vec2::new(1.0, 0.0), Some(a));
    let c = body(&mut sim, "C", Vec2::new(1.0, 0.0), Some(b));
    let poses = [world_pos(&sim, a), world_pos(&sim, b), world_pos(&sim, c)];

    let mut bridge = PhysicsBridge::new();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    bridge.dispatch(&mut sim, false, 0);

    for (e, want) in [a, b, c].into_iter().zip(poses) {
        let got = world_pos(&sim, e);
        assert!(
            (got - want).length() < 1e-3,
            "nível não voltou: quer {want:?}, veio {got:?}"
        );
    }
}
