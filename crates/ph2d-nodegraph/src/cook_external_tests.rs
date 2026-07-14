//! **Externals** (doc 65) — the door from the app into the cook. Sibling of `cook_tests` (LOC cap).
//!
//! The gate that matters is the one that is easy to get wrong and impossible to notice: **the memo
//! has to see the external.** It is an input the graph does not describe, so nothing else in the
//! fingerprint changes when the artist edits the drawn curve — and a node that does not recompute
//! hands back the pre-edit shape **forever**, silently, while looking perfectly correct.
//!
//! (The driven-param fingerprint had to be rescued from exactly this, doc 58 §4.)

use super::*;
use crate::attr::{Column, Stream};
use crate::graph::Graph;
use crate::node::{LoweringKind, NodeManifest, PortSpec};
use crate::port::{Clock, Dim, Domain, PortType};
use std::sync::atomic::{AtomicU64, Ordering};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// A node that reads the external named by its text param `src` and emits it verbatim, counting
/// its own evaluations — so a test can ask *"did you recompute?"* and not merely *"is the answer
/// right?"*.
struct Echo {
    evals: AtomicU64,
}
static ECHO_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.external_echo"),
    name: "test.external_echo",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
impl NodeOp for Echo {
    fn manifest(&self) -> &'static NodeManifest {
        &ECHO_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        self.evals.fetch_add(1, Ordering::Relaxed);
        let name = ctx.text_param("src").unwrap_or_default().to_string();
        let s = ctx.external(&name).clone();
        ctx.emit(s);
    }
}
struct Ops(Echo);
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == ECHO_MAN.id).then_some(&self.0 as &dyn NodeOp)
    }
}

fn curve(pts: &[[f32; 2]]) -> Stream {
    Stream::new(pts.len()).with("P", Column::Vec2(pts.to_vec()))
}

fn ys(v: &[CookValue]) -> Vec<f32> {
    match v[0].as_stream().get("P") {
        Some(Column::Vec2(p)) => p.iter().map(|q| q[1]).collect(),
        _ => Vec::new(),
    }
}

/// The whole feature in one test: **the app publishes, the node reads.**
#[test]
fn a_node_reads_what_the_app_published() {
    let mut g = Graph::new();
    let n = g.add_node("test.external_echo");
    g.set_text_param(n, "src", "Track");
    let o = Ops(Echo {
        evals: AtomicU64::new(0),
    });
    let mut cook = Cook::new();
    cook.set_external("Track", curve(&[[0.0, 1.0], [0.0, 2.0]]));

    let out = cook.cook(&g, &o, n, 0.0).unwrap();
    assert_eq!(ys(out), vec![1.0, 2.0]);
}

/// **The memo has to SEE it.** Edit the curve and the node must recompute — nothing else in its
/// fingerprint changed (no inputs, no params, no tick), so if the external is not in there, the
/// node hands back the pre-edit shape **forever**.
#[test]
fn editing_the_external_recomputes_the_node_that_reads_it() {
    let mut g = Graph::new();
    let n = g.add_node("test.external_echo");
    g.set_text_param(n, "src", "Track");
    let o = Ops(Echo {
        evals: AtomicU64::new(0),
    });
    let mut cook = Cook::new();

    cook.set_external("Track", curve(&[[0.0, 1.0]]));
    assert_eq!(ys(cook.cook(&g, &o, n, 0.0).unwrap()), vec![1.0]);

    // The artist drags the curve.
    cook.set_external("Track", curve(&[[0.0, 9.0]]));
    assert_eq!(
        ys(cook.cook(&g, &o, n, 0.0).unwrap()),
        vec![9.0],
        "the node must follow the edited curve - nothing else in its fingerprint moved"
    );
}

/// …and **publishing the SAME curve does not.** The revision is the content, so a shell that
/// republishes every frame (which is the simple thing to do) invalidates nothing — and a graph
/// full of pure nodes does not re-cook sixty times a second for no reason.
#[test]
fn republishing_the_same_value_reuses_the_memo() {
    let mut g = Graph::new();
    let n = g.add_node("test.external_echo");
    g.set_text_param(n, "src", "Track");
    let o = Ops(Echo {
        evals: AtomicU64::new(0),
    });
    let mut cook = Cook::new();

    cook.set_external("Track", curve(&[[0.0, 1.0]]));
    let _ = cook.cook(&g, &o, n, 0.0).unwrap();
    let after_first = o.0.evals.load(Ordering::Relaxed);

    for _ in 0..5 {
        cook.set_external("Track", curve(&[[0.0, 1.0]])); // the same curve, again and again
        let _ = cook.cook(&g, &o, n, 0.0).unwrap();
    }
    assert_eq!(
        o.0.evals.load(Ordering::Relaxed),
        after_first,
        "the same content is the same revision: nothing recomputed"
    );
}

/// A name nobody published reads as **empty**, exactly like an unconnected input — a node asking
/// for a shape that is not there emits nothing, it does not fail. And a curve **appearing** (the
/// artist draws it, or names it) recomputes the node that was asking.
#[test]
fn an_absent_external_is_empty_and_its_arrival_recomputes() {
    let mut g = Graph::new();
    let n = g.add_node("test.external_echo");
    g.set_text_param(n, "src", "NotDrawnYet");
    let o = Ops(Echo {
        evals: AtomicU64::new(0),
    });
    let mut cook = Cook::new();

    assert!(
        ys(cook.cook(&g, &o, n, 0.0).unwrap()).is_empty(),
        "a shape that does not exist is an empty stream, not an error"
    );

    cook.set_external("NotDrawnYet", curve(&[[0.0, 5.0]]));
    assert_eq!(
        ys(cook.cook(&g, &o, n, 0.0).unwrap()),
        vec![5.0],
        "…and when the artist finally draws it, the node notices"
    );

    // …and when they delete it, it notices that too.
    cook.clear_externals();
    assert!(
        ys(cook.cook(&g, &o, n, 0.0).unwrap()).is_empty(),
        "a deleted shape must not linger in the memo"
    );
}

/// **Pointing at a DIFFERENT curve recomputes** — the text param that names it is already in the
/// fingerprint, which is why the external channel does not need to hash the name a second time.
#[test]
fn renaming_the_source_recomputes() {
    let mut g = Graph::new();
    let n = g.add_node("test.external_echo");
    g.set_text_param(n, "src", "A");
    let o = Ops(Echo {
        evals: AtomicU64::new(0),
    });
    let mut cook = Cook::new();
    cook.set_external("A", curve(&[[0.0, 1.0]]));
    cook.set_external("B", curve(&[[0.0, 2.0]]));

    assert_eq!(ys(cook.cook(&g, &o, n, 0.0).unwrap()), vec![1.0]);
    g.set_text_param(n, "src", "B");
    assert_eq!(
        ys(cook.cook(&g, &o, n, 0.0).unwrap()),
        vec![2.0],
        "the node points somewhere else now"
    );
}
