//! `value.math` — the value-domain COMBINER: fold two **value** fields into one
//! with an arithmetic operation (Motion Nodes M2, the value domain — doc 12/14).
//! This is the first node that takes TWO value fields, so it is where the doc-12
//! broadcast rule is first exercised between two *fields* (until now only the
//! consumer `motion.drive` broadcast, and only a value against the transform
//! stream). It is TouchDesigner's **Math CHOP** (Combine), Cavalry's **Math**,
//! Nuke's Merge(math): one node, an operation selector — not a per-op node
//! explosion (the mature node editors all converge on the single multi-op node,
//! and the value domain's whole ethos is *author once*, doc 12).
//!
//! **The one broadcast rule (the load-bearing decision, doc 12):** the output is
//! `max(len_a, len_b)` long; a length-1 field is HELD (broadcast) at every index;
//! two length-N fields combine element-wise; any other pairing is a mismatch
//! (`debug_assert`, then a lenient element-wise read — no silent no-op), exactly
//! the `1→N`-only rule `motion.drive` uses. It is what makes
//! `value.instance_field × value.lfo → …` — a per-element **spatial gradient
//! modulated in time** — a single wire: the length-N gradient times the length-1
//! global oscillation, broadcast. A disconnected input reads as the zero field
//! `{0}` (the additive-identity degenerate field, matching `motion.drive`'s
//! `value_at`).
//!
//! **The value type** is the continuous per-instance field `(Instances, Scalar,
//! Frame)` on the `v` column (doc 12). `Pure` (no clock, no state — a straight
//! combinator). Transcendental-free (HR-5): only `+ − × ÷` and `min`/`max`;
//! division is IEEE-deterministic and guarded against a (near-)zero divisor
//! (collapses to `0.0`) so a downstream field never sees `inf`/`NaN`.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, both inputs and the output (the canonical `value`-domain
/// column).
const VALUE_COL: &str = "v";
/// Below this magnitude a divisor is treated as zero (the quotient collapses to
/// `0.0`), so `Divide` never emits `inf`/`NaN` into a downstream field.
const MIN_DIVISOR: f32 = 1e-9;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.math"),
    name: "value.math",
    inputs: &[
        PortSpec {
            name: "a",
            ty: VALUE,
        },
        PortSpec {
            name: "b",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Add · 1 Subtract · 2 Multiply · 3 Divide · 4 Min · 5 Max · 6 Modulo ·
        // 7 Floored Modulo — the reference-convergent core (TD Math CHOP / Cavalry
        // Math / Blender Math). Power/log are omitted by HR-5.
        // 8..13 are the COMPARISONS (Blender *Compare*'s six modes, in its order),
        // which fold the same pair into a 0/1 MASK — see [`Op`].
        ParamSpec {
            name: "op",
            default: 0.0,
        },
        // The equality TOLERANCE (`Equal`/`Not Equal` only — `ParamGate`d, so it is
        // not painted under an op that never reads it). Blender's Compare node ships
        // the same param under the same two modes, with the same 0.001 default.
        ParamSpec {
            name: "epsilon",
            default: 0.001,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The arithmetic operation two value fields are folded by (the TD Math CHOP /
/// Cavalry Math combine vocabulary, restricted to the HR-5-safe core).
#[derive(Copy, Clone, PartialEq, Eq)]
enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
    Min,
    Max,
    /// The remainder with the sign of the DIVIDEND (C `fmod`, Houdini `%`,
    /// Blender *Modulo*): `−7 % 3 = −1`.
    Modulo,
    /// The remainder with the sign of the DIVISOR (Python `%`, GLSL `mod`,
    /// Blender *Floored Modulo*): `−7 mod 3 = 2`. For a positive divisor the
    /// result always lands in `[0, b)` — this is the one a WRAP wants, and the
    /// reason both ship instead of one.
    FlooredModulo,
    /// `a < b`
    Less,
    /// `a <= b`
    LessOrEqual,
    /// `a > b`
    Greater,
    /// `a >= b`
    GreaterOrEqual,
    /// `|a − b| <= epsilon` — equality WITH a tolerance, because exact float
    /// equality on a computed field is a coin toss.
    Equal,
    /// `|a − b| > epsilon`
    NotEqual,
}

impl Op {
    /// Does this op fold the pair into a **0/1 mask** rather than a number?
    ///
    /// ⚠️ **Test-only, and the reason is worth stating so nobody "promotes" it:**
    /// the natural second caller would be [`PARAM_GATES`], but that is a `static`
    /// with a literal array — a `const fn` cannot fill it — and it answers a
    /// DIFFERENT question anyway (*which ops read the tolerance?* is the two
    /// equality ops, not all six comparisons). So this classifier exists for the
    /// gate that asserts the mask law, and pretending it is a shared door would be
    /// a doc-comment the code does not honour.
    #[cfg(test)]
    const fn is_comparison(self) -> bool {
        matches!(
            self,
            Op::Less
                | Op::LessOrEqual
                | Op::Greater
                | Op::GreaterOrEqual
                | Op::Equal
                | Op::NotEqual
        )
    }
}

impl Op {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Op::Subtract,
            2 => Op::Multiply,
            3 => Op::Divide,
            4 => Op::Min,
            5 => Op::Max,
            6 => Op::Modulo,
            7 => Op::FlooredModulo,
            8 => Op::Less,
            9 => Op::LessOrEqual,
            10 => Op::Greater,
            11 => Op::GreaterOrEqual,
            12 => Op::Equal,
            13 => Op::NotEqual,
            _ => Op::Add,
        }
    }
    /// Combine one pair of samples. Division by a (near-)zero divisor collapses
    /// to `0.0` rather than producing `inf`/`NaN` — the documented guard, and the
    /// two moduli share it (they divide too).
    ///
    /// ⚠️ **The two moduli are computed in DERIVED form** (`a − b·trunc(a/b)` and
    /// `a − b·floor(a/b)`), never by Rust's `%`. Two reasons, both load-bearing:
    /// `%` on `f32` lowers to a **`fmodf` call in the libm** (the same class of
    /// cost the drying pass of the Wet Paint measured at 2.51 ns against 0.54),
    /// and — the one that decides it — `fmod` is *exact* (computed as if in
    /// infinite precision) while `trunc(a/b)` rounds, so an exact CPU against a
    /// derived WGSL would disagree at the ulp on exactly the inputs a wrap lands
    /// on. Deriving on BOTH sides makes the parity bit-for-bit by construction.
    fn apply(self, a: f32, b: f32, eps: f32) -> f32 {
        match self {
            Op::Add => a + b,
            Op::Subtract => a - b,
            Op::Multiply => a * b,
            Op::Divide => {
                if b.abs() < MIN_DIVISOR {
                    0.0
                } else {
                    a / b
                }
            }
            Op::Min => a.min(b),
            Op::Max => a.max(b),
            Op::Modulo => {
                if b.abs() < MIN_DIVISOR {
                    0.0
                } else {
                    a - b * (a / b).trunc()
                }
            }
            Op::FlooredModulo => {
                if b.abs() < MIN_DIVISOR {
                    0.0
                } else {
                    a - b * (a / b).floor()
                }
            }
            // The six comparisons. Each is a DIRECT IEEE comparison, spelled the
            // same way on both sides (`select(0.0, 1.0, cond)` in WGSL), so parity
            // is bit-for-bit by construction: the operands are the same bits and
            // IEEE-754 specifies the predicate exactly.
            //
            // ⚠️ `Not Equal` is `|a−b| > eps`, NOT `!(|a−b| <= eps)`. The two forms
            // agree everywhere except at NaN, where every comparison is false and
            // the negation would flip — and a NaN that reached here would then be
            // reported *not equal to itself* on one side and *equal* on the other.
            // Mirroring the hardware on both sides is the one spelling that cannot
            // disagree. (A NaN is unreachable anyway: the divisor guard above is
            // exactly what keeps one out of a downstream field.)
            Op::Less => f32::from(a < b),
            Op::LessOrEqual => f32::from(a <= b),
            Op::Greater => f32::from(a > b),
            Op::GreaterOrEqual => f32::from(a >= b),
            Op::Equal => f32::from((a - b).abs() <= eps),
            Op::NotEqual => f32::from((a - b).abs() > eps),
        }
    }
}

/// The sample of value field `v` at index `i`, applying the `1→N` broadcast
/// rule: a length-1 field is held at every index; a length-N field is read
/// element-wise; a missing field reads as `0.0` (the zero degenerate field).
fn field_at(v: &[f32], i: usize) -> f32 {
    match v.len() {
        0 => 0.0,
        1 => v[0], // broadcast: one value → every index (the 1→N rule)
        _ => v.get(i).copied().unwrap_or(0.0),
    }
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Combine two value fields by `op` under the broadcast rule. Output length is
/// `max(len_a, len_b)`; a length that is neither 1 nor the other's length is a
/// mismatch — `debug_assert`ed loudly, then read leniently (element-wise, `0.0`
/// past the end) so a release build degrades rather than panics.
fn combine(a: &[f32], b: &[f32], op: Op, eps: f32) -> Vec<f32> {
    debug_assert!(
        a.len() <= 1 || b.len() <= 1 || a.len() == b.len(),
        "value.math: field lengths {} and {} are neither broadcast (1) nor equal",
        a.len(),
        b.len()
    );
    let n = a.len().max(b.len());
    (0..n)
        .map(|i| op.apply(field_at(a, i), field_at(b, i), eps))
        .collect()
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`Op::apply`] under the
/// broadcast rule.
///
/// **The third count law, and the first that reads more than one port.** The
/// engine's default — *"as wide as port 0"* — is wrong here in the case the node
/// exists for: `value.instance_field × value.lfo` is a length-N field times a
/// length-1 one, and port 0 is whichever the artist happened to wire first. The
/// output is `max(len_a, len_b)`, which is the same expression `combine` uses,
/// and it has to be the same expression: two laws that disagree do not crash,
/// they render a different number of things.
///
/// Both inputs are [`ColumnAccess::ReadBroadcast`], which is the `1 => v[0]` arm
/// of the CPU's `field_at` decided per dispatch from a uniform bit — so one
/// compiled module serves *field × field* and *field × global* alike.
///
/// An unconnected port is absent, reads its identity `0.0`, and that is exactly
/// the CPU's `0 => 0.0` zero degenerate field.
///
/// `vm_round` is round-half-AWAY-from-zero to match Rust's `f32::round`: `op`
/// picks a BRANCH, so a half-even disagreement would select a different
/// operation, not shift a value by an ε.
///
/// **The comparisons are `select`, and the CPU is `f32::from(bool)`** — the two
/// spellings of the same thing, and the parity is bit-for-bit by CONSTRUCTION
/// rather than by tuning: the operands are the same bits, IEEE-754 specifies each
/// predicate exactly, and both sides produce the literals `0.0`/`1.0`. There is
/// no epsilon in the ORDER comparisons for the same reason a `>` needs none, and
/// the equality pair reads `params.epsilon` — the one param the artist only sees
/// under those two ops.
///
/// The divisor guard is a branch and not a `select`, mirroring the CPU
/// literally. Its threshold is spelled twice — [`MIN_DIVISOR`] here and the
/// literal `1e-9` in the WGSL — because a `&'static str` cannot interpolate a
/// Rust constant, and a THRESHOLD is the worst kind of number to spell twice: a
/// divisor landing between two spellings takes a different arm on each side and
/// nothing else about the graph changes. Since the language cannot pin them, a
/// GATE does — `dividing_by_a_threshold_divisor_agrees_on_both_sides` straddles
/// it, and it is the reason to edit both lines or neither.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vm_a = read_a_v(i);\n\
        let vm_b = read_b_v(i);\n\
        let vm_op = i32(vm_round(params.op));\n\
        var vm_r = vm_a + vm_b;\n\
        if (vm_op == 1) {\n\
        \x20   vm_r = vm_a - vm_b;\n\
        } else if (vm_op == 2) {\n\
        \x20   vm_r = vm_a * vm_b;\n\
        } else if (vm_op == 3) {\n\
        \x20   if (abs(vm_b) < 1e-9) { vm_r = 0.0; } else { vm_r = vm_a / vm_b; }\n\
        } else if (vm_op == 4) {\n\
        \x20   vm_r = min(vm_a, vm_b);\n\
        } else if (vm_op == 5) {\n\
        \x20   vm_r = max(vm_a, vm_b);\n\
        } else if (vm_op == 6) {\n\
        \x20   if (abs(vm_b) < 1e-9) { vm_r = 0.0; } else { vm_r = vm_a - vm_b * trunc(vm_a / vm_b); }\n\
        } else if (vm_op == 7) {\n\
        \x20   if (abs(vm_b) < 1e-9) { vm_r = 0.0; } else { vm_r = vm_a - vm_b * floor(vm_a / vm_b); }\n\
        } else if (vm_op == 8) {\n\
        \x20   vm_r = select(0.0, 1.0, vm_a < vm_b);\n\
        } else if (vm_op == 9) {\n\
        \x20   vm_r = select(0.0, 1.0, vm_a <= vm_b);\n\
        } else if (vm_op == 10) {\n\
        \x20   vm_r = select(0.0, 1.0, vm_a > vm_b);\n\
        } else if (vm_op == 11) {\n\
        \x20   vm_r = select(0.0, 1.0, vm_a >= vm_b);\n\
        } else if (vm_op == 12) {\n\
        \x20   vm_r = select(0.0, 1.0, abs(vm_a - vm_b) <= params.epsilon);\n\
        } else if (vm_op == 13) {\n\
        \x20   vm_r = select(0.0, 1.0, abs(vm_a - vm_b) > params.epsilon);\n\
        }\n\
        write_v(i, vm_r);\n",
    wgsl_lib: "\
        fn vm_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["op", "epsilon"],
    count_law: Some(math_count),
    variant_by_param: None,
    applicable: None,
};

/// **How wide is the output?** — `max(len_a, len_b)`, the same expression
/// `combine` uses. Written as a `max` over every port rather than over two named
/// ones: the law is *"as wide as the widest input"*, and spelling it that way
/// means it stays correct rather than merely true.
fn math_count(ctx: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(ctx.inputs.iter().copied().max().unwrap_or(0) as usize)
}

struct ValueMath;

impl NodeOp for ValueMath {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let op = Op::from_param(ctx.param("op"));
        let a = scalar_col(ctx.input(0), VALUE_COL);
        let b = scalar_col(ctx.input(1), VALUE_COL);
        let out = combine(&a, &b, op, ctx.param("epsilon"));
        ctx.emit(Stream::new(out.len()).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueMath))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Math",
            // Utility grey: a value→value combiner, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // `epsilon` is a TOLERANCE, and only the two equality ops have anything to be
    // tolerant about. Painted under those and nowhere else — a control that does
    // nothing is not painted (doc 88 B3, the `amount_y` law of `motion.scale`).
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamHardMax, ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "op",
        label: "Op",
        min: 0.0,
        max: 13.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            // ⚠️ The six comparisons are in **Blender's Compare order** (Less Than,
            // Less Than or Equal, Greater Than, Greater Than or Equal, Equal, Not
            // Equal), not in an order of our own: an artist who knows the reference
            // finds them where they expect. The arithmetic core keeps indices 0..7
            // untouched, so every document already authored reads the same op.
            labels: &[
                "Add",
                "Subtract",
                "Multiply",
                "Divide",
                "Min",
                "Max",
                "Modulo",
                "Floored Mod",
                "Less",
                "Less or Eq",
                "Greater",
                "Greater or Eq",
                "Equal",
                "Not Equal",
            ],
        },
    },
    ParamUiHint {
        param: "epsilon",
        label: "Epsilon",
        // ⚠️ **The soft max is DERIVED, not chosen**, and a convention gate produced
        // the number: the panel's track is 154 px and the mapping is linear, so
        // `span / 154` is the smallest step a DRAG can make — above
        // `span / default` the smallest drag already jumps past the default itself.
        // With `default = 0.001` that puts the ceiling at `0.154`; `0.1` is the
        // round number under it. (Written as `1.0` first, and
        // `the_slider_drags_where_the_hand_works` refused it.)
        //
        // ⚠️ And it is NOT the ceiling: a tolerance has no resource behind it — it
        // is a distance in the field's OWN unit, and a value field is unbounded —
        // so the typed entry reaches as far as a quantity in this library goes (the
        // `PARAM_HARD_MAX` below, the emitter's `rate` number for the same reason).
        // A field authored in the hundreds wants an epsilon in the units.
        min: 0.0,
        max: 0.1,
        step: 0.001,
        widget: ParamWidget::Slider,
    },
];

/// `epsilon` exists only for the two ops that ask about EQUALITY (indices 12/13 —
/// [`Op::is_comparison`]'s equality half). Under `Add` or `Greater` it would be a
/// row the artist can drag that changes nothing.
static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: "epsilon",
    when: "op",
    values: &[12, 13],
}];

/// The typed ceiling of the tolerance — see the hint above for why the slider stops
/// at `0.1` (a derived number) and the box does not.
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "epsilon",
    max: 1_000_000.0,
}];
#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
