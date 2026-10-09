//! The GPU kernels for `motion.stagger` — one per target column.
//!
//! The stagger's delta is a pure function of the element INDEX (`i / (n−1)`,
//! eased, remapped into `min..max`, masked by `falloff`), which is why it ports
//! to the device without state: every element's answer depends on its own index
//! and the dispatch width, both of which the kernel already has.
//!
//! **Every channel**, by shipping one variant per target column
//! ([`GpuKernel::variant_by_param`]). It used to cover X/Y only: Rotation writes
//! `rot` and Size writes `size`, and a static binding set cannot switch its
//! output column on a param, so those two channels fell back to the CPU. The
//! four channel values map exactly as the CPU's `channel_column` does —
//! including its `_ => size` catch-all for an out-of-range value.
//!
//! The delta is computed identically in all three variants (the shared
//! `SG_LIB`); only the column it lands on differs, which is precisely why the
//! variants exist and not a body-level branch. `SG_LIB` is one constant read
//! three times rather than three copies of the same text: an easing table that
//! must agree with itself in three places is a drift waiting to be found by a
//! parity gate that only fixtures one of them.
//!
//! `params.count` is the dispatch width — the same `n` the CPU divides by, with
//! the same `n <= 1` guard (dividing by `n−1` without it is a divide by zero).
//!
//! `sg_round` is round-half-AWAY-from-zero because `channel` / `ease_curve` /
//! `ease_dir` all pick BRANCHES: WGSL's builtin `round` is half-to-EVEN, and a
//! disagreement there would select a different curve, not shift a value by an ε.

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, LutSpec};
use ph2d_nodegraph::port::Dim;

/// A resolução da LUT da ease [`super::ease::EASE_CUSTOM`] — amostras da curva autorada
/// sobre `t ∈ [0,1]`.
///
/// ⚠️ **512, o mesmo do irmão `motion.oscillator` e o dobro do `field.remap`**, e pela
/// mesma razão medida: uma tabela uniforme converge com `1/n` numa esquina e não converge
/// de todo num degrau — o que a densidade encolhe é a LARGURA da banda errada. Aqui a
/// tabela é lida como POSIÇÃO de peças ao longo de uma fileira, então essa banda é um
/// salto visível entre dois vizinhos. Custo: 2 KiB por cozimento.
const LUT_RESOLUTION: u32 = 512;

/// O canal de LUT deste nó (A1-gpu) — ver o gémeo em `motion.oscillator`. O nome
/// `sg_curve` faz o acessor chamar-se `sg_curve_sample`, casando com a chamada no WGSL.
pub(crate) static LUTS: &[LutSpec] = &[LutSpec {
    name: "sg_curve",
    text_key: super::CURVE_KEY,
    resolution: LUT_RESOLUTION,
    fill: fill_curve_lut,
}];

/// Amostra a curva autorada em `t = k/(n−1)`. Uma string ausente ou malformada enche a
/// **identidade** (`out[k] = t`), que é o `custom.map_or(t, …)` da CPU — os dois lados
/// concordam em *"nada autorado = o Linear"*.
fn fill_curve_lut(text: &str, out: &mut [f32]) {
    let curve = ph2d_curve::parse(text).unwrap_or_else(ph2d_curve::Curve::identity);
    let n = out.len();
    for (k, slot) in out.iter_mut().enumerate() {
        let t = if n <= 1 {
            0.0
        } else {
            k as f32 / (n - 1) as f32
        };
        *slot = curve.eval(t);
    }
}

const SG_PARAMS: &[&str] = &[
    "channel",
    "min",
    "max",
    "ease_curve",
    "ease_dir",
    "reverse",
    "offset",
    // ⭐ A ORDEM (ciclo 2, W2) — o device faz a MESMA conta que o `order::raw_at`.
    //
    // ⛔⛔ **ESTE PAR ESTAVA AQUI DUAS VEZES**, e o efeito não era cosmético: o gerador declara
    // um campo por nome no `struct` de uniformes, e `order: f32` duas vezes é
    // `redefinition of 'order'` — o WGSL **não compila**. Ou seja, desde a W2 do ciclo 2 o
    // `motion.stagger` deixou de ter kernel no dispositivo, em silêncio, e o gate que o diz
    // (`every_registered_kernel_validates_across_the_whole_presence_space`) vive numa crate que
    // o fecho daquela wave não corria. *Uma lista de nomes com um duplicado lê-se como uma
    // lista mais longa; só o compilador do device sabe que é a MESMA coisa duas vezes.*
    "order",
    "seed",
    // ⭐ CICLO 17 — a forma das famílias e a ordem no espaço (o `ph2d_ease::wgsl_lib!` lê-os).
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

/// A posição, LIDA pelo `From Point` nas variantes que não a escrevem (a X/Y já a liga em
/// leitura-escrita). Ausente ⇒ a origem, como a CPU.
const SG_P_READ: ColumnBinding = ColumnBinding {
    column: "P",
    dim: Dim::Vec2,
    access: ColumnAccess::Read,
    identity: [0.0; 4],
    port: 0,
};

/// The falloff mask, bound identically by every variant.
const SG_FALLOFF: ColumnBinding = ColumnBinding {
    column: "falloff",
    dim: Dim::Scalar,
    access: ColumnAccess::Read,
    identity: [1.0; 4],
    port: 0,
};

/// A tabela de easing e a ordem são as da casa (`ph2d_ease::wgsl_lib!`, ciclo 17 — a MESMA conta
/// da CPU); aqui só mora o que é deste nó: ler os params e a LUT da `Custom`.
///
/// Partilhado pelas três variantes — ver a nota do módulo.
const SG_LIB: &str = concat!(
    ph2d_ease::wgsl_lib!(),
    "\
    fn sg_round(x: f32) -> f32 {\n\
        return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
    }\n\
    fn sg_ease(curve: i32, dir: i32, t: f32) -> f32 {\n\
        // A NONA familia: a DESENHADA, lida da LUT que o sequenciador assa do\n\
        // text param. Curva ausente => a tabela e' a identidade, o proprio Linear.\n\
        if (curve == 8) { return sg_curve_sample(t); }\n\
        let s0 = vec4<f32>(params.overshoot, params.period, params.steps, params.damping);\n\
        let s1 = vec4<f32>(params.bez_x1, params.bez_y1, params.bez_x2, params.bez_y2);\n\
        return pe_ease(curve, dir, t, s0, s1);\n\
    }\n\
    fn sg_raw_at(i: u32) -> f32 {\n\
        return pe_raw_at(i32(sg_round(params.order)),\n\
        \x20   u32(max(sg_round(params.seed), 0.0)),\n\
        \x20   u32(max(sg_round(params.grid_cols), 1.0)),\n\
        \x20   i32(sg_round(params.grid_from)),\n\
        \x20   u32(max(sg_round(params.grid_cell), 0.0)),\n\
        \x20   i32(sg_round(params.axis)),\n\
        \x20   vec2<f32>(params.origin_x, params.origin_y), params.reach,\n\
        \x20   i, params.count, read_P(i));\n\
    }\n\
    fn sg_delta(i: u32) -> f32 {\n\
        var raw = sg_raw_at(i);\n\
        if (params.reverse >= 0.5) { raw = 1.0 - raw; }\n\
        if (params.offset != 0.0) {\n\
        \x20   // Ver o `eval`: com o knob no neutro o `frac` nem corre,\n\
        \x20   // senao a ponta da rampa (exactamente 1.0) iria para 0.\n\
        \x20   let sg_s = raw + params.offset;\n\
        \x20   raw = sg_s - floor(sg_s);\n\
        }\n\
        let e = sg_ease(i32(sg_round(params.ease_curve)),\n\
        \x20   i32(sg_round(params.ease_dir)), raw);\n\
        return (params.min + e * (params.max - params.min)) * read_falloff(i);\n\
    }\n"
);

/// **X / Y** — adds the delta to one component of `P`. The channel test is
/// `< 0.5`, which agrees with the CPU's `round()` for both values this variant
/// is selected for.
const SG_P: GpuKernel = GpuKernel {
    wgsl: "\
        let sg_d = sg_delta(i);\n\
        var sg_p = read_P(i);\n\
        if (params.channel < 0.5) {\n\
        \x20   sg_p.x = sg_p.x + sg_d;\n\
        } else {\n\
        \x20   sg_p.y = sg_p.y + sg_d;\n\
        }\n\
        write_P(i, sg_p);\n",
    wgsl_lib: SG_LIB,
    bindings: &[
        // The target channel is materialized from its identity when absent —
        // the CPU's `apply_channel_delta` does the same (`base_vec2`).
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        SG_FALLOFF,
    ],
    params: SG_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Rotation** — adds the delta to `rot` (identity `0`).
const SG_ROT: GpuKernel = GpuKernel {
    wgsl: "        write_rot(i, read_rot(i) + sg_delta(i));\n",
    wgsl_lib: SG_LIB,
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        SG_FALLOFF,
        SG_P_READ,
    ],
    params: SG_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size** — adds the delta to BOTH components, from the UNIT identity (never
/// `[0,0]`: unit scale is what "no size" means, and a bipolar stagger based at
/// zero would drive the sprite through zero and negative).
const SG_SIZE: GpuKernel = GpuKernel {
    wgsl: "\
        let sg_d = sg_delta(i);\n\
        let sg_s = read_size(i);\n\
        write_size(i, vec2<f32>(sg_s.x + sg_d, sg_s.y + sg_d));\n",
    wgsl_lib: SG_LIB,
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        SG_FALLOFF,
        SG_P_READ,
    ],
    params: SG_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// The registered kernel: the X/Y variant's shape, with the channel switch.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    // The top-level shape IS the X/Y variant, so a caller that never resolves
    // still sees a real kernel rather than the empty (pass-through) one.
    wgsl: SG_P.wgsl,
    wgsl_lib: SG_LIB,
    bindings: SG_P.bindings,
    params: SG_PARAMS,
    count_law: None,
    variant_by_param: Some(|param| match param("channel").round() as i32 {
        2 => &SG_ROT,
        0 | 1 => &SG_P,
        _ => &SG_SIZE,
    }),
    applicable: None,
};
