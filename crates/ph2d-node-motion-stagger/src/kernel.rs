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
];

/// The falloff mask, bound identically by every variant.
const SG_FALLOFF: ColumnBinding = ColumnBinding {
    column: "falloff",
    dim: Dim::Scalar,
    access: ColumnAccess::Read,
    identity: [1.0; 4],
    port: 0,
};

/// The whole easing table, ported verbatim from the CPU: 8 curve families × 3
/// directions, all polynomial or `sqrt` (HR-5 — no `pow`, no transcendental),
/// with In-Out built by reflecting the In base exactly as the CPU does.
///
/// Shared by all three variants — see the module note on why this is one
/// constant and not three copies.
const SG_LIB: &str = "\
    fn sg_round(x: f32) -> f32 {\n\
        return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
    }\n\
    fn sg_bounce_out(t: f32) -> f32 {\n\
        let n1 = 7.5625;\n\
        let d1 = 2.75;\n\
        if (t < 1.0 / d1) { return n1 * t * t; }\n\
        if (t < 2.0 / d1) {\n\
            let u = t - 1.5 / d1;\n\
            return n1 * u * u + 0.75;\n\
        }\n\
        if (t < 2.5 / d1) {\n\
            let u = t - 2.25 / d1;\n\
            return n1 * u * u + 0.9375;\n\
        }\n\
        let u = t - 2.625 / d1;\n\
        return n1 * u * u + 0.984375;\n\
    }\n\
    fn sg_ease_in(curve: i32, t: f32) -> f32 {\n\
        if (curve == 1) { return t * t; }\n\
        if (curve == 2) { return t * t * t; }\n\
        if (curve == 3) { return t * t * t * t; }\n\
        if (curve == 4) { return t * t * t * t * t; }\n\
        if (curve == 5) { return 1.0 - sqrt(max(1.0 - t * t, 0.0)); }\n\
        if (curve == 6) {\n\
            let c1 = 1.70158;\n\
            let c3 = c1 + 1.0;\n\
            return c3 * t * t * t - c1 * t * t;\n\
        }\n\
        if (curve == 7) { return 1.0 - sg_bounce_out(1.0 - t); }\n\
        return t;\n\
    }\n\
    fn sg_ease(curve: i32, dir: i32, t: f32) -> f32 {\n\
        if (curve == 0) { return t; }\n\
        // A NONA familia: a DESENHADA, lida da LUT que o sequenciador assa do\n\
        // text param. Ela ignora o `dir` pela mesma razao que o Linear -- a\n\
        // curva e' a declaracao de intencao do artista. Curva ausente => a\n\
        // tabela e' a identidade, que e' o proprio Linear.\n\
        if (curve == 8) { return sg_curve_sample(t); }\n\
        if (dir == 1) { return 1.0 - sg_ease_in(curve, 1.0 - t); }\n\
        if (dir == 2) {\n\
            if (t < 0.5) { return sg_ease_in(curve, 2.0 * t) * 0.5; }\n\
            return 1.0 - sg_ease_in(curve, 2.0 - 2.0 * t) * 0.5;\n\
        }\n\
        return sg_ease_in(curve, t);\n\
    }\n\
    fn sg_hash01(i: u32, seed: u32) -> f32 {\n\
        var x = i * 2654435769u + seed * 2246822507u;\n\
        x = x ^ (x >> 16u);\n\
        x = x * 2146121005u;\n\
        x = x ^ (x >> 15u);\n\
        x = x * 2221713035u;\n\
        x = x ^ (x >> 16u);\n\
        return f32(x) / 4294967296.0;\n\
    }\n\
    fn sg_raw_at(i: u32) -> f32 {\n\
        if (params.count <= 1u) { return 0.0; }\n\
        let last = f32(params.count) - 1.0;\n\
        let ord = i32(sg_round(params.order));\n\
        if (ord == 1) {\n\
        \x20   return abs(2.0 * f32(i) - last) / last;\n\
        }\n\
        if (ord == 2) {\n\
        \x20   return sg_hash01(i, u32(max(sg_round(params.seed), 0.0)));\n\
        }\n\
        return f32(i) / last;\n\
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
    }\n";

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
