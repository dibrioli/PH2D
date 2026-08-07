//! **O que o DISPOSITIVO computa** — o kernel de GPU deste nó, cortado do `lib.rs` no teto
//! de LOC (HR-18) pela costura que ele já tinha: o `lib.rs` responde *o que um oscilador É*
//! (o manifesto, a onda, as duas leis) e este arquivo *como um device chega no mesmo número*.
//!
//! É uma costura limpa porque a aritmética não mora aqui: os três variants compartilham
//! `wgsl_lib`, que é o port linha-a-linha do `waveform`/`cycles_per_second`/`fade_gain` do
//! módulo pai — e quem prova que os dois lados concordam é o gate de paridade CPU×GPU, não
//! esta divisão de arquivos.

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::port::Dim;

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
    "wave",
    "amplitude",
    "frequency",
    "phase_stagger",
    "offset",
    "phase",
    "time_mode",
    "bpm",
];

/// The falloff mask, bound identically by every variant.
const OSC_FALLOFF: ColumnBinding = ColumnBinding {
    column: "falloff",
    dim: Dim::Scalar,
    access: ColumnAccess::Read,
    identity: [1.0; 4],
    port: 0,
};

/// **X / Y** — adds the delta to one component of `P`. The channel test is
/// `< 0.5`, which agrees with the CPU's `round()` for both values this variant
/// is selected for.
pub(crate) const OSC_P: GpuKernel = GpuKernel {
    wgsl: "\
        let osc_d = osc_delta(i, params.playhead);\n\
        var osc_p = read_P(i);\n\
        if (params.channel < 0.5) {\n\
            osc_p.x = osc_p.x + osc_d;\n\
        } else {\n\
            osc_p.y = osc_p.y + osc_d;\n\
        }\n\
        write_P(i, osc_p);\n",
    wgsl_lib: "\
        fn osc_delta(i: u32, t: f32) -> f32 {\n\
            let cps = osc_cycles_per_second();\n\
            let phase = t * cps + f32(i) * params.phase_stagger + params.phase;\n\
            let w = osc_wave(i32(osc_round(params.wave)), phase);\n\
            return (w * params.amplitude + params.offset) * read_falloff(i);\n\
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
        fn osc_wave(kind: i32, phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
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
        }\n",
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
    ],
    params: OSC_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Rotation** — adds the delta to `rot`.
pub(crate) const OSC_ROT: GpuKernel = GpuKernel {
    wgsl: "\
        write_rot(i, read_rot(i) + osc_delta(i, params.playhead));\n",
    wgsl_lib: "\
        fn osc_delta(i: u32, t: f32) -> f32 {\n\
            let phase = t * params.frequency + f32(i) * params.phase_stagger\n\
            \x20   + params.phase;\n\
            let w = osc_wave(i32(osc_round(params.wave)), phase);\n\
            return (w * params.amplitude + params.offset) * read_falloff(i);\n\
        }\n\
        fn osc_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn osc_wave(kind: i32, phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
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
        }\n",
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        OSC_FALLOFF,
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
        let osc_d = osc_delta(i, params.playhead);\n\
        let osc_s = read_size(i);\n\
        write_size(i, vec2<f32>(osc_s.x + osc_d, osc_s.y + osc_d));\n",
    wgsl_lib: "\
        fn osc_delta(i: u32, t: f32) -> f32 {\n\
            let phase = t * params.frequency + f32(i) * params.phase_stagger\n\
            \x20   + params.phase;\n\
            let w = osc_wave(i32(osc_round(params.wave)), phase);\n\
            return (w * params.amplitude + params.offset) * read_falloff(i);\n\
        }\n\
        fn osc_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn osc_wave(kind: i32, phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
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
        }\n",
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        OSC_FALLOFF,
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
