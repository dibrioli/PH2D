//! ⭐ **Os gates da SELEÇÃO como sujeito do gesto** (W27).
//!
//! ⚠️ **O defeito era de coerência, e é dos que o artista lê como «partido»:** a seleção deste
//! módulo é a do app — clicar na Hierarquia com `Ctrl` escolhe vários, e a fileira de operações
//! contava com isso desde a W9 —, e o arrasto do gizmo movia **um**. Duas linhas acesas, uma a
//! andar.
//!
//! Os gates dirigem `apply_motion` pela porta de produção, com o mundo de verdade.

use ph2d_ecs::SimWorld;
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

/// Três esferas irmãs sob uma união, em `x = -1, 0, +1`.
fn three_in_a_row() -> FieldDoc {
    let ball = |x: f32| Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
        mods: Vec::new(),
    };
    FieldDoc::new(
        vec![
            ball(-1.0),
            ball(0.0),
            ball(1.0),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1), NodeId(2)],
                },
                mods: Vec::new(),
            },
        ],
        NodeId(3),
    )
    .expect("três esferas")
}

/// A cena montada, e as entidades das três folhas em ordem de x.
fn scene_of_three() -> (SimWorld, Vec<bevy_ecs::entity::Entity>) {
    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(&mut sim, Some(&three_in_a_row()), 0.0);
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    let mut leaves: Vec<bevy_ecs::entity::Entity> = ph2d_field_ecs::walk(world, root)
        .into_iter()
        .map(|(e, _)| e)
        .filter(|e| {
            matches!(
                world.get::<ph2d_field_ecs::FieldNode>(*e).map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Leaf(_))
            )
        })
        .collect();
    leaves.sort_by(|a, b| {
        let (xa, xb) = (world_x(world, *a), world_x(world, *b));
        xa.partial_cmp(&xb).expect("finito")
    });
    (sim, leaves)
}

fn world_x(world: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity) -> f32 {
    ph2d_field_ecs::world_xform(world, e).translation[0]
}

fn world_pos(world: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity) -> [f32; 3] {
    ph2d_field_ecs::world_xform(world, e).translation
}

/// ⭐ **O GATE-MÃE: arrastar move TODOS os escolhidos.**
#[test]
fn a_drag_moves_every_selected_node() {
    let (mut sim, leaves) = scene_of_three();
    let before: Vec<[f32; 3]> = leaves.iter().map(|e| world_pos(sim.world(), *e)).collect();

    crate::field3d_scene::apply_motion_for_test(
        &mut sim,
        leaves[0].to_bits(),
        &[leaves[0], leaves[2]],
        crate::field3d_gizmo::Motion::Translate([0.0, 0.5, 0.0]),
    );

    let after: Vec<[f32; 3]> = leaves.iter().map(|e| world_pos(sim.world(), *e)).collect();
    assert!(
        (after[0][1] - before[0][1] - 0.5).abs() < 1e-5,
        "o principal tem de andar 0,5"
    );
    assert!(
        (after[2][1] - before[2][1] - 0.5).abs() < 1e-5,
        "e o OUTRO escolhido tem de andar o mesmo — era isto que faltava"
    );
    assert!(
        (after[1][1] - before[1][1]).abs() < 1e-6,
        "e quem não foi escolhido não se mexe"
    );
}

/// ⭐ **Um giro roda o CONJUNTO em torno do meio dele** — não cada peça sobre si mesma.
///
/// ⚠️ É a diferença entre «rodar a seleção» e «rodar N objetos ao mesmo tempo», e ela vê-se na
/// POSIÇÃO: meia volta em torno do meio troca as duas pontas de sítio.
#[test]
fn rotating_a_selection_swings_them_around_the_shared_pivot() {
    let (mut sim, leaves) = scene_of_three();
    let (left, right) = (leaves[0], leaves[2]);
    let before = (world_pos(sim.world(), left), world_pos(sim.world(), right));

    crate::field3d_scene::apply_motion_for_test(
        &mut sim,
        left.to_bits(),
        &[left, right],
        crate::field3d_gizmo::Motion::Rotate {
            axis: [0.0, 1.0, 0.0],
            angle: std::f32::consts::PI,
        },
    );

    let after = (world_pos(sim.world(), left), world_pos(sim.world(), right));
    assert!(
        (after.0[0] - before.1[0]).abs() < 1e-4,
        "meia volta em torno do meio põe a da esquerda onde estava a da direita: {} vs {}",
        after.0[0],
        before.1[0]
    );
    assert!(
        (after.1[0] - before.0[0]).abs() < 1e-4,
        "…e vice-versa: {} vs {}",
        after.1[0],
        before.0[0]
    );
}

/// ⭐ **Com UM nó escolhido, a lei é a de sempre** — o pivô é a origem dele e ele não sai do sítio.
///
/// ⚠️ É o controle que dá valor aos dois gates acima: a lei nova **contém** a antiga, e se ela
/// tivesse mudado o comportamento de um objeto só, todas as waves anteriores estariam a mentir.
#[test]
fn with_one_node_the_law_is_the_old_one() {
    let (mut sim, leaves) = scene_of_three();
    let one = leaves[2];
    let before = world_pos(sim.world(), one);

    crate::field3d_scene::apply_motion_for_test(
        &mut sim,
        one.to_bits(),
        &[one],
        crate::field3d_gizmo::Motion::Rotate {
            axis: [0.0, 1.0, 0.0],
            angle: std::f32::consts::FRAC_PI_2,
        },
    );
    let after = world_pos(sim.world(), one);
    assert!(
        after.iter().zip(before).all(|(a, b)| (a - b).abs() < 1e-6),
        "rodar UM objeto não o pode tirar do sítio: {before:?} -> {after:?}"
    );

    // …e o mesmo para o tamanho.
    crate::field3d_scene::apply_motion_for_test(
        &mut sim,
        one.to_bits(),
        &[one],
        crate::field3d_gizmo::Motion::Scale(2.0),
    );
    let after = world_pos(sim.world(), one);
    assert!(
        after.iter().zip(before).all(|(a, b)| (a - b).abs() < 1e-6),
        "escalar UM objeto também não: {before:?} -> {after:?}"
    );
}

/// ⭐ **Escalar a seleção AFASTA as peças** — o tamanho do conjunto, não de cada uma.
#[test]
fn scaling_a_selection_spreads_them_from_the_shared_pivot() {
    let (mut sim, leaves) = scene_of_three();
    let (left, right) = (leaves[0], leaves[2]);

    crate::field3d_scene::apply_motion_for_test(
        &mut sim,
        left.to_bits(),
        &[left, right],
        crate::field3d_gizmo::Motion::Scale(2.0),
    );

    assert!(
        (world_x(sim.world(), left) + 2.0).abs() < 1e-4,
        "a −1 com pivô em 0 e fator 2 tem de ir para −2; foi para {}",
        world_x(sim.world(), left)
    );
    assert!(
        (world_x(sim.world(), right) - 2.0).abs() < 1e-4,
        "e a +1 para +2; foi para {}",
        world_x(sim.world(), right)
    );
}

/// ⭐ **Um filho de outro escolhido NÃO anda duas vezes.**
///
/// ⚠️ O defeito clássico de mover uma seleção: com o grupo e uma peça dele ambos acesos, a peça
/// recebe o gesto **e** herda o do grupo pela hierarquia — anda o dobro, e só ela. Um artista que
/// escolhe um grupo e algo lá dentro não está a pedir isso.
#[test]
fn a_child_of_a_selected_node_does_not_move_twice() {
    let (mut sim, leaves) = scene_of_three();
    let world = sim.world_mut();
    let group = world
        .get::<bevy_ecs::hierarchy::ChildOf>(leaves[0])
        .map(|c| c.0)
        .expect("as folhas estão sob a união");
    let before = world_pos(sim.world(), leaves[0]);

    crate::field3d_scene::apply_motion_for_test(
        &mut sim,
        group.to_bits(),
        &[group, leaves[0]],
        crate::field3d_gizmo::Motion::Translate([0.0, 0.0, 0.25]),
    );

    let after = world_pos(sim.world(), leaves[0]);
    assert!(
        (after[2] - before[2] - 0.25).abs() < 1e-5,
        "o filho tem de andar 0,25 (a herança do pai), e não 0,5: andou {}",
        after[2] - before[2]
    );
}

/// ⭐ **O PIVÔ SOBREVIVE AO GESTO QUE ELE APLICA** — e é isto que torna seguro recalculá-lo.
///
/// ⚠️ **A pergunta que este gate responde é de desenho, não de aritmética.** O pivô é recalculado a
/// cada quadro a partir das origens (uma função, [`super::selection_pivot`], usada tanto pelo gizmo
/// como pelo gesto) — mas o gesto **move as origens**. Se o giro do quadro 2 usasse um pivô diferente
/// do quadro 1, um arrasto contínuo descreveria uma espiral em vez de um arco, e o defeito só
/// apareceria num gesto longo.
///
/// A propriedade que salva: **rodar e escalar em torno do centróide preservam o centróide**. Ela é
/// matemática, mas a implementação pode partir-se — e é por isso que ela é medida aqui em vez de
/// escrita num comentário.
#[test]
fn the_pivot_survives_the_motion_it_applies() {
    let (mut sim, leaves) = scene_of_three();
    let sel = [leaves[0], leaves[1], leaves[2]];
    let before = super::selection_pivot(sim.world(), &sel);

    for motion in [
        crate::field3d_gizmo::Motion::Rotate {
            axis: [0.0, 1.0, 0.0],
            angle: 0.6458,
        },
        crate::field3d_gizmo::Motion::Scale(1.7),
        crate::field3d_gizmo::Motion::Rotate {
            axis: [0.3, 0.7, 0.2],
            angle: -1.2,
        },
    ] {
        crate::field3d_scene::apply_motion_for_test(&mut sim, sel[0].to_bits(), &sel, motion);
        let now = super::selection_pivot(sim.world(), &sel);
        assert!(
            now.iter().zip(before).all(|(a, b)| (a - b).abs() < 1e-4),
            "o pivô andou com o gesto ({motion:?}): {before:?} -> {now:?} — um arrasto contínuo \
             descreveria uma espiral"
        );
    }
}

/// ⭐ **O gizmo pousa no MEIO da seleção** — e é o mesmo ponto que o gesto usa.
///
/// ⚠️ Duas contas de *"onde está o pivô"* — uma para desenhar, outra para aplicar — divergiriam, e o
/// sintoma seria o pior possível: a peça a girar em torno de um ponto que **não é** aquele onde as
/// argolas estão desenhadas.
#[test]
fn the_gizmo_sits_at_the_middle_of_the_selection() {
    let (mut sim, leaves) = scene_of_three();
    let anchor = super::anchor_for(&mut sim, Some(leaves[0].to_bits()), &[leaves[0], leaves[2]])
        .expect("há gizmo");
    assert!(
        anchor.origin[0].abs() < 1e-5,
        "entre −1 e +1 o gizmo tem de pousar em 0; pousou em {}",
        anchor.origin[0]
    );
    assert_eq!(
        anchor.entity,
        leaves[0].to_bits(),
        "e a identidade continua a ser a do PRINCIPAL — é ele que o gesto congela"
    );

    // ⚠️ O CONTROLE: com um só escolhido, o gizmo continua em cima dele.
    let one = super::anchor_for(&mut sim, Some(leaves[2].to_bits()), &[leaves[2]]).expect("gizmo");
    assert!(
        (one.origin[0] - 1.0).abs() < 1e-5,
        "com um só, o gizmo fica na origem dele; ficou em {}",
        one.origin[0]
    );
}

/// **`top_level` guarda a ordem da entrada** — quem chama depende dela para saber quem é o principal.
#[test]
fn the_top_of_each_branch_keeps_the_order_it_came_in() {
    let (mut sim, leaves) = scene_of_three();
    let world = sim.world_mut();
    let group = world
        .get::<bevy_ecs::hierarchy::ChildOf>(leaves[0])
        .map(|c| c.0)
        .expect("as folhas estão sob a união");
    assert_eq!(
        ph2d_field_ecs::top_level(world, &[leaves[2], leaves[0], leaves[1]]),
        vec![leaves[2], leaves[0], leaves[1]],
        "sem parentesco entre eles, a lista sai como entrou"
    );
    assert_eq!(
        ph2d_field_ecs::top_level(world, &[leaves[1], group, leaves[0]]),
        vec![group],
        "com o grupo na lista, os filhos dele saem"
    );
}
