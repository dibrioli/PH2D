//! O estado do Render por malha: mover não extrai, a verificação não pisca, encostar funde.

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

fn duas_bolas() -> (SimWorld, Vec<Entity>) {
    let ball = |x: f32| {
        Node::new(
            Xform::at(x, 0.0, 0.0),
            NodeKind::Leaf(Primitive::Sphere { radius: 0.3 }),
        )
    };
    let doc = FieldDoc::new(
        vec![
            ball(-0.8),
            ball(0.8),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("doc");
    let mut sim = SimWorld::new();
    crate::scene::sync_scene(&mut sim, Some(&doc), 0.0);
    let world = sim.world_mut();
    let mut q = world.query::<(Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    let us = crate::malha_render::unidades(sim.world(), root);
    (sim, us)
}

/// Chama o sincronismo até a extração em voo chegar (ou desistir aos 20 s).
fn ate_assentar(sim: &mut SimWorld, gesto: bool) {
    let t = std::time::Instant::now();
    loop {
        super::sync(sim, true, gesto);
        let voando =
            super::ESTADO.with(|c| c.borrow().as_ref().is_some_and(|e| e.em_voo.is_some()));
        if !voando || t.elapsed().as_secs() > 20 {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn geracao() -> u64 {
    super::com(|e| e.geracao).unwrap_or(0)
}

fn n_objetos() -> usize {
    super::com(|e| e.objetos.len()).unwrap_or(0)
}

#[test]
fn moving_does_not_extract_and_the_check_does_not_flicker_and_touching_fuses() {
    super::ESTADO.with(|c| *c.borrow_mut() = None);
    let (mut sim, us) = duas_bolas();
    ate_assentar(&mut sim, false);
    assert_eq!(n_objetos(), 2);
    assert_eq!(geracao(), 1);

    // 1) Durante o gesto: nada se extrai, e a matriz do objeto movido leva o deslocamento.
    ph2d_field_ecs::translate_world(sim.world_mut(), us[0], [0.0, 0.25, 0.0]);
    super::sync(&mut sim, true, true);
    let em_voo = super::ESTADO.with(|c| c.borrow().as_ref().is_some_and(|e| e.em_voo.is_some()));
    assert!(!em_voo, "um arrasto não pode lançar extração");
    let movido = super::com(|e| {
        e.objetos
            .iter()
            .zip(&e.modelos)
            .find(|(o, _)| o.unidades[0] == us[0])
            .map(|(_, m)| m[3][1])
    })
    .flatten()
    .expect("o objeto da primeira bola");
    assert!(
        (movido - 0.25).abs() < 1e-5,
        "a matriz não levou o gesto: {movido}"
    );

    // 2) Fim do gesto: a verificação corre e, com a mesma partição, NÃO troca as malhas.
    ate_assentar(&mut sim, false);
    assert_eq!(
        geracao(),
        1,
        "a verificação trocou malhas iguais — o quadro piscaria"
    );

    // 3) Encostar as duas bolas funde-as num objeto só.
    ph2d_field_ecs::translate_world(sim.world_mut(), us[0], [1.3, -0.25, 0.0]);
    ate_assentar(&mut sim, false);
    assert_eq!(n_objetos(), 1, "duas bolas sobrepostas são UM objeto");
    assert_eq!(geracao(), 2);
}

#[test]
fn leaving_the_render_drops_the_state() {
    let (mut sim, _) = duas_bolas();
    ate_assentar(&mut sim, false);
    assert!(super::com(|_| ()).is_some());
    super::sync(&mut sim, false, false);
    assert!(super::com(|_| ()).is_none());
}
