//! **O KERNEL WGSL do `motion.kaleidoscope`** — cortado do `lib.rs` no teto de LOC do HR-18
//! (700 para `crates/`), pela mesma costura que o `motion.drive` e o `motion.noise` já usam.
//!
//! O corte é por RESPONSABILIDADE e não por tamanho: o `lib.rs` responde *o que um caleidoscópio
//! É* (o manifesto, o `eval`, o registo) e este responde *o que o dispositivo corre*. Nada aqui
//! é lido pela CPU — se fosse, o corte estaria no sítio errado.

use super::{MAX_SEGMENTS, REINDEX, VALUE_COL};
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, ROWS_COL, SourceWindow};
use ph2d_nodegraph::port::Dim;

/// The GPU kernel (GPU/M5, ADR-0136 `StreamOp::SourceRows`): a COUNT-CHANGING
/// deformer, output length `segments · n`, slice-major (output `i` is slice
/// `i / n`, source row `i % n`).
///
/// ⚠️ **This is the first `SourceRows` kernel that READS its template.**
/// `sim.spawn` only writes `id`/`cp_rows` and lets the gather copy the template;
/// kaleidoscope reads the source `P` at row `i % window_src_n`
/// ([`ColumnAccess::SourceRead`], the length-decouple that makes a template-port
/// read present) and writes a ROTATED output `P` — so `P` is TWO bindings
/// (SourceRead in, Write out; they are different buffers). The sequencer then
/// gathers every OTHER template column at `cp_rows`, duplicating `size`/`tint`/
/// `id` onto each slice exactly like the CPU's `dup_n`.
///
/// ⚠️ **The trig is the CPU's parabolic sine, ported** (see `motion.bend`): HR-5
/// keeps the canonical path transcendental-free, so the device's real `sin` would
/// be a different curve, not a tighter ε. The rotation matches [`kaleidoscope`]
/// operation for operation, including the odd-slice mirror (`reflect`).
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: concat!(
        // ⚠️ O divisor é a contagem da ENTRADA (`window_src_n`) e não `params.count`:
        // este kernel emite `segments · n`, e com a contagem da saída o centroide sairia
        // `segments` vezes menor. ⚠️ E é a expressão crua, não o `k_srcn` — o prólogo é
        // colado ANTES do corpo, então a variável local ainda não existe aqui.
        ph2d_nodegraph::pivot_wgsl!("f32(max(params.window_src_n, 1u))"),
        "\
        let k_srcn = max(params.window_src_n, 1u);\n\
        let k_seg = clamp(round(params.segments), 1.0, 256.0);\n\
        let k_s = i / k_srcn;\n\
        let k_row = i % k_srcn;\n\
        write_cp_rows(i, f32(k_row));\n\
        let k_src = read_in_P(k_row);\n\
        let k_lx = k_src.x - pv_pivot.x;\n\
        var k_ly = k_src.y - pv_pivot.y;\n\
        if (round(params.reflect) != 0.0 && (k_s % 2u) == 1u) { k_ly = -k_ly; }\n\
        let k_ph = f32(k_s) / k_seg + read_spin_v(0u) / 360.0;\n\
        let k_c = kal_sin_cycles(k_ph + 0.25);\n\
        let k_sn = kal_sin_cycles(k_ph);\n\
        write_P(i, vec2<f32>(\n\
        \x20   k_lx * k_c - k_ly * k_sn + pv_pivot.x,\n\
        \x20   k_lx * k_sn + k_ly * k_c + pv_pivot.y));\n"
    ),
    wgsl_lib: "\
        // The corrected parabolic sine at `phase` CYCLES — the port of `trig.rs`.\n\
        fn kal_sin_cycles(phase: f32) -> f32 {\n\
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
        // The source position, read at the mapped row `i % src_n` — length
        // decoupled from the dispatch (the template is `n`, the output `n·seg`).
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::SourceRead,
            identity: [0.0; 4],
            port: 0,
        },
        // The rotated OUTPUT position — a separate buffer from the read above.
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        // The template row each output element is born from — the SourceRows
        // machinery gathers every other column at these rows and drops this one.
        ColumnBinding {
            column: ROWS_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        // The global spin (degrees), broadcast at index 0 — the CPU's `first()`.
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 1,
        },
    ],
    params: &[
        "segments",
        "reflect",
        ph2d_nodegraph::pivot::PARAM,
        "pivot_x",
        "pivot_y",
    ],
    // Output = `n · segments`, the CPU's `positions.len()`. `segments` is clamped
    // to the SAME `[1, MAX_SEGMENTS]` the `eval` uses, so both sides mint the same
    // count and the kernel's `k_seg` divisor matches its slice count.
    count_law: Some(|c| {
        let n = c.inputs.first().copied().unwrap_or(0) as usize;
        let segments = ((c.param)("segments").round() as i64).clamp(1, MAX_SEGMENTS) as usize;
        SourceWindow::of_count(n * segments)
    }),
    variant_by_param: None,
    // ⚠️ **O device RECUA quando a renumeração está ligada** — o mesmo `applicable`
    // que o `motion.combine` já declara, pelo mesmo motivo estrutural: este é um
    // kernel `SourceRows`, e as colunas que não são `P` chegam à saída por um
    // GATHER de `cp_rows` (uma cópia do template), não por uma escrita do corpo.
    // Renumerar é escrever `Index`/`Count` **novos**, que é outra operação — e uma
    // renumeração em `segments · n` elementos é um passe de escrita linear, não o
    // caminho quente que este nó existe para acelerar.
    applicable: Some(|p| p(REINDEX) < 0.5),
};
