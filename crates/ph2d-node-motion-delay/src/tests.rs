//! Gates for `motion.delay` (doc 63).
//!
//! The properties that matter are the ones that tell this node apart from the four that already
//! touch time: **the neutral point is byte-identical**, **Delay really is the past**, **Average
//! really kills jitter**, **Blend NEVER overshoots** (that is the whole reason it is not
//! `motion.spring`), and **the history follows the ELEMENT, not the row** (without which the node
//! is a silent no-op inside a simulation zone — green, wired, doing nothing).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A source whose position (and, optionally, ids) the test drives tick by tick. Interior mutability
/// because `NodeOp::eval` takes `&self` — the whole point is to move it between cooks.
struct Src {
    pos: std::sync::Mutex<Vec<[f32; 2]>>,
    ids: std::sync::Mutex<Option<Vec<f32>>>,
}
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.delay.test.src"),
    name: "motion.delay.test.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Temporal, // it changes per tick, and the cook must not memoize it away
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let p = self.pos.lock().expect("test").clone();
        let mut s = Stream::new(p.len()).with("P", Column::Vec2(p));
        if let Some(ids) = self.ids.lock().expect("test").clone() {
            s = s.with("id", Column::Scalar(ids));
        }
        ctx.emit(s);
    }
}
struct Ops(Src);
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == MANIFEST.id {
            Some(&MotionDelay as &dyn NodeOp)
        } else if ty == SRC_MAN.id {
            Some(&self.0 as &dyn NodeOp)
        } else {
            None
        }
    }
}

/// The `src -> delay` chain, with the `pre` self-loop the editor plumbs on drop.
fn rig(params: &[(&str, f32)]) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("motion.delay.test.src");
    let dly = g.add_node("motion.delay");
    g.connect(Edge {
        from: (src, 0),
        to: (dly, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (dly, 0),
        to: (dly, 1), // the `state` port: out --pre--> state
        delayed: true,
    })
    .unwrap();
    for (k, v) in params {
        g.set_param(dly, *k, *v);
    }
    (g, src, dly)
}

/// Run `n` ticks, moving the source to `path[t]` on tick `t`, and return what the node emitted
/// each tick (element 0's y).
fn run(params: &[(&str, f32)], path: &[f32]) -> Vec<f32> {
    let (g, _src, dly) = rig(params);
    let ops = Ops(Src {
        pos: std::sync::Mutex::new(vec![[0.0, path[0]]]),
        ids: std::sync::Mutex::new(None),
    });
    let mut cook = Cook::new();
    let mut out = Vec::new();
    for (t, y) in path.iter().enumerate() {
        *ops.0.pos.lock().expect("test") = vec![[0.0, *y]];
        let cooked = cook.cook(&g, &ops, dly, t as f64).unwrap();
        match cooked[0].as_stream().get("P") {
            Some(Column::Vec2(v)) => out.push(v[0][1]),
            _ => panic!("the delay emits P"),
        }
        cook.advance_tick(&g, &ops, t as f64).unwrap();
    }
    out
}

/// **The neutral point is byte-identical.** Dropping the node into a chain must change NOTHING
/// until it is asked for something — in every mode, not just the default.
#[test]
fn zero_ticks_is_a_no_op_in_every_mode() {
    let path: Vec<f32> = (0..12).map(|i| i as f32 * 0.7).collect();
    for mode in [0.0, 1.0, 2.0] {
        let out = run(&[("mode", mode), ("ticks", 0.0)], &path);
        assert_eq!(out, path, "mode {mode} with 0 ticks must be transparent");
    }
}

/// **Delay really is the past.** After the line fills, the output is where the input WAS.
#[test]
fn delay_mode_emits_the_position_from_n_ticks_ago() {
    let path: Vec<f32> = (0..16).map(|i| i as f32).collect(); // y = t
    let out = run(&[("mode", 0.0), ("ticks", 4.0)], &path);
    // From tick 4 on, the line is full: out(t) == in(t-4) == t-4.
    for (t, v) in out.iter().enumerate().skip(4) {
        assert!(
            (v - (t as f32 - 4.0)).abs() < 1e-4,
            "at tick {t} the node should be showing tick {}: got {v}",
            t - 4
        );
    }
    // …and before it filled, it seeded FLAT (at the live value), not at garbage.
    assert!(out[0].abs() < 1e-6, "tick 0 has no past: it shows itself");
}

/// **Average really kills jitter** — that is what a boxcar is FOR. An alternating ±1 averaged over
/// a window is ~0; a broken Average (one that returns the live value) keeps the full ±1.
#[test]
fn average_mode_flattens_an_alternating_input() {
    let path: Vec<f32> = (0..24)
        .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
    let out = run(&[("mode", 1.0), ("ticks", 5.0)], &path);
    let settled = &out[12..];
    let worst = settled.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    assert!(
        worst < 0.25,
        "a 5-tick average of a +/-1 square should sit near zero, but it swings {worst}"
    );
    // And the live input really was swinging — the gate is not measuring a flat input.
    assert!(path[12..].iter().any(|v| v.abs() > 0.9));
}

/// **Blend NEVER overshoots.** This is the whole reason the node exists: `motion.spring` chases
/// with overshoot and RINGS on a jittery input; a one-pole can only approach.
///
/// Step the input from 0 to 1 and the output must climb monotonically and **never pass 1**. A
/// spring in the same rig sails past it.
#[test]
fn blend_mode_approaches_and_never_overshoots() {
    let mut path = vec![0.0; 4];
    path.extend(std::iter::repeat_n(1.0, 40));
    let out = run(&[("mode", 2.0), ("ticks", 6.0)], &path);

    let after = &out[4..];
    assert!(
        after.iter().all(|v| *v <= 1.0 + 1e-6),
        "a one-pole must never pass its target: {:?}",
        after.iter().cloned().fold(f32::MIN, f32::max)
    );
    assert!(
        after.windows(2).all(|w| w[1] >= w[0] - 1e-6),
        "…and it must climb monotonically, never ring"
    );
    assert!(
        *after.last().expect("ticks") > 0.98,
        "it must actually GET there: {}",
        after.last().expect("ticks")
    );
    // The lag is real: one tick after the step, a 6-tick pole has covered 1/6 of the way.
    assert!(
        (after[0] - 1.0 / 6.0).abs() < 1e-4,
        "the one-pole's first step should be 1/6: {}",
        after[0]
    );
}

/// **The history follows the ELEMENT, not the row.** Inside a simulation zone the count changes on
/// almost every tick (spawn, cull), so an order-matched ring re-seeds constantly and the node
/// becomes a silent no-op — wired, green, doing nothing.
///
/// Here element `7` keeps moving while a stranger is born beside it and shifts its row. Its delay
/// must survive that.
#[test]
fn a_newborn_neighbour_does_not_erase_an_elements_past() {
    let (g, _src, dly) = rig(&[("mode", 0.0), ("ticks", 2.0)]);
    let ops = Ops(Src {
        pos: std::sync::Mutex::new(vec![[0.0, 0.0]]),
        ids: std::sync::Mutex::new(Some(vec![7.0])),
    });
    let mut cook = Cook::new();

    // Ticks 0..3: element 7 alone, marching up.
    for t in 0..3u64 {
        *ops.0.pos.lock().expect("test") = vec![[0.0, t as f32]];
        let _ = cook.cook(&g, &ops, dly, t as f64).unwrap();
        cook.advance_tick(&g, &ops, t as f64).unwrap();
    }

    // Tick 3: a NEWBORN (id 99) is spawned AHEAD of it — element 7 is now row 1, not row 0.
    *ops.0.pos.lock().expect("test") = vec![[0.0, 50.0], [0.0, 3.0]];
    *ops.0.ids.lock().expect("test") = Some(vec![99.0, 7.0]);
    let out = cook.cook(&g, &ops, dly, 3.0).unwrap();
    let p = match out[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    };

    // Element 7 (row 1) is 2 ticks late: it must show y = 1, its own past.
    assert!(
        (p[1][1] - 1.0).abs() < 1e-4,
        "the newborn shifted its row and it lost its history: {:?}",
        p[1]
    );
    // The newborn (row 0) has NO past, so it shows itself — not the stranger's history, and not 0.
    assert!(
        (p[0][1] - 50.0).abs() < 1e-4,
        "a newborn must start where it IS, not inherit a stranger's past: {:?}",
        p[0]
    );
}

/// The multiplicative `falloff` field gates the EFFECT (a falloff of 0 is transparent) — and the
/// line keeps filling regardless, so a field that opens later does not start from nothing.
#[test]
fn the_falloff_field_gates_the_delay() {
    // A source with a half falloff on its only element.
    struct Half;
    static HALF_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.delay.test.half"),
        name: "motion.delay.test.half",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Temporal,
        clock: Clock::Frame,
        params: &[ParamSpec {
            name: "y",
            default: 0.0,
        }],
        lowerings: &[LoweringKind::Cpu],
    };
    impl NodeOp for Half {
        fn manifest(&self) -> &'static NodeManifest {
            &HALF_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let y = ctx.param("y");
            ctx.emit(
                Stream::new(1)
                    .with("P", Column::Vec2(vec![[0.0, y]]))
                    .with("falloff", Column::Scalar(vec![0.5])),
            );
        }
    }
    struct O2;
    impl OpResolver for O2 {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            if ty == MANIFEST.id {
                Some(&MotionDelay as &dyn NodeOp)
            } else if ty == HALF_MAN.id {
                Some(&Half as &dyn NodeOp)
            } else {
                None
            }
        }
    }

    let mut g = Graph::new();
    let src = g.add_node("motion.delay.test.half");
    let dly = g.add_node("motion.delay");
    g.connect(Edge {
        from: (src, 0),
        to: (dly, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (dly, 0),
        to: (dly, 1),
        delayed: true,
    })
    .unwrap();
    g.set_param(dly, "mode", 0.0);
    g.set_param(dly, "ticks", 3.0);

    let o = O2;
    let mut cook = Cook::new();
    // Three ticks at y = 0, then jump to 10. Fully delayed it would still read 0; live it reads 10.
    // At half falloff it must read exactly half way: 5.
    for t in 0..4u64 {
        g.set_param(src, "y", 0.0);
        let _ = cook.cook(&g, &o, dly, t as f64).unwrap();
        cook.advance_tick(&g, &o, t as f64).unwrap();
    }
    g.set_param(src, "y", 10.0);
    let out = cook.cook(&g, &o, dly, 4.0).unwrap();
    let y = match out[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v[0][1],
        _ => panic!("P"),
    };
    assert!(
        (y - 5.0).abs() < 1e-4,
        "half the field is half the delay: expected 5, got {y}"
    );
}
