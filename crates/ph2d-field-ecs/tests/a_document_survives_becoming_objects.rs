//! ⛔⛔⛔ **UM DOCUMENTO PERDIA A PILHA E O VERBO AO VIRAR OBJETOS** — report do Enio, 2026-08-30.
//!
//! *«nada torcido»*, com a foto de três barras **idênticas** onde a do meio devia estar torcida 156°.
//!
//! # A causa, e ela não era da torção
//!
//! O `cook` lê **quatro** componentes — `FieldNode`, `FieldPose`, `FieldMods` e `FieldVerb` — e o
//! `spawn_doc` escrevia **dois**. Um documento que chegasse com modificadores via-os desaparecer no
//! instante em que virava objetos, **sem uma palavra**, e a tela mostrava a forma crua.
//!
//! ⚠️ **A casca, o afastamento, os espelhos, as matrizes e a inclinação caíam pela mesma porta desde
//! que o `FieldMods` existe.** Nenhuma cena de smoke trazia modificadores, e por isso ninguém o viu —
//! o defeito esperou a primeira cena que trouxesse.
//!
//! # ⭐ Por que o gate é uma IDA E VOLTA, e não uma lista
//!
//! Uma lista de componentes escrita aqui à mão seria a **terceira** cópia da mesma pergunta (o `cook`
//! tem uma, o `spawn_doc` tinha outra), e as três divergiriam no dia em que a quinta nascesse. O
//! ciclo `doc → spawn_doc → cook → doc` não tem lista nenhuma: ele exige que a travessia inteira não
//! perca nada, e um componente novo entra nele **de graça**.

use bevy_ecs::world::World;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Unary, UnaryKind, Xform};

/// Uma peça com **tudo o que um nó pode carregar**: pose, verbo próprio e uma pilha de dois
/// modificadores — um sem números (o espelho) e um com três (a torção).
fn peca_completa(com_mods: bool) -> FieldDoc {
    let mut a = Node::new(
        Xform::at(-0.3, 0.0, 0.0),
        NodeKind::Leaf(Primitive::Box {
            half: [0.34, 0.11, 0.6],
            round: 0.02,
            chamfer: 0.0,
        }),
    );
    let mut b = Node::new(
        Xform::at(0.3, 0.0, 0.0),
        NodeKind::Leaf(Primitive::Sphere { radius: 0.25 }),
    );
    if com_mods {
        a.mods = vec![
            Unary::MirrorZ,
            Unary::Twist {
                turns: 0.35,
                lower: -1.0,
                upper: 1.0,
                falloff: 0.0,

                axis: ph2d_field::mods::TWIST_AXIS,
            },
        ];
        b.mods = vec![Unary::Shell { thickness: 0.04 }];
    }
    // ⚠️ O verbo **do segundo** irmão: o primeiro semeia, e o dele não é perguntado.
    b.verb = Some(Op::Difference(Blend::Exact { radius: 0.05 }));
    let grupo = Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(Blend::Exact { radius: 0.08 }),
            children: vec![NodeId(0), NodeId(1)],
        },
    );
    FieldDoc::new(vec![a, b, grupo], NodeId(2)).expect("peça")
}

fn ida_e_volta(doc: &FieldDoc) -> FieldDoc {
    let mut world = World::new();
    let root = ph2d_field_ecs::spawn_doc(&mut world, doc, "peça");
    ph2d_field_ecs::cook(&world, root)
        .expect("há peça")
        .expect("válida")
}

#[test]
fn a_document_survives_becoming_objects() {
    let antes = peca_completa(true);
    let depois = ida_e_volta(&antes);
    assert_eq!(
        depois.nodes().len(),
        antes.nodes().len(),
        "a árvore mudou de tamanho na travessia"
    );
    for (i, (a, d)) in antes.nodes().iter().zip(depois.nodes()).enumerate() {
        assert_eq!(a.kind, d.kind, "nó {i}: a FORMA não sobreviveu");
        assert_eq!(a.xform, d.xform, "nó {i}: a POSE não sobreviveu");
        assert_eq!(
            a.mods, d.mods,
            "nó {i}: a PILHA DE MODIFICADORES não sobreviveu — é o defeito de 2026-08-30, e o \
             sintoma é a forma CRUA na tela, sem uma palavra"
        );
        assert_eq!(a.verb, d.verb, "nó {i}: o VERBO próprio não sobreviveu");
    }
}

/// ⛔ **O CONTROLE**: a sonda tem de saber ver uma diferença, senão ela compararia dois vazios.
#[test]
fn the_round_trip_probe_can_tell_two_documents_apart() {
    let com = peca_completa(true);
    let sem = peca_completa(false);
    assert_ne!(
        com.nodes()[0].mods,
        sem.nodes()[0].mods,
        "a sonda não distingue uma pilha cheia de uma vazia"
    );
    // E a peça SEM modificadores também tem de sobreviver — senão o gate acima passaria por acaso.
    assert_eq!(ida_e_volta(&sem).nodes()[0].mods, Vec::new());
}

/// ⭐ E a travessia aguenta **todos** os modificadores, não só os dois da fixtura.
///
/// ⚠️ Derivado do [`UnaryKind::ALL`], nunca uma lista à mão — um modificador novo entra de graça.
#[test]
fn every_modifier_kind_survives_the_round_trip() {
    for k in UnaryKind::ALL {
        let mut n = Node::new(
            Xform::IDENTITY,
            NodeKind::Leaf(Primitive::Box {
                half: [0.3, 0.2, 0.4],
                round: 0.0,
                chamfer: 0.0,
            }),
        );
        n.mods = vec![Unary::born(k, 0.3)];
        let antes = FieldDoc::new(vec![n], NodeId(0)).expect("peça");
        assert_eq!(
            antes.nodes()[0].mods,
            ida_e_volta(&antes).nodes()[0].mods,
            "{k:?} não sobreviveu à travessia"
        );
    }
}
