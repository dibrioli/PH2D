//! Os gates do bordo (W33).
//!
//! ⚠️ **Todos medem a mesma coisa por dois lados**: o bordo tem de **conter** a peça (senão a
//! exportação corta) e não deve ser absurdamente maior do que ela (senão a grade desperdiça
//! resolução). O primeiro é uma lei; o segundo é uma medição com folga.

use super::*;
use ph2d_field::{Blend, Node, NodeId, NodeKind, Primitive, Xform};

/// O campo, avaliado num ponto — a única forma honesta de perguntar *"há peça aqui?"*.
fn inside(doc: &FieldDoc, p: [f32; 3]) -> bool {
    let reg = crate::hybrid::Registry::new();
    let mut h = crate::hybrid::Hybrid::new(doc, &reg);
    h.eval(&[p[0]], &[p[1]], &[p[2]]).expect("avalia")[0] < 0.0
}

/// ⭐ **O bordo CONTÉM a peça** — varrido, não afirmado.
///
/// ⚠️ O gate procura um contra-exemplo: qualquer ponto **fora** da esfera onde o campo diga *dentro*
/// é a exportação a cortar. A varredura é grosseira de propósito — o que interessa não é a
/// precisão do bordo, é ele nunca mentir.
#[test]
fn the_ball_contains_every_point_the_field_calls_solid() {
    let reg = crate::hybrid::Registry::new();
    let docs = [
        // Uma peça LONGE da origem — o caso que a caixa fixa `[-1,1]` cortava.
        FieldDoc::new(
            vec![Node {
                xform: Xform::at(2.5, -1.0, 0.5),
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.4 }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("esfera longe"),
        // Um grupo girado e escalado, com dois filhos afastados.
        FieldDoc::new(
            vec![
                Node {
                    xform: Xform::at(-0.8, 0.0, 0.0),
                    kind: NodeKind::Leaf(Primitive::Box {
                        half: [0.3, 0.2, 0.25],
                        round: 0.05,
                        chamfer: 0.0,
                    }),
                    mods: Vec::new(),
                    verb: None,
                },
                Node {
                    xform: Xform::at(0.9, 0.3, -0.2),
                    kind: NodeKind::Leaf(Primitive::Cylinder {
                        radius: 0.25,
                        half_height: 0.5,
                        round: 0.0,
                        chamfer: 0.0,
                    }),
                    mods: Vec::new(),
                    verb: None,
                },
                Node {
                    xform: Xform {
                        translation: [0.4, 0.2, -0.3],
                        rotation: ph2d_field::xform::quat_from_euler([0.5, -0.9, 0.3]),
                        scale: 1.6,
                    },
                    kind: NodeKind::Combine {
                        op: ph2d_field::Op::Union(Blend::Sharp),
                        children: vec![NodeId(0), NodeId(1)],
                    },
                    mods: Vec::new(),
                    verb: None,
                },
            ],
            NodeId(2),
        )
        .expect("grupo girado"),
        // Com modificadores que CRESCEM a peça: casca, afastamento e matriz.
        FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.3 }),
                mods: vec![
                    ph2d_field::Unary::Offset { distance: 0.2 },
                    ph2d_field::Unary::Array {
                        count: 4,
                        spacing: 0.9,
                        joint: ph2d_field::Joint::SHARP,
                    },
                ],
                verb: None,
            }],
            NodeId(0),
        )
        .expect("matriz afastada"),
    ];

    for (k, doc) in docs.iter().enumerate() {
        let ball = bounding_ball(doc, &reg).expect("a peça tem bordo");
        let mut solid = 0usize;
        // Uma varredura grosseira sobre uma caixa MAIOR que o bordo — se houver peça lá fora, aqui
        // se vê.
        let span = ball.radius * 2.5 + 1.0;
        const N: i32 = 24;
        for i in 0..=N {
            for j in 0..=N {
                for l in 0..=N {
                    let f = |t: i32| (f32::from(t as i16) / f32::from(N as i16)) * 2.0 - 1.0;
                    let p = [
                        ball.center[0] + f(i) * span,
                        ball.center[1] + f(j) * span,
                        ball.center[2] + f(l) * span,
                    ];
                    if !inside(doc, p) {
                        continue;
                    }
                    solid += 1;
                    let d = [
                        p[0] - ball.center[0],
                        p[1] - ball.center[1],
                        p[2] - ball.center[2],
                    ];
                    let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    assert!(
                        dist <= ball.radius + 1e-3,
                        "peça {k}: há matéria a {dist:.3} do centro e o bordo diz {:.3} — a \
                         exportação cortaria isto",
                        ball.radius
                    );
                }
            }
        }
        assert!(solid > 0, "peça {k}: a varredura tem de encontrar matéria");
    }
}

/// ⚠️ **E o bordo não pode ser absurdo** — senão a grade gasta a resolução em vazio.
///
/// A barra é folgada de propósito (**4× o raio da peça**): o que se está a impedir é o bordo crescer
/// por composição — três agrupamentos girados com uma caixa re-envolvida a cada nível.
#[test]
fn the_ball_is_not_absurdly_bigger_than_the_piece() {
    let reg = crate::hybrid::Registry::new();
    let doc = FieldDoc::new(
        vec![
            Node {
                xform: Xform::at(0.2, 0.0, 0.0),
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.25 }),
                mods: Vec::new(),
                verb: None,
            },
            Node {
                xform: Xform {
                    translation: [0.0; 3],
                    rotation: ph2d_field::xform::quat_from_euler([0.7, 0.4, -1.1]),
                    scale: 1.0,
                },
                kind: NodeKind::Combine {
                    op: ph2d_field::Op::Union(Blend::Sharp),
                    children: vec![NodeId(0)],
                },
                mods: Vec::new(),
                verb: None,
            },
            Node {
                xform: Xform {
                    translation: [0.0; 3],
                    rotation: ph2d_field::xform::quat_from_euler([-0.3, 1.2, 0.6]),
                    scale: 1.0,
                },
                kind: NodeKind::Combine {
                    op: ph2d_field::Op::Union(Blend::Sharp),
                    children: vec![NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("três níveis girados");
    let ball = bounding_ball(&doc, &reg).expect("bordo");
    assert!(
        ball.radius < 4.0 * 0.25,
        "o bordo cresceu com a composição: {:.3} para uma esfera de 0,25",
        ball.radius
    );
}

/// **Uma subtração não cresce com o cortador** — o que se corta não acrescenta matéria.
#[test]
fn a_cutter_does_not_grow_the_piece() {
    let reg = crate::hybrid::Registry::new();
    let doc = FieldDoc::new(
        vec![
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.3 }),
                mods: Vec::new(),
                verb: None,
            },
            // Um cortador ENORME e longe — ele não pode inflar a caixa da peça.
            Node {
                xform: Xform::at(9.0, 0.0, 0.0),
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 5.0 }),
                mods: Vec::new(),
                verb: None,
            },
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: ph2d_field::Op::Difference(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("a subtração");
    let ball = bounding_ball(&doc, &reg).expect("bordo");
    assert!(
        ball.radius < 0.35,
        "o cortador inflou a peça: raio {:.3}",
        ball.radius
    );
}
