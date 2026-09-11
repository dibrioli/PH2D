//! ⭐⭐⭐ **O QUE SE PARTE QUANDO A PEÇA NÃO TEM TAMANHO** (W140) — a medição que o
//! [plano 09](../../../docs/3DModeling/09_plano_das_dez_que_faltam.md) manda fazer **antes** de
//! escrever um `sd_plane`.
//!
//! *«Não é uma forma a construir: é a bola de recorte admitir uma peça INFINITA.»*
//!
//! ⚠️ Há **duas** perguntas, e elas não são a mesma:
//!
//! 1. **A composição já exprime um plano?** — uma caixa muito grande é um plano para todos os
//!    efeitos práticos. O que ela CUSTA é o que esta sonda mede.
//! 2. **O que faz um raio de bordo infinito** aos consumidores dele.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

fn peca_com_verbo(k: f32, op: Op) -> FieldDoc {
    let mut mover = Xform::IDENTITY;
    mover.translation = [0.0, 0.0, -0.55];
    FieldDoc::new(
        vec![
            Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Sphere { radius: 0.5 }),
            ),
            Node::new(
                mover,
                NodeKind::Leaf(Primitive::Box {
                    half: [k, k, 0.02],
                    round: 0.0,
                    chamfer: 0.0,
                }),
            ),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op,
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("o documento aceita a peça")
}

/// ⭐⭐⭐ **A PERGUNTA QUE DECIDE O PLANE: um plano que CORTA infla o bordo?**
///
/// ⚠️ *Um plano tem dois usos, e eles não são o mesmo* — ser **chão** (parte da peça) ou ser
/// **faca** (tirar-lhe metade). Se a subtracção já ignora a caixa de quem corta, então metade da
/// pergunta do plano **já está respondida pela composição**, e a outra metade é inexportável por
/// definição.
#[test]
#[ignore = "sonda: o verbo muda o bordo?"]
fn probe_whether_the_verb_changes_the_bound() {
    let reg = ph2d_field_eval::hybrid::Registry::default();
    println!("  meia-aresta | raio (UNIÃO) | raio (SUBTRACÇÃO) | raio (INTERSECÇÃO)");
    for k in [0.6_f32, 5.0, 100.0, 1000.0] {
        let raio = |op: Op| {
            ph2d_field_eval::bounds::bounding_ball(&peca_com_verbo(k, op), &reg)
                .map_or(f32::NAN, |b| b.radius)
        };
        println!(
            "  {k:>11.1} | {:12.3} | {:17.3} | {:18.3}",
            raio(Op::Union(Blend::Sharp)),
            raio(Op::Difference(Blend::Sharp)),
            raio(Op::Intersection(Blend::Sharp))
        );
    }
}

fn chao_mais_peca(k: f32) -> FieldDoc {
    let mover = |z: f32| {
        let mut x = Xform::IDENTITY;
        x.translation = [0.0, 0.0, z];
        x
    };
    FieldDoc::new(
        vec![
            Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Sphere { radius: 0.5 }),
            ),
            Node::new(
                mover(-0.55),
                NodeKind::Leaf(Primitive::Box {
                    half: [k, k, 0.02],
                    round: 0.0,
                    chamfer: 0.0,
                }),
            ),
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
    .expect("o documento aceita a peça")
}

/// ⭐⭐⭐ **O PREÇO DE UM «PLANO» FEITO DE CAIXA** — e ele não é o traçado, é a EXPORTAÇÃO.
#[test]
#[ignore = "sonda: o que um plano grande custa aos consumidores do bordo"]
fn probe_what_a_big_box_costs() {
    let reg = ph2d_field_eval::hybrid::Registry::default();
    println!("  meia-aresta do chão | raio do bordo | aresta da célula (256³) | células na esfera");
    for k in [0.6_f32, 1.0, 5.0, 25.0, 100.0, 1000.0] {
        let doc = chao_mais_peca(k);
        let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
        let passo = ph2d_field_eval::extract::cell_size(&doc, &reg, 8);
        println!(
            "  {k:>19.1} | {:13.3} | {passo:23.6} | {:17.2}",
            bola.radius,
            1.0 / passo
        );
    }
}

/// ⭐⭐ **E o que o documento faz com uma medida NÃO-FINITA** — a pergunta 2, feita à porta.
#[test]
#[ignore = "sonda: o documento aceita uma peça infinita?"]
fn probe_whether_the_document_accepts_an_infinite_piece() {
    for (nome, half) in [
        ("infinito", [f32::INFINITY, f32::INFINITY, 0.02_f32]),
        ("enorme (f32::MAX)", [f32::MAX, f32::MAX, 0.02]),
        ("1e30", [1.0e30, 1.0e30, 0.02]),
    ] {
        let r = FieldDoc::new(
            vec![Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Box {
                    half,
                    round: 0.0,
                    chamfer: 0.0,
                }),
            )],
            NodeId(0),
        );
        match r {
            Err(e) => println!("  {nome:20} RECUSADO: {e:?}"),
            Ok(doc) => {
                let reg = ph2d_field_eval::hybrid::Registry::default();
                let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg);
                let passo = ph2d_field_eval::extract::cell_size(&doc, &reg, 8);
                println!(
                    "  {nome:20} ACEITE — raio {:?}, aresta da célula {passo:?}",
                    bola.map(|b| b.radius)
                );
            }
        }
    }
}
