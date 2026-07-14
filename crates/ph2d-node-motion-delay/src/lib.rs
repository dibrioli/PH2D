#![forbid(unsafe_code)]
//! `motion.delay` — **the set arrives LATE, and it arrives SMOOTH** (Motion Nodes M2, the History
//! family — doc 01 §1.3 / doc 63).
//!
//! The reference is Cinema 4D's **Delay Effector**, which is not a delay line at all in the way the
//! name suggests: it sits after the things that move your set and *lags* their result, and its
//! default mode is **Blend** — an exponential ease toward the live value. Its three modes are
//! Average, Blend and Spring; ours are **Delay**, **Average** and **Blend**, because we already have
//! a spring and it is a better one.
//!
//! ## Why this node exists, when four others already touch time
//!
//! The plan named `motion.delay` before `time_remap`, `trail` and `slit_scan` existed. So the first
//! honest question was whether it still has a job. It does — exactly one, and it is not the one in
//! its name:
//!
//! | you want | you already have |
//! |---|---|
//! | the same **pure** sub-tree, cooked at `t − d` | **`motion.time_remap`** — exact, stateless, scrub-perfect, free |
//! | past generations kept as fading **copies** | **`motion.trail`** |
//! | a **ramp** of delays across the set (element *i* sees `t − i·lag`) | **`motion.slit_scan`** |
//! | a chase with **overshoot** and settle | **`motion.spring`** |
//! | **lag with NO overshoot** — a smoother | **nothing.** This node. |
//!
//! A spring cannot be a smoother: it overshoots by construction, and a jittery input makes it ring.
//! A one-pole (Blend) can only approach — which is why every compositing package hands you both.
//!
//! And `time_remap` cannot delay a **simulation**: a sim is not a function of `t`, so you cannot
//! re-cook it a second ago. The only way to see where it *was* is to have kept it. That is the ring.
//!
//! ## The three modes
//!
//! - **Delay** (`0`) — the position `ticks` ago. Fractional: a lookback of 3.4 lerps between the
//!   slots 3 and 4 ticks back, so a small delay slides smoothly instead of stair-stepping.
//! - **Average** (`1`) — the mean of the last `ticks` positions. A boxcar low-pass: it *kills*
//!   jitter (an alternating ±1 averages to 0) at the cost of half the window in lag.
//! - **Blend** (`2`, default — C4D's) — a one-pole ease: `out += (live − out) / ticks`. Lags,
//!   smooths, and **never overshoots**. This is the mode the node is for.
//!
//! **`ticks = 0` is a byte-identical no-op** in every mode — the neutral point, so dropping the
//! node into a chain changes nothing until you ask it to.
//!
//! What is delayed is **position**. The appearance columns (tint, size, rot) stay live: echoing
//! whole rows, colour and all, is what `motion.trail` is for. The multiplicative `falloff` field
//! blends the output between the live value (0) and the delayed one (1), so a falloff makes the
//! smoothing itself local.
//!
//! Sequential (the `pre` self-loop the editor plumbs on drop), `Effect::Pure` (the tick enters the
//! fingerprint through the consumed `pre` edge), HR-5: arithmetic only.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod ring;
use ring::MAX_LAG;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// `mode`: the three ways to be late.
const MODE_DELAY: i32 = 0;
const MODE_AVERAGE: i32 = 1;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.delay"),
    name: "motion.delay",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The feedback port — auto-wired `out --pre--> state` on add (the sequential-node
        // convention: an input named `state` with the output's type).
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
        // 0 Delay · 1 Average · 2 Blend. Blend is the default — it is what the node is FOR, and
        // it is C4D's default too.
        ParamSpec {
            name: "mode",
            default: 2.0,
        },
        // How late, in ticks. **0 is a byte-identical no-op** in every mode.
        ParamSpec {
            name: "ticks",
            default: 8.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The multiplicative `falloff` weight for element `i` (absent → `1.0`).
fn falloff_at(s: &Stream, i: usize) -> f32 {
    match s.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The live positions (absent → empty: a stream with no `P` has nothing to be late about).
fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// The delayed position of element `i`, by mode.
///
/// `past[k-1][i]` is where it was `k` ticks ago; `prev[i]` is what this node emitted for it last
/// tick (the one-pole's own state).
fn delayed(
    mode: i32,
    ticks: f32,
    i: usize,
    live: [f32; 2],
    past: &[Vec<[f32; 2]>],
    prev: &[[f32; 2]],
) -> [f32; 2] {
    match mode {
        MODE_DELAY => {
            // Fractional lookback: 3.4 ticks back is 60% of the way from slot 3 toward slot 4.
            let whole = ticks.floor();
            let frac = ticks - whole;
            let k = (whole as usize).clamp(1, MAX_LAG); // CLAMP-OK: guarded by the caller's `ticks > 0`
            let near = past[k - 1][i];
            let far = past[k.min(MAX_LAG - 1)][i];
            lerp(near, far, frac)
        }
        MODE_AVERAGE => {
            // A boxcar over the window, INCLUDING the live sample — a mean that ignored `now`
            // would lag by a whole extra tick and, at `ticks = 1`, would just be `Delay`.
            let n = (ticks.round() as usize).clamp(1, MAX_LAG); // CLAMP-OK: the ring's bounds
            let mut sum = live;
            for p in past.iter().take(n) {
                sum[0] += p[i][0];
                sum[1] += p[i][1];
            }
            let inv = 1.0 / (n + 1) as f32;
            [sum[0] * inv, sum[1] * inv]
        }
        // Blend: the one-pole. `out += (live - out) / ticks` — it approaches and never passes, and
        // that is the whole difference between it and `motion.spring`.
        _ => lerp(prev[i], live, 1.0 / ticks.max(1.0)),
    }
}

struct MotionDelay;

impl NodeOp for MotionDelay {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i32;
        // 0 = no lag at all. Clamped to the ring, so a hand-edited 900 cannot ask for state that
        // does not exist.
        let ticks = ctx.param("ticks").clamp(0.0, MAX_LAG as f32); // CLAMP-OK: the ring's depth

        let out = {
            let input = ctx.input(0);
            let state = ctx.input(1);
            let live = positions(input);

            // Everything but this node's own state columns passes through untouched — the
            // appearance columns stay LIVE (that is `motion.trail`'s job, not ours).
            let mut out = Stream::new(input.count());
            for (name, col) in input.columns() {
                if !ring::is_state(name) {
                    out.set(name.clone(), col.clone());
                }
            }

            let rows = ring::rows_of(state, input);
            let past = ring::past(state, &rows, &live);
            let prev = ring::prev_out(state, &rows, &live);

            // **The neutral point is byte-identical.** No lag, no smoothing, nothing to compute —
            // the node is transparent until it is asked for something.
            let emitted: Vec<[f32; 2]> = if ticks <= 0.0 {
                live.clone()
            } else {
                (0..live.len())
                    .map(|i| {
                        let d = delayed(mode, ticks, i, live[i], &past, &prev);
                        // The field gates the EFFECT, not the state: the line keeps filling
                        // regardless, so a falloff that opens later does not start from nothing.
                        lerp(live[i], d, falloff_at(input, i))
                    })
                    .collect()
            };

            if !live.is_empty() {
                out.set("P", Column::Vec2(emitted.clone()));
            }
            ring::push(&mut out, past, &live, &emitted);
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionDelay))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Delay",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "ticks",
        label: "Ticks",
        min: 0.0,
        max: 32.0,
        step: 0.5,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
