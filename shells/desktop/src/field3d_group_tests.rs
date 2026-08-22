//! ⭐ **Os gates do GRUPO** (W31) — os dois sintomas que o Enio reportou, e o mecanismo único.
//!
//! Enio, 2026-08-22: *"ainda não temos como criar novos grupos. Se coloco um objeto como filho do
//! outro ele some."*
//!
//! ⚠️ **As duas frases são UM defeito:** no idioma do campo, **só uma operação pode ter filhos**. Uma
//! forma é uma folha — o cozimento emite-a e nunca olha para os filhos dela —, então um nó largado
//! ali fica no mundo, aparece na Hierarquia e **não entra em documento nenhum**; e a única forma de
//! aninhar era o botão de operação, que exigia **dois** selecionados. *Uma árvore que a UI aceita e a
//! linguagem não exprime é um objeto que desaparece em silêncio.*

use ph2d_ecs::SimWorld;
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

fn ball(x: f32) -> Node {
    Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
        mods: Vec::new(),
    }
}

/// Uma peça com uma **subtração**: a base em `x = 0` menos o cortador em `x = 0,3`.
fn a_difference() -> FieldDoc {
    FieldDoc::new(
        vec![
            ball(0.0),
            ball(0.3),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Difference(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
            },
        ],
        NodeId(2),
    )
    .expect("a subtração")
}

fn scene() -> (
    SimWorld,
    bevy_ecs::entity::Entity,
    Vec<bevy_ecs::entity::Entity>,
) {
    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(&mut sim, Some(&a_difference()), 0.0);
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    let leaves: Vec<bevy_ecs::entity::Entity> = ph2d_field_ecs::walk(world, root)
        .into_iter()
        .map(|(e, _)| e)
        .filter(|e| {
            matches!(
                world.get::<ph2d_field_ecs::FieldNode>(*e).map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Leaf(_))
            )
        })
        .collect();
    (sim, root, leaves)
}

fn leaves_in_doc(doc: &FieldDoc) -> usize {
    doc.nodes()
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Leaf(_)))
        .count()
}

/// Uma **união** de duas esferas — a fixture do desaparecimento.
fn a_union() -> FieldDoc {
    FieldDoc::new(
        vec![
            ball(0.0),
            ball(0.6),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
            },
        ],
        NodeId(2),
    )
    .expect("a união")
}

/// ⭐ **A peça tem matéria NESTE ponto?** — a única pergunta que responde a *"ele some"*.
///
/// ⚠️ **Contar nós na arena NÃO responde**, e a primeira versão deste gate contava: um nó que ninguém
/// referencia continua **escrito** na arena (o cozimento emite-o na subida e o pai é que o deixa de
/// fora), então a contagem passava com a cura desligada. Uma prova de mutação apanhou-o. *O que o
/// artista viu foi a TELA, e é a tela que se tem de medir.*
fn solid_at(doc: &FieldDoc, p: [f32; 3]) -> bool {
    let reg = crate::field3d_smoke::sampled_registry();
    let mut h = ph2d_field_eval::hybrid::Hybrid::new(doc, &reg);
    h.eval(&[p[0]], &[p[1]], &[p[2]]).expect("avalia")[0] < 0.0
}

/// ⭐ **O GATE-MÃE: uma forma largada em cima de outra NÃO some.**
///
/// Enio, 2026-08-22: *"Se coloco um objeto como filho do outro ele some."*
#[test]
fn a_shape_dropped_onto_a_shape_is_not_lost() {
    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(&mut sim, Some(&a_union()), 0.0);
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    let leaves: Vec<bevy_ecs::entity::Entity> = ph2d_field_ecs::walk(world, root)
        .into_iter()
        .map(|(e, _)| e)
        .filter(|e| {
            matches!(
                world.get::<ph2d_field_ecs::FieldNode>(*e).map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Leaf(_))
            )
        })
        .collect();

    let before = crate::field3d_scene::sync_scene(&mut sim, None, 0.0).expect("cozinha");
    assert!(
        solid_at(&before, [0.6, 0.0, 0.0]),
        "a fixture tem de ter matéria onde a segunda esfera está"
    );

    // O que o arrasto da Hierarquia faz: o nó passa a ser filho de uma FORMA.
    sim.world_mut()
        .entity_mut(leaves[1])
        .insert(bevy_ecs::hierarchy::ChildOf(leaves[0]));

    let after = crate::field3d_scene::sync_scene(&mut sim, None, 0.0).expect("continua a cozinhar");
    assert!(
        solid_at(&after, [0.6, 0.0, 0.0]),
        "a forma largada SUMIU da peça — uma folha não tem filhos, e o nó ficou órfão na arena"
    );
    assert!(
        solid_at(&after, [0.0, 0.0, 0.0]),
        "…e a que recebeu o filho continua lá"
    );
}

/// ⭐ **A promoção guarda a ORDEM dos irmãos** — e numa subtração a ordem é quem corta quem.
///
/// ⚠️ Sem isto, promover a **base** de uma subtração punha o grupo novo no fim da lista: o cortador
/// virava base, a peça invertia-se, e o artista veria a forma toda mudar por ter arrastado uma linha.
#[test]
fn promoting_a_host_keeps_who_cuts_whom() {
    let (mut sim, root, leaves) = scene();
    // O terceiro objeto entra COMO FILHO DA BASE — o caso do Enio, sobre a base da subtração.
    let extra = ph2d_field_ecs::add_leaf(
        sim.world_mut(),
        root,
        Primitive::Sphere { radius: 0.15 },
        [0.0, 0.4, 0.0],
    )
    .expect("nasce um terceiro");
    sim.world_mut()
        .entity_mut(extra)
        .insert(bevy_ecs::hierarchy::ChildOf(leaves[0]));

    let doc = crate::field3d_scene::sync_scene(&mut sim, None, 0.0).expect("cozinha");
    let root_node = &doc.nodes()[doc.root().0 as usize];
    let NodeKind::Combine { op, children } = &root_node.kind else {
        panic!("a raiz continua a ser a subtração");
    };
    assert!(
        matches!(op, Op::Difference(_)),
        "a operação da raiz não pode mudar"
    );
    // O primeiro filho da subtração tem de ser o GRUPO que tomou o lugar da base — nunca o cortador.
    let first = &doc.nodes()[children[0].0 as usize];
    assert!(
        matches!(first.kind, NodeKind::Combine { .. }),
        "o lugar da base é do grupo que a promoveu; ficou {:?}",
        first.kind
    );
    assert_eq!(children.len(), 2, "e a subtração continua com dois filhos");
}

/// ⭐ **Uma forma escolhida SOZINHA vira um grupo** — o gesto de criar grupo, que não existia.
#[test]
fn one_selected_shape_becomes_a_group() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, _root, leaves) = scene();
    let before = crate::field3d_scene::sync_scene(&mut sim, None, 0.0).expect("cozinha");
    let ops_before = before
        .nodes()
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Combine { .. }))
        .count();

    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::ApplyOp {
        slot: 0,
    });
    let after = crate::field3d_scene::sync_scene_and_birth(&mut sim, None, &[leaves[1]], 0.0)
        .0
        .expect("cozinha");

    assert_eq!(
        after
            .nodes()
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Combine { .. }))
            .count(),
        ops_before + 1,
        "escolher UMA forma e carregar numa operação tem de criar um grupo"
    );
    assert_eq!(
        leaves_in_doc(&after),
        leaves_in_doc(&before),
        "…sem perder nem inventar formas"
    );
}

/// ⚠️ **O CONTROLE: uma OPERAÇÃO escolhida sozinha continua a TROCAR de operação**, e não a
/// embrulhar-se noutra.
///
/// Sem esta metade, o gate acima passaria com o gesto antigo destruído — e trocar `Union` por
/// `Subtract` numa operação é o gesto mais usado do módulo.
#[test]
fn an_operation_selected_alone_still_swaps_its_op() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, root, _leaves) = scene();
    let group = ph2d_field_ecs::walk(sim.world(), root)
        .into_iter()
        .map(|(e, _)| e)
        .find(|e| {
            matches!(
                sim.world()
                    .get::<ph2d_field_ecs::FieldNode>(*e)
                    .map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Combine(_))
            )
        })
        .expect("a raiz é a subtração");
    let before = crate::field3d_scene::sync_scene(&mut sim, None, 0.0).expect("cozinha");
    let ops_before = before
        .nodes()
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Combine { .. }))
        .count();

    // O slot 0 é a UNIÃO — a operação tem de passar a ser união, sem grupo novo nenhum.
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::ApplyOp {
        slot: 0,
    });
    let after = crate::field3d_scene::sync_scene_and_birth(&mut sim, None, &[group], 0.0)
        .0
        .expect("cozinha");

    assert_eq!(
        after
            .nodes()
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Combine { .. }))
            .count(),
        ops_before,
        "uma operação escolhida sozinha NÃO ganha um grupo por cima"
    );
    let root_node = &after.nodes()[after.root().0 as usize];
    let NodeKind::Combine { op, .. } = &root_node.kind else {
        panic!("a raiz é a operação");
    };
    assert!(
        matches!(op, Op::Union(_)),
        "…e a operação dela tem de ter TROCADO para união; ficou {op:?}"
    );
}
