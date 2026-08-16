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
pub(crate) struct Src {
    pub(crate) pos: std::sync::Mutex<Vec<[f32; 2]>>,
    pub(crate) ids: std::sync::Mutex<Option<Vec<f32>>>,
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
pub(crate) struct Ops(pub(crate) Src);
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
pub(crate) fn rig(params: &[(&str, f32)]) -> (Graph, NodeId, NodeId) {
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
pub(crate) fn run(params: &[(&str, f32)], path: &[f32]) -> Vec<f32> {
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

/// A source that emits whatever stream the test last handed it — the rig for the CHANNEL gates,
/// which need `rot` / `size` / `tint` and not just a marching `P`.
struct Any(std::sync::Mutex<Stream>);
static ANY_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.delay.test.any"),
    name: "motion.delay.test.any",
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
impl NodeOp for Any {
    fn manifest(&self) -> &'static NodeManifest {
        &ANY_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let s = self.0.lock().expect("test").clone();
        ctx.emit(s);
    }
}
struct OpsAny(Any);
impl OpResolver for OpsAny {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == MANIFEST.id {
            Some(&MotionDelay as &dyn NodeOp)
        } else if ty == ANY_MAN.id {
            Some(&self.0 as &dyn NodeOp)
        } else {
            None
        }
    }
}

/// Run `frames` ticks over the flexible source, handing tick `t` the stream `at` returns, and
/// collect what the node emitted each tick. `at` also gets the graph and the delay's id, so a gate
/// can change a param mid-run (which is exactly what switching the channel is).
pub(crate) fn run_any(
    params: &[(&str, f32)],
    frames: usize,
    mut at: impl FnMut(&mut Graph, NodeId, usize) -> Stream,
) -> Vec<Stream> {
    let mut g = Graph::new();
    let src = g.add_node("motion.delay.test.any");
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
    for (k, v) in params {
        g.set_param(dly, *k, *v);
    }
    let ops = OpsAny(Any(std::sync::Mutex::new(Stream::new(0))));
    let mut cook = Cook::new();
    let mut out = Vec::new();
    for t in 0..frames {
        let s = at(&mut g, dly, t);
        *ops.0.0.lock().expect("test") = s;
        let cooked = cook.cook(&g, &ops, dly, t as f64).unwrap();
        out.push(cooked[0].as_stream().clone());
        cook.advance_tick(&g, &ops, t as f64).unwrap();
    }
    out
}

/// One column of an emitted stream, as plain floats.
///
/// ⚠️ Deliberately **not** `ring::flat`: an oracle that reshapes with the code under test agrees
/// with it precisely when the reshaping is what is wrong.
fn col(s: &Stream, name: &str) -> Vec<f32> {
    match s
        .get(name)
        .unwrap_or_else(|| panic!("the stream has no `{name}` column"))
    {
        Column::Scalar(v) => v.clone(),
        Column::Vec2(v) => v.iter().flat_map(|c| [c[0], c[1]]).collect(),
        Column::Vec3(v) => v.iter().flat_map(|c| [c[0], c[1], c[2]]).collect(),
        Column::Vec4(v) => v.iter().flat_map(|c| [c[0], c[1], c[2], c[3]]).collect(),
    }
}

/// **The channel picks WHAT arrives late — and nothing else moves.**
///
/// C4D's Delay Effector lags a set *"with regard to position, scale and rotation"* and its Field
/// adds colour; ours lagged position and a second `motion.delay` downstream just lagged position
/// again, so the other quantities were unreachable through the graph.
///
/// One source carrying all four columns, one lag, six channels: the chosen slice reads `t − LAG`
/// and **every other component reads `t`**. The second half is the one that catches a channel
/// that lags too much — the X channel must hand `P.y` through live.
#[test]
fn each_channel_lags_its_own_quantity_and_leaves_the_others_live() {
    // (channel, the column it owns, which components of it)
    const CASES: &[(f32, &str, &[usize])] = &[
        (0.0, "P", &[0]),
        (1.0, "P", &[1]),
        (2.0, "rot", &[0]),
        (3.0, "size", &[0, 1]),
        (4.0, "P", &[0, 1]),
        (5.0, "tint", &[0, 1, 2, 3]),
    ];
    const LAG: usize = 3;
    for (ch, owned_col, owned) in CASES {
        let outs = run_any(
            &[("mode", 0.0), ("ticks", LAG as f32), ("channel", *ch)],
            10,
            |_, _, t| {
                let v = t as f32;
                Stream::new(1)
                    .with("P", Column::Vec2(vec![[v, v]]))
                    .with("rot", Column::Scalar(vec![v]))
                    .with("size", Column::Vec2(vec![[v, v]]))
                    .with("tint", Column::Vec4(vec![[v, v, v, v]]))
            },
        );
        for (t, s) in outs.iter().enumerate().skip(LAG) {
            for (name, width) in [("P", 2usize), ("rot", 1), ("size", 2), ("tint", 4)] {
                let got = col(s, name);
                for (k, g) in got.iter().enumerate().take(width) {
                    let lagged = name == *owned_col && owned.contains(&k);
                    let want = t as f32 - if lagged { LAG as f32 } else { 0.0 };
                    assert!(
                        (g - want).abs() < 1e-4,
                        "channel {ch}: {name}[{k}] at tick {t} should be {want}, got {g}"
                    );
                }
            }
        }
    }
}

/// **An angle is a circle and a filter is not.**
///
/// `motion.look_at` writes an `atan2`, so a target circling an element makes `rot` saw-tooth
/// between `+180` and `−180`. A one-pole handed those two numbers eases the LONG way — the sprite
/// unwinds most of a turn over `ticks` frames, at the one seam a spin crosses every revolution.
///
/// The oracle is the APPEARANCE, not the unwrapper: a smoother following a steady spin must keep
/// turning **the same way**.
#[test]
fn the_rotation_channel_does_not_unwind_at_the_seam() {
    // Written the way `look_at` writes it: folded into (−180, 180].
    let wrapped = |t: usize| {
        let a = 20.0 * t as f32;
        a - 360.0 * (a / 360.0 + 0.5).floor()
    };
    let outs = run_any(
        &[("mode", 2.0), ("ticks", 4.0), ("channel", 2.0)],
        40,
        |_, _, t| Stream::new(1).with("rot", Column::Scalar(vec![wrapped(t)])),
    );

    // The fixture CONTAINS the phenomenon: the input really does step backwards at the seam.
    let worst_in = (1..40)
        .map(|t| wrapped(t - 1) - wrapped(t))
        .fold(0.0f32, f32::max);
    assert!(worst_in > 300.0, "the input never wraps: {worst_in}");

    let rot: Vec<f32> = outs.iter().map(|s| col(s, "rot")[0]).collect();
    let worst_back = rot.windows(2).map(|w| w[0] - w[1]).fold(0.0f32, f32::max);
    assert!(
        worst_back < 1e-3,
        "the smoother swept BACK {worst_back} degrees at the seam: {rot:?}"
    );
    // …and it really did follow the spin, rather than sitting still.
    assert!(
        rot.last().expect("ticks") - rot[0] > 600.0,
        "the output barely moved: {:?}",
        rot.last()
    );
}

/// **A line is only history if it is history of the same quantity.**
///
/// Position and Size are both `Vec2`, so a state built for one would be handed to the other with
/// the types raising no objection — and the sprites would inherit a position line as their scale.
/// The state carries the channel it was built for; switching re-seeds flat.
#[test]
fn switching_the_channel_re_seeds_the_line() {
    const SIZE: f32 = 2.0;
    let outs = run_any(
        &[("mode", 2.0), ("ticks", 8.0), ("channel", 4.0)],
        12,
        |g, dly, t| {
            if t == 11 {
                g.set_param(dly, "channel", 3.0); // Size — for the last tick only
            }
            let y = t as f32 * 5.0;
            Stream::new(1)
                .with("P", Column::Vec2(vec![[0.0, y]]))
                .with("size", Column::Vec2(vec![[SIZE, SIZE]]))
        },
    );

    // The position line is FULL and far from the size: it is worth inheriting wrongly.
    let carried = col(&outs[10], "P")[1];
    assert!(
        carried > 10.0,
        "the position line never got far from the size, so the gate proves nothing: {carried}"
    );
    let s = col(&outs[11], "size");
    assert!(
        (s[0] - SIZE).abs() < 1e-4 && (s[1] - SIZE).abs() < 1e-4,
        "the size channel inherited the position line: {s:?}"
    );
}

/// **You cannot lag a quantity that is not there** — and the node must not invent one.
///
/// Materialising an identity `size` where the artist had none would ADD a column downstream (a
/// delayed constant is the constant anyway), and keeping a line for it would carry 33 columns of
/// nothing.
#[test]
fn a_channel_the_stream_does_not_carry_is_a_pass_through() {
    let outs = run_any(
        &[("mode", 2.0), ("ticks", 8.0), ("channel", 3.0)],
        6,
        |_, _, t| Stream::new(1).with("P", Column::Vec2(vec![[0.0, t as f32]])),
    );
    for (t, s) in outs.iter().enumerate() {
        assert!(
            s.get("size").is_none(),
            "tick {t}: the node invented a `size` column for a stream that had none"
        );
        assert_eq!(
            col(s, "P"),
            vec![0.0, t as f32],
            "tick {t}: …and it must hand the rest of the stream through untouched"
        );
        assert!(
            s.get("dl_1").is_none() && s.get("dl_out").is_none(),
            "tick {t}: it kept a delay line for a quantity that is not there"
        );
    }
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

// ─────────────────────────────────────────────────────────────────────────────
// `ticks_down` — subir e descer são dois tempos (o Lag CHOP do TD)
// ─────────────────────────────────────────────────────────────────────────────

/// Um degrau: sobe a `1.0` no tick 1, desce a `0.0` no tick `fall`, e fica. É a
/// entrada canônica de um envelope — a única em que *"quanto tempo para subir"* e
/// *"quanto tempo para descer"* são perguntas separadas.
fn step_path(fall: usize, total: usize) -> Vec<f32> {
    (0..total)
        .map(|t| if (1..fall).contains(&t) { 1.0 } else { 0.0 })
        .collect()
}

/// **A DESCIDA É MAIS LENTA QUE A SUBIDA** — o que o Lag CHOP entrega e um
/// `ticks` só não conseguia dizer.
///
/// ⚠️ O oráculo mede as DUAS metades do mesmo degrau na MESMA corrida: quanto
/// falta para chegar ao topo `k` ticks depois de subir, e quanto ainda resta `k`
/// ticks depois de descer. Um gate que só olhasse a descida ficaria verde com um
/// `ticks_down` que atrasasse tudo.
#[test]
fn the_fall_can_be_slower_than_the_rise() {
    const RISE: f32 = 2.0;
    const FALL: f32 = 16.0;
    let path = step_path(12, 24);
    let sym = run(&[("mode", 2.0), ("ticks", RISE)], &path);
    let asym = run(
        &[("mode", 2.0), ("ticks", RISE), ("ticks_down", FALL)],
        &path,
    );

    // A SUBIDA é a mesma nos dois — `ticks_down` não a toca.
    assert_eq!(
        sym[..12],
        asym[..12],
        "a régua da subida não pode mudar: {sym:?} / {asym:?}"
    );
    // E a DESCIDA do assimétrico fica MUITO mais alta quatro ticks depois do
    // degrau — ele ainda está a soltar o valor que o simétrico já largou.
    let (s, a) = (sym[15], asym[15]);
    assert!(
        a > s * 3.0,
        "quatro ticks após o degrau: simétrico {s:.4}, assimétrico {a:.4}"
    );
    eprintln!("degrau: subida idêntica · descida {s:.4} contra {a:.4}");
}

/// **A SENTINELA É *"o mesmo da subida"*, BYTE A BYTE** — o param ausente coza o
/// que um `ticks_down` igual ao `ticks` coza.
///
/// ⚠️ **A 1ª versão deste gate era VAZIA e uma mutação a nomeou:** ela comparava
/// *param ausente* contra *param explicitamente `0`*, e os dois resolvem ao MESMO
/// número — era o nó comparado consigo mesmo, verde sob qualquer lei que a
/// sentinela pudesse ter. Trocar `0` por *"descida instantânea"* passava. O
/// oráculo falsificável é o OUTRO lado da promessa: zero tem de significar
/// **cinco**, e é isso que também prova que um grafo salvo antes deste param coza
/// o que cozia.
#[test]
fn the_zero_sentinel_means_the_rise_rule_to_the_bit() {
    let path = step_path(10, 20);
    let absent = run(&[("mode", 2.0), ("ticks", 5.0)], &path);
    let spelled = run(&[("mode", 2.0), ("ticks", 5.0), ("ticks_down", 5.0)], &path);
    for (t, (a, b)) in absent.iter().zip(&spelled).enumerate() {
        assert_eq!(a.to_bits(), b.to_bits(), "tick {t}: {a} contra {b}");
    }
    // ⚠️ E o CONTROLE: a fixture tem de CONTER uma descida, senão a igualdade é
    // verdadeira por vácuo (só o ramo da subida foi percorrido).
    assert!(
        absent.windows(2).any(|w| w[1] < w[0]),
        "a fixture tem de descer: {absent:?}"
    );
}

/// **`ticks_down = 1` É a descida instantânea** — a medição que torna a
/// sentinela gratuita.
///
/// ⚠️ Sem este número a escolha de `0` como *"o mesmo da subida"* seria uma troca
/// (um tempo pedível gasto numa sentinela). A lei do one-pole divide por
/// `rule.max(1.0)`, então `1` já leva o valor ao vivo num tick — nada se perde.
#[test]
fn a_fall_of_one_tick_is_already_instant() {
    let path = step_path(10, 14);
    let out = run(&[("mode", 2.0), ("ticks", 8.0), ("ticks_down", 1.0)], &path);
    // No tick 10 o degrau desceu; com régua 1 o nó já está no vivo (0.0).
    assert_eq!(out[10], 0.0, "a descida de um tick é instantânea: {out:?}");
    // E o CONTROLE: a subida continua lenta (régua 8), senão o gate estaria a
    // medir um nó que virou passa-adiante.
    assert!(out[2] < 0.5, "a subida continua lagada: {out:?}");
}

/// **A régua da descida NÃO alcança `Delay` nem `Average`** — os dois não têm
/// direção, e é por isso que a row é gateada ao `Blend`.
///
/// ⚠️ Um gate só sobre o `ParamGate` (a row some) provaria a UI e não a LEI: um
/// documento pode carregar `ticks_down` num nó em modo `Average`, e o kernel tem
/// de o ignorar.
#[test]
fn the_fall_rule_is_blend_only() {
    let path = step_path(10, 20);
    for mode in [0.0f32, 1.0] {
        let plain = run(&[("mode", mode), ("ticks", 4.0)], &path);
        let with = run(
            &[("mode", mode), ("ticks", 4.0), ("ticks_down", 20.0)],
            &path,
        );
        assert_eq!(plain, with, "modo {mode}: a descida não pode ser lida");
    }
}

/// **A direção é POR COMPONENTE** — uma cor que clareia num canal e escurece
/// noutro usa os dois tempos no mesmo tick.
///
/// ⚠️ A alternativa (decidir pela norma do vetor) faria o canal que anda pouco
/// herdar a direção do que anda muito, e é exatamente o que este oráculo separa:
/// os dois canais partem do MESMO valor e vão para lados opostos.
#[test]
fn the_direction_is_decided_per_component() {
    let mid = |v: f32| Stream::new(1).with("tint", Column::Vec4(vec![[v, 1.0 - v, 0.5, 1.0]]));
    // Assenta em (0.5, 0.5), depois o vermelho SOBE e o verde DESCE.
    let out = run_any(
        &[
            ("channel", CH_COLOR as f32),
            ("mode", 2.0),
            ("ticks", 2.0),
            ("ticks_down", 24.0),
        ],
        40,
        |_, _, t| mid(if t < 30 { 0.5 } else { 1.0 }),
    );
    let last = col(out.last().expect("frames"), "tint");
    let (red, green) = (last[0], last[1]);
    // O vermelho subiu de 0,5 para ~1,0 com régua 2 (rápido); o verde desceu de
    // 0,5 para 0,0 com régua 24 (devagar) e ainda está bem acima de zero.
    assert!(red > 0.95, "o canal que SOBE é rápido: {red:.4}");
    assert!(green > 0.25, "o canal que DESCE é lento: {green:.4}");
    eprintln!("por componente: vermelho {red:.4} (sobe) · verde {green:.4} (desce)");
}
