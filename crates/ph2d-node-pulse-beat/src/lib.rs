//! `pulse.beat` — the beat SOURCE: a metronome that emits a PULSE directly from
//! the playhead (Motion Nodes M2, pulse family; handoff doc `09_handoff_pulse_*`).
//!
//! This is the node the family was missing. `pulse.threshold` turns a *signal*
//! into a pulse — but the module had no signal *source*, so the demo faked a
//! clock by oscillating the invisible Rotation channel and thresholding it: two
//! nodes coupled through a transform channel nobody renders (the "clock hack",
//! doc 09 §1). Every mature tool keeps the clock in its utility family instead:
//! MiniCavalry `lfo` (a `pulse` out per cycle), Max `metro`, TouchDesigner Beat
//! CHOP. `pulse.beat` is that source: period in, pulse out — no channel, nothing
//! to mis-wire.
//!
//! **Semantics:** the beat grid is `t = offset + k·period`. Each tick computes
//! the cycle index `k = floor((t − offset)/period)` and fires when `k` differs
//! from the one carried on the `pre` self-loop — the producer-side edge
//! detection shared by the whole family (`pulse.threshold`'s `armed`,
//! `motion.step`'s `count_tick`). The very first primed tick fires too (Max's
//! `metro` bangs on start), so a scene beats the moment it starts playing.
//! `floor` on IEEE doubles is correctly rounded → deterministic (HR-5); no
//! transcendentals anywhere.
//!
//! **Effect::Temporal, deliberately** (a deviation from the doc 09 §4.1 sketch,
//! which copied the threshold's `Pure`): this node reads `ctx.playhead()`, and
//! only a `Temporal` manifest folds the playhead into the memo fingerprint
//! (`cook.rs`). Declaring it `Pure` would let a same-tick re-cook at a moved
//! playhead return a stale beat. The precedent is `motion.oscillator` — reads
//! the playhead → `Temporal`.
//!
//! Uniform across instances (a global beat, `phase_stagger = 0` by nature):
//! every row fires on the same tick. Per-row swing/stagger is a follow-up for
//! when a per-instance value domain exists (doc 09 §4.3).

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The instance stream this node paces: read only for its row count, passed
/// through nowhere — the beat is a pure source, the stream just tells it N.
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The pulse type (mirror of `ph2d_node_pulse_threshold::PULSE`; kept local so
/// this crate stays a leaf drop-crate — the shared vocabulary is the port
/// `(Instances, Scalar, Event)`, not a shared symbol).
pub const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The canonical column of a pulse stream: `1.0` on the tick it fires.
pub const PULSE_COL: &str = "pulse";
/// The cycle index `k` carried on the `pre` self-loop (the edge memory).
const CYCLE_COL: &str = "beat_cycle";
/// `1.0` once the loop has carried a real cycle index. Distinguishes "no state
/// yet" (first tick → fire the start beat) from a legitimate `k = 0.0`, for any
/// `offset` — a sentinel value inside `beat_cycle` itself could collide.
const PRIMED_COL: &str = "beat_primed";

/// Shortest honoured period, in seconds. A zero/negative `period` would make
/// the cycle index infinite (division by zero) — clamped here, not honoured.
const MIN_PERIOD: f32 = 1e-3;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.beat"),
    name: "pulse.beat",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Feedback: last tick's pulse output carries the cycle index (the edge
        // memory). Named `state` so the editor plumbs its `pre` self-loop on drop.
        PortSpec {
            name: "state",
            ty: PULSE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: PULSE,
    }],
    // Temporal: reads the playhead, so the playhead must gate the memo (see the
    // module doc — the doc 09 sketch said Pure; that would serve stale beats).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Seconds per beat. Clamped to `MIN_PERIOD` at eval.
        ParamSpec {
            name: "period",
            default: 1.0,
        },
        // Phase shift of the beat grid, in seconds: beats land at
        // `offset + k·period`.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The beat-grid cycle index at playhead `t`: `floor((t − offset)/period)`.
/// `f64` division — the playhead is `f64`, and hours of runtime stay exact.
fn cycle_index(t: f64, period: f32, offset: f32) -> f32 {
    let period = period.max(MIN_PERIOD) as f64;
    ((t - offset as f64) / period).floor() as f32
}

/// One tick: fire iff the cycle index moved since the carried one — or on the
/// very first primed tick (the start beat). Uniform: one decision, N rows.
fn step(n: usize, k: f32, state: &Stream) -> Stream {
    let prev_k = match state.get(CYCLE_COL) {
        Some(Column::Scalar(v)) => v.first().copied(),
        _ => None,
    };
    let primed = matches!(state.get(PRIMED_COL), Some(Column::Scalar(v)) if v.first().copied().unwrap_or(0.0) > 0.5);
    let fire = if primed {
        prev_k != Some(k)
    } else {
        true // first tick under this loop: the start beat
    };
    let pulse = if fire { 1.0 } else { 0.0 };
    Stream::new(n)
        .with(PULSE_COL, Column::Scalar(vec![pulse; n]))
        .with(CYCLE_COL, Column::Scalar(vec![k; n]))
        .with(PRIMED_COL, Column::Scalar(vec![1.0; n]))
}

struct PulseBeat;

impl NodeOp for PulseBeat {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let period = ctx.param("period");
        let offset = ctx.param("offset");
        let k = cycle_index(ctx.playhead(), period, offset);
        let n = ctx.input(0).count();
        let out = step(n, k, ctx.input(1));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseBeat))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Beat",
            // Utility grey: pulse plumbing, not a visible transform.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "period",
        label: "Period",
        min: 0.05,
        max: 8.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -4.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    /// Run the metronome over `ticks` frames at 60 Hz, feeding the output back
    /// as `state` (what the `pre` self-loop does). Returns the fired playheads.
    fn beats(ticks: u64, period: f32, offset: f32) -> Vec<f64> {
        let mut state = Stream::new(1);
        let mut fired = Vec::new();
        for i in 0..ticks {
            let t = i as f64 / 60.0;
            state = step(1, cycle_index(t, period, offset), &state);
            if let Some(Column::Scalar(v)) = state.get(PULSE_COL)
                && v[0] > 0.5
            {
                fired.push(t);
            }
        }
        fired
    }

    /// FALSIFICATION of the edge detection: over one second at 60 Hz with a
    /// 0.5 s period, the metronome fires exactly twice — the start beat and the
    /// one boundary inside the window (t = 0.5). Firing "while inside a cycle"
    /// — the bug the carried cycle index exists to prevent — would fire on all
    /// 60 ticks.
    #[test]
    fn it_beats_on_the_cycle_boundary_not_once_per_tick() {
        let fired = beats(60, 0.5, 0.0);
        assert_eq!(fired, vec![0.0, 0.5], "start beat + one boundary, not 60");
    }

    /// FALSIFICATION of "period is wired": halving the period doubles the beat
    /// count over the same window. A metronome that ignored its period (fixed
    /// rate, or the un-clamped division) could not show this ratio.
    #[test]
    fn halving_the_period_doubles_the_beats() {
        let slow = beats(240, 1.0, 0.0).len(); // 4 s → beats at 0,1,2,3
        let fast = beats(240, 0.5, 0.0).len(); // 4 s → beats at 0,.5,…,3.5
        assert_eq!((slow, fast), (4, 8));
    }

    /// The offset shifts the beat GRID: with `offset 0.3` the boundaries land
    /// at 0.3, 1.3, … The start beat still fires at t = 0 (a metronome bangs
    /// when it starts, wherever it is in the cycle — Max `metro`).
    #[test]
    fn the_offset_shifts_the_beat_grid() {
        let fired = beats(120, 1.0, 0.3);
        assert_eq!(fired.len(), 3);
        assert_eq!(fired[0], 0.0, "the start beat");
        // Boundaries land on the first tick at/after 0.3 and 1.3.
        assert!((fired[1] - 0.3).abs() < 1.0 / 60.0 + 1e-9, "{}", fired[1]);
        assert!((fired[2] - 1.3).abs() < 1.0 / 60.0 + 1e-9, "{}", fired[2]);
    }

    /// A NEGATIVE offset must not fake the start beat away: `beat_primed` is a
    /// separate column, so a first-tick cycle index that happens to be non-zero
    /// (offset −2.5 → k = 2 at t = 0) still reads as "no state yet" → fire.
    /// (A sentinel value inside `beat_cycle` would collide exactly here.)
    #[test]
    fn a_negative_offset_still_fires_the_start_beat() {
        let fired = beats(31, 1.0, -2.5);
        assert_eq!(fired[0], 0.0, "primed flag, not a magic cycle value");
        assert_eq!(fired.len(), 2, "then the 3.0-boundary at t = 0.5");
    }

    /// A degenerate `period ≤ 0` is clamped to `MIN_PERIOD`, never divided by:
    /// the cycle index stays finite and the node keeps ticking (at most one
    /// pulse per tick — a pulse column cannot express more).
    #[test]
    fn a_degenerate_period_is_clamped_not_divided_by_zero() {
        let fired = beats(10, 0.0, 0.0);
        assert!(fired.len() == 10, "every tick crosses ≥1 clamped boundary");
        for t in [0.0, 1.0, -3.0] {
            assert!(cycle_index(t, 0.0, 0.0).is_finite());
            assert!(cycle_index(t, -1.0, 0.0).is_finite());
        }
    }

    /// The beat is UNIFORM: every instance fires on the same tick (a global
    /// beat — the whole point of replacing the per-channel clock hack).
    #[test]
    fn every_instance_beats_together() {
        let state = step(3, 0.0, &Stream::new(3));
        match state.get(PULSE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![1.0, 1.0, 1.0]),
            _ => panic!(),
        }
    }

    /// The playhead standing still (paused transport re-cooking) does NOT
    /// retrigger: same cycle index, primed state → silence until t moves on.
    #[test]
    fn a_paused_playhead_does_not_retrigger() {
        let mut state = Stream::new(1);
        let mut total = 0.0;
        for _ in 0..5 {
            state = step(1, cycle_index(1.25, 1.0, 0.0), &state);
            if let Some(Column::Scalar(v)) = state.get(PULSE_COL) {
                total += v[0];
            }
        }
        assert_eq!(total, 1.0, "the start beat only; a held k never refires");
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
