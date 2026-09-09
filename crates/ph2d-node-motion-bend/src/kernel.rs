//! **O KERNEL WGSL do `motion.bend`** — cortado do `lib.rs` no teto de LOC do HR-18 (700 para
//! `crates/`), pela mesma costura que o `motion.drive`, o `motion.kaleidoscope` e o
//! `motion.spherize` já usam.
//!
//! O corte é por RESPONSABILIDADE: o `lib.rs` responde *como a dobra funciona* e este responde
//! *o que o dispositivo corre*. O `ui.rs` ao lado responde *como ela se apresenta*.

use super::VALUE_COL;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::port::Dim;

/// ⭐⭐⭐ **O WGSL QUE O KERNEL E AS REDUÇÕES PARTILHAM** — e é ele que tira o `direction` de
/// fora do dispositivo (ciclo 3, doc 106).
///
/// ⛔⛔ **A recusa que ele desfaz:** o extent num quadro rodado é um `Max`/`Min` sobre a
/// PROJECÇÃO `v.x·cos + v.y·sin`, e uma [`ReduceSpec::value`] só alcançava `params` — o
/// polinómio HR-5 teria de ser escrito **uma segunda vez** dentro da string da redução, que é
/// exactamente como as duas metades divergem. Com
/// [`wgsl_shared`](ph2d_nodegraph::gpu::KernelResolver::wgsl_shared) há **uma** fonte, colada
/// nos dois módulos pelo gerador.
///
/// ⚠️ **É o polinómio da CPU, portado operação a operação** — não o `sin`/`cos` do WGSL. A CPU é
/// livre de transcendentais por HR-5 (a seno parabólica corrigida, ~0,09% fora do trig
/// verdadeiro), então chamar o `sin` do dispositivo aqui não seria um ε mais apertado: seria
/// **outra curva**.
///
/// ⚠️ **`bd_dir_cos(0)` é `1,0` e `bd_dir_sin(0)` é `0,0` AO BIT** — conferido, não assumido:
/// em `phase = 0,25` a parábola dá `p = 1` e `0,225·(1·1 − 1) + 1 = 1`; em `phase = 0` dá
/// `p = 0` e `0,225·(0 − 0) + 0 = 0`. Mesmo assim o caminho de `direction == 0` é um **ramo
/// literal** nos dois lados, pelo caso degenerado do `±inf` e do `−0,0` (onde `x·1 + y·0` não é
/// `x`) — o mesmo precedente que a CPU já escreve em [`super::DIRECTION`].
pub(crate) const WGSL_SHARED: &str = "\
    // The corrected parabolic sine at `phase` CYCLES — the port of `trig.rs`.\n\
    fn bend_sin_cycles(phase: f32) -> f32 {\n\
        let f = phase - floor(phase);\n\
        var p: f32;\n\
        if (f < 0.5) {\n\
            let u = f * 2.0;\n\
            p = 4.0 * u * (1.0 - u);\n\
        } else {\n\
            let u = (f - 0.5) * 2.0;\n\
            p = -4.0 * u * (1.0 - u);\n\
        }\n\
        return 0.225 * (p * abs(p) - p) + p;\n\
    }\n\
    // O quadro LOCAL da dobra, em GRAUS (a unidade autorada do param).\n\
    fn bd_dir_cos(deg: f32) -> f32 { return bend_sin_cycles(deg / 360.0 + 0.25); }\n\
    fn bd_dir_sin(deg: f32) -> f32 { return bend_sin_cycles(deg / 360.0); }\n\
    // A projeccao de um ponto no eixo da dobra. Em `direction == 0` devolve `p.x` AO BIT,\n\
    // por RAMO — ver o doc-comment acima.\n\
    fn bd_axis(p: vec2<f32>, deg: f32) -> f32 {\n\
        if (deg == 0.0) { return p.x; }\n\
        return p.x * bd_dir_cos(deg) + p.y * bd_dir_sin(deg);\n\
    }\n";

/// The device form of [`bend`] (GPU/M5). One invocation per element, reading the
/// layout's X extent from the reduction above.
///
/// ⚠️ **The trig is the CPU's polynomial, ported operation for operation** — not
/// WGSL's `sin`/`cos`. The CPU is transcendental-free by HR-5 (the corrected
/// parabolic sine, ~0.09% off true trig), so calling the device's real `sin`
/// here would not be a tighter ε, it would be a *different curve*: the arc would
/// visibly differ from the canonical one wherever the approximation does.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: concat!(
        ph2d_nodegraph::pivot_wgsl!("f32(params.count)"),
        "\
        let bd_p = read_in_P(i);\n\
        // O quadro LOCAL da dobra — a MESMA costura da CPU (`bend::local`), e em\n\
        // `direction == 0` o ramo literal devolve `dx`/`dy` inalterados AO BIT.\n\
        let bd_rx = bd_p.x - pv_pivot.x;\n\
        let bd_ry = bd_p.y - pv_pivot.y;\n\
        let bd_dc = bd_dir_cos(params.direction);\n\
        let bd_ds = bd_dir_sin(params.direction);\n\
        var bd_dx = bd_rx;\n\
        var bd_dy = bd_ry;\n\
        if (params.direction != 0.0) {\n\
        \x20   bd_dx = bd_rx * bd_dc + bd_ry * bd_ds;\n\
        \x20   bd_dy = -bd_rx * bd_ds + bd_ry * bd_dc;\n\
        }\n\
        let bd_theta = params.angle * read_amount_v(i) * 3.1415927 / 180.0;\n\
        // A extensao, DERIVADA de duas reducoes independentes do pivo:\n\
        // `max|x - p|` e' `max(xmax - p, p - xmin)`, e a igualdade e' AO BIT\n\
        // (a subtraccao e' correctamente arredondada e o arredondamento e'\n\
        // monotono, entao o maximo dos arredondados e' o arredondado do maximo).\n\
        // Sem isto o `Centroid` seria uma reducao que depende de outra reducao,\n\
        // que o sequenciador nao sabe correr.\n\
        // ⚠️ Contra a projeccao do PIVO no mesmo eixo — as duas reducoes dobram\n\
        // `bd_axis(v)`, entao o `p` desta conta tem de ser `bd_axis(pivo)`. Em\n\
        // `direction == 0` isto e' `pv_pivot.x`, AO BIT (ramo literal em `bd_axis`).\n\
        let bd_pax = bd_axis(pv_pivot, params.direction);\n\
        let bd_ext = max(reduce_xmax() - bd_pax, bd_pax - reduce_xmin());\n\
        // A FATIA, ordenada — o `min`/`max` é a mesma lei da CPU (`slice_of`).\n\
        let bd_a = min(params.limit_lo, params.limit_hi) * bd_ext;\n\
        let bd_b = max(params.limit_lo, params.limit_hi) * bd_ext;\n\
        let bd_mid = (bd_a + bd_b) * 0.5;\n\
        let bd_half = (bd_b - bd_a) * 0.5;\n\
        let bd_mode = i32(bd_round(params.mode));\n\
        var bd_bent = vec2<f32>(bd_dx, bd_dy);\n\
        if (bd_half >= 1e-4 && abs(bd_theta) >= 1e-4) {\n\
        \x20   var bd_held = bd_dx;\n\
        \x20   if (bd_mode != 0) { bd_held = clamp(bd_dx, bd_a, bd_b); }\n\
        \x20   let bd_run = bd_dx - bd_held;\n\
        \x20   if (bd_mode != 2 || bd_run == 0.0) {\n\
        \x20       let bd_k = bd_theta / bd_half;\n\
        \x20       let bd_r = 1.0 / bd_k;\n\
        \x20       let bd_ph = (bd_k * (bd_held - bd_mid)) / 6.2831855;\n\
        \x20       let bd_c = bend_sin_cycles(bd_ph + 0.25);\n\
        \x20       let bd_s = bend_sin_cycles(bd_ph);\n\
        \x20       bd_bent = vec2<f32>((bd_r - bd_dy) * bd_s, bd_r * (1.0 - bd_c) + bd_dy * bd_c);\n\
        \x20       if (bd_run != 0.0) {\n\
        \x20           bd_bent = vec2<f32>(bd_bent.x + bd_run * bd_c, bd_bent.y + bd_run * bd_s);\n\
        \x20       }\n\
        \x20   }\n\
        }\n\
        // De volta ao MUNDO — a mesma base, no sentido contrario (CPU: `world`).\n\
        var bd_w = bd_bent;\n\
        if (params.direction != 0.0) {\n\
        \x20   bd_w = vec2<f32>(bd_bent.x * bd_dc - bd_bent.y * bd_ds,\n\
        \x20                    bd_bent.x * bd_ds + bd_bent.y * bd_dc);\n\
        }\n\
        let bd_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        write_P(i, vec2<f32>(\n\
        \x20   bd_p.x + (pv_pivot.x + bd_w.x - bd_p.x) * bd_f,\n\
        \x20   bd_p.y + (pv_pivot.y + bd_w.y - bd_p.y) * bd_f));\n"
    ),
    wgsl_lib: "\
        fn bd_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            // ReadWrite, not ReadWriteExisting: the CPU materialises an absent
            // `P` from the origin and always emits one (`out.set("P", …)`).
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            // The `amount_at` rule, declared: absent reads 1.0 (full static
            // bend), length-1 broadcasts, length-N is per element.
            access: ColumnAccess::ReadBroadcast,
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 1,
        },
    ],
    params: &[
        "angle",
        "direction",
        "mode",
        "limit_lo",
        "limit_hi",
        ph2d_nodegraph::pivot::PARAM,
        "pivot_x",
        "pivot_y",
    ],
    count_law: None,
    variant_by_param: None,
    // ⭐ **Sem recusa.** A do `direction` caiu quando o canal [`WGSL_SHARED`] deu à redução
    // acesso ao mesmo polinómio HR-5 que o kernel usa — ver o doc-comment dele.
    applicable: None,
};
