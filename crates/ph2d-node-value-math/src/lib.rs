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
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Direct unit tests of the core (no cook needed for the arithmetic).

    /// Each op folds a pair the way its name says — the reference-convergent
    /// combine vocabulary (TD Math CHOP).
    #[test]
    fn every_op_combines_the_pair_as_named() {
        assert_eq!(Op::Add.apply(2.0, 3.0, 0.0), 5.0);
        assert_eq!(Op::Subtract.apply(2.0, 3.0, 0.0), -1.0);
        assert_eq!(Op::Multiply.apply(2.0, 3.0, 0.0), 6.0);
        assert_eq!(Op::Divide.apply(6.0, 3.0, 0.0), 2.0);
        assert_eq!(Op::Min.apply(2.0, 3.0, 0.0), 2.0);
        assert_eq!(Op::Max.apply(2.0, 3.0, 0.0), 3.0);
    }

    /// **Cada comparação dobra o par numa MÁSCARA 0/1** — as seis, na ordem da
    /// referência, e o resultado é EXATAMENTE `0.0` ou `1.0`.
    ///
    /// ⚠️ A metade do `is_comparison` não é cerimônia: uma máscara é o que cinco
    /// famílias deste grafo consomem (a §5 do CLAUDE.md nomeia-as), e um `0.999`
    /// no lugar de `1.0` não falha em lugar nenhum — ele **dilui** o que quer que
    /// a leia, em silêncio.
    #[test]
    fn every_comparison_folds_the_pair_into_a_zero_or_one_mask() {
        assert_eq!(Op::Less.apply(1.0, 2.0, 0.0), 1.0);
        assert_eq!(Op::Less.apply(2.0, 2.0, 0.0), 0.0);
        assert_eq!(Op::LessOrEqual.apply(2.0, 2.0, 0.0), 1.0);
        assert_eq!(Op::Greater.apply(3.0, 2.0, 0.0), 1.0);
        assert_eq!(Op::Greater.apply(2.0, 2.0, 0.0), 0.0);
        assert_eq!(Op::GreaterOrEqual.apply(2.0, 2.0, 0.0), 1.0);
        assert_eq!(Op::Equal.apply(2.0, 2.0, 0.0), 1.0);
        assert_eq!(Op::NotEqual.apply(2.0, 3.0, 0.0), 1.0);
        // E TODA saída de comparação é um dos dois literais, nunca um número perto
        // deles: varrido sobre pares que cruzam a fronteira nas duas direções.
        for op in [
            Op::Less,
            Op::LessOrEqual,
            Op::Greater,
            Op::GreaterOrEqual,
            Op::Equal,
            Op::NotEqual,
        ] {
            assert!(op.is_comparison(), "a porta única concorda com a lista");
            for (a, b) in [(-3.0, 2.0), (2.0, 2.0), (7.5, 2.0), (0.0, 0.0)] {
                let m = op.apply(a, b, 0.25);
                assert!(m == 0.0 || m == 1.0, "máscara {m} para ({a}, {b})");
            }
        }
        // E os oito aritméticos NÃO são comparações — a porta separa as famílias.
        for op in [Op::Add, Op::Divide, Op::Max, Op::FlooredModulo] {
            assert!(!op.is_comparison());
        }
    }

    /// **A igualdade é COM tolerância, e a tolerância é LIDA.**
    ///
    /// Falsificável nas duas direções: um kernel que ignorasse `eps` daria a mesma
    /// máscara para 0,001 e para 0,5, e um que o lesse com o sinal trocado
    /// inverteria a banda.
    #[test]
    fn equality_is_within_a_tolerance_and_the_tolerance_is_read() {
        // Duas amostras que a igualdade EXATA separa e uma tolerância junta.
        assert_eq!(
            Op::Equal.apply(1.0, 1.05, 0.0),
            0.0,
            "eps 0 = igualdade exata"
        );
        assert_eq!(Op::Equal.apply(1.0, 1.05, 0.1), 1.0, "dentro da banda");
        assert_eq!(Op::Equal.apply(1.0, 1.2, 0.1), 0.0, "fora da banda");
        // A fronteira é FECHADA (`<=`), o que torna `eps = 0` a igualdade exata em
        // vez de "nunca igual" — um `<` ali faria `Equal` com eps 0 ser sempre 0.
        //
        // ⚠️ A fixture usa 1,25 e 0,25 — **potências de dois, exatas em binário** —
        // e não 1,1 e 0,1. Este gate nasceu VERMELHO com aqueles: `(1.0f32 −
        // 1.1).abs()` vale `0.100000024`, que é MAIOR que `0.1`, então o teste
        // media a representação do decimal e não o predicado. *Um gate de
        // FRONTEIRA precisa de operandos cuja diferença seja exatamente o número
        // que ele afirma.*
        assert_eq!(
            Op::Equal.apply(1.0, 1.25, 0.25),
            1.0,
            "a fronteira pertence"
        );
        assert_eq!(
            Op::Equal.apply(1.0, 1.0, 0.0),
            1.0,
            "eps 0 é igualdade exata"
        );
        // E `Not Equal` é o COMPLEMENTO exato nos finitos.
        for (a, b, e) in [(1.0, 1.05, 0.1), (1.0, 1.2, 0.1), (0.0, 0.0, 0.0)] {
            assert_eq!(
                Op::Equal.apply(a, b, e) + Op::NotEqual.apply(a, b, e),
                1.0,
                "({a}, {b}, {e}): as duas máscaras particionam"
            );
        }
    }

    /// **`Not Equal` é a comparação DIRETA, não a negação de `Equal`** — as duas
    /// formas só divergem no NaN, e é lá que o gate mede.
    ///
    /// ⚠️ O ponto não é o NaN em si (o guard do divisor existe justamente para
    /// nenhum chegar aqui): é que a forma escrita tem de ser a MESMA dos dois lados
    /// da fronteira CPU/WGSL. Uma negação em Rust contra um `>` no device
    /// discordaria exatamente aqui, e nada mais no grafo mudaria.
    #[test]
    fn not_equal_is_the_direct_comparison_not_the_negation() {
        let nan = f32::NAN;
        assert_eq!(Op::Equal.apply(nan, 0.0, 0.1), 0.0);
        assert_eq!(
            Op::NotEqual.apply(nan, 0.0, 0.1),
            0.0,
            "toda comparação com NaN é falsa — a negação daria 1.0 aqui"
        );
        // Idem para a ordem: nenhuma das quatro é verdadeira sobre um NaN.
        for op in [Op::Less, Op::LessOrEqual, Op::Greater, Op::GreaterOrEqual] {
            assert_eq!(op.apply(nan, 0.0, 0.0), 0.0);
            assert_eq!(op.apply(0.0, nan, 0.0), 0.0);
        }
    }

    /// **A tolerância não vaza para as comparações de ORDEM** — um `>` não tem do
    /// que ser tolerante, e um epsilon enorme não pode mover a fronteira dele.
    ///
    /// ⚠️ **A fixture tem de conter pares dos DOIS lados de cada predicado**, e esta
    /// nasceu sem: os três primeiros pares têm todos `a <= b`, e sobre eles a
    /// mutação `a > b + eps` devolve *falso* com qualquer epsilon — **verde sobre o
    /// defeito exato que este gate persegue**. Um epsilon somado só muda a resposta
    /// onde a comparação era VERDADEIRA; sem um par com `a > b` não há o que mudar.
    #[test]
    fn the_order_comparisons_ignore_the_tolerance() {
        for op in [Op::Less, Op::LessOrEqual, Op::Greater, Op::GreaterOrEqual] {
            for (a, b) in [
                (1.0, 1.05),
                (2.0, 2.0),
                (-3.0, 7.0),
                // …e os dois que faltavam: `a > b`, onde um epsilon somado morde.
                (7.0, 2.0),
                (-3.0, -7.0),
            ] {
                assert_eq!(
                    op.apply(a, b, 0.0),
                    op.apply(a, b, 1e6),
                    "({a}, {b}): a ordem não lê a tolerância"
                );
            }
        }
    }

    /// **Os oito ops ARITMÉTICOS não são tocados pelo param novo** — o campo
    /// `epsilon` entrou no manifesto e nenhum documento já autorado muda um bit.
    ///
    /// ⚠️ É a metade que torna a wave segura: apender um param a um `NodeManifest`
    /// é aditivo por construção, mas *ser aditivo* e *não ser lido* são coisas
    /// diferentes, e só a segunda é o que um grafo salvo precisa.
    #[test]
    fn the_arithmetic_ops_are_untouched_by_the_new_epsilon() {
        for op in [
            Op::Add,
            Op::Subtract,
            Op::Multiply,
            Op::Divide,
            Op::Min,
            Op::Max,
            Op::Modulo,
            Op::FlooredModulo,
        ] {
            for (a, b) in [(7.0, 3.0), (-7.0, 3.0), (2.5, -0.75), (5.0, 0.0)] {
                assert_eq!(
                    op.apply(a, b, 0.0).to_bits(),
                    op.apply(a, b, 999.0).to_bits(),
                    "({a}, {b}): a aritmética é surda ao epsilon, BIT A BIT"
                );
            }
        }
    }

    /// **Os índices 0..7 continuam a significar o que significavam** — as seis
    /// comparações são APENDADAS, e um documento salvo com `op = 5` ainda é `Max`.
    #[test]
    fn the_comparisons_are_appended_so_every_authored_op_still_means_what_it_meant() {
        let core = [
            Op::Add,
            Op::Subtract,
            Op::Multiply,
            Op::Divide,
            Op::Min,
            Op::Max,
            Op::Modulo,
            Op::FlooredModulo,
        ];
        for (i, want) in core.into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "i <= 7")]
            let got = Op::from_param(i as f32);
            assert!(got == want, "o índice {i} mudou de significado");
        }
        // E os seis novos ocupam 8..13, na ordem da referência.
        let new = [
            Op::Less,
            Op::LessOrEqual,
            Op::Greater,
            Op::GreaterOrEqual,
            Op::Equal,
            Op::NotEqual,
        ];
        for (k, want) in new.into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "k <= 5")]
            let got = Op::from_param((8 + k) as f32);
            assert!(got == want, "o índice {} não é o esperado", 8 + k);
        }
    }

    /// **O `epsilon` é pintado sob as duas igualdades e sob mais nada**, e a
    /// expectativa é DERIVADA do enum em vez de escrita à mão: uma sétima
    /// comparação que esquecesse o gate sangra aqui.
    #[test]
    fn epsilon_is_painted_only_under_the_two_equality_ops() {
        let gate = PARAM_GATES
            .iter()
            .find(|g| g.param == "epsilon")
            .expect("o epsilon é gateado");
        assert_eq!(gate.when, "op");
        for i in 0..=13i32 {
            #[expect(clippy::cast_precision_loss, reason = "i <= 13")]
            let op = Op::from_param(i as f32);
            let reads = matches!(op, Op::Equal | Op::NotEqual);
            assert_eq!(
                gate.values.contains(&i),
                reads,
                "o índice {i} {} o epsilon, e a tabela de gate discorda",
                if reads { "LÊ" } else { "não lê" }
            );
        }
    }

    /// FALSIFICATION of the divide guard: dividing by a (near-)zero divisor
    /// collapses to `0.0` — a downstream field never sees `inf`/`NaN`. An
    /// unguarded `a / 0.0` would be non-finite and poison the whole graph.
    #[test]
    fn divide_by_zero_collapses_to_zero_not_infinity() {
        let q = Op::Divide.apply(5.0, 0.0, 0.0);
        assert!(q.is_finite(), "guarded: finite, not inf/NaN");
        assert_eq!(q, 0.0, "collapses to 0");
        // A divisor below the epsilon is treated as zero too.
        assert_eq!(Op::Divide.apply(5.0, 1e-12, 0.0), 0.0);
        // A real divisor still divides.
        assert_eq!(Op::Divide.apply(5.0, 2.0, 0.0), 2.5);
    }

    /// The broadcast rule (doc 12): a length-1 field is HELD at every index of a
    /// length-N field — the whole point of the combiner, and what makes
    /// `gradient(N) × global(1)` one wire. Falsifiable: an element-wise-only
    /// implementation would read `b` past its single entry as 0 and multiply the
    /// tail to 0.
    #[test]
    fn a_length_one_field_broadcasts_across_a_length_n_field() {
        // gradient [0, 0.5, 1] × global 2.0 → [0, 1, 2] (b broadcast to all 3).
        let out = combine(&[0.0, 0.5, 1.0], &[2.0], Op::Multiply, 0.0);
        assert_eq!(out, vec![0.0, 1.0, 2.0], "the single b held at every index");
        // Symmetric: a length-1 `a` broadcasts against a length-N `b`.
        let out = combine(&[10.0], &[1.0, 2.0, 3.0], Op::Add, 0.0);
        assert_eq!(
            out,
            vec![11.0, 12.0, 13.0],
            "the single a held at every index"
        );
    }

    /// Two equal-length fields combine element-wise, length preserved.
    #[test]
    fn two_length_n_fields_combine_element_wise() {
        let out = combine(&[1.0, 2.0, 3.0], &[10.0, 20.0, 30.0], Op::Add, 0.0);
        assert_eq!(out, vec![11.0, 22.0, 33.0]);
    }

    /// A disconnected (empty) input reads as the zero field: `a + {} = a`
    /// (additive identity passthrough), while `a × {} = 0` (the documented
    /// consequence of the zero degenerate field). The output still tracks the
    /// connected input's length.
    #[test]
    fn a_disconnected_input_is_the_zero_field() {
        assert_eq!(
            combine(&[1.0, 2.0], &[], Op::Add, 0.0),
            vec![1.0, 2.0],
            "add: passthrough of the connected field"
        );
        assert_eq!(
            combine(&[1.0, 2.0], &[], Op::Multiply, 0.0),
            vec![0.0, 0.0],
            "multiply: the zero field collapses it"
        );
        // Both empty → empty (no field at all).
        assert!(combine(&[], &[], Op::Add, 0.0).is_empty());
    }

    // Two value sources with distinct type ids (the `motion.drive` two-source
    // harness): field `a` is length-3, field `b` is length-1, so the cook sees
    // the broadcast max.
    static SRC_A_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.math.test.a"),
        name: "value.math.test.a",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    static SRC_B_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.math.test.b"),
        name: "value.math.test.b",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct SrcA;
    impl NodeOp for SrcA {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_A_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(3).with(VALUE_COL, Column::Scalar(vec![0.0, 0.5, 1.0])));
        }
    }
    struct SrcB;
    impl NodeOp for SrcB {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_B_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![2.0])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_A_MAN.id => Some(&SrcA),
                t if t == SRC_B_MAN.id => Some(&SrcB),
                t if t == MANIFEST.id => Some(&ValueMath),
                _ => None,
            }
        }
    }

    /// End-to-end through the cook: a length-3 gradient `a` and a length-1 global
    /// `b` are multiplied, and the output is the broadcast max (length 3) —
    /// exactly the `instance_field × lfo` shape the boot scene wires.
    #[test]
    fn combines_two_value_sources_through_the_cook() {
        let mut g = Graph::new();
        let a = g.add_node("value.math.test.a");
        let b = g.add_node("value.math.test.b");
        let m = g.add_node("value.math");
        g.connect(Edge {
            from: (a, 0),
            to: (m, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (b, 0),
            to: (m, 1),
            delayed: false,
        })
        .unwrap();
        g.set_param(m, "op", 2.0); // Multiply
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 1.0, 2.0], "length-3 × broadcast 2"),
            _ => panic!("v"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }

    /// **Os dois modulos diferem no SINAL que seguem, e e' essa a razao de os
    /// dois existirem.** O truncado segue o DIVIDENDO (`-7 mod 3 = -1`, o `%` do
    /// C/Houdini), o aterrado segue o DIVISOR (`= 2`, o `%` do Python / o `mod`
    /// do GLSL). Um modulo so' obrigaria metade dos usos a uma cadeia de
    /// correcao de sinal -- e acima de zero eles COINCIDEM, que e' por que a
    /// fixture tem de descer abaixo dele.
    #[test]
    fn the_two_moduli_differ_by_the_sign_they_follow() {
        assert_eq!(Op::Modulo.apply(7.0, 3.0, 0.0), 1.0);
        assert_eq!(
            Op::FlooredModulo.apply(7.0, 3.0, 0.0),
            1.0,
            "acima de zero os dois coincidem"
        );
        assert_eq!(Op::Modulo.apply(-7.0, 3.0, 0.0), -1.0, "sinal do DIVIDENDO");
        assert_eq!(
            Op::FlooredModulo.apply(-7.0, 3.0, 0.0),
            2.0,
            "sinal do DIVISOR"
        );
        // Divisor negativo: o aterrado o segue, o truncado nao.
        assert_eq!(Op::Modulo.apply(7.0, -3.0, 0.0), 1.0);
        assert_eq!(Op::FlooredModulo.apply(7.0, -3.0, 0.0), -2.0);
    }

    /// **O aterrado aterra em `[0, b)` para todo `b > 0`** -- a propriedade que
    /// faz dele o modulo que alguem quer ao escrever *"repita a cada N"*, e que
    /// um ponto isolado nao afirma.
    ///
    /// A tolerancia de 1e-6 e' honesta e nao folga: o resultado e' `a - b·k`, e
    /// para um `a/b` que roda para logo abaixo de um inteiro a subtracao pode
    /// devolver um negativo do tamanho de um ulp de `a`.
    #[test]
    fn the_floored_modulo_wraps_into_the_half_open_range() {
        let b = 0.75_f32;
        for k in -40..40 {
            let a = k as f32 * 0.13;
            let m = Op::FlooredModulo.apply(a, b, 0.0);
            assert!(m > -1e-6 && m < b + 1e-6, "a={a} -> {m}, fora de [0,{b})");
        }
    }

    /// FALSIFICACAO da guarda: um divisor (quase) nulo colapsa em `0.0` nos DOIS
    /// modulos -- eles dividem, entao herdam a guarda do Divide, e um campo a
    /// jusante nunca ve' `inf`/`NaN`.
    #[test]
    fn a_zero_divisor_collapses_both_moduli() {
        for op in [Op::Modulo, Op::FlooredModulo] {
            assert_eq!(op.apply(5.0, 0.0, 0.0), 0.0);
            assert_eq!(op.apply(5.0, 1e-12, 0.0), 0.0);
            assert!(op.apply(5.0, 0.0, 0.0).is_finite());
        }
    }
}
