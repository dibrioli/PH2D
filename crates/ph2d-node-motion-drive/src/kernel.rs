//! **Os KERNELS WGSL do `motion.drive`** — cortados do `lib.rs` no teto de LOC do HR-18
//! (700 para `crates/`), pela mesma costura que o `motion.noise` já usava.
//!
//! O corte é por RESPONSABILIDADE e não por tamanho: o `lib.rs` responde *o que um drive É*
//! (o manifesto, o `eval`, o registo) e este responde *o que o device corre*. Nada aqui é
//! lido pela CPU — se fosse, o corte estaria no sítio errado.
//!
//! ⚠️ **A aritmética é a MESMA dos dois lados por construção:** a `drive_resolve` daqui é o
//! gémeo literal da `Combine::resolve` do `combine.rs`, e a `drive_local_axis` o do
//! `channel::local_axis`. Há gate a cruzá-las pelos literais da string.

use crate::VALUE_COL;
use crate::channel::{
    CH_CUSTOM, CH_FALLOFF, CH_HUE, CH_OPACITY, CH_SAT, CH_SIZE_X, CH_SIZE_Y, CH_VAL,
};
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::port::Dim;

/// ⚠️ **`space` apendado**: sem esta linha o kernel lê um `params.space` inexistente — ou,
/// pior, calcula o espaço do MUNDO em silêncio enquanto a CPU calcula o do elemento.
pub(crate) const DRIVE_PARAMS: &[&str] = &["channel", "scale", "mode", "space"];

/// The shared prologue: resolve the mode, the scaled value and the falloff mask.
/// Every variant pastes this and then writes ITS column.
///
/// `drive_round` is round-half-away-from-zero to match Rust's `f32::round` —
/// `mode` picks a BRANCH ([[feedback_cpu_gpu_rounding_conventions_diverge]]).
/// The falloff clamp MIRRORS the CPU's; no node writes a falloff outside `[0,1]`
/// today, so it is defensive on both sides rather than load-bearing.
pub(crate) const DRIVE_LIB: &str = "\
    fn drive_round(x: f32) -> f32 {\n\
        // Rust f32::round = half away from zero (WGSL round is half-even).\n\
        return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
    }\n\
    fn drive_combine(cur: f32, v: f32, mode: i32) -> f32 {\n\
        if (mode == 1) { return v; }\n\
        if (mode == 2) { return cur * v; }\n\
        // Apendados (folha 06 linha 40) -- o gemeo de `Combine::apply`.\n\
        if (mode == 3) { return cur - v; }\n\
        if (mode == 4) {\n\
            // A MESMA guarda da CPU, com o MESMO limiar: um `inf` num canal de\n\
            // transform envenena a posicao e todo NaN a jusante vem sem endereco.\n\
            if (abs(v) < 1e-9) { return 0.0; }\n\
            return cur / v;\n\
        }\n\
        if (mode == 5) { return min(cur, v); }\n\
        if (mode == 6) { return max(cur, v); }\n\
        // O Remap (folha 06 linha 41) -- o gemeo de `Combine::Remap`.\n\
        if (mode == 7) { return v; }\n\
        return cur + v;\n\
    }\n\
    // **De onde a mascara mede** -- o gemeo de `Combine::base`. So' o Remap\n\
    // mede a partir do ZERO; todo o resto mede a partir do canal.\n\
    fn drive_base(cur: f32, mode: i32) -> f32 {\n\
        if (mode == 7) { return 0.0; }\n\
        return cur;\n\
    }\n\
    // **A porta unica** -- o gemeo de `Combine::resolve`. Ela existe do lado da\n\
    // CPU porque a lei estava escrita OITO vezes; aqui pelo mesmo motivo, e a\n\
    // paridade entre as duas so' e' verificavel se as duas tiverem UMA porta.\n\
    fn drive_resolve(cur: f32, v: f32, mode: i32, f: f32) -> f32 {\n\
        let b = drive_base(cur, mode);\n\
        return b + (drive_combine(cur, v, mode) - b) * f;\n\
    }\n\
    // A senoide parabolica corrigida (Capens/devmaster) -- o porte linha-a-linha do\n\
    // `trig.rs`. HR-5: o `sin` do WGSL nao tem garantia cross-vendor, entao o eixo\n\
    // local so' bate com o da CPU se as duas correrem a MESMA conta.\n\
    fn drive_sin_cycles(phase: f32) -> f32 {\n\
        let fr = phase - floor(phase);\n\
        var pp: f32;\n\
        if (fr < 0.5) {\n\
            let u = fr * 2.0;\n\
            pp = 4.0 * u * (1.0 - u);\n\
        } else {\n\
            let u = (fr - 0.5) * 2.0;\n\
            pp = -4.0 * u * (1.0 - u);\n\
        }\n\
        return 0.225 * (pp * abs(pp) - pp) + pp;\n\
    }\n\
    // O eixo LOCAL do elemento -- o gemeo de `channel::local_axis`.\n\
    fn drive_local_axis(rot_deg: f32, comp: i32) -> vec2<f32> {\n\
        let ph = rot_deg / 360.0;\n\
        let c = drive_sin_cycles(ph + 0.25);\n\
        let sn = drive_sin_cycles(ph);\n\
        if (comp == 1) { return vec2<f32>(-sn, c); }\n\
        return vec2<f32>(c, sn);\n\
    }\n";

/// `falloff` and the value port, bound identically by every variant.
macro_rules! drive_common {
    () => {
        [
            ColumnBinding {
                column: "falloff",
                dim: Dim::Scalar,
                access: ColumnAccess::Read,
                // Absent falloff = full effect, the CPU's `falloff_at` fallback.
                identity: [1.0, 0.0, 0.0, 0.0],
                port: 0,
            },
            ColumnBinding {
                column: VALUE_COL,
                dim: Dim::Scalar,
                access: ColumnAccess::ReadBroadcast,
                // Absent value = 0.0, the `0 =>` arm of `value_at`.
                identity: [0.0; 4],
                port: 1,
            },
        ]
    };
}

/// **X / Y** — writes one component of `P`.
const DRIVE_P: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_comp = i32(drive_round(params.channel));\n\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_p = read_in_P(i);\n\
        var dr_cur = dr_p.x;\n\
        if (dr_comp == 1) { dr_cur = dr_p.y; }\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_out = drive_resolve(dr_cur, dr_v, dr_mode, dr_f);\n\
        var dr_next = dr_p;\n\
        if (params.space >= 0.5) {\n\
            // O MODO decide a magnitude, o ESPACO decide a direccao -- e o que se\n\
            // projecta e' o DELTA que o drive teria aplicado no eixo do mundo.\n\
            let dr_ax = drive_local_axis(read_in_rot(i), dr_comp);\n\
            dr_next = dr_p + (dr_out - dr_cur) * dr_ax;\n\
        } else if (dr_comp == 1) {\n\
            dr_next.y = dr_out;\n\
        } else {\n\
            dr_next.x = dr_out;\n\
        }\n\
        write_P(i, dr_next);\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // ⚠️ **O `rot`, para o espaço do ELEMENTO.** A identidade é `0`, que é o mesmo que
        // o `base_scalar(input, "rot", n, 0.0)` da CPU faz — e um `rot` ausente dá o eixo
        // `(1, 0)`, ou seja o espaço do mundo. A ausência da coluna nunca muda a resposta.
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Rotation** — writes `rot`, in degrees like the CPU's.
const DRIVE_ROT: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_cur = read_in_rot(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        write_rot(i, drive_resolve(dr_cur, dr_v, dr_mode, dr_f));\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size** — drives BOTH components uniformly, from the unit identity.
const DRIVE_SIZE: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_s = read_in_size(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_x = drive_resolve(dr_s.x, dr_v, dr_mode, dr_f);\n\
        let dr_y = drive_resolve(dr_s.y, dr_v, dr_mode, dr_f);\n\
        write_size(i, vec2<f32>(dr_x, dr_y));\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            // An element with no size starts UNIT, not zero (`base_vec2`).
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size X / Size Y** — writes ONE component of `size`, from the unit identity.
///
/// ⚠️ **Um kernel para os dois eixos, ramificando em `params.channel`** — o molde exacto
/// do [`DRIVE_P`], e pelo mesmo motivo: os dois escrevem a MESMA coluna com a MESMA
/// binding, então dois kernels seriam duas cópias de uma lei que não difere.
const DRIVE_SIZE_AXIS: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_comp = i32(drive_round(params.channel)) - 10;\n\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_s = read_in_size(i);\n\
        var dr_cur = dr_s.x;\n\
        if (dr_comp == 1) { dr_cur = dr_s.y; }\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_out = drive_resolve(dr_cur, dr_v, dr_mode, dr_f);\n\
        var dr_next = dr_s;\n\
        if (dr_comp == 1) { dr_next.y = dr_out; } else { dr_next.x = dr_out; }\n\
        write_size(i, dr_next);\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            // A MESMA identidade unitaria do `DRIVE_SIZE`: uma peca sem tamanho parte de 1.
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Opacity** — the ALPHA of `tint`, clamped to `[0,1]`. An element with no
/// tint starts from opaque white, so driving the opacity of an uncoloured stream
/// does what it says instead of silently nothing (doc 51).
const DRIVE_TINT: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_t = read_in_tint(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_a = drive_resolve(dr_t.w, dr_v, dr_mode, dr_f);\n\
        write_tint(i, vec4<f32>(dr_t.x, dr_t.y, dr_t.z, clamp(dr_a, 0.0, 1.0)));\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "tint",
            dim: Dim::Vec4,
            access: ColumnAccess::ReadWrite,
            identity: [1.0, 1.0, 1.0, 1.0],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// O prólogo mais a ida-e-volta HSV, **verbatim** o que `ph2d_color::rgb_to_hsv` /
/// `hsv_to_rgba` computam em Rust.
///
/// ⚠️ **É a segunda expressão da lei, e é inevitável** — o dispositivo não chama Rust. O que
/// a mantém honesta é o gate de paridade CPU×GPU deste nó, não a disciplina de quem edita.
/// (A `motion.luminance` carrega a metade da IDA pelo mesmo motivo; extrair as duas para um
/// `wgsl_lib` compartilhado é wave própria — o substrato hoje só tem lib POR KERNEL, e a
/// convenção da biblioteca é a mesma que já copia o `falloff_at` nove vezes.)
const DRIVE_LIB_HSV: &str = concat!(
    "\
    fn drive_rgb_to_hsv(c: vec4<f32>) -> vec3<f32> {\n\
        let mx = max(max(c.x, c.y), c.z);\n\
        let mn = min(min(c.x, c.y), c.z);\n\
        let d = mx - mn;\n\
        var h = 0.0;\n\
        if (d > 0.0) {\n\
            if (mx == c.x) { h = (c.y - c.z) / d + select(0.0, 6.0, c.y < c.z); }\n\
            else if (mx == c.y) { h = (c.z - c.x) / d + 2.0; }\n\
            else { h = (c.x - c.y) / d + 4.0; }\n\
            h = h / 6.0;\n\
        }\n\
        var s = 0.0;\n\
        if (mx > 0.0) { s = d / mx; }\n\
        return vec3<f32>(h, s, mx);\n\
    }\n\
    fn drive_hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {\n\
        // `rem_euclid(1.0)` do Rust: o matiz envolve AQUI, como na porta de Rust.\n\
        let hw = (h - floor(h)) * 6.0;\n\
        let i = floor(hw);\n\
        let f = hw - i;\n\
        let p = v * (1.0 - s);\n\
        let q = v * (1.0 - s * f);\n\
        let t = v * (1.0 - s * (1.0 - f));\n\
        let k = i32(i) % 6;\n\
        if (k == 0) { return vec3<f32>(v, t, p); }\n\
        if (k == 1) { return vec3<f32>(q, v, p); }\n\
        if (k == 2) { return vec3<f32>(p, v, t); }\n\
        if (k == 3) { return vec3<f32>(p, q, v); }\n\
        if (k == 4) { return vec3<f32>(t, p, v); }\n\
        return vec3<f32>(v, p, q);\n\
    }\n",
    "\
    fn drive_round(x: f32) -> f32 {\n\
        return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
    }\n\
    fn drive_combine(cur: f32, v: f32, mode: i32) -> f32 {\n\
        if (mode == 1) { return v; }\n\
        if (mode == 2) { return cur * v; }\n\
        return cur + v;\n\
    }\n"
);

/// **A COR sobre a cor que já está lá** ([`CH_HUE`]) — matiz, saturação e valor do `tint`.
///
/// ⚠️ **UMA variante para os TRÊS canais, e a régua é a BINDING, não o gosto:** uma variante
/// existe quando a lista de colunas ligadas difere (o módulo gerado só define `write_<col>`
/// para coluna BOUND), e estes três leem e escrevem exatamente o que o [`DRIVE_TINT`] lê e
/// escreve. Três variantes seriam três cópias do mesmo par de bindings esperando divergir; o
/// `channel` é uniforme no dispatch inteiro, então o ramo não diverge entre invocações.
const DRIVE_HSV: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_ch = i32(drive_round(params.channel));\n\
        let dr_t = read_in_tint(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_hsv = drive_rgb_to_hsv(dr_t);\n\
        var dr_cur = dr_hsv.z;\n\
        if (dr_ch == 6) { dr_cur = dr_hsv.x; }\n\
        else if (dr_ch == 7) { dr_cur = dr_hsv.y; }\n\
        let dr_next = drive_resolve(dr_cur, dr_v, dr_mode, dr_f);\n\
        var dr_h = dr_hsv.x;\n\
        var dr_s = dr_hsv.y;\n\
        var dr_val = dr_hsv.z;\n\
        if (dr_ch == 6) { dr_h = dr_next; }\n\
        else if (dr_ch == 7) { dr_s = clamp(dr_next, 0.0, 1.0); }\n\
        else { dr_val = max(dr_next, 0.0); }\n\
        let dr_rgb = drive_hsv_to_rgb(dr_h, dr_s, dr_val);\n\
        write_tint(i, vec4<f32>(dr_rgb.x, dr_rgb.y, dr_rgb.z, dr_t.w));\n",
    wgsl_lib: DRIVE_LIB_HSV,
    bindings: DRIVE_TINT.bindings,
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **The mask as a target** ([`CH_FALLOFF`]). The scalar template, with ONE difference that
/// carries the whole decision: every other variant binds `falloff` as a `Read` to blend with,
/// so this one — whose target IS `falloff` — binds it once as `ReadWrite` and simply has no
/// common read. The self-mask is not refused by taste; it is **inexpressible** in the binding
/// list, which is the kind of refusal that survives the next person.
///
/// No `dr_f` blend and no clamp, mirroring the CPU arm line for line.
const DRIVE_FALLOFF: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_cur = read_in_falloff(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        write_falloff(i, drive_combine(dr_cur, dr_v, dr_mode));\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            // Absent falloff = 1.0 — every reader's fallback, and the CPU's `base_scalar`
            // identity. A writer that started from 0 would disagree with the whole library
            // about what "no mask" means.
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 0,
        },
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// GPU compute kernel (ADR-0126) — the value domain's WRITE side on the device,
/// and the node that named [`GpuKernel::variant_by_param`].
///
/// `drive_channel` writes a DIFFERENT column per channel — `P` for X/Y, `rot`
/// for Rotation, `size` for Size, `tint` for Opacity — and materialises the
/// target from its identity when the stream lacks it. One static shape could not
/// express that: binding all four would emit columns **the CPU's output does not
/// carry** (a different stream SHAPE, not an ε), and binding one meant claiming
/// only the two channels that write `P`. So the node ships four variants and the
/// engine picks by `channel` — the SAME mapping `channel_column` uses, including
/// its `_ => size` catch-all for an out-of-range value.
///
/// The value port BROADCASTS: length 1 is one number held across the field (the
/// `1 => vals[0]` arm of `value_at`), length N is per-element.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    // The top-level shape IS the X/Y variant, so a caller that never resolves
    // still sees a real kernel rather than the empty (pass-through) one.
    wgsl: DRIVE_P.wgsl,
    wgsl_lib: DRIVE_P.wgsl_lib,
    bindings: DRIVE_P.bindings,
    params: DRIVE_PARAMS,
    count_law: None,
    // ⚠️ **O CUSTOM RECUSA o device** — ver [`CH_CUSTOM`]: uma `ColumnBinding`
    // carrega o nome como `&'static str`, e o nome que o artista digita só existe
    // em tempo de cook. O sequenciador recua para o `eval` da CPU, que é a porta
    // que a `Median` do `value.reduce` já usa.
    applicable: Some(|param| param("channel").round() as i32 != CH_CUSTOM),
    variant_by_param: Some(|param| {
        // The same rounding and the same mapping as `channel_column`.
        match param("channel").round() as i32 {
            2 => &DRIVE_ROT,
            CH_OPACITY => &DRIVE_TINT,
            CH_FALLOFF => &DRIVE_FALLOFF,
            CH_HUE | CH_SAT | CH_VAL => &DRIVE_HSV,
            0 | 1 => &DRIVE_P,
            CH_SIZE_X | CH_SIZE_Y => &DRIVE_SIZE_AXIS,
            _ => &DRIVE_SIZE,
        }
    }),
};
