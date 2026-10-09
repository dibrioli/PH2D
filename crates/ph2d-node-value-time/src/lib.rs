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

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rate",
        label: "node.value.time.param.rate",
        min: -8.0,
        max: 8.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "node.value.time.param.offset",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "stagger",
        // ⚠️ **`Phase Stagger`** (ciclo 6 W3): o `pulse.beat` pergunta a MESMA coisa — a fase
        // POR LINHA — e escrevia o nome inteiro. Três nós, uma palavra.
        label: "node.value.time.param.stagger",
        min: -2.0,
        max: 2.0,
        step: 0.02,
        widget: ParamWidget::Slider,
    },
    // ⭐ CICLO 17 — o ATRASO pela ordem e pela curva da casa (doc 126 §3.3).
    ParamUiHint {
        param: "delay",
        label: "node.value.time.param.delay",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "order",
        label: "node.value.time.param.order",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.time.param.order.0",
                "node.value.time.param.order.1",
                "node.value.time.param.order.2",
                "node.value.time.param.order.3",
                "node.value.time.param.order.4",
            ],
        },
    },
    ParamUiHint {
        param: "seed",
        label: "node.value.time.param.seed",
        min: 0.0,
        max: 999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "ease_curve",
        label: "node.value.time.param.ease_curve",
        min: 0.0,
        max: 14.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.time.param.ease_curve.0",
                "node.value.time.param.ease_curve.1",
                "node.value.time.param.ease_curve.2",
                "node.value.time.param.ease_curve.3",
                "node.value.time.param.ease_curve.4",
                "node.value.time.param.ease_curve.5",
                "node.value.time.param.ease_curve.6",
                "node.value.time.param.ease_curve.7",
                "node.value.time.param.ease_curve.8",
                "node.value.time.param.ease_curve.9",
                "node.value.time.param.ease_curve.10",
                "node.value.time.param.ease_curve.11",
                "node.value.time.param.ease_curve.12",
                "node.value.time.param.ease_curve.13",
                "node.value.time.param.ease_curve.14",
            ],
        },
    },
    // A curva DESENHADA da `Custom` — um TEXT param, não um `ParamSpec`.
    ParamUiHint {
        param: CURVE_KEY,
        label: "node.value.time.param.curve",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Curve,
    },
    ParamUiHint {
        param: "ease_dir",
        label: "node.value.time.param.ease_dir",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.time.param.ease_dir.0",
                "node.value.time.param.ease_dir.1",
                "node.value.time.param.ease_dir.2",
            ],
        },
    },
    ParamUiHint {
        param: "overshoot",
        label: "node.value.time.param.overshoot",
        min: 0.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "period",
        label: "node.value.time.param.period",
        min: 0.05,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "steps",
        label: "node.value.time.param.steps",
        min: 1.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "damping",
        label: "node.value.time.param.damping",
        min: 0.05,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "bez_x1",
        label: "node.value.time.param.bez_x1",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "bez_y1",
        label: "node.value.time.param.bez_y1",
        min: -1.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "bez_x2",
        label: "node.value.time.param.bez_x2",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "bez_y2",
        label: "node.value.time.param.bez_y2",
        min: -1.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "grid_cols",
        label: "node.value.time.param.grid_cols",
        min: 1.0,
        max: 1000.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "grid_from",
        label: "node.value.time.param.grid_from",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.time.param.grid_from.0",
                "node.value.time.param.grid_from.1",
                "node.value.time.param.grid_from.2",
                "node.value.time.param.grid_from.3",
            ],
        },
    },
    ParamUiHint {
        param: "grid_cell",
        label: "node.value.time.param.grid_cell",
        min: 0.0,
        max: 10000.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "axis",
        label: "node.value.time.param.axis",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.time.param.axis.0",
                "node.value.time.param.axis.1",
                "node.value.time.param.axis.2",
            ],
        },
    },
    ParamUiHint {
        param: "origin_x",
        label: "node.value.time.param.origin_x",
        min: -50.0,
        max: 50.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "origin_y",
        label: "node.value.time.param.origin_y",
        min: -50.0,
        max: 50.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "reach",
        label: "node.value.time.param.reach",
        min: 0.0,
        max: 50.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

use ph2d_node_registry::{ParamGate, ParamGateAbove, ParamUnit, ParamUnitDecl};

/// **Um controlo que o cozimento não abre não é pintado.** A ordem e a curva escolhem-se como no
/// `motion.stagger`; e TODOS os números da onda só existem com o `delay` armado ([`WAVE_GATES`]).
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: CURVE_KEY,
        when: "ease_curve",
        values: &[ph2d_ease::CUSTOM],
    },
    ParamGate {
        param: "ease_dir",
        when: "ease_curve",
        values: ph2d_ease::USES_DIRECTION,
    },
    ParamGate {
        param: "overshoot",
        when: "ease_curve",
        values: &[ph2d_ease::BACK],
    },
    ParamGate {
        param: "period",
        when: "ease_curve",
        values: &[ph2d_ease::ELASTIC],
    },
    ParamGate {
        param: "steps",
        when: "ease_curve",
        values: &[ph2d_ease::STEPS],
    },
    ParamGate {
        param: "damping",
        when: "ease_curve",
        values: &[ph2d_ease::SPRING],
    },
    ParamGate {
        param: "bez_x1",
        when: "ease_curve",
        values: &[ph2d_ease::BEZIER],
    },
    ParamGate {
        param: "bez_y1",
        when: "ease_curve",
        values: &[ph2d_ease::BEZIER],
    },
    ParamGate {
        param: "bez_x2",
        when: "ease_curve",
        values: &[ph2d_ease::BEZIER],
    },
    ParamGate {
        param: "bez_y2",
        when: "ease_curve",
        values: &[ph2d_ease::BEZIER],
    },
    ParamGate {
        param: "seed",
        when: "order",
        values: &[ph2d_ease::order::ORDER_RANDOM],
    },
    ParamGate {
        param: "grid_cols",
        when: "order",
        values: &[ph2d_ease::order::ORDER_GRID],
    },
    ParamGate {
        param: "grid_from",
        when: "order",
        values: &[ph2d_ease::order::ORDER_GRID],
    },
    ParamGate {
        param: "grid_cell",
        when: "order",
        values: &[ph2d_ease::order::ORDER_GRID],
    },
    ParamGate {
        param: "grid_cell",
        when: "grid_from",
        values: &[ph2d_ease::order::FROM_CELL],
    },
    ParamGate {
        param: "axis",
        when: "order",
        values: &[
            ph2d_ease::order::ORDER_GRID,
            ph2d_ease::order::ORDER_FROM_POINT,
        ],
    },
    ParamGate {
        param: "origin_x",
        when: "order",
        values: &[ph2d_ease::order::ORDER_FROM_POINT],
    },
    ParamGate {
        param: "origin_y",
        when: "order",
        values: &[ph2d_ease::order::ORDER_FROM_POINT],
    },
    ParamGate {
        param: "reach",
        when: "order",
        values: &[ph2d_ease::order::ORDER_FROM_POINT],
    },
];

/// ⭐ **A onda só aparece com o `delay` armado** (`delay > 0`, a faixa do slider começa em `0`):
/// com o atraso a zero o termo nem corre, e vinte controlos inertes no cartão seriam o knob morto
/// que o doc 90 caçou. Os gates SOMAM-SE aos de [`PARAM_GATES`].
static WAVE_GATES: &[ParamGateAbove] = &[
    ParamGateAbove {
        param: "order",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "seed",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "ease_curve",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: CURVE_KEY,
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "ease_dir",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "overshoot",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "period",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "steps",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "damping",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "bez_x1",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "bez_y1",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "bez_x2",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "bez_y2",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "grid_cols",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "grid_from",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "grid_cell",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "axis",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "origin_x",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "origin_y",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "reach",
        when: "delay",
        above: 0.0,
    },
];

/// O atraso é TEMPO; o ponto e o alcance são lugares do mundo.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "delay",
        unit: ParamUnit::Seconds,
    },
    ParamUnitDecl {
        param: "origin_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "origin_y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "reach",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // A grid source: `n` instances at the origin, so `value.time` can read a count.
    static GRID_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.time.test.grid"),
        name: "value.time.test.grid",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Grid;
    impl NodeOp for Grid {
        fn manifest(&self) -> &'static NodeManifest {
            &GRID_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == GRID_MAN.id => Some(&Grid),
                t if t == MANIFEST.id => Some(&ValueTime),
                _ => None,
            }
        }
    }

    fn vals(s: &Stream) -> Vec<f32> {
        match s.get(VALUE_COL).unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("v"),
        }
    }

    /// Cook `value.time` at `playhead`; `connect_grid` decides whether it reads a
    /// count from a source (length-N) or stands alone (length-1 global).
    fn time_at(
        playhead: f64,
        connect_grid: bool,
        setup: impl FnOnce(&mut Graph, NodeId),
    ) -> Vec<f32> {
        let mut g = Graph::new();
        let time = g.add_node("value.time");
        if connect_grid {
            let grid = g.add_node("value.time.test.grid");
            g.connect(Edge {
                from: (grid, 0),
                to: (time, 0),
                delayed: false,
            })
            .unwrap();
        }
        setup(&mut g, time);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, time, playhead).unwrap();
        vals(out[0].as_stream())
    }

    /// UNCONNECTED input → one global clock (length-1). This is the field that
    /// `motion.drive` broadcasts across every instance — *the* clock (doc 12).
    #[test]
    fn an_unconnected_time_emits_a_single_global_clock() {
        // Default rate 1 → the value IS the playhead.
        assert_eq!(time_at(0.0, false, |_, _| {}), vec![0.0], "t=0 -> 0");
        assert_eq!(time_at(2.5, false, |_, _| {}), vec![2.5], "t=2.5 -> 2.5");
        // Rate scales the clock; offset shifts its start.
        let v = time_at(2.0, false, |g, t| {
            g.set_param(t, "rate", 3.0);
            g.set_param(t, "offset", 1.0);
        });
        assert_eq!(v, vec![7.0], "2*3 + 1 = 7");
    }

    /// **Monotonic, not periodic** — the clock keeps CLIMBING past any period,
    /// where `value.lfo(Saw)` would fold back. Falsifiable: a wrapped clock would
    /// read the same at `t` and `t + 1/rate`.
    #[test]
    fn the_clock_climbs_and_never_folds() {
        let at = |t: f64| time_at(t, false, |g, node| g.set_param(node, "rate", 1.0))[0];
        let a = at(0.5);
        let b = at(10.5); // ten "periods" later — a Saw would read the same
        assert!(
            b > a + 9.5,
            "the clock climbed by ~10, did not fold: {a} -> {b}"
        );
    }

    /// CONNECTED input → the field's length follows the geometry (N=3), and
    /// `stagger` sends a travelling ramp across it: at t=0 with stagger 0.5,
    /// instance i reads `i·0.5`. As t advances the whole ramp shifts up.
    #[test]
    fn a_connected_time_emits_a_travelling_ramp() {
        let v = time_at(0.0, true, |g, t| g.set_param(t, "stagger", 0.5));
        assert_eq!(v, vec![0.0, 0.5, 1.0], "spatial ramp at t=0");
        // t=2 shifts every element up by 2 (rate 1) — the ramp travels.
        let v = time_at(2.0, true, |g, t| g.set_param(t, "stagger", 0.5));
        assert_eq!(v, vec![2.0, 2.5, 3.0], "the ramp travelled up by t");
    }

    /// A negative rate runs the clock BACKWARDS — the value decreases as the
    /// playhead advances (reverse scroll / counter-rotation).
    #[test]
    fn a_negative_rate_runs_backwards() {
        let v = time_at(3.0, false, |g, t| g.set_param(t, "rate", -2.0));
        assert_eq!(v, vec![-6.0], "3*(-2) = -6");
    }

    /// ⭐ **O ATRASO NO TEMPO** (ciclo 17): `delay` distribui-se pela ordem e pela curva — a
    /// primeira linha não espera, a última espera o `delay` inteiro; com `Quad In` o meio espera
    /// só um quarto.
    #[test]
    fn the_delay_spreads_over_the_order_and_the_curve() {
        let v = time_at(2.0, true, |g, t| g.set_param(t, "delay", 1.0));
        assert_eq!(v, vec![2.0, 1.5, 1.0], "linear: a cauda espera 1 s");
        let v = time_at(2.0, true, |g, t| {
            g.set_param(t, "delay", 1.0);
            g.set_param(t, "ease_curve", ph2d_ease::QUAD as f32);
        });
        assert_eq!(v, vec![2.0, 1.75, 1.0], "Quad In: o meio espera 0,25");
        // Do centro para fora: o do meio arranca primeiro.
        let v = time_at(2.0, true, |g, t| {
            g.set_param(t, "delay", 1.0);
            g.set_param(t, "order", ph2d_ease::order::ORDER_FROM_CENTER as f32);
        });
        assert_eq!(v, vec![1.0, 2.0, 1.0], "o centro não espera");
    }

    /// Com o `delay` a zero o relógio é o de sempre AO BIT, mesmo com a onda toda configurada.
    #[test]
    fn a_zero_delay_is_the_old_clock_bit_for_bit() {
        let antes = time_at(1.37, true, |g, t| g.set_param(t, "stagger", 0.13));
        let depois = time_at(1.37, true, |g, t| {
            g.set_param(t, "stagger", 0.13);
            g.set_param(t, "order", 3.0);
            g.set_param(t, "ease_curve", 11.0);
        });
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&antes), bits(&depois));
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
