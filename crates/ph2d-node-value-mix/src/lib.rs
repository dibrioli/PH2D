#![forbid(unsafe_code)]
//! `value.mix` — the value-domain SOFT BLEND: crossfade between two **value**
//! fields by a factor, `mix = a + t·(b − a)` (Motion Nodes M2, the value domain —
//! doc 12/70). It completes the COMBINE trilogy the value vocabulary converged on:
//! `value.math` does arithmetic (`+ − × ÷ min max`), `value.switch` does a HARD
//! select (pick `a` OR `b`), and `value.mix` does the SOFT blend between them —
//! the crossfader every mature graph ships: Blender's **Mix** (Float), Nuke's
//! **Merge(mix)**, TouchDesigner's **Cross CHOP**, the fundamental "combine two
//! behaviours" node.
//!
//! **The factor is a VALUE, not (only) a param** — so an LFO, a `value.noise`, or
//! any field can DRIVE the crossfade and animate it (the domain philosophy
//! `value.switch` set: controls are values). But a bare node still wants a knob,
//! so `t` is a PORT with a `factor` PARAM fallback: **`t` connected overrides
//! `factor`; `t` unconnected reads `factor`** — exactly Blender's Mix, whose
//! Factor socket carries an inline default a wire overrides. The GPU kernel reads
//! the generated `HAS_t_v` presence const to make the SAME choice on the device.
//!
//! **`clamp`** (Blender's Clamp Factor, default On) holds `t` in `[0,1]` so the
//! blend stays between `a` and `b`; Off lets `t` past the ends OVERSHOOT (`t > 1`
//! past `b`) / UNDERSHOOT (`t < 0` before `a`) — an authored extreme, not a bug.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). **Per-element AND broadcast** (the
//! one `1→N` rule, doc 12): a length-1 field is HELD at every index; the output is
//! `max` of all connected input lengths (the same law `value.math`/`value.switch`
//! use). `Pure` (no clock, no state). Transcendental-free (HR-5): `+ − ×` and
//! `clamp` only. The kernel is the WGSL port of the same blend, so the node is
//! **device-resident** — it cooks on the GPU, no CPU fallback.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_value_math::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, inputs and output (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.mix"),
    name: "value.mix",
    inputs: &[
        PortSpec {
            name: "a",
            ty: VALUE,
        },
        PortSpec {
            name: "b",
            ty: VALUE,
        },
        PortSpec {
            name: "t",
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
        ParamSpec {
            name: "factor",
            default: 0.5,
        },
        ParamSpec {
            name: "clamp",
            default: 1.0,
        },
        // 0 Mix · 1 Add · 2 Subtract · 3 Multiply · 4 Screen · 5 Difference ·
        // 6 Darken · 7 Lighten · 8 Overlay — Blender's Mix dropdown restricted to
        // the modes that mean something on a SCALAR. `Mix` is the default and
        // reduces literally to the lerp this node always was.
        ParamSpec {
            name: "blend",
            default: 0.0,
        },
        // ⚠️ **Apendado**: o SEGUNDO clamp do Mix do Blender — o do RESULTADO, que
        // é outra grandeza que o do factor (ver o doc do nó). `0` = desligado, o
        // nó que sempre shipou.
        ParamSpec {
            name: "clamp_result",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// What `b` DOES to `a` before the crossfade weighs the two.
///
/// ⚠️ **This is not a second door onto `value.math`, and the difference is the
/// factor.** `value.math(Multiply)` answers `a·b`; this answers `lerp(a, a·b, t)`
/// — *`a` faded toward the product*, which is what a blend mode IS and what the
/// arithmetic node cannot say without a second `value.mix` wired behind it. With
/// `t = 1` the two do coincide, and that coincidence is the proof the law is the
/// right one rather than a copy.
///
/// **The tonal modes assume the `[0,1]` convention** the value domain already
/// documents (a normalised driver): `Screen` and `Overlay` are algebra about
/// *how far from full* a number is, so on a `[0,100]` field they still compute,
/// they just stop meaning what their names promise. Put a `value.map_range`
/// before them, which is the same advice `value.step` gives about its threshold.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BlendMode {
    Mix,
    Add,
    Subtract,
    Multiply,
    Screen,
    Difference,
    Darken,
    Lighten,
    Overlay,
}

impl BlendMode {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => BlendMode::Add,
            2 => BlendMode::Subtract,
            3 => BlendMode::Multiply,
            4 => BlendMode::Screen,
            5 => BlendMode::Difference,
            6 => BlendMode::Darken,
            7 => BlendMode::Lighten,
            8 => BlendMode::Overlay,
            _ => BlendMode::Mix,
        }
    }

    /// The target `a` is faded TOWARD. `Mix` returns `b` unchanged, so the whole
    /// node reduces to `a + t·(b − a)` — byte-identical to the world before this
    /// param existed. Transcendental-free (HR-5): `+ − ×` and one comparison.
    fn apply(self, a: f32, b: f32) -> f32 {
        match self {
            BlendMode::Mix => b,
            BlendMode::Add => a + b,
            BlendMode::Subtract => a - b,
            BlendMode::Multiply => a * b,
            BlendMode::Screen => 1.0 - (1.0 - a) * (1.0 - b),
            BlendMode::Difference => (a - b).abs(),
            BlendMode::Darken => a.min(b),
            BlendMode::Lighten => a.max(b),
            BlendMode::Overlay => {
                if a < 0.5 {
                    2.0 * a * b
                } else {
                    1.0 - 2.0 * (1.0 - a) * (1.0 - b)
                }
            }
        }
    }
}

/// The sample of value field `v` at index `i` under the `1→N` broadcast rule: a
/// length-1 field is held at every index; a length-N field is read element-wise;
/// a missing field reads as `0.0`. Mirror of `value.math`'s `field_at`.
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

/// **COMO** os dois campos se misturam — os quatro controles que não são dados.
///
/// ⚠️ **Eles viajam juntos porque um `clippy::too_many_arguments` os separou, e a
/// separação é a certa:** `a`/`b`/`t` são os DADOS, isto é a LEI. E há um segundo
/// motivo, mais concreto que o lint — `clamp` e `clamp_result` são dois `bool`
/// ADJACENTES numa lista posicional, que é exactamente onde dois argumentos se
/// trocam sem o compilador ver nada. Nomeados no sítio da chamada, não podem.
#[derive(Clone, Copy)]
struct Blend {
    factor: f32,
    clamp: bool,
    mode: BlendMode,
    clamp_result: bool,
}

/// Crossfade `a` and `b` by the factor. `t` is the field from the port if
/// connected (`t_connected`), else the constant `factor`; `clamp` holds it in
/// `[0,1]`. Output length is `max` of the input lengths under the broadcast rule;
/// a length that is neither 1 nor the max is read leniently (element-wise, `0.0`
/// past the end).
fn blend(a: &[f32], b: &[f32], t: &[f32], t_connected: bool, how: Blend) -> Vec<f32> {
    let Blend {
        factor,
        clamp,
        mode,
        clamp_result,
    } = how;
    let n = a.len().max(b.len()).max(t.len());
    (0..n)
        .map(|i| {
            let mut tt = if t_connected { field_at(t, i) } else { factor };
            if clamp {
                tt = tt.clamp(0.0, 1.0);
            }
            let va = field_at(a, i);
            let vb = mode.apply(field_at(a, i), field_at(b, i));
            let o = va + tt * (vb - va);
            if clamp_result { o.clamp(0.0, 1.0) } else { o }
        })
        .collect()
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`blend`], **fully device-
/// resident**. `read_a_v`/`read_b_v`/`read_t_v` apply the broadcast rule per
/// dispatch; `HAS_t_v` is the generated presence const that answers *"is `t`
/// wired?"* — `select(factor, port, HAS_t_v)` makes the SAME port-overrides-param
/// choice the CPU makes. No `applicable` gate — the sequencer never falls back to
/// the CPU (the "maximize GPU" north).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        var vx_t = select(params.factor, read_t_v(i), HAS_t_v);\n\
        if (params.clamp >= 0.5) { vx_t = clamp(vx_t, 0.0, 1.0); }\n\
        let vx_a = read_a_v(i);\n\
        let vx_raw = read_b_v(i);\n\
        let vx_m = i32(vx_round(params.blend));\n\
        var vx_b = vx_raw;\n\
        if (vx_m == 1) { vx_b = vx_a + vx_raw; }\n\
        else if (vx_m == 2) { vx_b = vx_a - vx_raw; }\n\
        else if (vx_m == 3) { vx_b = vx_a * vx_raw; }\n\
        else if (vx_m == 4) { vx_b = 1.0 - (1.0 - vx_a) * (1.0 - vx_raw); }\n\
        else if (vx_m == 5) { vx_b = abs(vx_a - vx_raw); }\n\
        else if (vx_m == 6) { vx_b = min(vx_a, vx_raw); }\n\
        else if (vx_m == 7) { vx_b = max(vx_a, vx_raw); }\n\
        else if (vx_m == 8) {\n\
        \x20   if (vx_a < 0.5) { vx_b = 2.0 * vx_a * vx_raw; }\n\
        \x20   else { vx_b = 1.0 - 2.0 * (1.0 - vx_a) * (1.0 - vx_raw); }\n\
        }\n\
        var vx_o = vx_a + vx_t * (vx_b - vx_a);\n\
        // O SEGUNDO clamp -- o do RESULTADO, nao o do factor.\n\
        if (params.clamp_result >= 0.5) { vx_o = clamp(vx_o, 0.0, 1.0); }\n\
        write_v(i, vx_o);\n",
    wgsl_lib: "\
        fn vx_round(x: f32) -> f32 {\n\
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
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 2,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    // ⚠️ Esta lista não é derivada do manifesto: um param novo compila, coza na
    // CPU, e o device recusa o shader (`invalid field accessor`).
    params: &["factor", "clamp", "blend", "clamp_result"],
    count_law: Some(mix_count),
    variant_by_param: None,
    applicable: None,
};

/// **How wide is the output?** — `max` over every input port (the same law
/// `value.math`/`value.switch` use), written as a `max` over all ports rather than
/// named ones so it stays correct rather than merely true. An unconnected `t`
/// contributes 0 (the `factor` fallback carries no length), so a bare-factor
/// crossfade of two length-N fields is length N.
fn mix_count(c: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(c.inputs.iter().copied().max().unwrap_or(0) as usize)
}

struct ValueMix;

impl NodeOp for ValueMix {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let factor = ctx.param("factor");
        let clamp = ctx.param("clamp") >= 0.5;
        let a = scalar_col(ctx.input(0), VALUE_COL);
        let b = scalar_col(ctx.input(1), VALUE_COL);
        let mode = BlendMode::from_param(ctx.param("blend"));
        let t = scalar_col(ctx.input(2), VALUE_COL);
        // `t` connected == a non-empty field on port 2 (matches `HAS_t_v` on GPU).
        let out = blend(
            &a,
            &b,
            &t,
            !t.is_empty(),
            Blend {
                factor,
                clamp,
                mode,
                clamp_result: ctx.param("clamp_result") >= 0.5,
            },
        );
        ctx.emit(Stream::new(out.len()).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueMix))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Mix",
            // Utility grey: a value→value combiner, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    // The constant blend, used when the `t` port is unconnected (a wire overrides
    // it). `[0,1]` is the crossfade range; overshoot is authored by driving `t`
    // with `clamp` off.
    ParamUiHint {
        param: "factor",
        label: "Factor",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // Hold `t` in `[0,1]` (On) or let the ends overshoot/undershoot (Off) — the
    // same Off/On enum `value.map_range`'s clamp uses.
    //
    // ⚠️ O rótulo diz **Factor** desde que o irmão `Clamp Result` existe: os dois
    // são o par que o Mix do Blender shipa, e um deles chamar-se só "Clamp" já foi
    // o que fez a conferência ter de MEDIR qual dos dois este era.
    ParamUiHint {
        param: "clamp",
        label: "Clamp Factor",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Off", "On"],
        },
    },
    // What `b` does to `a` before the factor weighs them. `Mix` is the plain
    // crossfade this node always was.
    ParamUiHint {
        param: "blend",
        label: "Blend",
        min: 0.0,
        max: 8.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "Mix",
                "Add",
                "Subtract",
                "Multiply",
                "Screen",
                "Difference",
                "Darken",
                "Lighten",
                "Overlay",
            ],
        },
    },
    // O clamp do RESULTADO. Desligado = o nó de sempre. Ligado, segura a saída em
    // `[0,1]` — o que os modos de blend (Add/Screen/Subtract) fazem transbordar.
    ParamUiHint {
        param: "clamp_result",
        label: "Clamp Result",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Off", "On"],
        },
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
