#![forbid(unsafe_code)]
//! `value.smooth` — the value-domain FILTER: soften a field by averaging each
//! element with its index-neighbours, a weighted moving average over the
//! instance order (Motion Nodes M2, the value domain — doc 12/77). It is the
//! de-noisier: a jagged driver — a `value.noise`, an `instance_field` Random —
//! smoothed into a gradual one. The Filter/Lag CHOP of TouchDesigner, the Smooth
//! of Cavalry, the Blur Attribute of Blender.
//!
//! **Unlike every other value node so far, element `i`'s answer reads its
//! NEIGHBOURS** — `v[i−r] … v[i+r]` — not just `v[i]`. It is the `Filter CHOP`
//! shape: a moving average over the ordered field. The order is meaningful when
//! the instances are laid out in sequence (a row, a grid) — the common case for a
//! per-instance driver.
//!
//! **`radius`** is the half-window (`0` = a bit-exact passthrough, the neutral
//! default). **`weight`** is the SHAPE of that window — what each tap counts for:
//!
//! - **Box** (default) — every tap counts `1`. `out[i]` is the plain mean of the
//!   window. This is what the node always did, and it is **bit-exact**: the
//!   weights are `1.0`, `1.0 * x` is exactly `x` in IEEE-754, and `Σ 1.0` over
//!   `2r+1` taps is exactly `2r+1`, which is the divisor that shipped.
//! - **Triangle** — the tap `d` away counts `r+1−d`, a linear falloff to the rim.
//! - **Smooth** — the same falloff run through the house's own smoothstep
//!   (`t²(3−2t)`, the one `value.step` and `value.map_range` already speak): a
//!   bell whose tails land softly. ⚠️ **Polynomial, never `exp`** — a Gaussian's
//!   transcendental would be the platform libm on one side and the vendor's on
//!   the other, and the parity of a *weight* is not the place to spend that.
//!
//! The edges **extend** (a clamped index repeats the boundary value), so the
//! window is always `2r+1` taps and the divisor is the weight sum. The window is
//! accumulated **left to right** on BOTH paths — a per-element fixed-order sum,
//! NOT the tree reduction whose order the `reduce` channel documents an ε for.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). `Pure` (no clock, no state);
//! length preserved. The GPU kernel reads the neighbours off the input buffer, so
//! it is **device-resident** (no CPU fallback) with the existing kernel channel —
//! no reduction, no scan.

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

/// **Onde um raio deixa de ser um filtro e passa a ser o quadro** — MEDIDO.
///
/// O custo é `O(N · (2r+1))` nos dois caminhos, e a sonda
/// `measure_what_a_radius_costs` mede-o em **0,76-1,06 ns por tap** na CPU, num
/// campo de **10 000** elementos (a ordem de grandeza de um driver
/// por-instância):
///
/// | raio | taps    | ms     |
/// |------|---------|--------|
/// | 1    | 30 k    | 0,032  |
/// | 64   | 1,29 M  | 0,984  |
/// | 128  | 2,57 M  | 2,010  |
/// | 256  | 5,13 M  | 4,530  |
/// | 512  | 10,25 M | **10,151** |
///
/// O teto sai da última linha: a `512` um único filtro come **10,2 ms**, ou seja
/// o quadro de 60 fps inteiro. É esse o recurso — **TEMPO**, linear no raio — e
/// não a memória nem a precisão.
///
/// ⚠️ **O outro limite, o do SIGNIFICADO, é função do campo e não cabe aqui:**
/// com `r ≥ N` toda janela cobre o conjunto inteiro e a saída fica CONSTANTE —
/// subir mais deixa de mudar qualquer coisa. Isso é visível na tela (um campo
/// chato) e depende de um número que a UI não conhece no momento em que
/// desenha o slider, então o teto que se escreve é o do relógio.
const RADIUS_LAST_USEFUL: f32 = 512.0;

/// Which shape the window's taps carry — the `Weight` of Blender's Blur
/// Attribute, the filter TYPE of TouchDesigner's Filter CHOP.
///
/// ⚠️ **Os índices são a face que o documento GUARDA** — `Box` é o `0` que todo
/// grafo salvo carrega, e a lista só cresce pelo FIM.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Weight {
    /// Every tap counts `1` — the plain mean. What the node always did.
    Box,
    /// Linear falloff to the rim: the tap `d` away counts `r+1−d`.
    Triangle,
    /// The linear falloff run through the house smoothstep — a bell.
    Smooth,
}

impl Weight {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Weight::Triangle,
            2 => Weight::Smooth,
            _ => Weight::Box,
        }
    }
}

/// **O peso do tap a `dist` do centro**, para meia-janela `r`. Não-normalizado:
/// quem divide é a soma dos pesos, computada no mesmo laço.
///
/// `dist ≤ r` sempre, então `r + 1 − dist ≥ 1` — nenhum tap da janela pesa zero,
/// nem no aro. Um perfil que zerasse na borda desperdiçaria dois taps e faria o
/// raio mentir sobre o próprio alcance.
fn tap_weight(weight: Weight, dist: u32, r: u32) -> f32 {
    match weight {
        Weight::Box => 1.0,
        Weight::Triangle => (r + 1 - dist) as f32,
        Weight::Smooth => {
            let t = (r + 1 - dist) as f32 / (r + 1) as f32;
            t * t * (3.0 - 2.0 * t)
        }
    }
}

/// Weighted moving average of one field by `radius`. `radius = 0` is a
/// passthrough (bit-exact). The window is accumulated LEFT TO RIGHT with
/// edge-clamped indices, exactly as the WGSL does, so the two paths agree — a
/// fixed-order per-element sum, not a tree reduction.
///
/// ⚠️ **`Box` is bit-identical to the unweighted code that shipped**, and that is
/// arithmetic rather than promise: its weights are `1.0`, `1.0 * x` is exactly
/// `x`, and `Σ 1.0` over `2r+1` taps is exactly the integer divisor `2r+1`.
fn smooth(field: &[f32], radius: usize, weight: Weight) -> Vec<f32> {
    let n = field.len();
    if radius == 0 || n == 0 {
        return field.to_vec();
    }
    let r = radius as isize;
    let last = n as isize - 1;
    (0..n)
        .map(|i| {
            let mut sum = 0.0f32;
            let mut wsum = 0.0f32;
            let mut k = i as isize - r;
            let hi = i as isize + r;
            while k <= hi {
                let w = tap_weight(
                    weight,
                    (k - i as isize).unsigned_abs() as u32,
                    radius as u32,
                );
                let idx = k.clamp(0, last) as usize;
                sum += w * field[idx];
                wsum += w;
                k += 1;
            }
            sum / wsum
        })
        .collect()
}

/// The static contract of this node type (ADR-0031). The kernel and the param
/// side-tables are side-metadata (ADR-0126); `NodeManifest` stays the frozen 8
/// fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.smooth"),
    name: "value.smooth",
    inputs: &[PortSpec {
        name: "in",
        ty: VALUE,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "radius",
            default: 0.0,
        },
        ParamSpec {
            name: "weight",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`smooth`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU. VALUE in, VALUE out; the binding is `ReadWrite` because the kernel
/// READS the input (its neighbours, off `in_v`) and WRITES a fresh `out_v` — the
/// two are separate buffers, so a write never corrupts a neighbour read. The
/// window accumulates left to right, matching the CPU order exactly.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vs_r = i32(vs_round(params.radius));\n\
        if (vs_r <= 0) {\n\
            write_v(i, read_v(i));\n\
        } else {\n\
            let vs_w = i32(vs_round(params.weight));\n\
            let vs_last = i32(params.count) - 1;\n\
            let vs_span = f32(vs_r + 1);\n\
            var vs_sum = 0.0;\n\
            var vs_wsum = 0.0;\n\
            var vs_k = -vs_r;\n\
            loop {\n\
                if (vs_k > vs_r) { break; }\n\
                let vs_lin = f32(vs_r + 1 - abs(vs_k));\n\
                var vs_tw = 1.0;\n\
                if (vs_w == 1) {\n\
                    vs_tw = vs_lin;\n\
                } else if (vs_w == 2) {\n\
                    let vs_t = vs_lin / vs_span;\n\
                    vs_tw = vs_t * vs_t * (3.0 - 2.0 * vs_t);\n\
                }\n\
                let vs_idx = clamp(i32(i) + vs_k, 0, vs_last);\n\
                vs_sum = vs_sum + vs_tw * read_v(u32(vs_idx));\n\
                vs_wsum = vs_wsum + vs_tw;\n\
                vs_k = vs_k + 1;\n\
            }\n\
            write_v(i, vs_sum / vs_wsum);\n\
        }\n",
    wgsl_lib: "\
        fn vs_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["radius", "weight"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueSmooth;

impl NodeOp for ValueSmooth {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Radius is a non-negative half-window; round and clamp at 0.
        let radius = ctx.param("radius").round().max(0.0) as usize;
        let weight = Weight::from_param(ctx.param("weight"));
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        let out = smooth(&input, radius, weight);
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueSmooth))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Smooth",
            // Utility grey: a value->value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        // The half-window of the moving average. `0` is a passthrough; higher
        // softens more AND reaches further. The slider stops where the hand
        // works; the typable box goes to [`RADIUS_LAST_USEFUL`].
        param: "radius",
        label: "Radius",
        min: 0.0,
        max: 16.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        // ⚠️ A ordem dos rótulos É a ordem dos índices — o `weight` guardado num
        // documento é o índice, então a lista só cresce pelo FIM.
        param: "weight",
        label: "Weight",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Box", "Triangle", "Smooth"],
        },
    },
];

/// O teto que a MÁQUINA impõe, alcançável por DIGITAÇÃO — o slider fica onde a
/// MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes (16) é o que
/// o dedo percorre; nada ficou inalcançável.
///
/// ⚠️ **E este teto é load-bearing para a wave, não decoração:** é ele que torna
/// o `weight` capaz de alcançar o que uma pilha de passes de box alcançaria —
/// ver o gate `a_wide_smooth_window_reaches_what_repeated_box_passes_reach`.
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "radius",
    max: RADIUS_LAST_USEFUL,
}];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
