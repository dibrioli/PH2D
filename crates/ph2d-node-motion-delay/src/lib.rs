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
//! ## Subir e descer são dois tempos (`ticks_down`)
//!
//! O **Lag CHOP** do TouchDesigner — a referência que o próprio `value.smooth` cita — tem *Lag
//! up* e *Lag down* separados, e é o que faz de um smoother um envelope: um medidor que salta
//! e desce devagar, uma luz que acende no ato e apaga com calma. Aqui um `ticks` governava as
//! duas direções.
//!
//! `ticks_down` é o tempo da DESCIDA, e vale **só no `Blend`** — é o único modo em que o número
//! é uma TAXA. O `Delay` é uma consulta ao passado e o `Average` é um boxcar: nenhum dos dois
//! tem direção sobre a qual perguntar, e um controle pintado ali seria um knob morto (daí o
//! [`ParamGate`]).
//!
//! ⚠️ **`0` significa *"o mesmo da subida"*, e nada se torna inalcançável por isso** — a
//! sentinela custaria um valor de verdade se `0` fosse um tempo pedível, e ele não é: a lei do
//! one-pole já divide por `ticks.max(1.0)`, então **`ticks_down = 1` JÁ É a descida
//! instantânea**. O par alternativo (um toggle *Same | Custom* mais o número, o idioma do
//! `Mass: Auto | Manual` da física) seria dois params para uma grandeza que a sentinela
//! exprime sem perda.
//!
//! ⚠️ **A direção é decidida POR COMPONENTE**, não pela posição inteira: uma cor que clareia no
//! vermelho e escurece no azul usa os dois tempos no mesmo tick. É o que um Lag CHOP faz (ele
//! é por-canal), e a alternativa — decidir pela norma do vetor — faria o eixo que anda pouco
//! herdar a direção do que anda muito.
//!
//! ## What is delayed — the `channel`
//!
//! C4D's Delay Effector lags a set *"with regard to **position, scale and rotation**"*, and the
//! Delay **Field** adds *"**colour** changes"*. Ours lagged position and nothing else, and a second
//! `motion.delay` downstream just lagged position again — so the other three quantities were not
//! reachable through the graph at all.
//!
//! `channel` picks one, and **chaining is how you get several**: `delay(Position) → delay(Rotation)`
//! is two lines with their own `ticks`, which is strictly more than C4D's shared setting.
//!
//! ⚠️ **The first four indices are the catalogue's shared channel vocabulary** — `0` X, `1` Y,
//! `2` Rotation, `3` Size, the same numbers `motion.step` / `noise` / `oscillator` / `stagger` /
//! `wiggle` / `drive` mean. Keeping them is not cosmetic: the index is what the *document* stores,
//! and those crates each carry a copy of the same `channel.rs` that a later wave will want to
//! collapse into one door. A node that re-numbered the shared prefix would turn that collapse into
//! a silent behaviour change in every saved graph. Past the prefix each node extends on its own —
//! `motion.drive` already added Opacity/Falloff/Hue/Sat/Val at `4..8`; this one adds **`4` Position
//! and `5` Colour**, the two *whole quantities* the reference names.
//!
//! **Position is the default**, because it is what this node did before the param existed.
//!
//! ⚠️ And the magnitude param does **not** need [`ParamUnit::FromChannel`]: `ticks` is a count of
//! ticks on every channel. That variant exists for behaviours whose *amount* changes unit with the
//! channel (metres / degrees / scale); a duration does not.
//!
//! The multiplicative `falloff` field blends the output between the live value (0) and the delayed
//! one (1), so a falloff makes the smoothing itself local.
//!
//! ## The filter does not care what it is lagging
//!
//! Everything works on **`dim` floats per element**: a position is two, an angle is one, a colour
//! is four, and lerp / boxcar / one-pole are the same arithmetic per component. The channel only
//! says which column, which components, and how wide — which is why Position stayed byte-identical
//! through the change.
//!
//! ⚠️ Two things do care. **An angle is a circle and a filter is not** — see [`unwrap_live`]. And
//! **a line is only history if it is history of the same quantity**: the state carries the channel
//! it was built for, because Position and Size are both `Vec2` and the types would not object.
//!
//! Sequential (the `pre` self-loop the editor plumbs on drop), `Effect::Pure` (the tick enters the
//! fingerprint through the consumed `pre` edge), HR-5: arithmetic only.

use ph2d_node_registry::{
    NodeRegistry, ParamGate, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
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

/// `channel`: the shared prefix (`0..3`) plus this node's two whole quantities.
const CH_X: i32 = 0;
const CH_Y: i32 = 1;
const CH_ROTATION: i32 = 2;
const CH_SIZE: i32 = 3;
const CH_COLOR: i32 = 5;

/// **Which slice of which column the line is made of.** A channel is a column name plus a window
/// into its components, and that is the whole of what the filter needs to know.
struct Chan {
    col: &'static str,
    /// The first component of the column this channel owns.
    lo: usize,
    /// How many components — 1 for an angle or a single axis, 2 for a point, 4 for a colour.
    dim: usize,
    /// Degrees on a circle. See [`unwrap_live`].
    angular: bool,
}

/// The channel table. An unknown index falls through to **Position**, the default: a hand-edited
/// value cannot ask for a column that does not exist.
fn chan(channel: i32) -> Chan {
    match channel {
        CH_X => Chan {
            col: "P",
            lo: 0,
            dim: 1,
            angular: false,
        },
        CH_Y => Chan {
            col: "P",
            lo: 1,
            dim: 1,
            angular: false,
        },
        CH_ROTATION => Chan {
            col: "rot",
            lo: 0,
            dim: 1,
            angular: true,
        },
        CH_SIZE => Chan {
            col: "size",
            lo: 0,
            dim: 2,
            angular: false,
        },
        CH_COLOR => Chan {
            col: "tint",
            lo: 0,
            dim: 4,
            angular: false,
        },
        _ => Chan {
            col: "P",
            lo: 0,
            dim: 2,
            angular: false,
        },
    }
}

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
        // 0 X · 1 Y · 2 Rotation · 3 Size · 4 Position · 5 Colour — the catalogue's shared prefix,
        // extended (see the header). **4 is the default**: it is what this node did before the
        // param existed, so a saved graph that never set it is byte-identical.
        ParamSpec {
            name: "channel",
            default: 4.0,
        },
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
        // O tempo da DESCIDA (só no `Blend`). **`0` = o mesmo da subida**, e é
        // por isso que um grafo salvo que nunca o escreveu é byte-idêntico — ver
        // o cabeçalho para por que a sentinela não torna nada inalcançável.
        ParamSpec {
            name: "ticks_down",
            default: 0.0,
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

/// The channel's live values, `dim` floats per element — `None` when the column is absent or too
/// narrow for the slice the channel asks for.
///
/// **You cannot lag a quantity that is not there**, so the node hands its input through untouched
/// rather than materialising one. Writing an identity `size` where the artist had none would ADD a
/// column downstream, and a delayed constant is the constant anyway.
fn read_channel(s: &Stream, c: &Chan) -> Option<Vec<f32>> {
    let (all, w) = s.get(c.col).map(ring::flat)?;
    let n = s.count();
    if c.lo + c.dim > w || all.len() < n * w {
        return None;
    }
    let mut out = Vec::with_capacity(n * c.dim);
    for i in 0..n {
        let at = i * w + c.lo;
        out.extend_from_slice(&all[at..at + c.dim]);
    }
    Some(out)
}

/// Write the channel's values back into its slice of the column, leaving every other component
/// where it was — the X channel lags `P.x` and hands `P.y` through live.
fn write_channel(out: &mut Stream, input: &Stream, c: &Chan, vals: &[f32]) {
    let Some((mut all, w)) = input.get(c.col).map(ring::flat) else {
        return;
    };
    for i in 0..input.count() {
        for k in 0..c.dim {
            all[i * w + c.lo + k] = vals[i * c.dim + k];
        }
    }
    out.set(c.col, ring::unflat(all, w));
}

/// A full turn of the `rot` column, whose unit is **degrees** (`motion.look_at` writes
/// `atan2 · 180/π`).
const TURN: f32 = 360.0;

/// Continue an ANGULAR channel's live value from where the line left it.
///
/// An angle is a circle and a filter is not. `motion.look_at` writes an `atan2`, so a target
/// circling an element makes `rot` saw-tooth between `+180` and `−180` — and a one-pole handed
/// those two numbers eases the **long** way, unwinding almost a whole turn over `ticks` frames.
/// Adding the *short* step to the previous unwrapped value keeps the filter working on a
/// continuous quantity, with the entire node unchanged behind it. The line stores the unwrapped
/// value, so the continuation compounds correctly across ticks.
///
/// HR-5: a rounding and a multiply, no call. The limit is the honest one every unwrapper has — an
/// element that turns more than half a turn in ONE tick (>10800°/s at 60 Hz) is aliased, and the
/// short way is then the wrong way.
fn unwrap_live(live: &mut [f32], prev_live: &[f32]) {
    for (v, p) in live.iter_mut().zip(prev_live) {
        let d = *v - p;
        *v = p + (d - TURN * (d / TURN).round());
    }
}

/// The delayed value of one component, by mode.
///
/// `past[k-1][j]` is what it was `k` ticks ago; `prev[j]` is what this node emitted for it last
/// tick (the one-pole's own state).
///
/// ⚠️ `down` é a régua da DESCIDA e só o `Blend` a lê — ver o cabeçalho.
fn delayed(
    mode: i32,
    ticks: f32,
    down: f32,
    j: usize,
    live: f32,
    past: &[Vec<f32>],
    prev: &[f32],
) -> f32 {
    match mode {
        MODE_DELAY => {
            // Fractional lookback: 3.4 ticks back is 60% of the way from slot 3 toward slot 4.
            let whole = ticks.floor();
            let frac = ticks - whole;
            let k = (whole as usize).clamp(1, MAX_LAG); // CLAMP-OK: guarded by the caller's `ticks > 0`
            let near = past[k - 1][j];
            let far = past[k.min(MAX_LAG - 1)][j];
            near + (far - near) * frac
        }
        MODE_AVERAGE => {
            // A boxcar over the window, INCLUDING the live sample — a mean that ignored `now`
            // would lag by a whole extra tick and, at `ticks = 1`, would just be `Delay`.
            let n = (ticks.round() as usize).clamp(1, MAX_LAG); // CLAMP-OK: the ring's bounds
            let mut sum = live;
            for p in past.iter().take(n) {
                sum += p[j];
            }
            sum * (1.0 / (n + 1) as f32)
        }
        // Blend: the one-pole. `out += (live - out) / ticks` — it approaches and never passes, and
        // that is the whole difference between it and `motion.spring`.
        //
        // ⚠️ **A régua é escolhida pela direção DESTE componente**: `live < prev` é
        // uma descida. `down <= 0` = a sentinela *"o mesmo da subida"*, e nesse
        // caso a expressão é literalmente a de antes.
        _ => {
            let rule = if down > 0.0 && live < prev[j] {
                down
            } else {
                ticks
            };
            prev[j] + (live - prev[j]) * (1.0 / rule.max(1.0))
        }
    }
}

struct MotionDelay;

impl NodeOp for MotionDelay {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i32;
        let channel = ctx.param("channel").round() as i32;
        // 0 = no lag at all. Clamped to the ring, so a hand-edited 900 cannot ask for state that
        // does not exist.
        let ticks = ctx.param("ticks").clamp(0.0, MAX_LAG as f32); // CLAMP-OK: the ring's depth
        // 0 = a sentinela "o mesmo da subida"; negativo é lixo de documento e cai
        // nela também.
        let ticks_down = ctx.param("ticks_down").clamp(0.0, MAX_LAG as f32); // CLAMP-OK: the ring's depth

        let out = {
            let input = ctx.input(0);
            let state = ctx.input(1);
            let c = chan(channel);

            // Everything but this node's own state columns passes through untouched — every
            // channel the artist did not pick stays LIVE (echoing whole rows is what
            // `motion.trail` is for).
            let mut out = Stream::new(input.count());
            for (name, col) in input.columns() {
                if !ring::is_state(name) {
                    out.set(name.clone(), col.clone());
                }
            }

            match read_channel(input, &c) {
                // Nothing to lag, and so no line worth keeping: pure pass-through.
                None => out,
                Some(mut live) => {
                    // A line built for another channel is not old history, it is **someone
                    // else's** — Position and Size are both `Vec2`, so the types would not
                    // object and the sprites would inherit a position line as their scale.
                    let empty = Stream::new(0);
                    let hist = if ring::is_for_channel(state, channel) {
                        state
                    } else {
                        &empty
                    };
                    let rows = ring::rows_of(hist, input);
                    let past = ring::past(hist, &rows, &live, c.dim);
                    let prev = ring::prev_out(hist, &rows, &live, c.dim);

                    if c.angular {
                        // `past[0]` is one tick ago, already unwrapped (and a newborn's is its own
                        // live value, so the first tick is the identity).
                        unwrap_live(&mut live, &past[0]);
                    }

                    // **The neutral point is byte-identical.** No lag, no smoothing, nothing to
                    // compute — the node is transparent until it is asked for something.
                    let emitted: Vec<f32> = if ticks <= 0.0 {
                        live.clone()
                    } else {
                        (0..live.len())
                            .map(|j| {
                                let d = delayed(mode, ticks, ticks_down, j, live[j], &past, &prev);
                                // The field gates the EFFECT, not the state: the line keeps
                                // filling regardless, so a falloff that opens later does not
                                // start from nothing.
                                let f = falloff_at(input, j / c.dim);
                                live[j] + (d - live[j]) * f
                            })
                            .collect()
                    };

                    write_channel(&mut out, input, &c, &emitted);
                    ring::push(&mut out, past, &live, &emitted, c.dim, channel);
                    out
                }
            }
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
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    // CPU-only: this node reads `falloff` only at eval runtime (no GPU kernel), so the
    // diagnoser cannot derive the role from a `ColumnBinding` — declare it (ADR-0155).
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("falloff")],
    );
    Ok(())
}

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "Channel",
        min: 0.0,
        max: 5.0,
        step: 1.0,
        // The words the shared prefix already paints, plus the two whole quantities. They are
        // listed in INDEX order — the default (Position) is not first, and that is the price of
        // the shared numbering being worth more than the list reading prettily.
        widget: ParamWidget::Enum {
            labels: &["X", "Y", "Rotation", "Size", "Position", "Colour"],
        },
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        // The `ParamSpec` above already names the three (`0 Delay · 1 Average · 2 Blend`); an
        // `IntSlider` made the artist read those NUMBERS. Seventeen other `mode` params in the
        // catalogue paint words.
        widget: ParamWidget::Enum {
            labels: &["Delay", "Average", "Blend"],
        },
    },
    ParamUiHint {
        param: "ticks",
        label: "Rise",
        min: 0.0,
        max: 32.0,
        step: 0.5,
        widget: ParamWidget::Slider,
    },
    // ⚠️ O rótulo de cima virou **Rise** e não é cosmético: com uma régua de
    // descida ao lado, *"Ticks"* deixou de nomear a única duração do nó.
    ParamUiHint {
        param: "ticks_down",
        label: "Fall",
        min: 0.0,
        max: 32.0,
        step: 0.5,
        widget: ParamWidget::Slider,
    },
];

/// **A descida só existe no `Blend`.** O `Delay` consulta o passado e o `Average`
/// é um boxcar — nenhum dos dois tem direção, e um slider ali não faria nada.
static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: "ticks_down",
    when: "mode",
    values: &[2],
}];

/// ⚠️ **`ticks` is a `Count` on every channel, not [`ParamUnit::FromChannel`]** — that variant is
/// for a behaviour whose *magnitude* changes unit with what it drives (metres on Position, degrees
/// on Rotation, a bare factor on Size). A duration does not: eight ticks is eight ticks whether it
/// is lagging a point, an angle or a colour. Declaring `FromChannel` here would hand the display
/// boundary a length conversion to apply to a frame count.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "ticks",
        unit: ParamUnit::Count,
    },
    ParamUnitDecl {
        param: "ticks_down",
        unit: ParamUnit::Count,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
