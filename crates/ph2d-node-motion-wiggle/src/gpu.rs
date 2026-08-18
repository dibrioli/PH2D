//! **O que o DISPOSITIVO computa** — o kernel de GPU deste nó, cortado do `lib.rs` no
//! teto de LOC (HR-18) pela costura que o irmão `motion.oscillator` já tinha: o `lib.rs`
//! responde *o que um wiggle É* (o manifesto, o campo, o laço) e este arquivo *como um
//! device chega no mesmo número*.
//!
//! É uma costura limpa porque a aritmética não mora aqui: os três variants compartilham
//! `WG_LIB`, que é o port linha-a-linha do `eval` do módulo pai — e quem prova que os
//! dois lados concordam é o gate de paridade CPU×GPU, não esta divisão de arquivos.

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::port::Dim;

/// **A PORTA DE TEMPO** — ligada por todo variant, `ReadBroadcast` para herdar a regra
/// 1→N do `motion.drive`. A `identity` é inerte: o neutro desta porta é
/// `params.playhead`, que não é constante de compilação, então quem responde *"a porta
/// está ligada?"* é o `const HAS_time_v` do módulo gerado (fixo por pipeline, logo
/// ramificar nele é de graça).
const WG_TIME: ColumnBinding = ColumnBinding {
    column: "v",
    dim: Dim::Scalar,
    access: ColumnAccess::ReadBroadcast,
    identity: [0.0; 4],
    port: 1,
};

/// The params every variant declares, in one order — the uniform layout is
/// per-variant, and identical lists mean a reader never has to ask which variant
/// a `params.<x>` belongs to.
const WG_PARAMS: &[&str] = &[
    "channel",
    "amplitude",
    "frequency",
    "seed",
    "octaves",
    "amp_mult",
    "loop_len",
];

/// **A biblioteca WGSL que os TRÊS variants compartilham.**
///
/// ⚠️ Ela era LITERAL em cada um deles — três blocos byte-idênticos de 1527
/// caracteres —, e é isso que faz uma lei nova de ruído ter de ser escrita três
/// vezes e poder divergir em duas. Os variants existem pela COLUNA que cada um
/// escreve; a aritmética é a MESMA nos três (o padrão que o `motion.noise` e o
/// `motion.oscillator` já usam).
const WG_LIB: &str = "\
        fn wg_time(i: u32) -> f32 {\n\
            // A porta `time` DESLIGADA e' o relogio global -- ver o binding WG_TIME.\n\
            if (HAS_time_v) { return read_time_v(i); }\n\
            return params.playhead;\n\
        }\n\
        fn wg_delta(i: u32) -> f32 {\n\
            let ny = f32(i) + params.seed;\n\
            let oct = min(max(i32(wg_round(params.octaves)), 1), 8);\n\
            // O tempo WRAPA antes de entrar no campo -- ver `loop_times`. Por\n\
            // ELEMENTO: com a porta ligada cada peca fecha o PROPRIO ciclo.\n\
            let wg_t = wg_time(i);\n\
            var ta = wg_t;\n\
            var tb = ta;\n\
            var w = 0.0;\n\
            if (params.loop_len > 0.0) {\n\
            \x20   let u0 = wg_t / params.loop_len;\n\
            \x20   let u = u0 - floor(u0);\n\
            \x20   ta = u * params.loop_len;\n\
            \x20   tb = ta - params.loop_len;\n\
            \x20   w = u * u * (3.0 - 2.0 * u);\n\
            }\n\
            let sa = wg_fbm(ta * params.frequency, ny, oct, params.amp_mult);\n\
            var s = sa;\n\
            if (w != 0.0) {\n\
            \x20   let sb = wg_fbm(tb * params.frequency, ny, oct, params.amp_mult);\n\
            \x20   s = sa + (sb - sa) * w;\n\
            }\n\
            return s * params.amplitude * read_in_falloff(i);\n\
        }\n\
        fn wg_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn wg_hash2(ix: i32, iy: i32) -> f32 {\n\
            // Same mix as noise::hash2 -- u32 wraps mod 2^32 (== Rust wrapping_*),\n\
            // bitcast<u32> == Rust `as u32` (bit reinterpretation, not a value cast).\n\
            var h: u32 = bitcast<u32>(ix) * 0x27d4eb2du + bitcast<u32>(iy) * 0x165667b1u;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x2c1b3c6du;\n\
            h = h ^ (h >> 12u);\n\
            h = h * 0x297175f9u;\n\
            h = h ^ (h >> 15u);\n\
            return (f32(h) / f32(0xffffffffu)) * 2.0 - 1.0;\n\
        }\n\
        fn wg_fade(t: f32) -> f32 {\n\
            return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);\n\
        }\n\
        fn wg_noise(x: f32, y: f32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let u = wg_fade(x - x0);\n\
            let v = wg_fade(y - y0);\n\
            let n00 = wg_hash2(ix, iy);\n\
            let n10 = wg_hash2(ix + 1, iy);\n\
            let n01 = wg_hash2(ix, iy + 1);\n\
            let n11 = wg_hash2(ix + 1, iy + 1);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return nx0 + v * (nx1 - nx0);\n\
        }\n\
        fn wg_fbm(x0: f32, y0: f32, octaves: i32, amp_mult: f32) -> f32 {\n\
            // A MESMA lei da folha `ph2d_fbm::eval`, na MESMA ordem.\n\
            // ATENCAO: as DUAS coordenadas escalam por oitava. A primeira\n\
            // versao deste laco escalava so o X, e a paridade na RTX apanhou-a\n\
            // com |dif| 0,668 -- da ordem da amplitude, nao de um ulp.\n\
            let gain = clamp(amp_mult, 0.0, 1.0);\n\
            var x = x0;\n\
            var y = y0;\n\
            var amp = 1.0;\n\
            var sum = 0.0;\n\
            var total = 0.0;\n\
            for (var o = 0; o < octaves; o = o + 1) {\n\
            \x20   sum = sum + amp * wg_noise(x + f32(o) * 1013.0, y);\n\
            \x20   total = total + amp;\n\
            \x20   amp = amp * gain;\n\
            \x20   x = x * 2.0;\n\
            \x20   y = y * 2.0;\n\
            }\n\
            return sum / total;\n\
        }\n";

/// **X / Y** — adds the delta to one component of `P`. The channel test is
/// `< 0.5`, which agrees with the CPU's `round()` for both values this variant
/// is selected for.
const WG_P: GpuKernel = GpuKernel {
    wgsl: "\
        let d = wg_delta(i);\n\
        var p = read_in_P(i);\n\
        if (params.channel < 0.5) { p.x = p.x + d; } else { p.y = p.y + d; }\n\
        write_P(i, p);\n",
    wgsl_lib: WG_LIB,
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
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        WG_TIME,
    ],
    params: WG_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Rotation** — adds the delta to `rot`.
const WG_ROT: GpuKernel = GpuKernel {
    wgsl: "\
        write_rot(i, read_in_rot(i) + wg_delta(i));\n",
    wgsl_lib: WG_LIB,
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        WG_TIME,
    ],
    params: WG_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size** — adds the delta to BOTH components, from the UNIT identity (never
/// `[0,0]`: unit scale is what "no size" means).
const WG_SIZE: GpuKernel = GpuKernel {
    wgsl: "\
        let d = wg_delta(i);\n\
        let s = read_in_size(i);\n\
        write_size(i, vec2<f32>(s.x + d, s.y + d));\n",
    wgsl_lib: WG_LIB,
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        WG_TIME,
    ],
    params: WG_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// GPU compute kernel (ADR-0126) — **every channel**, by shipping one variant per
/// target column ([`GpuKernel::variant_by_param`]).
///
/// It used to cover X/Y only: Rotation and Size write a DIFFERENT column, which a
/// static binding set cannot switch on, so those fell back to the CPU. The delta
/// is computed identically in all three variants (the shared `wg_delta`);
/// only the column it lands on differs, which is precisely why they are variants
/// and not a branch inside one body — the generated module defines `write_<col>`
/// only for BOUND columns.
///
/// The channels map exactly as `channel_column` does, including its `_ => size`
/// catch-all for an out-of-range value.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    // The top-level shape IS the X/Y variant, so a caller that never resolves
    // still sees a real kernel rather than the empty (pass-through) one.
    wgsl: WG_P.wgsl,
    wgsl_lib: WG_P.wgsl_lib,
    bindings: WG_P.bindings,
    params: WG_PARAMS,
    count_law: None,
    variant_by_param: Some(|param| match param("channel").round() as i32 {
        2 => &WG_ROT,
        0 | 1 => &WG_P,
        _ => &WG_SIZE,
    }),
    applicable: None,
};
