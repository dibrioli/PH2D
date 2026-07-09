//! `motion.trail` — an echo trail: every element leaves N fading, shrinking
//! copies of where it was on the previous ticks (Motion Nodes M2, plan §1.3).
//!
//! Comet tails, motion smears, cursor sparkle wakes. The reference catalogue's
//! `trail`: *"Rastro de cometa, trail de cursor, partícula com cauda colorida"*.
//!
//! **The ring buffer IS the value of the self-loop.** The plan reaches for a
//! `History { slots, head, len }` carried as an opaque value; it turns out the
//! stream itself is already that ring. Each tick the node emits
//!
//! ```text
//! out = carried(previous out) ++ live
//! ```
//!
//! where `carried` drops the generation that just aged past `length` and
//! multiplies the survivors' alpha by `fade` and their size by `shrink`. The
//! decay is therefore **geometric** — applied once per tick to the rows carried
//! forward — so it can never double-count, and the state needs no pristine copy
//! of the original colour. The `state` port (the substrate's sequential-node
//! convention) gets its `pre` self-loop wired by the editor, exactly like
//! `spring` and `integrate`.
//!
//! Carried rows draw FIRST so the live head paints on top of its own tail.
//!
//! Two things to know when placing it:
//! - It **duplicates `id`s** (an echo shares its source's identity), so it
//!   belongs *downstream* of anything that pairs state by id — put it after
//!   `motion.integrate`, never before.
//! - It is **sequential** (it reads a `pre`), so it may not sit upstream of a
//!   `motion.time_remap`: the editor refuses that wire, and the cook would
//!   refuse it too (`CookError::SequentialInTimeScope`).

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod carry;
use carry::{ages, concat, fade_alpha, gather, scale_vec2};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The column holding how many ticks ago a row was live. `0` = the live head.
/// Namespaced away from the emitter's own `age` (a particle's lifetime), which
/// means something else entirely and must survive untouched.
const AGE: &str = "trail_age";

/// Hard ceiling on the echo count, independent of the `length` param's own
/// slider range — a document loaded from disk (or authored over MCP) can carry
/// any `f32`.
const MAX_LENGTH: usize = 32;

/// Hard ceiling on the emitted element count. `length × live` is an allocation
/// size driven by an untrusted param times an untrusted upstream: 4096 live
/// particles × 32 echoes is already 131k quads. The trail is CLAMPED (fewer
/// generations), never truncated mid-generation — a half-drawn echo reads as a
/// bug, a shorter tail reads as a setting.
const MAX_INSTANCES: usize = 65_536;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.trail"),
    name: "motion.trail",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The feedback port: last tick's output IS the ring buffer. The editor
        // plumbs its `pre` self-loop on drop (the `state` convention).
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Pure: the tick enters the fingerprint through the consumed `pre` edge.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // A dropped Trail must SHOW something (an inert default reads as a
        // broken node), so it lands at 8 echoes. `length = 1` is the identity.
        ParamSpec {
            name: "length",
            default: 8.0,
        },
        ParamSpec {
            name: "fade",
            default: 0.72,
        },
        ParamSpec {
            name: "shrink",
            default: 0.94,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// How many generations to keep, clamped by both the hard ceiling and the
/// instance budget. Non-finite / negative → 1 (the identity), never 0 rows.
fn generations(length: f32, live: usize) -> usize {
    let requested = if length.is_finite() && length >= 1.0 {
        (length.round() as usize).min(MAX_LENGTH)
    } else {
        1
    };
    if live == 0 {
        return requested;
    }
    requested.max(1).min(MAX_INSTANCES / live.max(1)).max(1)
}

/// One tick of the echo: age last tick's rows, drop the generation that fell
/// off the end, fade + shrink the survivors, and put the live head in front.
fn step(live: &Stream, state: &Stream, length: f32, fade: f32, shrink: f32) -> Stream {
    let k = generations(length, live.count());
    if k <= 1 {
        // A length-1 trail is the identity — forward the live stream untouched,
        // with no `trail_age` column to confuse a downstream reader.
        return live.clone();
    }

    // Survivors: rows whose age, once bumped, still fits inside `k`.
    let prev_ages = ages(state, AGE);
    let keep: Vec<usize> = (0..state.count())
        .filter(|&i| (prev_ages[i] as usize) + 1 < k)
        .collect();
    let mut carried = gather(state, &keep);
    let bumped: Vec<f32> = keep.iter().map(|&i| prev_ages[i] + 1.0).collect();
    carried.set(AGE, Column::Scalar(bumped));
    // Geometric decay: applied once per tick to the rows carried forward, so a
    // row `n` ticks old has had it applied exactly `n` times. Nothing to undo.
    fade_alpha(&mut carried, "tint", fade);
    scale_vec2(&mut carried, "size", shrink);

    let mut head = live.clone();
    head.set(AGE, Column::Scalar(vec![0.0; live.count()]));
    // Tail first, head last: the live element paints over its own echoes.
    concat(&carried, &head)
}

struct MotionTrail;

impl NodeOp for MotionTrail {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (length, fade, shrink) = (ctx.param("length"), ctx.param("fade"), ctx.param("shrink"));
        let out = step(ctx.input(0), ctx.input(1), length, fade, shrink);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTrail))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Trail",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "length",
        label: "Length",
        min: 1.0,
        max: MAX_LENGTH as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "fade",
        label: "Fade",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "shrink",
        label: "Shrink",
        min: 0.5,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn dot(x: f32, alpha: f32) -> Stream {
        Stream::new(1)
            .with("P", Column::Vec2(vec![[x, 0.0]]))
            .with("tint", Column::Vec4(vec![[1.0, 1.0, 1.0, alpha]]))
            .with("size", Column::Vec2(vec![[1.0, 1.0]]))
    }

    fn xs(s: &Stream) -> Vec<f32> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.iter().map(|p| p[0]).collect(),
            _ => panic!(),
        }
    }
    fn alphas(s: &Stream) -> Vec<f32> {
        match s.get("tint").unwrap() {
            Column::Vec4(v) => v.iter().map(|c| c[3]).collect(),
            _ => panic!(),
        }
    }

    /// A moving dot, run for several ticks with the previous output fed back
    /// (exactly what the `pre` self-loop does). The trail must hold the last
    /// `length` positions, oldest first, each dimmer than the next.
    #[test]
    fn the_echo_holds_the_last_n_positions_oldest_first() {
        let mut state = Stream::new(0);
        for t in 0..6 {
            state = step(&dot(t as f32, 1.0), &state, 3.0, 0.5, 1.0);
        }
        // Ticks 3, 4, 5 survive; the live head (x=5) is LAST so it draws on top.
        assert_eq!(xs(&state), vec![3.0, 4.0, 5.0]);
        // Geometric fade: the head is opaque, each older echo halved once more.
        assert_eq!(alphas(&state), vec![0.25, 0.5, 1.0]);
    }

    /// FALSIFICATION of the geometric decay: the fade must be applied ONCE per
    /// tick to the carried rows, never recomputed from the age. Re-deriving it
    /// (`alpha = fade^age`) reads the same here — but re-applying it to an
    /// already-faded row would give `0.5^2` on a one-tick-old echo. Pin the
    /// head at full alpha and the one-tick echo at exactly `fade`.
    #[test]
    fn a_one_tick_old_echo_has_faded_exactly_once() {
        let s = step(&dot(0.0, 1.0), &Stream::new(0), 4.0, 0.5, 1.0);
        let s = step(&dot(1.0, 1.0), &s, 4.0, 0.5, 1.0);
        assert_eq!(alphas(&s), vec![0.5, 1.0]);
        assert_eq!(xs(&s), vec![0.0, 1.0]);
    }

    #[test]
    fn shrink_compounds_and_length_one_is_the_identity() {
        let mut state = Stream::new(0);
        for t in 0..3 {
            state = step(&dot(t as f32, 1.0), &state, 3.0, 1.0, 0.5);
        }
        match state.get("size").unwrap() {
            // Two ticks of 0.5, one tick, then the untouched head.
            Column::Vec2(v) => assert_eq!(
                v.iter().map(|s| s[0]).collect::<Vec<_>>(),
                vec![0.25, 0.5, 1.0]
            ),
            _ => panic!(),
        }

        // length = 1 → the live stream, verbatim, with no `trail_age` added.
        let live = dot(7.0, 1.0);
        let out = step(&live, &state, 1.0, 0.5, 0.5);
        assert_eq!(out.count(), 1);
        assert_eq!(xs(&out), vec![7.0]);
        assert!(out.get(AGE).is_none(), "the identity adds no column");
    }

    /// A run of ticks must not grow without bound: the ring drops the oldest
    /// generation every tick, so the count settles at `length × live`.
    #[test]
    fn the_element_count_settles_at_length_times_live() {
        let mut state = Stream::new(0);
        for t in 0..50 {
            state = step(&dot(t as f32, 1.0), &state, 8.0, 0.9, 0.99);
        }
        assert_eq!(state.count(), 8, "8 generations of a single dot");
    }

    /// `length` and the live count are both untrusted (a loaded document, an
    /// MCP edit). The instance budget clamps the GENERATIONS, so the trail gets
    /// shorter rather than allocating 4096 × 32 quads or emitting a half-drawn
    /// echo.
    #[test]
    fn the_instance_budget_clamps_generations_not_rows() {
        assert_eq!(generations(8.0, 100), 8);
        assert_eq!(generations(999.0, 1), MAX_LENGTH, "hard ceiling");
        assert_eq!(generations(f32::NAN, 10), 1, "junk → the identity");
        assert_eq!(generations(-5.0, 10), 1);
        // 32 generations × 4096 live = 131k > the 65_536 budget → 16 kept.
        assert_eq!(generations(32.0, 4096), MAX_INSTANCES / 4096);
        assert_eq!(generations(32.0, 999_999), 1, "never zero rows");
    }

    /// A stream with neither `tint` nor `size` still echoes (the fade/shrink
    /// simply have nothing to touch) — the trail is not silently a no-op on a
    /// bare positional stream.
    #[test]
    fn a_bare_positional_stream_still_echoes() {
        let bare = |x: f32| Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]]));
        let mut state = Stream::new(0);
        for t in 0..4 {
            state = step(&bare(t as f32), &state, 3.0, 0.5, 0.5);
        }
        assert_eq!(xs(&state), vec![1.0, 2.0, 3.0]);
        assert!(state.get("tint").is_none());
    }
}
