//! ⭐⭐⭐ **AS PEÇAS DO RENDER SÃO AS QUE SE TOCAM** — `extract::extract_parts` (ordem do dono, 02/10):
//! *«objetos que se sobrepõem em operações booleanas se fundem em um só; uma peça afastada das outras
//! (não toca em ninguém) vira objeto separado»*.
//!
//! ⚠️ A separação é de **SÓLIDO**, não de superfície: a esfera oca tem duas cascas (a de fora e a da
//! cavidade) e é UMA peça; a bolinha solta dentro da cavidade é OUTRA.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::extract::{extract, extract_parts};
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_eval::leaf;

const DEPTH: u8 = 6;

fn at(x: f32, y: f32, z: f32) -> Xform {
    Xform {
        translation: [x, y, z],
        ..Xform::IDENTITY
    }
}

fn sphere(r: f32, x: Xform) -> Node {
    leaf(Primitive::Sphere { radius: r }, x)
}

fn combine(op: Op, children: Vec<u32>) -> Node {
    Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op,
            children: children.into_iter().map(NodeId).collect(),
        },
    )
}

fn parts(doc: &FieldDoc) -> Vec<ph2d_mesh::Mesh> {
    extract_parts(doc, &Registry::new(), DEPTH).expect("extrai")
}

#[test]
fn two_spheres_apart_are_two_parts() {
    let doc = FieldDoc::new(
        vec![
            sphere(0.3, at(-0.6, 0.0, 0.0)),
            sphere(0.3, at(0.6, 0.0, 0.0)),
            combine(Op::Union(Blend::Sharp), vec![0, 1]),
        ],
        NodeId(2),
    )
    .expect("doc");
    assert_eq!(parts(&doc).len(), 2);
}

#[test]
fn two_overlapping_spheres_are_one_part() {
    let doc = FieldDoc::new(
        vec![
            sphere(0.4, at(-0.2, 0.0, 0.0)),
            sphere(0.4, at(0.2, 0.0, 0.0)),
            combine(Op::Union(Blend::Sharp), vec![0, 1]),
        ],
        NodeId(2),
    )
    .expect("doc");
    assert_eq!(parts(&doc).len(), 1);
}

/// A cavidade tem UMA casca a mais e não é peça a mais; a bolinha solta dentro dela é.
#[test]
fn a_hollow_ball_with_a_loose_marble_inside_is_two_parts() {
    let doc = FieldDoc::new(
        vec![
            sphere(0.8, Xform::IDENTITY),
            sphere(0.6, Xform::IDENTITY),
            combine(Op::Difference(Blend::Sharp), vec![0, 1]),
            sphere(0.2, Xform::IDENTITY),
            combine(Op::Union(Blend::Sharp), vec![2, 3]),
        ],
        NodeId(4),
    )
    .expect("doc");
    let p = parts(&doc);
    assert_eq!(p.len(), 2, "a casca oca e a bolinha");
}

/// Um toro cortado ao meio por uma laje é DUAS peças, embora seja um nó só da árvore.
#[test]
fn a_torus_cut_by_a_slab_is_two_parts() {
    let doc = FieldDoc::new(
        vec![
            leaf(
                Primitive::Torus {
                    major: 0.6,
                    minor: 0.15,
                },
                Xform::IDENTITY,
            ),
            leaf(
                Primitive::Box {
                    half: [0.1, 1.0, 1.0],
                    round: 0.0,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            ),
            combine(Op::Difference(Blend::Sharp), vec![0, 1]),
        ],
        NodeId(2),
    )
    .expect("doc");
    assert_eq!(parts(&doc).len(), 2);
}

/// ⭐ As peças PARTEM a malha da extração inteira: nenhum vértice nem face nasce ou some.
#[test]
fn the_parts_partition_the_whole_extraction() {
    let doc = FieldDoc::new(
        vec![
            sphere(0.3, at(-0.6, 0.0, 0.0)),
            sphere(0.3, at(0.6, 0.0, 0.0)),
            sphere(0.25, at(0.0, 0.6, 0.0)),
            combine(Op::Union(Blend::Sharp), vec![0, 1, 2]),
        ],
        NodeId(3),
    )
    .expect("doc");
    let whole = extract(&doc, &Registry::new(), DEPTH).expect("extrai");
    let p = parts(&doc);
    assert_eq!(p.len(), 3);
    let verts: usize = p.iter().map(|m| m.positions().len()).sum();
    let faces: usize = p.iter().map(|m| m.faces().len()).sum();
    assert_eq!(verts, whole.positions().len());
    assert_eq!(faces, whole.faces().len());
}
