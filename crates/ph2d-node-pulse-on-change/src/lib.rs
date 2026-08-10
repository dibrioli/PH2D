//! `pulse.on_change` — fire a PULSE the tick a **value** field changes (Motion
//! Nodes M2, the value domain — doc 12/17). This is Max/Pd's **`change`** object,
//! TouchDesigner's Trigger-on-change: the value→pulse trigger that watches the
//! value's *derivative* rather than its *level*. It is the complement of
//! `pulse.compare` (which fires on a threshold CROSSING): `on_change` fires
//! whenever the value STEPS to something new — exactly the clock you want off a
//! `pulse.counter`, a `pulse.sample_hold`, or a `value.switch` flip.
//!
//! **Sequential — it holds the previous value.** Last tick's value rides the
//! `pre` self-loop on the `state` port (like `pulse.sample_hold`/`pulse.counter`),
//! and this tick fires iff `|v − prev| > epsilon`. The `epsilon` guard ignores
//! float dither so a steady value never chatters (Max `change` is exact-equality;
//! a small tolerance is the honest floating-point version). On the FIRST tick it
//! primes (records the value, does NOT fire) so a fresh graph doesn't emit a
//! spurious pulse — the same first-tick discipline as the rest of the family.
//!
//! **Direction (`direction`):** Rise (the value stepped UP), Fall (stepped down)
//! or Both — Max's `edge~` has two outlets and TouchDesigner's Logic CHOP has
//! *Rising Edge* and *Falling Edge* as distinct modes. ⚠️ **The vocabulary is the
//! family's, deliberately**: `pulse.compare` and `pulse.threshold` already carry
//! an `edge` selector with these three labels in this order, and this node was
//! the odd one out. Same labels, same numbering (`0 Rise · 1 Fall · 2 Both`) —
//! only the DEFAULT differs, because the neutral here is `Both`: that is what the
//! node did before the param existed, and it does it bit for bit (the `Both` arm
//! contributes no arithmetic at all).
//!
//! **Unary over the field:** the pulse mirrors the value field's length `N` (each
//! instance watches its own value), so a length-N field fires a length-N pulse and
//! the downstream `motion.strobe`/`pulse.counter` reacts per element.
//! `Effect::Pure` — the tick enters the fingerprint through the consumed `pre`
//! edge. Transcendental-free (HR-5): subtract / abs / compare only.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type this node watches — the continuous per-instance scalar field on
/// the `v` column (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this
/// stays a leaf drop-crate — the shared vocabulary is the port, not a symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
/// The pulse type it emits — a discrete per-instance event. `Event` clock keeps it
/// off any `Frame` port (the membrane), so downstream can only be a pulse consumer.
pub const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The value stream's column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// The canonical column of a pulse stream: `1.0` on the tick it fires.
const PULSE_COL: &str = "pulse";
/// Last tick's value, carried on the `pre` self-loop for the change comparison.
const PREV_COL: &str = "oc_prev";
/// `1.0` once the first value has been recorded (the priming flag), on the `pre`
/// self-loop — so the first tick never fires.
const PRIMED_COL: &str = "oc_primed";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.on_change"),
    name: "pulse.on_change",
    inputs: &[
        PortSpec {
            name: "value",
            ty: VALUE,
        },
        // Feedback: last tick's pulse output carries `oc_prev` + `oc_primed`.
        // Named `state` so the editor plumbs its `pre` self-loop on drop.
        PortSpec {
            name: "state",
            ty: PULSE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: PULSE,
    }],
    // Pure: the tick enters the fingerprint through the consumed `pre` edge.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // The smallest change that counts as a change — ignores float dither so a
        // steady value never chatters. Exact-equality is `epsilon = 0`.
        ParamSpec {
            name: "epsilon",
            default: 0.001,
        },
        // 0 Rise · 1 Fall · 2 Both — the family's selector. APPENDED, and its
        // default is the NEUTRAL: `Both` is the pre-param behaviour.
        ParamSpec {
            name: "direction",
            default: 2.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Which way a step has to go to count. Mirror of `pulse.threshold`'s `EdgeDir`
/// — re-declared locally because this is a leaf drop-crate and the shared
/// vocabulary is the PORT, never a symbol.
#[derive(Copy, Clone, PartialEq, Eq)]
enum ChangeDir {
    Rise,
    Fall,
    Both,
}

impl ChangeDir {
    fn from_param(v: f32) -> Self {
        // ⚠️ NaN is checked FIRST, and the gate that asked for this line found a
        // live defect: `f32::NAN as i32` **saturates to 0** (Rust's defined
        // float→int cast), so without this a NaN would select variant 0 —
        // narrowing the node to Rise-only, in silence. It reads harmless in
        // `pulse.threshold` only because variant 0 happens to be ITS default;
        // here the neutral is 2, and the coincidence stops covering for us.
        if !v.is_finite() {
            return ChangeDir::Both;
        }
        match v.round() as i32 {
            0 => ChangeDir::Rise,
            1 => ChangeDir::Fall,
            // The NEUTRAL is the fallback: an unreadable number must not silently
            // NARROW what the node fires on — fall back to what the manifest
            // promises, which here is `Both`.
            _ => ChangeDir::Both,
        }
    }
    fn fires(self, delta: f32) -> bool {
        match self {
            ChangeDir::Rise => delta > 0.0,
            ChangeDir::Fall => delta < 0.0,
            ChangeDir::Both => true,
        }
    }
}

fn scalar_col(s: &Stream, name: &str, n: usize) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, 0.0);
    v
}

/// One tick of change detection. `N` mirrors the `value` field: each instance
/// fires iff its value moved more than `epsilon` since last tick; the first tick
/// primes (records, never fires).
fn step(value: &Stream, state: &Stream, epsilon: f32, dir: ChangeDir) -> Stream {
    let n = value.count();
    let vals = scalar_col(value, VALUE_COL, n);
    let prev = scalar_col(state, PREV_COL, n);
    let primed = scalar_col(state, PRIMED_COL, n);

    let mut pulses = Vec::with_capacity(n);
    for i in 0..n {
        // Prime on the first tick (nothing recorded yet) → never fire then.
        let delta = vals[i] - prev[i];
        let fired = primed[i] > 0.5 && delta.abs() > epsilon && dir.fires(delta);
        pulses.push(if fired { 1.0 } else { 0.0 });
    }
    Stream::new(n)
        .with(PULSE_COL, Column::Scalar(pulses))
        // This tick's value becomes next tick's `prev` (the change memory).
        .with(PREV_COL, Column::Scalar(vals))
        .with(PRIMED_COL, Column::Scalar(vec![1.0; n]))
}

struct PulseOnChange;

impl NodeOp for PulseOnChange {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let epsilon = ctx.param("epsilon").max(0.0);
        let dir = ChangeDir::from_param(ctx.param("direction"));
        let out = step(ctx.input(0), ctx.input(1), epsilon, dir);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseOnChange))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "On Change",
            // Utility grey: a value→pulse adapter, not a visible transform.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};

/// O teto DIGITÁVEL. Um dead-band de `1.0` continua exprimível — só não é mais o que
/// o slider percorre, porque **quem quer um dead-band grande não quer este nó**: o
/// doc-header já divide o trabalho (*"`pulse.compare` dispara num CRUZAMENTO de
/// limiar; `on_change` dispara quando o valor MUDA"*), e o `epsilon` daqui existe para
/// ignorar *dither* de float.
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "epsilon",
    max: 1.0,
}];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "epsilon",
        label: "Epsilon",
        min: 0.0,
        // O curso da MÃO: com o teto em `1.0` um pixel do track (~154 px) valia
        // `0,0065` — **seis vezes e meia o default de 0,001** —, então a guarda de
        // dither que este param É não podia ser afinada com o dedo. `1.0` segue
        // digitável (`PARAM_HARD_MAX`).
        max: 0.01,
        // O passo acompanha o curso: mantê-lo em `0,001` deixaria o slider com DEZ
        // degraus — trocar um curso ilegível por um seletor grosseiro não é conserto.
        step: 0.0001,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "direction",
        label: "Direction",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        // The family's three words, in the family's order.
        widget: ParamWidget::Enum {
            labels: &["Rise", "Fall", "Both"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    /// The premise the pre-direction fixtures DECLARE instead of inheriting: they
    ///measure the epsilon and the priming, not the selector.
    const ANY: ChangeDir = ChangeDir::Both;

    fn value(v: f32) -> Stream {
        Stream::new(1).with(VALUE_COL, Column::Scalar(vec![v]))
    }
    fn fired(s: &Stream) -> f32 {
        match s.get(PULSE_COL).unwrap() {
            Column::Scalar(v) => v[0],
            _ => panic!(),
        }
    }

    /// The core: a pulse fires the tick the value STEPS, and only then — a held
    /// value stays quiet. A staircase input fires once per step.
    #[test]
    fn it_fires_on_a_step_and_stays_quiet_while_held() {
        let mut state = Stream::new(1);
        // Prime on tick 0 (records 5, never fires).
        state = step(&value(5.0), &state, 0.001, ANY);
        assert_eq!(fired(&state), 0.0, "the first tick primes, never fires");
        // Held at 5 → no change → quiet.
        state = step(&value(5.0), &state, 0.001, ANY);
        assert_eq!(fired(&state), 0.0, "a held value is quiet");
        // Steps to 7 → fires once.
        state = step(&value(7.0), &state, 0.001, ANY);
        assert_eq!(fired(&state), 1.0, "the step fires");
        // Holds at 7 → quiet again (fired ONLY on the step, not while high).
        state = step(&value(7.0), &state, 0.001, ANY);
        assert_eq!(fired(&state), 0.0, "quiet once the new value is held");
    }

    /// FALSIFICATION with a staircase: a 3-step ramp fires exactly 3 times (once
    /// per step), not once per tick. A level detector would behave differently.
    #[test]
    fn a_staircase_fires_once_per_step() {
        // value 0,0,0,1,1,1,2,2,2 — steps at indices 3 and 6 (index 0 primes).
        let seq = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
        let mut state = Stream::new(1);
        let mut out = Vec::new();
        for &v in &seq {
            state = step(&value(v), &state, 0.001, ANY);
            out.push(fired(&state));
        }
        assert_eq!(out, vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    /// The `epsilon` guard swallows dither: a wobble smaller than epsilon is NOT a
    /// change; a step larger than it IS. Exact-equality noise never chatters.
    #[test]
    fn epsilon_ignores_dither_but_not_a_real_step() {
        let mut state = step(&value(1.0), &Stream::new(1), 0.05, ANY); // prime → 1.0
        // A 0.02 wobble (< 0.05 epsilon) → no fire.
        state = step(&value(1.02), &state, 0.05, ANY);
        assert_eq!(fired(&state), 0.0, "sub-epsilon dither is not a change");
        // A 0.2 step (> epsilon) → fires. (prev is 1.02 now.)
        state = step(&value(1.3), &state, 0.05, ANY);
        assert_eq!(fired(&state), 1.0, "a real step fires");
    }

    /// Unary over the FIELD: each instance watches its own value. Two dots, only
    /// one of which steps → only that one's pulse fires.
    #[test]
    fn it_watches_each_instance_of_the_field_independently() {
        let two = |a: f32, b: f32| Stream::new(2).with(VALUE_COL, Column::Scalar(vec![a, b]));
        let mut state = step(&two(1.0, 2.0), &Stream::new(2), 0.001, ANY); // prime
        state = step(&two(1.0, 9.0), &state, 0.001, ANY); // only dot 1 steps
        match state.get(PULSE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 1.0], "only the changed dot fires"),
            _ => panic!(),
        }
    }

    /// Walk a staircase that goes UP twice and DOWN twice, and read which steps
    /// each direction claims. The three answers PARTITION the four steps — that is
    /// what makes this a selector and not three unrelated knobs.
    #[test]
    fn it_fires_only_on_the_direction_it_was_asked_for() {
        // primes at 0, then +1, +1, −1, −1.
        let seq = [0.0, 1.0, 2.0, 1.0, 0.0];
        let run = |dir: ChangeDir| {
            let mut state = Stream::new(1);
            let mut out = Vec::new();
            for &v in &seq {
                state = step(&value(v), &state, 0.001, dir);
                out.push(fired(&state));
            }
            out
        };
        let up = run(ChangeDir::Rise);
        let down = run(ChangeDir::Fall);
        let both = run(ChangeDir::Both);
        assert_eq!(
            up,
            vec![0.0, 1.0, 1.0, 0.0, 0.0],
            "only the steps that rise"
        );
        assert_eq!(
            down,
            vec![0.0, 0.0, 0.0, 1.0, 1.0],
            "only the steps that fall"
        );
        assert_eq!(both, vec![0.0, 1.0, 1.0, 1.0, 1.0], "every step");
        // The partition: nothing is claimed twice, nothing is dropped.
        for i in 0..seq.len() {
            assert_eq!(
                up[i] + down[i],
                both[i],
                "step {i} belongs to exactly one direction"
            );
        }
    }

    /// **`Both` is the world before the param**, and the oracle is the old law
    /// written out by hand — `|v − prev| > epsilon`, with no selector anywhere.
    /// Equality is exact: the neutral arm contributes no arithmetic, so this is
    /// not "looks the same", it is the same operations in the same order.
    #[test]
    fn the_neutral_is_the_world_before_the_param() {
        let seq = [3.0, 3.0, 7.0, 7.0, 2.0, 2.0001, -5.0, -5.0];
        let mut state = Stream::new(1);
        let through_the_node: Vec<f32> = seq
            .iter()
            .map(|&v| {
                state = step(&value(v), &state, 0.001, ChangeDir::Both);
                fired(&state)
            })
            .collect();

        let (mut prev, mut primed) = (0.0f32, false);
        let old_law: Vec<f32> = seq
            .iter()
            .map(|&v| {
                let out = if primed && (v - prev).abs() > 0.001 {
                    1.0
                } else {
                    0.0
                };
                prev = v;
                primed = true;
                out
            })
            .collect();
        assert_eq!(
            through_the_node, old_law,
            "Both is the pre-param law, tick for tick"
        );
    }

    /// An unreadable `direction` falls back to the NEUTRAL, never to a narrower
    /// one — a number nobody can parse must not silently stop a node from firing.
    #[test]
    fn an_unreadable_direction_falls_back_to_the_neutral() {
        for bad in [-1.0f32, 3.0, 99.0, f32::NAN] {
            assert!(
                ChangeDir::from_param(bad).fires(-1.0) && ChangeDir::from_param(bad).fires(1.0),
                "{bad} must not narrow the node"
            );
        }
        // And the manifest's own default resolves to that same neutral.
        let default = MANIFEST
            .params
            .iter()
            .find(|p| p.name == "direction")
            .expect("the selector is declared")
            .default;
        assert!(ChangeDir::from_param(default).fires(-1.0));
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
