//! **O KERNEL WGSL do `motion.bend`** — cortado do `lib.rs` no teto de LOC do HR-18 (700 para
//! `crates/`), pela mesma costura que o `motion.drive`, o `motion.kaleidoscope` e o
//! `motion.spherize` já usam.
//!
//! O corte é por RESPONSABILIDADE: o `lib.rs` responde *como a dobra funciona* e este responde
//! *o que o dispositivo corre*. O `ui.rs` ao lado responde *como ela se apresenta*.

use super::VALUE_COL;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::port::Dim;

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
        let bd_dx = bd_p.x - pv_pivot.x;\n\
        let bd_dy = bd_p.y - pv_pivot.y;\n\
        let bd_theta = params.angle * read_amount_v(i) * 3.1415927 / 180.0;\n\
        // A extensao, DERIVADA de duas reducoes independentes do pivo:\n\
        // `max|x - p|` e' `max(xmax - p, p - xmin)`, e a igualdade e' AO BIT\n\
        // (a subtraccao e' correctamente arredondada e o arredondamento e'\n\
        // monotono, entao o maximo dos arredondados e' o arredondado do maximo).\n\
        // Sem isto o `Centroid` seria uma reducao que depende de outra reducao,\n\
        // que o sequenciador nao sabe correr.\n\
        let bd_ext = max(reduce_xmax() - pv_pivot.x, pv_pivot.x - reduce_xmin());\n\
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
        let bd_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        write_P(i, vec2<f32>(\n\
        \x20   bd_p.x + (pv_pivot.x + bd_bent.x - bd_p.x) * bd_f,\n\
        \x20   bd_p.y + (pv_pivot.y + bd_bent.y - bd_p.y) * bd_f));\n"
    ),
    wgsl_lib: "\
        fn bd_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
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
        "mode",
        "limit_lo",
        "limit_hi",
        ph2d_nodegraph::pivot::PARAM,
        "pivot_x",
        "pivot_y",
    ],
    count_law: None,
    variant_by_param: None,
    // ⚠️ Ver [`DIRECTION`]: a redução `x_extent` não roda com o quadro.
    applicable: Some(|p| p("direction") == 0.0),
};
