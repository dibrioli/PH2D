//! **O que o DISPOSITIVO computa** — o kernel de GPU deste nó, cortado do `lib.rs` no teto
//! de LOC (HR-18) pela costura que ele já tinha: o `lib.rs` responde *o que um oscilador É*
//! (o manifesto, a onda, as duas leis) e este arquivo *como um device chega no mesmo número*.
//!
//! É uma costura limpa porque a aritmética não mora aqui: os três variants compartilham
//! `wgsl_lib`, que é o port linha-a-linha do `waveform`/`cycles_per_second`/`fade_gain` do
//! módulo pai — e quem prova que os dois lados concordam é o gate de paridade CPU×GPU, não
//! esta divisão de arquivos.

use ph2d_curve::Curve;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, LutSpec};
use ph2d_nodegraph::port::Dim;

/// A resolução da LUT da onda [`super::WAVE_CUSTOM`] — amostras da curva autorada sobre
/// `f ∈ [0,1]`, o ciclo inteiro.
///
/// ⚠️ **Ela é o DOBRO da do `field.remap` (256), e o número tem motivo medido:** a lei da
/// tabela uniforme é que a esquina de uma parada converge com `1/n` e um DEGRAU não
/// converge de todo — o que a densidade encolhe ali é a LARGURA da banda errada
/// ([`project-memory/feedback_a_uniform_grid_cannot_represent_a_corner.md`]). Uma onda
/// autorada é lida como MOVIMENTO ao longo de um ciclo inteiro (não como a cor de um
/// pixel), então a banda errada vira um solavanco visível, e 512 põe a célula abaixo de
/// 0,2% do ciclo. Custo: 2 KiB por cozimento.
const LUT_RESOLUTION: u32 = 512;

/// O canal de LUT deste nó (A1-gpu): o sequenciador amostra a curva autorada
/// ([`super::CURVE_KEY`]) e liga a tabela, de modo que o ramo `kind == 5` do `osc_wave`
/// leia `osc_curve_sample(f)` **no device**. O nome `osc_curve` é o que faz o acessor
/// chamar-se `osc_curve_sample`, casando com a chamada no WGSL.
pub(crate) static LUTS: &[LutSpec] = &[LutSpec {
    name: "osc_curve",
    text_key: super::CURVE_KEY,
    resolution: LUT_RESOLUTION,
    fill: fill_curve_lut,
}];

/// Amostra a curva autorada em `f = k/(n−1)` — a metade deste crate do canal de LUT
/// (o `ph2d-nodegraph` não conhece biblioteca de curva nenhuma).
///
/// ⚠️ Uma string ausente ou malformada enche a **identidade** (`out[k] = f`), que é
/// exactamente o `curve.map_or(f, …)` que a CPU faz num `None` — os dois lados
/// concordam em *"nada autorado = a serra"*, e é isso que o gate de paridade mede.
fn fill_curve_lut(text: &str, out: &mut [f32]) {
    let curve = ph2d_curve::parse(text).unwrap_or_else(Curve::identity);
    let n = out.len();
    for (k, slot) in out.iter_mut().enumerate() {
        let f = if n <= 1 {
            0.0
        } else {
            k as f32 / (n - 1) as f32
        };
        *slot = curve.eval(f);
    }
}

/// GPU compute kernel (GPU/M5 Fase 1, ADR-0126). The waveform library is a
/// straight WGSL port of [`waveform`] — same piecewise polynomials, same
/// Capens 2nd-order sine correction (HR-5: WGSL `sin` has no cross-vendor
/// guarantee; the parabola is plain mul/abs arithmetic, so GPU-vs-CPU parity
/// holds within float ULPs). `osc_round` is round-half-AWAY-from-zero to match
/// Rust's `f32::round` (WGSL's builtin `round` is half-to-even, which would
/// pick a different waveform at `wave = 2.5`).
///
/// **Every channel**, by shipping one variant per target column
/// ([`GpuKernel::variant_by_param`]). It used to cover X/Y only: the Rotation and
/// Size channels write a DIFFERENT column, which a static binding set cannot
/// switch on, so those fell back to the CPU. The engine can switch now, and the
/// four channels map exactly as `channel_column` does — including its
/// `_ => size` catch-all for an out-of-range value.
///
/// The delta is computed identically in all three variants (the shared
/// `osc_delta`); only the column it lands on differs, which is precisely why the
/// variants exist and not a body-level branch.
const OSC_PARAMS: &[&str] = &[
    "channel",
    "pulse_width",
    "wave",
    "amplitude",
    "frequency",
    "phase_stagger",
    "offset",
    "phase",
    "time_mode",
    "bpm",
    // A FAIXA — `range_mode = 0` devolve a expressão que estava aqui antes.
    // ⚠️ Esta lista não é derivada do manifesto.
    "range_mode",
    "min",
    "max",
];

/// The falloff mask, bound identically by every variant.
const OSC_FALLOFF: ColumnBinding = ColumnBinding {
    column: "falloff",
    dim: Dim::Scalar,
    access: ColumnAccess::Read,
    identity: [1.0; 4],
    port: 0,
};

/// **A PORTA DE TEMPO** — ligada por todo variant, `ReadBroadcast` para herdar a regra
/// 1→N do `motion.drive` (um `value.time` desligado emite UM valor, e ele vale para
/// toda a instância).
///
/// ⚠️ A `identity` aqui é **inerte**: o corpo nunca a lê, porque o neutro desta porta é
/// `params.playhead` e uma identidade é uma constante de compilação. Quem responde
/// *"a porta está ligada?"* é o `const HAS_time_v` que o codegen emite ao lado do
/// leitor — fixo por pipeline compilado (a cache é chaveada exactamente nessa
/// assinatura), logo ramificar nele **não custa nada** no device.
const OSC_TIME: ColumnBinding = ColumnBinding {
    column: "v",
    dim: Dim::Scalar,
    access: ColumnAccess::ReadBroadcast,
    identity: [0.0; 4],
    port: 1,
};

/// **X / Y** — adds the delta to one component of `P`. The channel test is
/// `< 0.5`, which agrees with the CPU's `round()` for both values this variant
/// is selected for.
/// **A biblioteca WGSL que os TRES variants compartilham.**
///
/// AVISO: ela era LITERAL em cada um deles, e as tres copias JA TINHAM
/// DIVERGIDO -- medido em 2026-08-16, so o variant de `P` tinha o
/// `osc_cycles_per_second`, e os de `rot` e `size` liam `params.frequency`
/// direto: um grafo a dirigir a rotacao em BPM corria a uma taxa no device
/// e a outra na CPU, sem erro e sem aviso. Os variants existem pela COLUNA
/// que cada um escreve; a aritmetica e a MESMA nos tres, e mante-la em tres
/// lugares foi o que produziu o defeito.
const OSC_LIB: &str = "\
        fn osc_time(i: u32) -> f32 {\n\
            // A porta `time` DESLIGADA e' o relogio global. O neutro nao pode viajar\n\
            // na `identity` do binding (ela e' constante de compilacao e o playhead\n\
            // nao e'), entao a lei RAMIFICA na presenca -- e `HAS_time_v` e' const no\n\
            // modulo gerado, fixo por pipeline compilado.\n\
            if (HAS_time_v) { return read_time_v(i); }\n\
            return params.playhead;\n\
        }\n\
        fn osc_delta(i: u32) -> f32 {\n\
            let cps = osc_cycles_per_second();\n\
            let phase = osc_time(i) * cps + f32(i) * params.phase_stagger + params.phase;\n\
            let w = osc_wave(i32(osc_round(params.wave)), phase);\n\
            // A FAIXA: o gemeo de `gain_offset_for_range`, com a polaridade a\n\
            // sair da FORMA -- o Spike (4) e' unipolar [0,1], as outras quatro\n\
            // sao bipolares.\n\
            var amp = params.amplitude;\n\
            var off = params.offset;\n\
            if (params.range_mode >= 0.5) {\n\
                var lo = -1.0;\n\
                let k = i32(osc_round(params.wave));\n\
                // O Spike (4) e a Custom (5) sao unipolares [0,1] -- o gemeo do\n\
                // `natural_range`, e a Custom TEM de estar aqui: uma forma nova que\n\
                // nao declarasse a polaridade dela reabre a armadilha do Spike.\n\
                if (k == 4 || k == 5) { lo = 0.0; }\n\
                amp = (params.max - params.min) / (1.0 - lo);\n\
                off = params.min - lo * amp;\n\
            }\n\
            return (w * amp + off) * read_in_falloff(i);\n\
        }\n\
        fn osc_cycles_per_second() -> f32 {\n\
            // BPM: a MESMA grandeza noutra regua (batidas por minuto).\n\
            if (params.time_mode >= 0.5) { return params.bpm / 60.0; }\n\
            return params.frequency;\n\
        }\n\
        fn osc_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn osc_skew(f: f32, pw: f32) -> f32 {\n\
            // O warp de fase: pw = 0.5 e a IDENTIDADE (0.5/0.5 == 1.0).\n\
            let p = clamp(pw, 0.05, 0.95);\n\
            if (f < p) { return f * (0.5 / p); }\n\
            return 0.5 + (f - p) * (0.5 / (1.0 - p));\n\
        }\n\
        fn osc_wave(kind: i32, phase: f32) -> f32 {\n\
            let f = osc_skew(phase - floor(phase), params.pulse_width);\n\
            if (kind == 1) {\n\
                // Triangle: 0 at 0, +1 at 1/4, 0 at 1/2, -1 at 3/4.\n\
                if (f < 0.25) { return 4.0 * f; }\n\
                if (f < 0.75) { return 2.0 - 4.0 * f; }\n\
                return 4.0 * f - 4.0;\n\
            }\n\
            if (kind == 2) {\n\
                if (f < 0.5) { return 1.0; }\n\
                return -1.0;\n\
            }\n\
            if (kind == 3) { return 2.0 * f - 1.0; }\n\
            if (kind == 4) {\n\
                if (f < 0.08) { return 1.0; }\n\
                return 0.0;\n\
            }\n\
            // A forma DESENHADA (`WAVE_CUSTOM`), lida da LUT que o sequenciador\n\
            // assa do text param -- e' por isso que a onda custom cozinha no\n\
            // DEVICE em vez de derrubar o no' inteiro para a CPU. Uma curva\n\
            // ausente enche a tabela com a identidade (`out[k] = t`), que e'\n\
            // exactamente o `curve.map_or(f, ...)` do lado da CPU.\n\
            if (kind == 5) { return osc_curve_sample(f); }\n\
            // Parabolic sine-approximation + Capens correction (~0.09% off a\n\
            // true sine, transcendental-free) — the port of `waveform`'s default.\n\
            var p: f32;\n\
            if (f < 0.5) {\n\
                let u = f * 2.0;\n\
                p = 4.0 * u * (1.0 - u);\n\
            } else {\n\
                let u = (f - 0.5) * 2.0;\n\
                p = -4.0 * u * (1.0 - u);\n\
            }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n";

pub(crate) const OSC_P: GpuKernel = GpuKernel {
    wgsl: "\
        let osc_d = osc_delta(i);\n\
        var osc_p = read_in_P(i);\n\
        if (params.channel < 0.5) {\n\
            osc_p.x = osc_p.x + osc_d;\n\
        } else {\n\
            osc_p.y = osc_p.y + osc_d;\n\
        }\n\
        write_P(i, osc_p);\n",
    wgsl_lib: OSC_LIB,
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
        OSC_FALLOFF,
        OSC_TIME,
    ],
    params: OSC_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Rotation** — adds the delta to `rot`.
pub(crate) const OSC_ROT: GpuKernel = GpuKernel {
    wgsl: "\
        write_rot(i, read_in_rot(i) + osc_delta(i));\n",
    wgsl_lib: OSC_LIB,
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        OSC_FALLOFF,
        OSC_TIME,
    ],
    params: OSC_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size** — adds the delta to BOTH components, from the UNIT identity (never
/// `[0,0]`: unit scale is what "no size" means).
pub(crate) const OSC_SIZE: GpuKernel = GpuKernel {
    wgsl: "\
        let osc_d = osc_delta(i);\n\
        let osc_s = read_in_size(i);\n\
        write_size(i, vec2<f32>(osc_s.x + osc_d, osc_s.y + osc_d));\n",
    wgsl_lib: OSC_LIB,
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        OSC_FALLOFF,
        OSC_TIME,
    ],
    params: OSC_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    // The top-level shape IS the X/Y variant, so a caller that never resolves
    // still sees a real kernel rather than the empty (pass-through) one.
    wgsl: OSC_P.wgsl,
    wgsl_lib: OSC_P.wgsl_lib,
    bindings: OSC_P.bindings,
    params: OSC_PARAMS,
    count_law: None,
    variant_by_param: Some(|param| match param("channel").round() as i32 {
        2 => &OSC_ROT,
        0 | 1 => &OSC_P,
        _ => &OSC_SIZE,
    }),
    applicable: None,
};
