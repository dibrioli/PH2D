//! Gates for `source.object` (doc 86 §2).

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::Graph;

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == MANIFEST.id).then_some(&SourceObject as &dyn NodeOp)
    }
}

/// A sprite tile as the membrane would publish it: one instance at the origin
/// carrying the appearance (`texture_id`, `uv_rect`, `size`, `tint`).
fn tile(texture_id: f32) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![[2.0, 3.0]]))
        .with("tint", Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0]]))
        .with("uv_rect", Column::Vec4(vec![[0.1, 0.2, 0.3, 0.4]]))
        .with("texture_id", Column::Scalar(vec![texture_id]))
}

/// Publish `stream` under `published`, set the node's `object` to `named`, cook,
/// and hand back the output stream.
fn source(published: &str, named: &str, stream: Stream) -> Stream {
    let mut g = Graph::new();
    let n = g.add_node("source.object");
    g.set_text_param(n, OBJECT_PARAM, named);
    let mut cook = Cook::new();
    cook.set_external(published, stream);
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    out[0].as_stream().clone()
}

#[test]
fn the_source_emits_the_published_object_stream() {
    // The membrane published a sprite tile under `Ball`; the node names `Ball`
    // and emits exactly that appearance — the door the graph gains to say WHAT.
    let out = source("Ball", "Ball", tile(5.0));
    assert_eq!(out.count(), 1);
    let Some(Column::Scalar(ids)) = out.get("texture_id") else {
        panic!("texture_id")
    };
    assert_eq!(ids, &vec![5.0]);
    let Some(Column::Vec2(size)) = out.get("size") else {
        panic!("size")
    };
    assert_eq!(size, &vec![[2.0, 3.0]]);
    let Some(Column::Vec4(uv)) = out.get("uv_rect") else {
        panic!("uv_rect")
    };
    assert_eq!(uv, &vec![[0.1, 0.2, 0.3, 0.4]]);
    let Some(Column::Vec4(tint)) = out.get("tint") else {
        panic!("tint")
    };
    assert_eq!(tint, &vec![[1.0, 0.0, 0.0, 1.0]]);
}

#[test]
fn an_unpicked_source_emits_nothing() {
    // No object named (empty text param) → the empty external → an empty stream.
    // The node emits nothing rather than guessing or failing.
    let out = source("Ball", "", tile(5.0));
    assert_eq!(out.count(), 0);
}

#[test]
fn the_name_is_the_reference_a_mismatch_decouples() {
    // The object is published under `Box`, but the node names `Ball`. Nothing
    // resolves — renaming an object you referred to by name IS decoupling it.
    let out = source("Box", "Ball", tile(5.0));
    assert_eq!(out.count(), 0);
}
