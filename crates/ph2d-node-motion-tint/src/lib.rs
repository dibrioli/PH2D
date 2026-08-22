#![forbid(unsafe_code)]
//! `motion.tint` — a Motion **colour modifier**: sets the `tint` (Vec4 RGBA,
//! linear straight — §1.2) attribute to a target colour, masked per-instance by
//! the multiplicative `falloff` column. The per-instance result is
//! `lerp(existing, target_i, falloff_i)`, so at `falloff = 1` the instance takes
//! the target **exactly** and at `falloff = 0` it keeps its existing tint (absent
//! → opaque white). Every other column passes through unchanged. Pure.
//!
//! Two modes (`mode`): **Solid** (0, the default) applies one uniform colour
//! `(r,g,b,a)`; **Gradient** (1) sweeps from `(r,g,b,a)` (Start) to
//! `(r2,g2,b2,a2)` (End) across the stream, keyed by each instance's normalized
//! identity `Index/(Count−1)` (from the grid; absent → positional `i/(n−1)`), so
//! a grid reads as a colour ramp. The gradient lerp is in the wire's linear space
//! (transcendental-free; OKLab is a future, cbrt-gated refinement).
//!
//! Defaults: `mode` 0 (Solid); Start `r=g=b=a=1` (**opaque white** — the identity,
//! so a fresh Tint is a no-op until the artist picks a colour); End `(0,0,0,1)`
//! (black — so switching to Gradient shows a visible white→black ramp).
//!
//! ## `blend` — HOW the colour meets the one already there (doc 89 fam. 9)
//!
//! The node above computes a **target** colour; `blend` says how that target
//! combines with the instance's EXISTING tint before the `falloff` mask lerps
//! between them. It is the C4D MoGraph effector's *Color group → Blending Mode*
//! (Mix · Add · Subtract · Multiply · Divide), and the reason it has to live
//! **here** rather than be composed: `motion.mixer(Add)` sums every column the
//! two streams share — `P` included, so the positions double — and its `blend`
//! is `v.first()`, one global scalar rather than a field. There is no node in
//! the catalogue that combines two `tint`s per instance.
//!
//! **`Mix` (0) is the default and is the law this node always had**, bit for
//! bit: `blended` returns the target unchanged, so the pipeline is exactly
//! `lerp(existing, target, falloff)`. Every graph authored before this param
//! existed reads `0.0` and renders identically.
//!
//! ⚠️ **The four other modes act on all FOUR channels, alpha included, and
//! nothing is clamped.** Alpha is not special-cased because a `Multiply` of two
//! coverages is exactly the fade an artist means, and RGB is deliberately left
//! HDR — an `Add` of two whites is `2.0` in linear, which is a bloom source
//! `fx.glow` can see, not an error. Clamping here would be the slow path
//! choosing the fast path's ceiling.
//!
//! ⚠️ **`Divide` by a zero channel returns the EXISTING channel**, not an
//! infinity and not black. The function is genuinely singular there, so the
//! value at the singularity is a choice; this one makes *"divide by nothing"*
//! mean *"change nothing"*, keeps the column finite, and — the reason it is
//! written `t == 0.0` rather than a sign test — catches `-0.0` too, since IEEE
//! makes the two compare equal. A `NaN` or an `inf` leaving this node would
//! travel every consumer downstream and poison what it touched.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.tint"),
    name: "motion.tint",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // Start (Solid = the colour; Gradient = the low end).
        ParamSpec {
            name: "r",
            default: 1.0,
        },
        ParamSpec {
            name: "g",
            default: 1.0,
        },
        ParamSpec {
            name: "b",
            default: 1.0,
        },
        ParamSpec {
            name: "a",
            default: 1.0,
        },
        // End (Gradient only).
        ParamSpec {
            name: "r2",
            default: 0.0,
        },
        ParamSpec {
            name: "g2",
            default: 0.0,
        },
        ParamSpec {
            name: "b2",
            default: 0.0,
        },
        ParamSpec {
            name: "a2",
            default: 1.0,
        },
        // How the target meets the existing tint (see the module docs). 0 = Mix
        // = the law this node always had, so an untouched graph is unchanged.
        ParamSpec {
            name: "blend",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The blending modes, in the order the [`PARAM_HINTS`] enum paints them — the
/// C4D effector's list. The discriminants ARE the param values, so this enum is
/// the single place the mapping lives (a saved graph stores the number).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BlendMode {
    /// The target replaces what is there — the law before this param existed.
    Mix,
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl BlendMode {
    /// The labels the panel paints, and the ONLY list of them — [`PARAM_HINTS`]
    /// reads this, so a mode added to the enum without a label fails to compile
    /// rather than shipping a nameless row.
    pub const LABELS: [&'static str; 5] = ["Mix", "Add", "Subtract", "Multiply", "Divide"];

    /// The mode a param value names. Rounded half-away-from-zero (Rust's own
    /// `f32::round`, mirrored in WGSL by `tn_round`); anything outside the list
    /// falls back to `Mix`, so a graph saved by a FUTURE build that grew a sixth
    /// mode degrades to the identity instead of picking an arbitrary neighbour.
    #[must_use]
    pub fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Add,
            2 => Self::Subtract,
            3 => Self::Multiply,
            4 => Self::Divide,
            _ => Self::Mix,
        }
    }
}

/// One channel of the blend. See the module docs for why `Divide` returns `e` at
/// a zero divisor and why nothing is clamped.
fn blend_channel(e: f32, t: f32, mode: BlendMode) -> f32 {
    match mode {
        BlendMode::Mix => t,
        BlendMode::Add => e + t,
        BlendMode::Subtract => e - t,
        BlendMode::Multiply => e * t,
        // `t == 0.0` is true for `-0.0` as well (IEEE) — see the module docs.
        BlendMode::Divide => {
            if t == 0.0 {
                e
            } else {
                e / t
            }
        }
    }
}

/// The target colour AFTER it meets the existing one. At [`BlendMode::Mix`] this
/// returns `target` itself — the same value, by the same expression — which is
/// what makes the default byte-identical to the node that had no `blend`.
fn blended(existing: [f32; 4], target: [f32; 4], mode: BlendMode) -> [f32; 4] {
    [
        blend_channel(existing[0], target[0], mode),
        blend_channel(existing[1], target[1], mode),
        blend_channel(existing[2], target[2], mode),
        blend_channel(existing[3], target[3], mode),
    ]
}

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// Read entry `i` of a Scalar column (absent / wrong-typed → `default`).
fn scalar_at(col: Option<&Column>, i: usize, default: f32) -> f32 {
    match col {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}

/// Per-channel RGBA lerp `a·(1−t) + b·t` (endpoint-exact; linear-straight space).
fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] * (1.0 - t) + b[0] * t,
        a[1] * (1.0 - t) + b[1] * t,
        a[2] * (1.0 - t) + b[2] * t,
        a[3] * (1.0 - t) + b[3] * t,
    ]
}

/// The falloff-masked colour for one instance: `lerp(existing, target, f)` per
/// RGBA channel via the endpoint-exact form `existing·(1-f) + target·f` — so it
/// returns exactly `existing` at `f = 0` and exactly `target` at `f = 1` (any
/// colour + alpha, no float drift).
fn mixed_tint(existing: [f32; 4], target: [f32; 4], f: f32) -> [f32; 4] {
    let lerp = |e: f32, t: f32| e * (1.0 - f) + t * f;
    [
        lerp(existing[0], target[0]),
        lerp(existing[1], target[1]),
        lerp(existing[2], target[2]),
        lerp(existing[3], target[3]),
    ]
}

/// GPU compute kernel (GPU/M5 Fase 2, ADR-0126): `tint' = lerp(existing, target,
/// falloff)` per RGBA channel — the exact `mixed_tint` form, so parity is
/// bit-exact (no transcendentals). `ReadWrite` on `tint` mirrors the CPU: an
/// absent `tint` column starts from opaque white `[1,1,1,1]` and the column is
/// always written.
///
/// **Gradient runs here too, and the positional key is why it could not before.**
/// The ramp keys off `Index/(Count−1)`, falling back to `i`/`n` when those
/// columns are absent — a *positional* identity, and `ColumnBinding.identity` is
/// a CONSTANT, so `read_Index`'s fallback cannot be `f32(i)`. The generated
/// `HAS_<col>` const closes it exactly as it did for `motion.color_ramp`: the
/// body asks whether the column was really there and takes the CPU's other
/// branch when it was not. This is the gap the Fase 2 handoff logged against
/// this very node, and it is what lets the emitter's documented affordance
/// ("ids ascend oldest-first, so Gradient paints the stream by AGE") survive on
/// the GPU path instead of dropping the whole fountain to the CPU.
///
/// **The short-column hazard `motion.color_ramp` recedes over does not exist
/// here**, and the difference is structural, not luck: its `t` arrives on
/// ANOTHER port, where a chain rooted at a different generator can be a
/// different length, and this engine calls a wrong-length column *absent* — which
/// there would silently mean "use the positional key" while the CPU pads. `Index`
/// and `Count` ride the SAME port as the base, and [`Stream::set`] asserts
/// `col.len() == count`, so present ⟺ exists and a present column is exactly `n`
/// long. The CPU's `unwrap_or(default)` is unreachable for these two — defensive,
/// not a semantic branch — so `HAS_x ? read_x(i) : positional` *is* `scalar_at`.
///
/// **HR-5:** the lerps are written as the CPU writes them (`a·(1−t) + b·t`, the
/// `lerp4`/`mixed_tint` form), never WGSL's `mix` — same value, different
/// expression, different rounding. `mode` is rounded half-away-from-zero
/// (Rust's `f32::round`), not by WGSL's half-even builtin.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let tn_e = read_tint(i);\n\
        var tn_t = vec4<f32>(params.r, params.g, params.b, params.a);\n\
        if (i32(tn_round(params.mode)) == 1) {\n\
        \x20   // Absent Index/Count → the POSITIONAL key, the CPU's own fallback.\n\
        \x20   var tn_idx = f32(i);\n\
        \x20   if (HAS_Index) { tn_idx = read_Index(i); }\n\
        \x20   var tn_cnt = f32(params.count);\n\
        \x20   if (HAS_Count) { tn_cnt = read_Count(i); }\n\
        \x20   var tn_g = 0.0;\n\
        \x20   if (tn_cnt > 1.0) { tn_g = clamp(tn_idx / (tn_cnt - 1.0), 0.0, 1.0); }\n\
        \x20   let tn_end = vec4<f32>(params.r2, params.g2, params.b2, params.a2);\n\
        \x20   tn_t = vec4<f32>(\n\
        \x20       tn_t.x * (1.0 - tn_g) + tn_end.x * tn_g,\n\
        \x20       tn_t.y * (1.0 - tn_g) + tn_end.y * tn_g,\n\
        \x20       tn_t.z * (1.0 - tn_g) + tn_end.z * tn_g,\n\
        \x20       tn_t.w * (1.0 - tn_g) + tn_end.w * tn_g);\n\
        }\n\
        // The blend meets the existing colour BEFORE the mask lerps (the\n\
        // CPU's `blended`); mode 0 (Mix) returns `tn_t` itself, so the\n\
        // default path is the same expression it always was.\n\
        let tn_m = i32(tn_round(params.blend));\n\
        tn_t = vec4<f32>(\n\
        \x20   tn_blend(tn_e.x, tn_t.x, tn_m),\n\
        \x20   tn_blend(tn_e.y, tn_t.y, tn_m),\n\
        \x20   tn_blend(tn_e.z, tn_t.z, tn_m),\n\
        \x20   tn_blend(tn_e.w, tn_t.w, tn_m));\n\
        let tn_f = read_falloff(i);\n\
        write_tint(i, vec4<f32>(\n\
            tn_e.x * (1.0 - tn_f) + tn_t.x * tn_f,\n\
            tn_e.y * (1.0 - tn_f) + tn_t.y * tn_f,\n\
            tn_e.z * (1.0 - tn_f) + tn_t.z * tn_f,\n\
            tn_e.w * (1.0 - tn_f) + tn_t.w * tn_f));\n",
    wgsl_lib: "\
        fn tn_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn tn_blend(e: f32, t: f32, mode: i32) -> f32 {\n\
            // The CPU's `blend_channel`, arm for arm and expression for\n\
            // expression. Written as an if-chain rather than `select`, because\n\
            // `select` evaluates BOTH sides and `e / 0.0` is exactly the value\n\
            // the Divide arm exists to avoid producing.\n\
            if (mode == 1) { return e + t; }\n\
            if (mode == 2) { return e - t; }\n\
            if (mode == 3) { return e * t; }\n\
            if (mode == 4) {\n\
            \x20   if (t == 0.0) { return e; }\n\
            \x20   return e / t;\n\
            }\n\
            return t;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "tint",
            dim: Dim::Vec4,
            access: ColumnAccess::ReadWrite,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        // Read for the ramp key only; the `HAS_` const above decides whether the
        // value or the positional fallback is used, so the identity is inert.
        ColumnBinding {
            column: "Index",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "Count",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["r", "g", "b", "a", "r2", "g2", "b2", "a2", "mode", "blend"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct MotionTint;

impl NodeOp for MotionTint {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let gradient = ctx.param("mode").round() as i32 == 1;
        let blend = BlendMode::from_param(ctx.param("blend"));
        let start = [
            ctx.param("r"),
            ctx.param("g"),
            ctx.param("b"),
            ctx.param("a"),
        ];
        let end = [
            ctx.param("r2"),
            ctx.param("g2"),
            ctx.param("b2"),
            ctx.param("a2"),
        ];
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Base per-instance tint (absent column → opaque white).
            let base: Vec<[f32; 4]> = match input.get("tint") {
                Some(Column::Vec4(v)) => v.clone(),
                _ => vec![[1.0, 1.0, 1.0, 1.0]; n],
            };
            // Identity columns for the gradient ramp (absent → positional).
            let index = input.get("Index");
            let count = input.get("Count");
            let tinted: Vec<[f32; 4]> = (0..n)
                .map(|i| {
                    let target = if gradient {
                        let idx = scalar_at(index, i, i as f32);
                        let cnt = scalar_at(count, i, n as f32);
                        // Normalized position 0..1 across the stream (single
                        // instance → 0, so it just takes the Start colour).
                        let t = if cnt > 1.0 {
                            (idx / (cnt - 1.0)).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        lerp4(start, end, t)
                    } else {
                        start
                    };
                    let e = base.get(i).copied().unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    // The blend combines target with what is there; the mask then
                    // lerps between the two. At Mix the first step is the identity.
                    mixed_tint(e, blended(e, target, blend), falloff_at(input, i))
                })
                .collect();
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "tint" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("tint", Column::Vec4(tinted));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTint))?;
    // M1.R1 — UI metadata (a colour effect → magenta Fx, rounded-rect).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Tint",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    // GPU/M5 Fase 2 (ADR-0126): the WGSL lowering (Solid mode), on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

/// **A segunda cor só existe no gradiente** (doc 90 §2, caça aos knobs mortos).
///
/// ⚠️ A cor `End` é lida **só** no braço `Gradient`, e `mode` nasce em `Solid`: o artista abria
/// o segundo swatch, escolhia uma cor no picker OKLCH e a imagem não mudava.
///
/// ⚠️ **Este defeito estava CONFESSADO por escrito no hint deste arquivo** — *"it paints
/// regardless — v1 has no per-mode row hiding"* — e o mecanismo que o cura já existia no
/// repositório, usado pelo irmão `motion.transform`. *Uma limitação anotada continua uma
/// limitação; a nota não é a cura, e escrevê-la pode até adiar a cura por parecer uma.*
///
/// ⚠️ **Gateia só a ÂNCORA.** Os outros três canais (`g2`/`b2`/`a2`) não pintam linha própria —
/// o construtor de rows dobra-os no swatch (`consumed`) e é `r2` que emite a row. Três linhas a
/// mais aqui não decidiriam nada.
///
/// `0 = Solid` · `1 = Gradient`.
static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: "r2",
    when: "mode",
    values: &[1],
}];

/// The last valid `blend` value. Written as a literal because a `usize as f32`
/// cast is not what a slider bound should cost — and pinned to the label list by
/// a COMPILE-TIME assert, so a sixth mode that forgets this line fails to build
/// instead of shipping a row the artist cannot reach.
const BLEND_MAX: f32 = 4.0;
const _: () = assert!(
    BlendMode::LABELS.len() == 5,
    "BLEND_MAX must be LABELS.len() - 1"
);

/// Param UI hints (M1.P1 → colour authoring): a **named** Solid/Gradient selector,
/// then one canonical colour swatch → OKLCH picker per colour (never raw linear
/// sliders — a raw `0.5` linear reads as light grey). Each [`ParamWidget::Color`]
/// hint anchors on its first channel and names the four params it drives; the
/// shell bridge reads the pick back (sRGB→linear). The `End` swatch is only used
/// in Gradient mode (it paints regardless — v1 has no per-mode row hiding).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Solid", "Gradient"],
        },
    },
    ParamUiHint {
        param: "r",
        label: "Color",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: ["r", "g", "b", "a"],
        },
    },
    ParamUiHint {
        param: "r2",
        label: "End",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: ["r2", "g2", "b2", "a2"],
        },
    },
    // How that colour meets the one already on the instance. The labels are
    // [`BlendMode::LABELS`], never a second list — a mode added to the enum
    // without a name would not compile.
    ParamUiHint {
        param: "blend",
        label: "Blend",
        min: 0.0,
        max: BLEND_MAX,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &BlendMode::LABELS,
        },
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
