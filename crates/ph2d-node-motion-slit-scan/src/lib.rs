#![forbid(unsafe_code)]
//! `motion.slit_scan` — **each element sees a different NOW**: the set is sampled
//! along a ramp of delays, so one pose, animated once, arrives spread across time
//! (Motion Nodes M3 — doc 01 §3 / doc 34). A rigid grid that all bobs together
//! becomes a travelling wave; a shape that snaps becomes a whip.
//!
//! The effect is **slit-scan** photography (Trumbull's Star Gate in *2001*; the
//! Hitchcock title sequences), whose motion-graphics descendants are After
//! Effects' **Time Displacement** and Cavalry's per-element time offset: read the
//! subject through a moving slit and the film's spatial axis becomes a *time*
//! axis. Here the "slit" is the element ramp: element `i` shows where the stream
//! was `lag · i/(n−1)` ticks ago, so the whole set spans `lag` ticks of history
//! no matter how many elements it has (the delay is a fraction of the set, not a
//! per-element constant — a constant would make the spread explode with the
//! count and the tail would fall off the end of the buffer).
//!
//! **Order is the axis.** The ramp follows stream order, which is the geometric
//! order of a `motion.grid` (row-major). To sweep along another axis — the true
//! photographic slit — put a `motion.sort` upstream: sorting by X and then
//! scanning makes the delay increase from left to right.
//!
//! **What is delayed is POSITION.** The appearance columns (tint, size, rot) stay
//! live: a slit-scan is a geometric shear of time, and echoing whole rows —
//! colour and all — is what `motion.trail` is for.
//!
//! ## The delay line
//!
//! Sequential, like the other stateful nodes: the past positions ride the `state`
//! feedback port (the editor plumbs its `pre` self-loop on drop) as plain columns
//! (`ring` module). At tick 0 the line is empty and every slot re-seeds to the
//! live pose — the scan forms over the next `lag` ticks instead of snapping out
//! of a garbage history — and the same re-seed catches a changed element count.
//! The delays are **fractional**: a lookback of 3.4 ticks lerps between the
//! slots 3 and 4 ticks back, so a small `lag` shears smoothly instead of
//! stair-stepping between whole ticks.
//!
//! The multiplicative `falloff` field attenuates the delay (0 = live, the node is
//! transparent there), so a falloff makes the shear itself local. `Effect::Pure`
//! (the tick enters the fingerprint through the consumed `pre` edge), HR-5:
//! arithmetic only.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod ring;
use ring::{MAX_LAG, is_slot, past, push};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.slit_scan"),
    name: "motion.slit_scan",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The feedback port: last tick's output carries the delay line. The
        // editor plumbs its `pre` self-loop on drop (the `state` convention,
        // shared with spring / integrate / trail).
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // Ticks of history spanned by the whole set (the last element's delay).
        // A dropped node must SHOW something, so it lands at a visible shear;
        // `0` is the identity (every element live).
        ParamSpec {
            name: "lag",
            default: 12.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The requested lag in ticks, clamped to what the delay line can hold. Non-finite
/// (a hand-edited document) reads as 0 — the identity — never as a NaN delay.
fn lag_ticks(lag: f32) -> f32 {
    if lag.is_finite() {
        lag.clamp(0.0, MAX_LAG as f32)
    } else {
        0.0
    }
}

/// The multiplicative `falloff` field, widened to `n` (absent = 1 everywhere).
fn falloff(s: &Stream, n: usize) -> Vec<f32> {
    match s.get("falloff") {
        Some(Column::Scalar(v)) if v.len() == n => v.clone(),
        _ => vec![1.0; n],
    }
}

/// The positions column, widened to `n`.
fn positions(s: &Stream, n: usize) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    }
}

/// Element `i`'s delay, in ticks: its place in the ramp (`0` at the head, the
/// full `lag` at the tail) attenuated by the falloff field. A single-element
/// stream has no ramp to walk, so it stays live.
fn delay_of(i: usize, n: usize, lag: f32, falloff: f32) -> f32 {
    if n < 2 || lag <= 0.0 {
        return 0.0;
    }
    let rank = i as f32 / (n - 1) as f32;
    let f = if falloff.is_finite() {
        falloff.clamp(0.0, 1.0)
    } else {
        0.0
    };
    (lag * rank * f).clamp(0.0, MAX_LAG as f32)
}

/// Where element `i` was `k` whole ticks ago — `k = 0` is live, deeper slots come
/// from the delay line.
fn at(k: usize, i: usize, live: &[[f32; 2]], past: &[Vec<[f32; 2]>]) -> [f32; 2] {
    if k == 0 { live[i] } else { past[k - 1][i] }
}

/// One tick of the scan: sample every element at its own delay, then advance the
/// delay line. The whole node, as a pure function of (live, state, lag).
fn step(input: &Stream, state: &Stream, lag: f32) -> Stream {
    let n = input.count();
    let live = positions(input, n);
    let field = falloff(input, n);
    let past = past(state, &live);
    let lag = lag_ticks(lag);

    let scanned: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let d = delay_of(i, n, lag, field[i]);
            // Fractional delay: lerp between the two neighbouring ticks of
            // history, so the shear is smooth rather than quantised to ticks.
            let lo = d as usize; // truncation is the floor: d >= 0 and finite
            let hi = (lo + 1).min(MAX_LAG);
            let frac = d - lo as f32;
            let (a, b) = (at(lo, i, &live, &past), at(hi, i, &live, &past));
            [a[0] + (b[0] - a[0]) * frac, a[1] + (b[1] - a[1]) * frac]
        })
        .collect();

    // Everything but the positions and the line's own slots rides through live.
    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != "P" && !is_slot(name) {
            out.set(name.clone(), col.clone());
        }
    }
    out.set("P", Column::Vec2(scanned));
    push(&mut out, past, &live);
    out
}

struct MotionSlitScan;

impl NodeOp for MotionSlitScan {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let lag = ctx.param("lag");
        let out = step(ctx.input(0), ctx.input(1), lag);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionSlitScan))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Slit Scan",
            // A stylistic time effect, like its neighbour Trail.
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "lag",
    label: "Lag",
    min: 0.0,
    max: 32.0,
    step: 0.5,
    widget: ParamWidget::Slider,
}];

#[cfg(test)]
mod tests {
    use super::*;

    /// `n` elements marching in lockstep: at tick `t` every one sits at `x = t`.
    fn marching(n: usize, t: f32) -> Stream {
        Stream::new(n).with("P", Column::Vec2(vec![[t, 0.0]; n]))
    }

    fn xs(s: &Stream) -> Vec<f32> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.iter().map(|p| p[0]).collect(),
            _ => panic!("P"),
        }
    }

    /// Drive `ticks` ticks of the marching set through the scan, feeding each
    /// output back as the next tick's state (what the `pre` self-loop does), and
    /// return the last tick's x positions.
    fn run(n: usize, lag: f32, ticks: usize) -> Vec<f32> {
        let mut state = Stream::new(0);
        let mut last = Stream::new(0);
        for t in 0..ticks {
            last = step(&marching(n, t as f32), &state, lag);
            state = last.clone();
        }
        xs(&last)
    }

    /// **The scan.** The set marches in lockstep (every element at `x = t`), so a
    /// slit-scan must fan it out into a RAMP of ages: the head is live at `x = t`,
    /// the tail lags the full `lag` ticks at `x = t − lag`, the middle in between.
    /// FALSIFIED if the node forwarded the live pose (all equal — the "it compiles
    /// and shows the input" failure) or if it delayed every element equally.
    #[test]
    fn the_set_fans_out_into_a_ramp_of_delays() {
        // 5 elements, lag 4 ticks: delays are 0, 1, 2, 3, 4 ticks.
        let x = run(5, 4.0, 12);
        let t = 11.0; // the last tick driven
        assert_eq!(x, vec![t, t - 1.0, t - 2.0, t - 3.0, t - 4.0]);
    }

    /// Fractional delays interpolate between the two neighbouring ticks of
    /// history instead of stair-stepping: with a lag of 1 across 3 elements the
    /// middle one sits HALF a tick back.
    #[test]
    fn a_fractional_delay_lerps_between_ticks() {
        let x = run(3, 1.0, 8);
        let t = 7.0;
        assert_eq!(x, vec![t, t - 0.5, t - 1.0]);
    }

    /// Tick 0 has no history: the delay line seeds flat at the live pose, so the
    /// node opens on the input instead of snapping out of a garbage past.
    #[test]
    fn the_first_tick_seeds_flat_on_the_live_pose() {
        let out = step(&marching(4, 7.0), &Stream::new(0), 8.0);
        assert_eq!(xs(&out), vec![7.0; 4], "all live at tick 0");
    }

    /// `lag = 0` is the identity (and so is a single element, which has no ramp).
    #[test]
    fn a_zero_lag_is_the_identity() {
        assert_eq!(run(5, 0.0, 6), vec![5.0; 5]);
        assert_eq!(run(1, 8.0, 6), vec![5.0]);
    }

    /// The falloff field attenuates the delay, so the shear can be made local: a
    /// zeroed element stays live even at the tail of the ramp.
    #[test]
    fn the_falloff_field_attenuates_the_delay() {
        let mut state = Stream::new(0);
        let mut last = Stream::new(0);
        for t in 0..8 {
            let mut input = marching(3, t as f32);
            input.set("falloff", Column::Scalar(vec![1.0, 1.0, 0.0]));
            last = step(&input, &state, 2.0);
            state = last.clone();
        }
        let t = 7.0;
        // Element 1 is mid-ramp (1 tick back); element 2 would be 2 ticks back but
        // its field is zero, so it is live.
        assert_eq!(xs(&last), vec![t, t - 1.0, t]);
    }

    /// A changed element count re-seeds the line (an emitter churned / the grid
    /// resized): the scan re-forms rather than pairing unrelated elements or
    /// panicking on a stale, shorter column.
    #[test]
    fn a_count_change_reseeds_the_delay_line() {
        let mut state = Stream::new(0);
        for t in 0..6 {
            state = step(&marching(4, t as f32), &state, 3.0);
        }
        let grown = step(&marching(9, 6.0), &state, 3.0);
        assert_eq!(xs(&grown), vec![6.0; 9], "re-seeded flat, no panic");
    }

    /// The delay line does not leak into the appearance: a non-position column
    /// rides through LIVE (the shear is geometric), and the slots do not pile up
    /// as duplicates tick after tick.
    #[test]
    fn other_columns_ride_through_live() {
        let mut input = marching(3, 1.0);
        input.set("size", Column::Vec2(vec![[2.0, 2.0]; 3]));
        let out = step(&input, &Stream::new(0), 4.0);
        match out.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[2.0, 2.0]; 3]),
            _ => panic!("size"),
        }
        assert_eq!(out.count(), 3, "the element count is untouched");
    }

    /// Cooks through the registry with the `pre` self-loop the editor wires, and
    /// the scan is there: after enough ticks the tail lags the head.
    #[test]
    fn registers_and_cooks_through_the_self_loop() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.slit_scan.test.src"),
            name: "motion.slit_scan.test.src",
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
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                // Every element marches together: x = playhead (in ticks).
                let t = ctx.playhead() as f32 * 60.0;
                ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[t, 0.0]; 3])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionSlitScan),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.slit_scan.test.src");
        let scan = g.add_node("motion.slit_scan");
        g.set_param(scan, "lag", 4.0);
        g.connect(Edge {
            from: (src, 0),
            to: (scan, 0),
            delayed: false,
        })
        .unwrap();
        // The self-loop the editor plumbs on drop: out --pre--> state.
        g.connect(Edge {
            from: (scan, 0),
            to: (scan, 1),
            delayed: true,
        })
        .unwrap();

        let mut cook = Cook::new();
        let dt = 1.0 / 60.0;
        let mut x = Vec::new();
        for k in 0..10 {
            let ph = k as f64 * dt;
            let out = cook.cook(&g, &Ops, scan, ph).unwrap();
            x = match out[0].as_stream().get("P").unwrap() {
                Column::Vec2(v) => v.iter().map(|p| p[0]).collect(),
                _ => panic!("P"),
            };
            cook.advance_tick(&g, &Ops, ph).unwrap();
        }
        assert!(
            x[0] > x[1] && x[1] > x[2],
            "the tail lags the head through the cook: {x:?}"
        );
    }
}
