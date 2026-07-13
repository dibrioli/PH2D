//! **Driven params** (doc 58) — a wire that lands on a parameter. Sibling of `cook_tests`
//! (LOC cap); reuses its `ParamEcho` node, which emits its own `k` param as a length-1 `"v"`
//! scalar and is therefore both a driver and a driven node.
//!
//! The gates that matter are the two nobody would write by accident: **the memo must see the
//! driver** (a driven param that is not in the fingerprint returns a stale number forever),
//! and **the cycle check must see the wire** (a dependency the check cannot see is not a
//! refused connect — it is the cook recursing until the stack runs out).

use super::tests::{PARAM_MAN, ParamEcho, ParamOps};
use super::*;
use crate::attr::Column;
use crate::graph::{EdgeError, Graph, NodeId};
use crate::node::{LoweringKind, NodeManifest, PortSpec};
use crate::port::{Clock, Dim, Domain, PortType};
use std::sync::atomic::{AtomicU64, Ordering};

fn ops() -> ParamOps {
    ParamOps {
        echo: ParamEcho {
            calls: AtomicU64::new(0),
        },
    }
}

/// The one number a node cooked (its `"v"` column).
fn value(out: &[CookValue]) -> f32 {
    match out[0].as_stream().get(crate::attr::VALUE_COLUMN) {
        Some(Column::Scalar(v)) => v[0],
        _ => panic!("the echo emits a scalar"),
    }
}

/// **wire > override > default** — the resolution hierarchy the plan reserved from day one,
/// and the whole feature in one assertion. Note what is NOT here: any change to the node.
/// `ParamEcho` reads `ctx.param("k")` exactly as it always did.
#[test]
fn a_driven_param_beats_the_override_which_beats_the_default() {
    let mut g = Graph::new();
    let driver = g.add_node("test.param_echo");
    let driven = g.add_node("test.param_echo");
    let o = ops();
    let mut cook = Cook::new();

    // Default: 7.
    assert_eq!(value(cook.cook(&g, &o, driven, 0.0).unwrap()), 7.0);
    // Override beats the default.
    g.set_param(driven, "k", 3.0);
    assert_eq!(value(cook.cook(&g, &o, driven, 0.0).unwrap()), 3.0);
    // A WIRE beats the override — and the driver is cooked by this recursion, not by
    // anything else: nothing else in the graph pulls it.
    g.set_param(driver, "k", 42.0);
    g.drive_param(driven, "k", (driver, 0)).unwrap();
    assert_eq!(
        value(cook.cook(&g, &o, driven, 0.0).unwrap()),
        42.0,
        "the parameter reads the wire"
    );
    // Pull the wire and the override is still there, untouched — driving is not editing.
    assert_eq!(g.undrive_param(driven, "k"), Some((driver, 0)));
    assert_eq!(value(cook.cook(&g, &o, driven, 0.0).unwrap()), 3.0);
}

/// **The memo has to see the driver.** The driven node's inputs did not change — it has no
/// inputs — so nothing but the driver's revision can tell it to recompute. Without it the
/// node returns the first number it ever read, forever: the artist drags the driver's knob
/// and the scene does not move.
#[test]
fn editing_the_driver_recomputes_the_node_it_drives() {
    let mut g = Graph::new();
    let driver = g.add_node("test.param_echo");
    let driven = g.add_node("test.param_echo");
    g.drive_param(driven, "k", (driver, 0)).unwrap();
    let o = ops();
    let mut cook = Cook::new();

    assert_eq!(value(cook.cook(&g, &o, driven, 0.0).unwrap()), 7.0);
    g.set_param(driver, "k", 11.0);
    assert_eq!(
        value(cook.cook(&g, &o, driven, 0.0).unwrap()),
        11.0,
        "the driven node re-cooked because its DRIVER's revision changed"
    );

    // …and a cook that changed nothing still reuses the memo: the driver is a dependency,
    // not a reason to recompute the world every frame.
    let before = o.echo.calls.load(Ordering::Relaxed);
    let _ = cook.cook(&g, &o, driven, 0.0).unwrap();
    assert_eq!(
        o.echo.calls.load(Ordering::Relaxed),
        before,
        "nothing changed, so nothing recomputed"
    );
}

/// **Re-pointing a param to another PORT of the SAME driver must recompute.**
///
/// This is the one case the revisions cannot see, and I had to be shown it: re-pointing to a
/// different *node* bumps `input_revs` all by itself, so my first version of this gate was
/// green with the fingerprint field deleted — it proved nothing. Two ports of one node share
/// one revision. Without the wiring in the fingerprint, the driven node keeps the number it
/// read from the old port, forever.
#[test]
fn re_pointing_a_param_to_another_port_of_the_same_driver_recomputes() {
    // One node, two outputs, two different numbers.
    struct TwoOut;
    static TWO_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.two_out"),
        name: "test.two_out",
        inputs: &[],
        outputs: &[
            PortSpec {
                name: "a",
                ty: PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame),
            },
            PortSpec {
                name: "b",
                ty: PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame),
            },
        ],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    impl NodeOp for TwoOut {
        fn manifest(&self) -> &'static NodeManifest {
            &TWO_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(1).with(crate::attr::VALUE_COLUMN, Column::Scalar(vec![10.0])));
            ctx.emit(Stream::new(1).with(crate::attr::VALUE_COLUMN, Column::Scalar(vec![20.0])));
        }
    }
    struct Both {
        echo: ParamEcho,
        two: TwoOut,
    }
    impl OpResolver for Both {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            if ty == PARAM_MAN.id {
                Some(&self.echo as &dyn NodeOp)
            } else if ty == TWO_MAN.id {
                Some(&self.two as &dyn NodeOp)
            } else {
                None
            }
        }
    }

    let mut g = Graph::new();
    let driver = g.add_node("test.two_out");
    let driven = g.add_node("test.param_echo");
    g.drive_param(driven, "k", (driver, 0)).unwrap();
    let o = Both {
        echo: ParamEcho {
            calls: AtomicU64::new(0),
        },
        two: TwoOut,
    };
    let mut cook = Cook::new();
    assert_eq!(value(cook.cook(&g, &o, driven, 0.0).unwrap()), 10.0);

    g.drive_param(driven, "k", (driver, 1)).unwrap(); // SAME node, same revision
    assert_eq!(
        value(cook.cook(&g, &o, driven, 0.0).unwrap()),
        20.0,
        "the wire moved to another port of the same driver - the revision did not change, so \
         only the WIRING in the fingerprint can tell the memo it is now reading somewhere else"
    );
}

/// **A driven param is a dependency, so it can close a cycle — and the check has to see it.**
/// It cannot be caught later: an unseen cycle is not a refused connect, it is `cook_node`
/// recursing into itself until the stack runs out.
#[test]
fn driving_a_param_cannot_close_a_cycle() {
    let mut g = Graph::new();
    let a = g.add_node("test.param_echo");
    let b = g.add_node("test.param_echo");
    assert_eq!(
        g.drive_param(a, "k", (a, 0)),
        Err(EdgeError::WouldCycle),
        "a node cannot drive itself"
    );
    g.drive_param(b, "k", (a, 0)).unwrap();
    assert_eq!(
        g.drive_param(a, "k", (b, 0)),
        Err(EdgeError::WouldCycle),
        "a -> b by a param wire, so b -> a closes the loop"
    );
    // The mirror: a plain EDGE must also see the param wire it would close a loop through.
    let mut g2 = Graph::new();
    let x = g2.add_node("test.param_echo");
    let y = g2.add_node("test.param_echo");
    g2.drive_param(y, "k", (x, 0)).unwrap();
    assert_eq!(
        g2.connect(crate::graph::Edge {
            from: (y, 0),
            to: (x, 0),
            delayed: false,
        }),
        Err(EdgeError::WouldCycle),
        "the cycle runs through a param wire, and an edge closes it - one dependency graph, \
         not two"
    );
    // Unknown endpoints are refused like any edge's.
    assert_eq!(
        g2.drive_param(x, "k", (NodeId(99), 0)),
        Err(EdgeError::UnknownNode)
    );
}

/// Deleting the driver un-drives the param, on both sides of the map. A source left pointing
/// at a deleted node is a socket wired to a ghost: it would cook `Empty` forever.
#[test]
fn deleting_a_node_takes_its_param_wires_with_it() {
    let mut g = Graph::new();
    let driver = g.add_node("test.param_echo");
    let driven = g.add_node("test.param_echo");
    g.drive_param(driven, "k", (driver, 0)).unwrap();
    g.remove_node(driver);
    assert!(
        g.param_sources(driven).is_none(),
        "the param that was driven BY it is free again"
    );

    let driver2 = g.add_node("test.param_echo");
    g.drive_param(driven, "k", (driver2, 0)).unwrap();
    g.remove_node(driven);
    assert!(
        g.all_param_sources().is_empty(),
        "and a deleted node takes the params IT drove with it"
    );
}

/// **An empty driver does not mean zero.** A wire that has produced no number yet has not
/// said the number is zero — falling back to the override/default keeps the scene still
/// instead of snapping every driven value to 0 on the frames before an emitter spawns.
#[test]
fn a_driver_that_emits_nothing_leaves_the_param_alone() {
    // `Silent` emits an EMPTY stream on the same port shape as the echo.
    struct Silent;
    static SILENT_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.silent"),
        name: "test.silent",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame),
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    impl NodeOp for Silent {
        fn manifest(&self) -> &'static NodeManifest {
            &SILENT_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(0));
        }
    }
    struct Both {
        echo: ParamEcho,
        silent: Silent,
    }
    impl OpResolver for Both {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            if ty == PARAM_MAN.id {
                Some(&self.echo as &dyn NodeOp)
            } else if ty == SILENT_MAN.id {
                Some(&self.silent as &dyn NodeOp)
            } else {
                None
            }
        }
    }

    let mut g = Graph::new();
    let silent = g.add_node("test.silent");
    let driven = g.add_node("test.param_echo");
    g.set_param(driven, "k", 4.0);
    g.drive_param(driven, "k", (silent, 0)).unwrap();
    let o = Both {
        echo: ParamEcho {
            calls: AtomicU64::new(0),
        },
        silent: Silent,
    };
    let mut cook = Cook::new();
    assert_eq!(
        value(cook.cook(&g, &o, driven, 0.0).unwrap()),
        4.0,
        "no number came down the wire, so the param kept the one it had"
    );
}
