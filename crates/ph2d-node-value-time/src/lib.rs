//! `value.time` — the value-domain PRODUCER of the raw animated clock: the
//! playhead as a plain value (Motion Nodes M2, the value domain — doc 12/80).
//! Every other value producer is spatial or random — `value.instance_field`
//! (Index/Ramp/Random), `value.noise`, `value.pattern` — and none of them is the
//! **clock**. `value.lfo`/`value.noise` bring time in but bake a WAVEFORM around
//! it; this emits the clock UN-shaped, so a value graph can build its own
//! function of time. It is Houdini's `@Time`/`$T`, TouchDesigner's Timer/Beat
//! CHOP, the `time` node of every node editor.
//!
//! **Monotonic, not periodic — that is the whole point.** `value.lfo(Saw)` folds
//! the clock back every period (`phase − floor(phase)`); this one keeps climbing:
//! `t · rate + offset`. That distinction is what it is FOR — endless rotation, an
//! ever-scrolling offset, an accumulating drift — and it is what makes it the
//! natural partner of `value.wrap`: `time → wrap(Repeat)` is a controllable
//! sawtooth clock, `time → wrap(Mirror)` a triangle one, each folded exactly where
//! you want, while the raw `time` still climbs for whatever wants it un-folded.
//!
//! **Cardinality follows the geometry** (the `value.lfo` pattern): the optional
//! `in` port is read for its **count only** — connected → a length-N field with a
//! per-instance `stagger` (a travelling ramp across the grid), **unconnected → a
//! length-1 global clock** held across every instance by `motion.drive`'s
//! broadcast rule (the common case: *the* clock). Nothing from the input rides
//! through — this mints a fresh value.
//!
//! Reads the playhead, holds no state → `Effect::Temporal` (pull-side, like the
//! LFO). Transcendental-free (HR-5): a multiply-add, bit-exact on the device.
//!
//! `value_i = playhead · rate + offset + i · stagger − delay · ease(raw_i)`.
//!
//! ⭐ **O ATRASO NO TEMPO** (ciclo 17, doc 126 §3.3): `delay` é o atraso TOTAL distribuído pela
//! ORDEM da casa (`ph2d_ease::order` — índice, centro, aleatório, grelha, ponto) e moldado pela
//! curva da casa (`ph2d_ease::ease`). É o `stagger: {amount, from, grid, ease}` do GSAP/anime.js:
//! cada linha começa a sua animação mais tarde, a partir do centro, do canto ou de um ponto. Com
//! `delay = 0` o termo nem corre — o relógio de sempre, ao bit.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, LutSpec, SourceWindow,
};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The instance stream type — read for its count only (the optional `in` port).
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.time"),
    name: "value.time",
    // Optional: connected → count N + per-instance stagger; unconnected → one
    // global clock. Read for its count only; never passed through.
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Reads the playhead → pull-side; HR-5-exempt for the clock (the arithmetic
    // is nonetheless a plain multiply-add, bit-exact cross-platform).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Clock speed — units of value per second. `1` = the raw playhead in
        // seconds; `2` runs twice as fast; negative runs backwards.
        ParamSpec {
            name: "rate",
            default: 1.0,
        },
        // A constant added to the clock — where the value starts at `t = 0`.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // Per-instance offset → a travelling ramp across the field (needs a
        // connected `in` for N > 1; 0 → every instance reads the same clock).
        ParamSpec {
            name: "stagger",
            default: 0.0,
        },
        // ⭐ CICLO 17 — o atraso pela ordem e pela curva da casa (apendados; defaults = o de sempre).
        ParamSpec {
            name: "delay",
            default: 0.0,
        },
        ParamSpec {
            name: "order",
            default: 0.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        ParamSpec {
            name: "ease_curve",
            default: 0.0,
        },
        ParamSpec {
            name: "ease_dir",
            default: 0.0,
        },
        ParamSpec {
            name: "overshoot",
            default: ph2d_ease::Shape::DEFAULT.overshoot,
        },
        ParamSpec {
            name: "period",
            default: ph2d_ease::Shape::DEFAULT.period,
        },
        ParamSpec {
            name: "steps",
            default: ph2d_ease::Shape::DEFAULT.steps,
        },
        ParamSpec {
            name: "damping",
            default: ph2d_ease::Shape::DEFAULT.damping,
        },
        ParamSpec {
            name: "bez_x1",
            default: ph2d_ease::Shape::DEFAULT.bezier[0],
        },
        ParamSpec {
            name: "bez_y1",
            default: ph2d_ease::Shape::DEFAULT.bezier[1],
        },
        ParamSpec {
            name: "bez_x2",
            default: ph2d_ease::Shape::DEFAULT.bezier[2],
        },
        ParamSpec {
            name: "bez_y2",
            default: ph2d_ease::Shape::DEFAULT.bezier[3],
        },
        ParamSpec {
            name: "grid_cols",
            default: 10.0,
        },
        ParamSpec {
            name: "grid_from",
            default: 1.0,
        },
        ParamSpec {
            name: "grid_cell",
            default: 0.0,
        },
        ParamSpec {
            name: "axis",
            default: 0.0,
        },
        ParamSpec {
            name: "origin_x",
            default: 0.0,
        },
        ParamSpec {
            name: "origin_y",
            default: 0.0,
        },
        ParamSpec {
            name: "reach",
            default: 5.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// A chave do text param da curva DESENHADA (`ease_curve = Custom`) — o mesmo canal do
/// `motion.stagger`.
pub const CURVE_KEY: &str = "curve";

/// GPU compute kernel (ADR-0126) — the WGSL port of [`ValueTime::eval`], **fully
/// device-resident**. VALUE out (mints a fresh `v`, does NOT ride the input
/// through — the sequencer derives that from the manifest, port 0 in-type vs
/// out-type differ, exactly like `value.lfo`). The clock is `params.playhead`,
/// the same uniform the LFO reads; the arithmetic is a bare multiply-add, so the
/// device result is bit-comparable to the CPU (the only divergence is an FMA the
/// driver may fuse, ε below the parity budget). No `applicable` — no CPU fallback.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: VT_BODY,
    wgsl_lib: VT_LIB,
    bindings: &[VT_V],
    params: VT_PARAMS,
    count_law: Some(time_count),
    // ⚠️ **Só o `From Point` LÊ a posição**, e é por isso que ela vive numa VARIANTE: declarada no
    // kernel de topo, toda `value.time` solta (o relógio global, sem entrada) era lida pelo
    // diagnóstico como «lê `P` sem fonte» — um `MissingSource(P)` FALSO na cena `=76`.
    variant_by_param: Some(|param| {
        if param("order").round() as i32 == ph2d_ease::order::ORDER_FROM_POINT {
            &VT_PONTO
        } else {
            &GPU_KERNEL
        }
    }),
    applicable: None,
};

/// A variante do `From Point`: a mesma conta, com a posição LIDA.
const VT_PONTO: GpuKernel = GpuKernel {
    wgsl: VT_BODY,
    wgsl_lib: VT_LIB_PONTO,
    bindings: &[
        VT_V,
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: VT_PARAMS,
    count_law: Some(time_count),
    variant_by_param: None,
    applicable: None,
};

const VT_V: ColumnBinding = ColumnBinding {
    column: VALUE_COL,
    dim: Dim::Scalar,
    access: ColumnAccess::Write,
    identity: [0.0; 4],
    port: 0,
};

const VT_BODY: &str = "\
        var vt_v = params.playhead * params.rate + params.offset\n\
        \x20   + f32(i) * params.stagger;\n\
        if (params.delay != 0.0) {\n\
        \x20   vt_v = vt_v - params.delay * vt_ease(vt_raw_at(i));\n\
        }\n\
        write_v(i, vt_v);\n";

const VT_PARAMS: &[&str] = &[
    "rate",
    "offset",
    "stagger",
    "delay",
    "order",
    "seed",
    "ease_curve",
    "ease_dir",
    "overshoot",
    "period",
    "steps",
    "damping",
    "bez_x1",
    "bez_y1",
    "bez_x2",
    "bez_y2",
    "grid_cols",
    "grid_from",
    "grid_cell",
    "axis",
    "origin_x",
    "origin_y",
    "reach",
];

/// O trecho comum: a curva e a ordem lidas dos params; a POSIÇÃO entra pelo literal `$p`.
macro_rules! vt_lib {
    ($p:literal) => {
        concat!(
            ph2d_ease::wgsl_lib!(),
            "\
    fn vt_ease(t: f32) -> f32 {\n\
        let curve = i32(pe_round(params.ease_curve));\n\
        if (curve == 8) { return vt_curve_sample(t); }\n\
        let s0 = vec4<f32>(params.overshoot, params.period, params.steps, params.damping);\n\
        let s1 = vec4<f32>(params.bez_x1, params.bez_y1, params.bez_x2, params.bez_y2);\n\
        return pe_ease(curve, i32(pe_round(params.ease_dir)), t, s0, s1);\n\
    }\n\
    fn vt_raw_at(i: u32) -> f32 {\n\
        return pe_raw_at(i32(pe_round(params.order)),\n\
        \x20   u32(max(pe_round(params.seed), 0.0)),\n\
        \x20   u32(max(pe_round(params.grid_cols), 1.0)),\n\
        \x20   i32(pe_round(params.grid_from)),\n\
        \x20   u32(max(pe_round(params.grid_cell), 0.0)),\n\
        \x20   i32(pe_round(params.axis)),\n\
        \x20   vec2<f32>(params.origin_x, params.origin_y), params.reach,\n\
        \x20   i, params.count, ",
            $p,
            ");\n\
    }\n"
        )
    };
}

/// A tabela e a ordem da casa (`ph2d_ease::wgsl_lib!`) e a leitura dos params deste nó — sem a
/// posição (toda ordem que não a lê).
const VT_LIB: &str = vt_lib!("vec2<f32>(0.0, 0.0)");
/// O mesmo, com a posição LIDA (a variante do `From Point`).
const VT_LIB_PONTO: &str = vt_lib!("read_P(i)");

/// A LUT da curva desenhada (`Custom`) — a mesma resolução do `motion.stagger` e pela mesma
/// razão (doc do kernel dele).
static LUTS: &[LutSpec] = &[LutSpec {
    name: "vt_curve",
    text_key: CURVE_KEY,
    resolution: 512,
    fill: fill_curve_lut,
}];

/// Amostra a curva em `t = k/(n−1)`; ausente/malformada ⇒ a identidade (o `Linear`).
fn fill_curve_lut(text: &str, out: &mut [f32]) {
    let curve = ph2d_curve::parse(text).unwrap_or_else(ph2d_curve::Curve::identity);
    let n = out.len();
    for (k, slot) in out.iter_mut().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "k < 512")]
        let t = if n <= 1 {
            0.0
        } else {
            k as f32 / (n - 1) as f32
        };
        *slot = curve.eval(t);
    }
}

/// **How wide is the field?** — the same expression `eval` uses. Connected, it is
/// one clock per instance (a travelling ramp with `stagger`); **unconnected it is
/// ONE global clock**, held across every instance by `motion.drive`'s broadcast
/// rule. The engine's default law — "as wide as port 0" — gets the connected case
/// right and the unconnected one silently wrong: an empty port is `0`, a
/// zero-count stage is SKIPPED, and the node would be unreachable on the device
/// the moment something consumed the global clock (the `value.lfo` lesson).
fn time_count(c: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(c.inputs.first().copied().unwrap_or(0).max(1) as usize)
}

struct ValueTime;

/// O que o atraso lê: a ordem, a família e a curva desenhada — uma vez por cozimento.
struct Atraso {
    ordem: ph2d_ease::order::Order,
    curve: i32,
    dir: i32,
    shape: ph2d_ease::Shape,
    custom: Option<ph2d_curve::Curve>,
}

impl Atraso {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "índices de enum e contagens, arredondados como o WGSL"
    )]
    fn of(ctx: &EvalCtx<'_>) -> Self {
        Self {
            ordem: ph2d_ease::order::Order {
                kind: ctx.param("order").round() as i32,
                seed: ctx.param("seed").max(0.0).round() as u32,
                cols: ctx.param("grid_cols").round().max(1.0) as u32,
                from: ctx.param("grid_from").round() as i32,
                cell: ctx.param("grid_cell").round().max(0.0) as u32,
                axis: ctx.param("axis").round() as i32,
                origin: [ctx.param("origin_x"), ctx.param("origin_y")],
                reach: ctx.param("reach"),
            },
            curve: ctx.param("ease_curve").round() as i32,
            dir: ctx.param("ease_dir").round() as i32,
            shape: ph2d_ease::Shape {
                overshoot: ctx.param("overshoot"),
                period: ctx.param("period"),
                steps: ctx.param("steps"),
                damping: ctx.param("damping"),
                bezier: [
                    ctx.param("bez_x1"),
                    ctx.param("bez_y1"),
                    ctx.param("bez_x2"),
                    ctx.param("bez_y2"),
                ],
            },
            custom: ctx.text_param(CURVE_KEY).and_then(ph2d_curve::parse),
        }
    }

    fn ease(&self, raw: f32) -> f32 {
        if self.curve == ph2d_ease::CUSTOM {
            return self.custom.as_ref().map_or(raw, |c| c.eval(raw));
        }
        ph2d_ease::ease(self.curve, self.dir, raw, &self.shape)
    }
}

impl NodeOp for ValueTime {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let rate = ctx.param("rate");
        let offset = ctx.param("offset");
        let stagger = ctx.param("stagger");
        let delay = ctx.param("delay");
        let t = ctx.playhead() as f32;
        // Cardinality follows the geometry: N from the (optional) input, else the
        // length-1 global clock (broadcast by `motion.drive`).
        let n = ctx.input(0).count().max(1);
        let atraso = (delay != 0.0).then(|| Atraso::of(ctx));
        let input = ctx.input(0);
        let posicoes: &[[f32; 2]] = match input.get("P") {
            Some(Column::Vec2(v)) => v,
            _ => &[],
        };
        let value: Vec<f32> = (0..n)
            .map(|i| {
                let base = t * rate + offset + i as f32 * stagger;
                match &atraso {
                    // ⚠️ Só com o knob armado: o neutro é o relógio de sempre, ao bit.
                    Some(a) => {
                        let p = posicoes.get(i).copied().unwrap_or([0.0, 0.0]);
                        #[expect(
                            clippy::cast_possible_truncation,
                            reason = "a contagem cabe num u32"
                        )]
                        let raw = ph2d_ease::order::raw_at(&a.ordem, i as u32, n as u32, p);
                        base - delay * a.ease(raw)
                    }
                    None => base,
                }
            })
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(value)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueTime))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_luts(MANIFEST.id, LUTS);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_key: "node.value.time.name",
            // Utility grey: a value SOURCE, plumbing (not a visible transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_gates_above(MANIFEST.id, WAVE_GATES);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

mod ui;
use ui::{PARAM_GATES, PARAM_HINTS, PARAM_UNITS, WAVE_GATES};

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
