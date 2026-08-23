//! ⭐ **Os gates do ISOLAR** (W38) — e das duas vozes que faltavam.
//!
//! ⚠️ **A lei foi LIDA, não decidida:** o módulo irmão (`sculpt3d_objects::toggle_isolate`) já a
//! tinha escrita — toggle, e **nada entra na história**. O que esta linha acrescentou foi o
//! mecanismo, e ele acabou por não ser mecanismo nenhum: o `cook` já dizia *"coze a **subárvore** de
//! `root`"*, então isolar **é** cozer a partir daquele nó.

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

/// `A ∪ (B − C)`, com o **grupo interno deslocado** — é o deslocamento que prova a pose da cadeia.
fn nested() -> FieldDoc {
    let ball = |x: f32, r: f32| Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: r }),
        mods: Vec::new(),
    };
    FieldDoc::new(
        vec![
            ball(0.0, 0.25),
            ball(0.0, 0.2),
            ball(0.15, 0.15),
            Node {
                xform: Xform::at(1.2, -0.4, 0.3),
                kind: NodeKind::Combine {
                    op: Op::Difference(Blend::Sharp),
                    children: vec![NodeId(1), NodeId(2)],
                },
                mods: Vec::new(),
            },
            Node {
                // ⚠️ **A RAIZ também sai da identidade, e isto é o gate a existir.** Com a raiz na
                // identidade, a pose LOCAL do grupo interno e a de MUNDO coincidem — e uma prova de
                // mutação que apagasse a composição da cadeia passava **verde**. *A fixture que
                // concorda é a que não prova nada.*
                xform: Xform::at(0.5, 0.25, -0.1),
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(3)],
                },
                mods: Vec::new(),
            },
        ],
        NodeId(4),
    )
    .expect("o aninhado")
}

fn scene() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(&mut sim, Some(&nested()), 0.0);
    let world = sim.world_mut();
    let mut q = world.query::<(Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    (sim, root)
}

fn inner_group(sim: &SimWorld, root: Entity) -> Entity {
    ph2d_field_ecs::walk(sim.world(), root)
        .into_iter()
        .map(|(e, _)| e)
        .find(|e| {
            matches!(
                sim.world()
                    .get::<ph2d_field_ecs::FieldNode>(*e)
                    .map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Combine(_))
            ) && sim
                .world()
                .get::<bevy_ecs::hierarchy::ChildOf>(*e)
                .is_some()
        })
        .expect("o grupo interno")
}

fn leaves(doc: &FieldDoc) -> usize {
    doc.nodes()
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Leaf(_)))
        .count()
}

/// ⭐ **O GATE-MÃE: cozer a partir do nó isolado mostra SÓ aquela subárvore.**
#[test]
fn isolating_shows_only_that_subtree_and_the_whole_part_comes_back() {
    let (mut sim, root) = scene();
    let group = inner_group(&sim, root);
    let whole = crate::field3d_scene::sync_scene(&mut sim, None, 0.0).expect("cozinha");
    assert_eq!(leaves(&whole), 3, "a peça inteira tem três formas");

    let from = crate::field3d_scene::cook_root(sim.world(), root, Some(group.to_bits()));
    let only = ph2d_field_ecs::cook(sim.world(), from)
        .expect("não vazia")
        .expect("válida");
    assert_eq!(
        leaves(&only),
        2,
        "isolado, só as duas formas DENTRO do grupo entram — a terceira é irmã e fica de fora"
    );

    // E sem isolamento nenhum, a peça inteira — byte a byte.
    let back = ph2d_field_ecs::cook(
        sim.world(),
        crate::field3d_scene::cook_root(sim.world(), root, None),
    )
    .expect("não vazia")
    .expect("válida");
    assert_eq!(back, whole, "a peça volta EXACTAMENTE como estava");
}

/// ⭐ **A peça isolada NÃO SALTA** — a metade que a linha nova do `cook` existe para cumprir.
///
/// ⚠️ Sem a pose da cadeia, o grupo interno (deslocado `1,2 · −0,4 · 0,3`) apareceria na origem, e
/// da cadeira isso lê como *"isolar mexeu no meu modelo"*.
#[test]
fn the_isolated_piece_stays_where_it_was() {
    let (sim, root) = scene();
    let group = inner_group(&sim, root);
    let from = crate::field3d_scene::cook_root(sim.world(), root, Some(group.to_bits()));
    let only = ph2d_field_ecs::cook(sim.world(), from)
        .expect("não vazia")
        .expect("válida");
    let top = &only.nodes()[only.root().0 as usize];
    let expected = ph2d_field_ecs::world_xform(sim.world(), group);
    for k in 0..3 {
        assert!(
            (top.xform.translation[k] - expected.translation[k]).abs() < 1e-6,
            "o eixo {k} saiu em {} e o nó está em {} no mundo",
            top.xform.translation[k],
            expected.translation[k]
        );
    }
    // ⭐ **O CONTROLE DA FIXTURE**, e ele foi escrito depois de uma prova de mutação passar verde:
    // se a pose LOCAL do nó e a de MUNDO coincidirem, apagar a composição da cadeia não muda nada e
    // este gate deixa de medir o que diz medir.
    let local = sim
        .world()
        .get::<ph2d_field_ecs::FieldPose>(group)
        .expect("o grupo tem pose")
        .xform;
    assert!(
        (local.translation[0] - expected.translation[0]).abs() > 1e-3,
        "a fixture TEM de ter a pose local ({}) diferente da de mundo ({}) — senão a composição \
         da cadeia é indistinguível de não haver composição nenhuma",
        local.translation[0],
        expected.translation[0]
    );
}

/// ⭐ **UM ISOLAMENTO PENDURADO NUMA ENTIDADE MORTA é LARGADO** — e a peça inteira volta.
///
/// ⚠️ Os bits de entidade morrem num undo (o restore respawna tudo com ids novos). Obedecer a um
/// alvo que já não existe apagaria a peça da tela **sem nada a explicar** — o modo de falha exacto
/// que este módulo já pagou cinco vezes com outro nome.
#[test]
fn an_isolation_pinned_to_a_dead_object_is_dropped() {
    let (mut sim, root) = scene();
    let group = inner_group(&sim, root);
    let bits = group.to_bits();
    assert!(ph2d_field_ecs::remove(sim.world_mut(), group), "o nó some");

    let from = crate::field3d_scene::cook_root(sim.world(), root, Some(bits));
    assert_eq!(
        from, root,
        "com o alvo morto, o cozimento tem de voltar à peça inteira"
    );
}

/// ⭐ **A LEI DO TOGGLE**, dirigida sem estado nenhum — ver [`crate::field3d_smoke::next_isolation`].
#[test]
fn the_isolation_law_toggles_swaps_and_refuses_nothing() {
    use crate::field3d_smoke::next_isolation;
    // Isolar «nada» é recusado: apagaria a cena sem nada para devolver.
    assert_eq!(next_isolation(None, None), None);
    assert_eq!(
        next_isolation(Some(7), None),
        Some(7),
        "e não sai por engano"
    );
    // Entra.
    assert_eq!(next_isolation(None, Some(7)), Some(7));
    // A MESMA porta sai — não há um «sair» separado.
    assert_eq!(next_isolation(Some(7), Some(7)), None);
    // Isolar outro TROCA: o gesto é "mostra-me este", e sair primeiro seriam dois gestos.
    assert_eq!(next_isolation(Some(7), Some(9)), Some(9));
}

/// ⭐ **ALGUÉM DIZ QUE O GRUPO NASCEU** (W38, a outra metade).
///
/// ⚠️ A W31 fez o gesto e deixou-o **mudo**: a Hierarquia ganha uma linha nova, o objeto escolhido
/// passa a estar um nível abaixo, e nada na tela explica porquê. O aviso diz **quantos** entraram,
/// que é o que distingue *"criei um grupo com esta forma"* de *"embrulhei as três"*.
#[test]
fn a_born_group_says_so() {
    let _ = ph2d_panel_model3d::drain_intents();
    crate::field3d_notice::clear();
    let _ = crate::field3d_notice::drain();

    let (mut sim, root) = scene();
    let leaf = ph2d_field_ecs::walk(sim.world(), root)
        .into_iter()
        .map(|(e, _)| e)
        .find(|e| {
            matches!(
                sim.world()
                    .get::<ph2d_field_ecs::FieldNode>(*e)
                    .map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Leaf(_))
            ) && sim
                .world()
                .get::<bevy_ecs::hierarchy::ChildOf>(*e)
                .is_some_and(|c| c.0 == root)
        })
        .expect("uma folha filha da raiz");

    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::ApplyOp {
        slot: 0,
    });
    crate::field3d_scene::sync_scene_and_birth(&mut sim, None, &[leaf], 0.0);

    let said = crate::field3d_notice::drain();
    assert!(
        said.iter().any(|m| m.contains("Group created")),
        "criar um grupo tem de DIZER que ele nasceu; o módulo disse {said:?}"
    );
}
