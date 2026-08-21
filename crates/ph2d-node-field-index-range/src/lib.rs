#![forbid(unsafe_code)]
//! `field.index_range` — a Motion **focus field keyed by INDEX, not position**
//! (Cavalry's Range Falloff). It writes the same multiplicative `falloff` column
//! `motion.falloff` does — the sister-of-`accel` mask contract (§1.2) that every
//! downstream modifier scales its effect by — but the mask is a function of an
//! instance's *ordinal* `i / (n-1)` within the count, never its `(x,y)`. That is
//! the one field a spatial shape cannot express: "the first third of the clones",
//! "instances 40 %..60 %", the stagger-by-rank a grid+falloff can't reach.
//!
//! It is a **soft band** `[start, end]` (fractions of the count, `0..1`) with a
//! ramp of width `soft` at each edge, shaped by the same 4-curve as `motion.falloff`
//! (Linear/Quad/Smooth/Smoother — HR-5, polynomials only). `start > end` auto-swaps
//! (drag either handle past the other). It **multiplies** into any existing `falloff`
//! so fields compose (the MOPs contract), and passes every other column through
//! unchanged (count preserved). Pure. **Transcendental-free** (HR-5): the mask is
//! bit-identical across platforms for the replay hash.
//!
//! Params (read via `ctx.param`): `start` (0.25), `end` (0.75), `soft` (0.10 —
//! a *living* default: a newly-dropped node masks a visible middle band, never the
//! inert full range, D12), `curve` (2 Smooth), `invert` (0/1 — flips to `1 − f`).
//! The neutral is `start=0, end=1, soft=0, invert=0` ⇒ mask `1` everywhere ⇒ the
//! `falloff` column is multiplied by the identity, byte-for-byte unchanged.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `attr` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// **O QUE ORDENA A BANDA** (doc 89 folha 10 — MOPs *"Falloff From Attribute"*, C4D
/// Random effector modo **Indexed**).
///
/// - `0` **Index** — a ordem do stream, `s = i/(n−1)`. O que este nó sempre fez.
/// - `1` **Attribute** — o **POSTO** do elemento no campo ligado à porta `attr`,
///   `s = rank/(n−1)`. A ordenação **que não reordena**.
///
/// ⚠️ **A composição que já existia REORDENA, e é isso que a torna outra coisa.**
/// `motion.sort(key) → field.index_range` dá o mesmo posto — é literalmente o
/// mecanismo que a §4 do plano 89 cita no `motion.slit_scan` — mas o `sort` permuta o
/// stream **para sempre a jusante**: a ordem-z muda, o pareamento por índice muda, e
/// nas referências o falloff-por-atributo não reordena coisa nenhuma. O gap é o
/// *não-destrutivo*, não o posto.
///
/// ⚠️ **O `Auto Range` da mesma citação NÃO entrou, e isso é uma MEDIÇÃO, não um
/// esquecimento.** *"remapeia atributo existente→falloff (min/max + Auto Range)"* já é
/// exprimível hoje, e sem cair para a CPU: `value.attribute → value.normalize(Range) →
/// motion.drive(Falloff, Set)` — o `value.normalize` **descobre** o extento do campo
/// (é a razão de ele existir) e é device-resident. Construir um segundo modo aqui seria
/// construir o que já existe. *Meça se a composição já o exprime antes de escrever o
/// item da lista.*
///
/// ⚠️ **Ligado, ele RECUSA o device** (`applicable`, a porta dos irmãos
/// `motion.combine`/`motion.cull`): um posto é uma ORDENAÇÃO global, e um kernel
/// por-elemento só a alcançaria contando `#{j : v_j < v_i}` — `O(n²)`, que a 262 mil
/// elementos são 6,9·10¹⁰ comparações. Desligado (o default) nada recua.
const KEY: &str = "key";
/// O valor de [`KEY`] que pede o posto por atributo.
const KEY_ATTRIBUTE: f32 = 1.0;

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126, `register_gpu_kernel`); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.index_range"),
    name: "field.index_range",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // **O ATRIBUTO pelo qual ordenar** (modo `Attribute`). APENDADO — o índice
        // da porta 0 não se mexe, e desligada o nó é byte-idêntico. Ver [`KEY`].
        PortSpec {
            name: "attr",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "start",
            default: 0.25,
        },
        ParamSpec {
            name: "end",
            default: 0.75,
        },
        ParamSpec {
            name: "soft",
            default: 0.10,
        },
        ParamSpec {
            name: "curve",
            default: 2.0,
        },
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
        // **O QUE ORDENA A BANDA.** APENDADO, default `0` = a ordem do stream, ao
        // bit. Ver [`KEY`].
        ParamSpec {
            name: "key",
            default: 0.0,
        },
    ],
    // Same as `motion.falloff`: the frozen manifest declares only the CPU
    // lowering; the WGSL kernel below is the ADR-0126 side channel.
    lowerings: &[LoweringKind::Cpu],
};

/// An edge curve on a pre-clamped `s ∈ [0,1]` — the SAME transcendental-free set as
/// `motion.falloff` (HR-5). `0` Linear · `1` Quad · `2` Smooth (smoothstep) · `3`
/// Smoother (smootherstep). Every curve is monotone and endpoint-exact (`0→0`, `1→1`).
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,                                     // Quad
        2 => s * s * (3.0 - 2.0 * s),                   // Smooth (smoothstep)
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0), // Smoother (smootherstep)
        _ => s,                                         // Linear
    }
}

/// The band mask at normalized ordinal `s ∈ [0,1]`. `[start, end]` auto-swap into
/// `[lo, hi]`; `soft` is the ramp width at each edge, clamped so the two ramps meet
/// at most at the middle (a wide `soft` degenerates the trapezoid to a triangle,
/// never past). Because every `curve` is monotone, `curve(min(rise, fall))` equals
/// `min(curve(rise), curve(fall))` — one eval, shaping both edges. `soft = 0` gives
/// a hard rectangle; `[0,1]` gives the constant `1` (the identity a field multiplies
/// by, so an untouched full range never darkens the scene).
fn band_mask(s: f32, start: f32, end: f32, soft: f32, curve_kind: i32) -> f32 {
    let lo = start.min(end);
    let hi = start.max(end);
    let w = soft.max(0.0).min((hi - lo) * 0.5);
    let rise = if w > 0.0 {
        ((s - lo) / w).clamp(0.0, 1.0)
    } else if s >= lo {
        1.0
    } else {
        0.0
    };
    let fall = if w > 0.0 {
        ((hi - s) / w).clamp(0.0, 1.0)
    } else if s <= hi {
        1.0
    } else {
        0.0
    };
    curve(curve_kind, rise.min(fall))
}

/// O **ordinal normalizado** de cada elemento, `s ∈ [0,1]`.
///
/// Sem atributo (o modo `Index`, ou a porta desligada) é a posição no vector; com
/// atributo é o **posto** dele — `s[i] = rank(i)/(n−1)`, e o stream não se mexe.
///
/// ⚠️ **O desempate é o ÍNDICE, e ele não é decorativo:** sem ele, dois elementos com o
/// mesmo valor receberiam postos numa ordem que depende do algoritmo de ordenação, e o
/// hash de replay deixaria de bater entre plataformas. `total_cmp` dá a ordem total
/// (inclusive sobre `NaN` e `-0.0`), o índice fecha-a. *Não se escolhe um desempate
/// melhor: não se tem empate.*
///
/// ⚠️ **A porta VAZIA cai no modo `Index`, e não num campo de zeros.** Um `key =
/// Attribute` com nada ligado é o pedido incompleto do artista; responder com "todos
/// empatados, desempate pelo índice" dá o MESMO número, mas por acidente — e deixaria de
/// dar no dia em que o desempate mudasse. Cair no modo explícito diz a verdade.
fn ordinals(n: usize, attr: &[f32]) -> Vec<f32> {
    // `n.max(2) − 1` mantém o denominador ≥ 1 — o mesmo inteiro que o `max(count, 2u) − 1u`
    // do WGSL.
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de elementos")]
    let denom = (n.max(2) - 1) as f32;
    #[expect(clippy::cast_precision_loss, reason = "um posto < n")]
    let norm = |k: usize| if n > 1 { k as f32 / denom } else { 0.0 };
    if attr.is_empty() {
        return (0..n).map(norm).collect();
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        let (va, vb) = (
            attr.get(a).copied().unwrap_or(0.0),
            attr.get(b).copied().unwrap_or(0.0),
        );
        va.total_cmp(&vb).then(a.cmp(&b))
    });
    let mut s = vec![0.0_f32; n];
    for (rank, &i) in order.iter().enumerate() {
        s[i] = norm(rank);
    }
    s
}

/// GPU compute kernel (ADR-0126): a straight WGSL port of [`band_mask`] × [`curve`]
/// multiplied into the existing `falloff` — same polynomials, same `min`/`max`/
/// `clamp` (HR-5), so parity holds within float ULPs. The `curve` enum is routed
/// through `ir_round` (round-half-AWAY, matching Rust's `f32::round`; WGSL's builtin
/// `round` is half-even and would pick a different branch at `x.5` —
/// [[feedback_cpu_gpu_rounding_conventions_diverge]]) and `invert` through the
/// CPU's own `>= 0.5` threshold. The ordinal `s` reads `params.count` (the dispatch
/// width, = port 0's element count = the CPU's `input.count()`) so the two `s`
/// arithmetics are bit-identical. `ReadWrite` on `falloff` mirrors the CPU: an absent
/// column starts from the `1.0` identity (fields multiply) and is always written.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let ir_denom = f32(max(params.count, 2u) - 1u);\n\
        let ir_s = select(0.0, f32(i) / ir_denom, params.count > 1u);\n\
        let ir_m = ir_band_mask(\n\
            ir_s,\n\
            params.start,\n\
            params.end,\n\
            params.soft,\n\
            i32(ir_round(params.curve)));\n\
        var ir_f = ir_m;\n\
        if (params.invert >= 0.5) { ir_f = 1.0 - ir_m; }\n\
        write_falloff(i, read_falloff(i) * ir_f);\n",
    wgsl_lib: "\
        fn ir_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn ir_curve(kind: i32, s: f32) -> f32 {\n\
            if (kind == 1) { return s * s; }\n\
            if (kind == 2) { return s * s * (3.0 - 2.0 * s); }\n\
            if (kind == 3) { return s * s * s * (s * (s * 6.0 - 15.0) + 10.0); }\n\
            return s;\n\
        }\n\
        fn ir_band_mask(s: f32, start: f32, end: f32, soft: f32, curve_kind: i32) -> f32 {\n\
            let lo = min(start, end);\n\
            let hi = max(start, end);\n\
            let w = min(max(soft, 0.0), (hi - lo) * 0.5);\n\
            var rise: f32;\n\
            if (w > 0.0) { rise = clamp((s - lo) / w, 0.0, 1.0); }\n\
            else { rise = select(0.0, 1.0, s >= lo); }\n\
            var fall: f32;\n\
            if (w > 0.0) { fall = clamp((hi - s) / w, 0.0, 1.0); }\n\
            else { fall = select(0.0, 1.0, s <= hi); }\n\
            return ir_curve(curve_kind, min(rise, fall));\n\
        }\n",
    bindings: &[ColumnBinding {
        column: "falloff",
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [1.0; 4],
        port: 0,
    }],
    params: &["start", "end", "soft", "curve", "invert"],
    count_law: None,
    variant_by_param: None,
    // A recusa do modo `Attribute` — ver [`KEY`] para a conta que a justifica.
    applicable: Some(|p| p(KEY) < 0.5),
};

struct FieldIndexRange;

impl NodeOp for FieldIndexRange {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let start = ctx.param("start");
        let end = ctx.param("end");
        let soft = ctx.param("soft");
        let curve_kind = ctx.param("curve").round() as i32;
        let invert = ctx.param("invert") >= 0.5;
        // ⚠️ O atributo é lido ANTES do input 0 e clonado: os dois `ctx.input` não
        // podem coexistir emprestados, e a coluna só é tocada no modo que a lê.
        let attr: Vec<f32> = if ctx.param(KEY) >= KEY_ATTRIBUTE - 0.5 {
            match ctx.input(1).get(VALUE_COL) {
                Some(Column::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Existing per-instance falloff (fields multiply); absent → 1.
            let prev: Option<&[f32]> = match input.get("falloff") {
                Some(Column::Scalar(v)) => Some(v.as_slice()),
                _ => None,
            };
            // O ordinal de cada peça: a posição no vector, ou o POSTO no atributo.
            let ord = ordinals(n, &attr);
            let fall = par_build(n, |i| {
                let s = ord[i];
                let m = band_mask(s, start, end, soft, curve_kind);
                let f = if invert { 1.0 - m } else { m };
                let base = prev.and_then(|v| v.get(i).copied()).unwrap_or(1.0);
                base * f
            });
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "falloff" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("falloff", Column::Scalar(fall));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via `ph2d-node-sync`
/// codegen) from `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FieldIndexRange))?;
    // A focus field → amber, diamond value silhouette (same class as Falloff).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Index Range",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // The WGSL lowering, registered on the side (ADR-0126).
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): fractional Start/End/Softness sliders (0..1), a named
/// Curve selector, an Invert checkbox — never number sliders for the enum.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "start",
        label: "Start",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "end",
        label: "End",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "soft",
        label: "Softness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "curve",
        label: "Curve",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Quad", "Smooth", "Smoother"],
        },
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⚠️ Um Enum NOMEADO, não um toggle: o segundo modo precisa de um FIO na porta
    // `attr`, e um rótulo que diz *"Attribute"* é o que faz o artista procurá-la.
    ParamUiHint {
        param: "key",
        label: "Order By",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Index", "Attribute"],
        },
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
