//! Os gates da ponte ECS.

use super::*;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_field::{Node, NodeId, NodeKind, NodeShape, Primitive, Xform};

fn doc(radius: f32) -> FieldDoc {
    FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Sphere { radius }),
            mods: Vec::new(),
        }],
        NodeId(0),
    )
    .expect("esfera")
}

/// ⭐ **O gate que justifica a chave existir.** A mesma cena, entregue em ordens diferentes — como
/// uma consulta ECS de facto a devolve — tem de produzir o **mesmo documento**, byte a byte.
///
/// Sem isto, cada quadro diferiria do anterior, o snapshot veria bytes novos, e **todo quadro
/// viraria um passo de undo**. É o bug que o `canonicalize()` do shell já pagou uma vez, e a cura
/// ali foi a mesma: ordenar por CONTEÚDO/nome, nunca pela ordem de alocação.
#[test]
fn the_scene_field_does_not_depend_on_the_order_the_query_returns() {
    let a = ("alfa", doc(0.3));
    let b = ("beta", doc(0.4));
    let c = ("gama", doc(0.5));

    let one = scene_field(vec![a.clone(), b.clone(), c.clone()], Blend::Sharp)
        .expect("três objetos")
        .expect("válido");
    let two = scene_field(vec![c, a, b], Blend::Sharp)
        .expect("três objetos")
        .expect("válido");

    assert_eq!(one, two, "a ordem da consulta vazou para o documento");
}

/// Cena vazia não tem campo — e um documento vazio inventado aqui seria uma forma que ninguém pediu.
#[test]
fn an_empty_scene_has_no_field() {
    let empty: Vec<(&str, FieldDoc)> = Vec::new();
    assert!(scene_field(empty, Blend::Sharp).is_none());
}

/// Os componentes sobrevivem à ida e volta que o `WorldSnapshot` faz — se um deles não serializar,
/// o objeto some ao desfazer, e o sintoma não é um erro: é o desaparecimento.
#[test]
fn the_components_round_trip_through_serde() {
    let node = FieldNode {
        shape: NodeShape::Leaf(Primitive::Sphere { radius: 0.42 }),
    };
    let bytes = postcard::to_allocvec(&node).expect("serializa");
    assert_eq!(
        postcard::from_bytes::<FieldNode>(&bytes).expect("desserializa"),
        node
    );

    let pose = FieldPose {
        xform: Xform::at(0.1, -0.2, 0.3),
    };
    let bytes = postcard::to_allocvec(&pose).expect("serializa");
    assert_eq!(
        postcard::from_bytes::<FieldPose>(&bytes).expect("desserializa"),
        pose
    );

    let bytes = postcard::to_allocvec(&FieldObject).expect("serializa");
    assert_eq!(
        postcard::from_bytes::<FieldObject>(&bytes).expect("desserializa"),
        FieldObject
    );
}

/// ⚠️ **O nome canônico é PARTE DO FORMATO SALVO**, não um rótulo: o id do componente é derivado
/// dele (`stable_type_id`), então renomeá-lo faz todo projeto já salvo perder o objeto — em
/// silêncio, porque um id desconhecido é simplesmente ignorado ao carregar.
#[test]
fn the_canonical_name_is_pinned_because_the_saved_id_derives_from_it() {
    let mut reg = ComponentRegistry::new();
    register_field_components(&mut reg);
    for name in [
        "ph2d::field::FieldObject",
        "ph2d::field::FieldNode",
        "ph2d::field::FieldPose",
    ] {
        assert!(
            reg.get_by_name(name).is_some(),
            "o nome canônico `{name}` mudou — todo projeto salvo perde o objeto ao abrir"
        );
    }
}
