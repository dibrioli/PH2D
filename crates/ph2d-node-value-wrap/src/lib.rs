#![forbid(unsafe_code)]
//! `value.wrap` — the value-domain ADDRESS mode: fold a field's out-of-range
//! values back into `[min, max]` by **Clamp**, **Repeat**, or **Mirror** (Motion
//! Nodes M2, the value domain — doc 12/79). Where `value.map_range` LINEARLY
//! rescales one interval onto another (and clamps at the ends), this one takes a
//! field that already runs off the ends of a range and decides *what happens past
//! the edge* — the same choice a sampler makes at a texture boundary, or an
//! animation makes past its last key.
//!
//! **The gold standard is the texture-address trio** every renderer ships —
//! `ClampToEdge` / `Repeat` / `MirroredRepeat` (Vulkan/GL/WebGPU), After Effects'
//! loopOut `Continue`/`Cycle`/`PingPong`, Houdini VEX `clamp`/`(x%w)`/triangle.
//! All three are **transcendental-free** (HR-5): a `clamp`, or a `floor`-based
//! fold — so the GPU port is bit-comparable to the CPU and the node is
//! **device-resident** (it cooks on the GPU, no CPU fallback).
//!
//! **`mode`** picks the edge behaviour over the range `[min, max]` (width
//! `w = max − min`):
//! - **Clamp** — hold at the edges: below `min` reads `min`, above `max` reads
//!   `max`. A plateau either side (`ClampToEdge` / loopOut Continue).
//! - **Repeat** — tile the range: the value wraps every `w`, a **sawtooth** that
//!   jumps from `max` back to `min` (`Repeat` / loopOut Cycle). `max` itself maps
//!   to `min` (a half-open `[min, max)` tile, the sampler convention).
//! - **Mirror** — fold back and forth: a **triangle** that rises to `max` then
//!   falls to `min` and back, period `2w` (`MirroredRepeat` / loopOut PingPong).
//!
//! **A composer, not a producer.** Feed it a `value.instance_field` Ramp scaled
//! past `1.0` (or any field with reach beyond the range) and Repeat tiles the
//! grid into `N` copies of the ramp, Mirror into a zig-zag — an authored spatial
//! period that no single producer gives. `value.wrap` after `value.map_range`
//! places the range where you want it; `value.quantize` after it stairs the tile.
//!
//! **The output is NOT a `[0,1]` mask** — it lands in `[min, max]`, on whatever
//! scale the range names (a comparison-free fold is meaningful on any scale). A
//! **degenerate range** (`max ≤ min`, `w ≤ 0`) has nothing to fold into, so the
//! whole field pins to `min` — finite, never a division by zero. `Pure` (no
//! clock, no state); a **unary** map, length preserved.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// How the fold treats values past the ends of `[lo, hi]`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Hold at the edges — a plateau either side (`ClampToEdge`).
    Clamp,
    /// Tile the range — a sawtooth, `max` wraps to `min` (`Repeat`).
    Repeat,
    /// Fold back and forth — a triangle, period `2w` (`MirroredRepeat`).
    Mirror,
}

impl Mode {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Mode::Repeat,
            2 => Mode::Mirror,
            _ => Mode::Clamp,
        }
    }
}

/// Fold one value into `[lo, hi]` by `mode`. A degenerate range (`hi ≤ lo`) pins
/// to `lo` — there is no interval to fold into, and this is what keeps every path
/// finite (no division by a zero width). Transcendental-free: a `clamp`, or a
/// `floor`-based positive modulo.
fn wrap_one(v: f32, lo: f32, hi: f32, mode: Mode) -> f32 {
    let w = hi - lo;
    if w <= 0.0 {
        return lo; // degenerate range: nothing to fold into
    }
    match mode {
        Mode::Clamp => v.clamp(lo, hi),
        Mode::Repeat => {
            // Positive modulo into [0, w): r − w·⌊r/w⌋. `max` (r = w) maps to `min`.
            let r = v - lo;
            lo + (r - w * (r / w).floor())
        }
        Mode::Mirror => {
            // Fold over a period 2w: rise on [0, w], fall on [w, 2w].
            let p = 2.0 * w;
            let r = v - lo;
            let m = r - p * (r / p).floor(); // m in [0, 2w)
            lo + if m > w { p - m } else { m }
        }
    }
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.wrap"),
    name: "value.wrap",
    inputs: &[
        PortSpec {
            name: "in",
            ty: VALUE,
        },
        // ⭐⭐ **A FAIXA COMO CAMPO** (doc 89, folha 15 linha 69) — o Blender recebe o
        // `Max`/`Min` do *Wrap* como **sockets**, logo como campos, e uma faixa
        // por-instância era **inexprimível** aqui: os params são uniformes no dispatch
        // inteiro, e nenhuma composição os torna por-elemento.
        //
        // ⚠️ **APENDADAS, nunca inseridas.** As arestas de um documento salvo guardam o
        // ÍNDICE da porta; a porta `in` continua a `0` e um doc de ontem abre igual.
        //
        // ⚠️ **Desligada ⇒ o param de hoje, ao bit** — e o comprimento manda: `0` valores
        // é o param, `1` **difunde** (uma faixa para o campo todo), `n` é por-elemento.
        // É a mesma escada da porta `time` do `motion.oscillator`.
        PortSpec {
            name: "lo",
            ty: VALUE,
        },
        PortSpec {
            name: "hi",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // `lo`/`hi`, not `min`/`max`: WGSL `min`/`max` are builtin functions, and the
    // param names become the uniform's struct-field names — `params.min` is a
    // field access, safe, but the sidestep costs nothing and reads clean.
    params: &[
        ParamSpec {
            name: "lo",
            default: 0.0,
        },
        ParamSpec {
            name: "hi",
            default: 1.0,
        },
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`wrap_one`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU (the "maximize GPU" north). VALUE in, VALUE out; the fold is ported
/// verbatim, `floor` and `clamp` matching the CPU's arithmetic. The only
/// device-divergence is the FMA the driver may fuse in `lo + (…)`, ε below the
/// parity budget; a value landing EXACTLY on a cell edge is the one place `floor`
/// could disagree by a whole period, so the parity fixture uses un-round params
/// (the `value.quantize`/`field.remap` precedent — a boundary is measure-zero and
/// the artist never sees it).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vw_mode = i32(vw_round(params.mode));\n\
        let vw_lo = params.lo;\n\
        let vw_hi = params.hi;\n\
        let vw_v = read_in_v(i);\n\
        let vw_w = vw_hi - vw_lo;\n\
        var vw_o: f32;\n\
        if (vw_w <= 0.0) {\n\
            vw_o = vw_lo;\n\
        } else if (vw_mode == 1) {\n\
            let vw_r = vw_v - vw_lo;\n\
            vw_o = vw_lo + (vw_r - vw_w * floor(vw_r / vw_w));\n\
        } else if (vw_mode == 2) {\n\
            let vw_p = 2.0 * vw_w;\n\
            let vw_r = vw_v - vw_lo;\n\
            let vw_m = vw_r - vw_p * floor(vw_r / vw_p);\n\
            vw_o = vw_lo + select(vw_m, vw_p - vw_m, vw_m > vw_w);\n\
        } else {\n\
            vw_o = clamp(vw_v, vw_lo, vw_hi);\n\
        }\n\
        write_v(i, vw_o);\n",
    wgsl_lib: "\
        fn vw_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // ⚠️⚠️ **ACRESCENTAR PORTAS RENOMEOU O ACESSOR DO KERNEL.** O
        // `accessor_suffix` do codegen qualifica pelo NOME DA PORTA assim que o nó tem
        // **mais de uma** entrada (senão `read_vel` do `motion.integrate` resolveria em
        // silêncio para a primeira das duas que o declaram). Este ficheiro era de entrada
        // única, logo o corpo estava escrito contra o `read_v` **nu** — e passou a ser
        // `read_in_v` no instante em que estas portas nasceram. *Uma porta nova reescreve o
        // vocabulário do kernel, e o WGSL é uma string que ninguém recompila.* Quem apanhou
        // foi o `every_registered_kernel_validates_across_the_whole_presence_space`.
        //
        // ⛔⛔ **A FAIXA POR FIO É CPU-ONLY, e o bloqueador tem nome.**
        //
        // O `identity` de um binding é uma **constante**: ele diz o que uma coluna AUSENTE
        // vale, e o padrão da casa para porta opcional com kernel é a contribuição ter
        // **identidade algébrica** (o `falloff` do `field.combine` é `1` porque multiplica;
        // o `accel` de uma força é `0` porque soma). ⚠️ **Uma FAIXA não tem identidade
        // algébrica**: o valor de recuo destas portas é *o param deste nó*, que muda por
        // instância de nó e não cabe numa constante de compilação.
        //
        // As saídas medidas eram três, e duas são piores: (a) semântica **aditiva**
        // (`param + porta`, identidade `0`) — exprime tudo e obriga o artista a zerar o
        // param para usar a porta como absoluto, que é acoplamento escondido; (b) identidade
        // = o **default** do param — parte toda cena que já tenha `lo`/`hi` autorados e a
        // porta desligada. ⇒ (c) **recuar**: com fio, o `eval` da CPU — que é o caminho
        // canónico — responde, e o nó desligado continua **inteiro no device**, que é o
        // norte declarado deste ficheiro para o caso normal.
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::RefuseIfPresent,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::RefuseIfPresent,
            identity: [0.0; 4],
            port: 2,
        },
    ],
    params: &["lo", "hi", "mode"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// Os valores escalares de uma porta (ausente ⇒ vazio).
fn scalars(stream: &Stream) -> Vec<f32> {
    match stream.get(VALUE_COL) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// A escada do comprimento de uma porta opcional: **vazia ⇒ o param**, **um valor
/// DIFUNDE** para o campo inteiro, e `n` é por-elemento.
///
/// ⚠️ O caso do meio não é conforto: uma faixa é a mesma para todo o campo com muito mais
/// frequência do que é diferente, e sem a difusão ligar um `value.lfo` a `hi` exigiria que
/// ele tivesse o comprimento do campo — o que um gerador de UM número não tem.
fn at(field: &[f32], i: usize, fallback: f32) -> f32 {
    match field.len() {
        0 => fallback,
        1 => field[0],
        _ => field.get(i).copied().unwrap_or(fallback),
    }
}

struct ValueWrap;

impl NodeOp for ValueWrap {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let lo = ctx.param("lo");
        let hi = ctx.param("hi");
        let mode = Mode::from_param(ctx.param("mode"));
        // As portas de faixa (opcionais) — ver o `PortSpec` de `lo`.
        let lo_field = scalars(ctx.input(1));
        let hi_field = scalars(ctx.input(2));
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input
            .iter()
            .enumerate()
            .map(|(i, &v)| wrap_one(v, at(&lo_field, i, lo), at(&hi_field, i, hi), mode))
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueWrap))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Wrap",
            // Utility grey: a value->value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    // The range the field folds into. `0..1` is the natural home (the normalised
    // convention), but the fold is scale-free: any `[min, max]` is a valid tile.
    ParamUiHint {
        param: "lo",
        label: "Min",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "hi",
        label: "Max",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Clamp", "Repeat", "Mirror"],
        },
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
