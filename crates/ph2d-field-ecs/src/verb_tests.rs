//! ⭐⭐⭐ **O VERBO ATRAVESSA A CENA** (W97) — os gates da metade que só o mundo sabe responder.
//!
//! A lei da dobra vive na `ph2d-field-eval`; aqui mede-se o que ela **recebe**: que a ausência do
//! componente atravesse como ausência, que a presença chegue ao documento, e que a *base* seja a
//! primeira forma que **contribui** — que não é a primeira da lista.

use super::*;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use ph2d_field::{Blend, Node, NodeId, NodeKind, Op, Primitive, Xform};

/// Uma peça com uma união e três esferas por baixo dela.
fn piece() -> (
    World,
    bevy_ecs::entity::Entity,
    Vec<bevy_ecs::entity::Entity>,
) {
    let ball = |r: f32| {
        Node::new(
            Xform::IDENTITY,
            NodeKind::Leaf(Primitive::Sphere { radius: r }),
        )
    };
    let doc = FieldDoc::new(
        vec![
            ball(0.3),
            ball(0.4),
            ball(0.5),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1), NodeId(2)],
                },
            ),
        ],
        NodeId(3),
    )
    .expect("peça");
    let mut world = World::new();
    let root = crate::spawn_doc(&mut world, &doc, "peça");
    let kids: Vec<_> = world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    assert_eq!(kids.len(), 3, "a fixtura tem de ter três formas");
    (world, root, kids)
}

/// ⭐⭐ **Sem componente, o documento sai com `None`** — e é isso que faz *ausência = herança*
/// atravessar sem se traduzir a meio.
#[test]
fn silence_in_the_world_is_silence_in_the_document() {
    let (world, root, _) = piece();
    let doc = crate::cook(&world, root).expect("peça").expect("válida");
    assert!(
        doc.nodes().iter().all(|n| n.verb.is_none()),
        "um nó sem `FieldVerb` chegou ao documento com verbo"
    );
}

/// ⭐⭐ **E o verbo escrito no mundo CHEGA ao documento**, no nó certo.
///
/// ⚠️ O gate afirma **qual** nó o recebeu, e não só que «algum» o tem: a travessia é pós-ordem e um
/// erro de índice ali poria o verbo na forma errada — a peça mudaria e nada apontaria a causa.
#[test]
fn a_verb_written_in_the_world_reaches_the_document() {
    let (mut world, root, kids) = piece();
    let corte = Op::Difference(Blend::Exact { radius: 0.07 });
    crate::set_verb(&mut world, kids[2], Some(corte)).expect("é um nó");

    let doc = crate::cook(&world, root).expect("peça").expect("válida");
    let com_verbo: Vec<_> = doc
        .nodes()
        .iter()
        .filter(|n| n.verb.is_some())
        .map(|n| (n.kind.shape(), n.verb))
        .collect();
    assert_eq!(com_verbo.len(), 1, "o verbo espalhou-se por mais de um nó");
    assert_eq!(com_verbo[0].1, Some(corte), "o verbo chegou alterado");
    assert!(
        matches!(
            &com_verbo[0].0,
            ph2d_field::NodeShape::Leaf(Primitive::Sphere { radius }) if (*radius - 0.5).abs() < 1e-6
        ),
        "o verbo pousou noutra forma que não a terceira"
    );
}

/// ⭐ **`None` devolve a forma à herança, e o componente SAI.**
///
/// ⚠️ Um valor guardado a fingir de ausente deixaria o nó a discordar do pai em silêncio no dia em
/// que o padrão do grupo mudasse — é por isso que o gate mede o componente, e não só o documento.
#[test]
fn asking_to_inherit_removes_the_component() {
    let (mut world, root, kids) = piece();
    crate::set_verb(&mut world, kids[1], Some(Op::Union(Blend::Sharp))).expect("é um nó");
    assert!(world.get::<FieldVerb>(kids[1]).is_some());

    crate::set_verb(&mut world, kids[1], None).expect("é um nó");
    assert!(
        world.get::<FieldVerb>(kids[1]).is_none(),
        "o componente ficou lá a fingir de ausente"
    );
    let doc = crate::cook(&world, root).expect("peça").expect("válida");
    assert!(doc.nodes().iter().all(|n| n.verb.is_none()));
}

/// ⭐⭐⭐ **A BASE é a primeira que CONTRIBUI — esconder a primeira PROMOVE a segunda.**
///
/// ⚠️ É a lei que uma segunda cópia da regra no painel quebraria em silêncio: a Hierarquia diria
/// `BSE` numa linha escondida e `SUB` na que de facto semeia o acumulado. A resposta sai do
/// [`crate::contributes`], que é a **mesma** função que o cozimento usa.
#[test]
fn the_base_is_the_first_that_contributes_not_the_first_in_the_list() {
    let (mut world, _, kids) = piece();
    assert_eq!(
        crate::verb_role(&world, kids[0]),
        Some(VerbRole::Base),
        "a primeira forma é a base"
    );
    assert_eq!(
        crate::verb_role(&world, kids[1]),
        Some(VerbRole::Inherited(Op::Union(Blend::Sharp))),
        "quem não se pronuncia herda o verbo do pai"
    );

    world
        .entity_mut(kids[0])
        .insert(ph2d_ecs::Visibility { hidden: true });
    assert_eq!(
        crate::verb_role(&world, kids[1]),
        Some(VerbRole::Base),
        "esconder a primeira tinha de promover a segunda a base"
    );
    assert_eq!(
        crate::verb_role(&world, kids[0]),
        None,
        "uma forma escondida não participa de receita nenhuma"
    );
}

/// ⭐ **Um GRUPO VAZIO não é a base** — ele não rende nada ao documento, e o cozimento nem o emite.
///
/// ⚠️ Sem isto, acrescentar um grupo vazio no topo da lista faria a Hierarquia selar `BSE` nele e a
/// forma que de facto semeia o acumulado aparecer a subtrair.
#[test]
fn an_empty_group_is_not_the_base() {
    let (mut world, root, kids) = piece();
    let vazio = world
        .spawn((
            ph2d_ecs::Name::new("Union"),
            FieldNode {
                shape: ph2d_field::NodeShape::Combine(Op::Union(Blend::Sharp)),
            },
            FieldPose::default(),
        ))
        .id();
    world.entity_mut(root).add_child(vazio);
    // ⚠️ E ele vai para o PRINCÍPIO da lista, que é o único sítio onde a confusão é observável.
    let ordem: Vec<_> = std::iter::once(vazio).chain(kids.iter().copied()).collect();
    for e in &ordem {
        world.entity_mut(*e).remove::<ChildOf>();
        world.entity_mut(*e).insert(ChildOf(root));
    }

    assert_eq!(
        crate::verb_role(&world, vazio),
        None,
        "um grupo vazio não participa"
    );
    assert_eq!(
        crate::verb_role(&world, kids[0]),
        Some(VerbRole::Base),
        "a base continua a ser a primeira forma que rende geometria"
    );
}

/// ⭐ **A raiz da peça não tem verbo** — não há nada acima dela com que dobrar.
#[test]
fn the_root_has_no_verb_to_choose() {
    let (world, root, _) = piece();
    assert_eq!(crate::verb_role(&world, root), None);
}

/// ⭐⭐ **A união de CENAS não herda o verbo de uma peça.**
///
/// ⚠️ Um verbo autorado dentro de uma peça fala dos **irmãos dela**; adoptado por
/// [`FieldDoc::union_all`] passaria a falar das **outras peças** — uma peça inteira a subtrair-se de
/// outra sem ninguém o ter pedido. A porta chama-se `union_all`, e a união é o contrato.
#[test]
fn joining_scenes_never_inherits_a_pieces_verb() {
    let uma = |r: f32| {
        let mut n = Node::new(
            Xform::IDENTITY,
            NodeKind::Leaf(Primitive::Sphere { radius: r }),
        );
        n.verb = Some(Op::Difference(Blend::Sharp));
        FieldDoc::new(vec![n], NodeId(0)).expect("esfera")
    };
    let cena = FieldDoc::union_all(&[uma(0.3), uma(0.4)], Blend::Sharp)
        .expect("duas peças")
        .expect("válida");
    assert!(
        cena.nodes().iter().all(|n| n.verb.is_none()),
        "uma peça adoptada pela cena trouxe o verbo dela"
    );
}
