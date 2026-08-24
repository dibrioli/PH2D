//! Os gates de FONTE do `motion.spring` — a perseguição, o estado e a máscara.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte é o
//! dos irmãos: o `lib.rs` responde *como a mola funciona* e os `*_tests.rs` provam-no.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

// A target that STEPS: y = 0 before t = 0.5, y = 2 after. The spring must
// lag it, overshoot it, and settle on it.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.spring.test.src"),
    name: "motion.spring.test.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let y = if ctx.playhead() < 0.5 { 0.0 } else { 2.0 };
        ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[0.0, y]])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionSpring),
            _ => None,
        }
    }
}

/// src → spring.in, with the pre self-loop the editor template creates.
fn spring_graph(params: &[(&str, f32)]) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("motion.spring.test.src");
    let sp = g.add_node("motion.spring");
    g.connect(Edge {
        from: (src, 0),
        to: (sp, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (sp, 0),
        to: (sp, 1),
        delayed: true,
    })
    .unwrap();
    for (name, v) in params {
        g.set_param(sp, *name, *v);
    }
    (g, sp)
}

/// Run `ticks` fixed steps at 60 Hz, returning the emitted Y per tick.
fn run(params: &[(&str, f32)], ticks: usize) -> Vec<f32> {
    let (g, sp) = spring_graph(params);
    let mut cook = Cook::new();
    let mut ys = Vec::new();
    for k in 0..ticks {
        let ph = k as f64 / 60.0;
        let out = cook.cook(&g, &Ops, sp, ph).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => ys.push(v[0][1]),
            _ => panic!("P"),
        }
        cook.advance_tick(&g, &Ops, ph).unwrap();
    }
    ys
}

#[test]
fn seeds_at_the_target_then_lags_overshoots_and_settles() {
    // 3 seconds at 60 Hz; the step lands at t = 0.5 (tick 30).
    let ys = run(&[("tension", 20.0), ("friction", 3.0)], 180);
    assert_eq!(ys[0], 0.0, "seeded at the target, no snap");
    // Right after the step the spring LAGS (well below the new target).
    assert!(ys[32] < 1.0, "lags the step, got {}", ys[32]);
    // It then OVERSHOOTS (crosses above 2) — the follow-through that
    // distinguishes a spring from a lerp; a lerp never crosses its target.
    let peak = ys[30..].iter().cloned().fold(f32::MIN, f32::max);
    assert!(peak > 2.05, "overshoots the target, peak {peak}");
    // And finally SETTLES on it.
    let last = *ys.last().unwrap();
    assert!(
        (last - 2.0).abs() < 0.05,
        "settles at the target, got {last}"
    );
}

#[test]
fn without_the_state_loop_it_follows_the_target_exactly() {
    // No pre self-loop wired → seeds every tick → output == target. The
    // reference's "only acts on targets that change" footnote, inverted.
    let mut g = Graph::new();
    let src = g.add_node("motion.spring.test.src");
    let sp = g.add_node("motion.spring");
    g.connect(Edge {
        from: (src, 0),
        to: (sp, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, sp, 0.9).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v[0][1], 2.0, "seeds at the live target"),
        _ => panic!("P"),
    }
}

#[test]
fn stiff_spring_stays_stable_via_sub_steps() {
    // tension 60 (the UI max) would explode a single 1/60 Euler step
    // (dt²·k = 0.0167²·60 ≈ 0.017 is fine, but MAX_DT-sized steps are not:
    // 0.1²·60 = 0.6 >> 0.05). Drive with dt = MAX_DT ticks: bounded, settles.
    let (g, sp) = spring_graph(&[("tension", 60.0), ("friction", 2.0)]);
    let mut cook = Cook::new();
    let mut last = 0.0f32;
    for k in 0..60 {
        let ph = k as f64 * 0.1;
        let out = cook.cook(&g, &Ops, sp, ph).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => last = v[0][1],
            _ => panic!("P"),
        }
        assert!(last.is_finite() && last.abs() < 10.0, "bounded at tick {k}");
        cook.advance_tick(&g, &Ops, ph).unwrap();
    }
    assert!((last - 2.0).abs() < 0.1, "settled, got {last}");
}

#[test]
fn replay_is_deterministic() {
    let a = run(&[("tension", 8.0)], 90);
    let b = run(&[("tension", 8.0)], 90);
    assert_eq!(a, b);
}

#[test]
fn size_channel_springs_around_unit_identity() {
    // A bare P-only stream on the Size channel: the target is the unit
    // identity → the spring holds size 1.0 (never 0 — sprites don't vanish).
    let (g, sp) = spring_graph(&[("channel", 3.0)]);
    let mut cook = Cook::new();
    for k in 0..3 {
        let ph = k as f64 / 60.0;
        let out = cook.cook(&g, &Ops, sp, ph).unwrap();
        match out[0].as_stream().get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [1.0, 1.0], "unit identity at tick {k}"),
            _ => panic!("size"),
        }
        cook.advance_tick(&g, &Ops, ph).unwrap();
    }
}

#[test]
fn falloff_zero_makes_the_spring_transparent() {
    // Poke the pure step fn directly: with falloff 0 the OUTPUT is the raw
    // target even while the internal state lags elsewhere.
    let input = Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 5.0]]))
        .with("falloff", Column::Scalar(vec![0.0]));
    let state = Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("spring_value", Column::Scalar(vec![0.0]))
        .with("spring_vel", Column::Scalar(vec![0.0]))
        .with("sim_t", Column::Scalar(vec![0.0]));
    let out = step(&input, &state, 1, 8.0, 1.5, 1.0 / 60.0);
    match out.get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v[0][1], 5.0, "falloff 0: raw target"),
        _ => panic!("P"),
    }
}

/// A **pinned** element (`motion.pin_constraint`'s `inv_mass = 0`) tracks its
/// target rigidly: no lag, no overshoot. Its free neighbour, same spring,
/// still lags behind. FALSIFIED if the spring ignored the pin weight (the
/// pinned element would lag with the other one).
#[test]
fn a_pinned_element_tracks_its_target_rigidly() {
    let input = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 5.0], [0.0, 5.0]]))
        .with("inv_mass", Column::Scalar(vec![0.0, 1.0]));
    let state = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]))
        .with("spring_value", Column::Scalar(vec![0.0, 0.0]))
        .with("spring_vel", Column::Scalar(vec![0.0, 0.0]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]));
    let out = step(&input, &state, 1, 8.0, 1.5, 1.0 / 60.0);
    match out.get("P").unwrap() {
        Column::Vec2(v) => {
            assert_eq!(v[0][1], 5.0, "pinned: exactly on target, no lag");
            assert!(v[1][1] < 5.0, "free: still lagging behind ({})", v[1][1]);
        }
        _ => panic!("P"),
    }
}

/// **SONDA (folha 03, linha 65)** — *uma massa por elemento já é exprimível?*
///
/// A célula diz *"POR ELEMENTO: gap real"*. A cadeia que a testa são DUAS
/// molas em série com `falloff` COMPLEMENTAR: cada nó tem os seus próprios
/// `tension`/`friction`, e o falloff 0 faz a mola ser transparente ali.
#[test]
#[ignore = "sonda"]
fn measure_whether_two_springs_give_two_effective_masses() {
    const N: usize = 2;
    let mut sa = Stream::new(N);
    let mut sb = Stream::new(N);
    let mut ctrl = Stream::new(N);
    println!("alvo salta 0 -> 5 no tique 5; A: tension 30 | B: tension 3");
    println!("tique |  elem0 (so' A)  |  elem1 (so' B)  |  controle (uma mola so')");
    for k in 0..40 {
        let t = k as f32 / 60.0;
        let target = if k < 5 { 0.0 } else { 5.0 };
        let base = Stream::new(N).with("P", Column::Vec2(vec![[0.0, target]; N]));

        let a_in = base.clone().with("falloff", Column::Scalar(vec![1.0, 0.0]));
        let a = step(&a_in, &sa, 1, 30.0, 3.0, t);
        let b_in = a.clone().with("falloff", Column::Scalar(vec![0.0, 1.0]));
        let b = step(&b_in, &sb, 1, 3.0, 1.0, t);
        let c = step(&base, &ctrl, 1, 30.0, 3.0, t);

        if k % 5 == 0 || k == 39 {
            let (Some(Column::Vec2(v)), Some(Column::Vec2(cv))) = (b.get("P"), c.get("P")) else {
                panic!("P")
            };
            println!(
                "{k:5} | {:14.4} | {:14.4} | {:10.4}",
                v[0][1], v[1][1], cv[0][1]
            );
        }
        sa = a;
        sb = b;
        ctrl = c;
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}
