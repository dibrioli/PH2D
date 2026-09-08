//! **O KERNEL WGSL do `motion.transform` e as reduções dele** — cortado do `lib.rs` no teto de
//! LOC do HR-18 (700 para `crates/`), pela mesma costura que o `motion.bend`, o
//! `motion.kaleidoscope`, o `motion.spherize` e o `motion.mirror` já usam neste grupo.
//!
//! O corte é por RESPONSABILIDADE: o `lib.rs` responde *o que o afim de layout É* (o manifesto,
//! o `eval`, o registo) e este responde *o que o dispositivo corre e o que ele mede antes*.

use super::{SKEW_X, SKEW_Y};
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, ReduceSpec};
use ph2d_nodegraph::port::Dim;

/// GPU compute kernel (GPU/M5 Fase 2, ADR-0126): the exact per-element map of
/// the CPU `eval` — `full = p·scale + (ox, oy)` then `p' = p + (full − p)·falloff`
/// — in the SAME multiply/add order, so parity holds within GPU-FMA ε (the ε
/// gate). No `applicable`: a plain affine covers the whole param space (no enum,
/// no partial coverage). `ReadWriteExisting` on `P` mirrors the CPU's
/// pattern-match — a stream WITHOUT a `P` column passes through untouched, so
/// absence means the same thing on both paths (the falloff read materializes
/// its `1.0` identity when absent = full effect).
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: concat!(
        // ⚠️ O MODO decide o pivo, e o prologo vem da PORTA
        // (`ph2d_nodegraph::pivot`) -- nao de uma copia aqui. O que estava neste
        // sitio perguntava «o ponto digitado e' diferente de zero?» e nao olhava o
        // modo: um `pivot_x` deixado para tras (a row esta' escondida pelo
        // `ParamGate`, o valor nao) vazava para o device e desenhava outra coisa.
        ph2d_nodegraph::pivot_wgsl!("f32(params.count)"),
        "\
        let xf_f = read_falloff(i);\n\
        let xf_p = read_P(i);\n\
        // O link de corrente, lido como no CPU (`authored_factors`): `>= 0.5`, e ligado os\n\
        // dois eixos sao o MESMO numero.\n\
        let xf_sx = params.scale;\n\
        let xf_sy = select(params.scale_y, params.scale, params.uniform >= 0.5);\n\
        // The pivot folded into the offset, the same expression and the same\n\
        // order as the CPU's `folded_offset` -- including its zero shortcut, so\n\
        // the neutral is structural on both paths and not an IEEE argument.\n\
        // O CISALHAMENTO -- ver [`Shear`]. O ramo NEUTRO fica escrito a' parte porque\n\
        // `a + 0.0` nao e' `a` quando `a` e' `-0.0`: a identidade e' da ESTRUTURA.\n\
        let xf_neutral = params.skew_x == 0.0 && params.skew_y == 0.0;\n\
        var xf_ox = params.offset_x;\n\
        var xf_oy = params.offset_y;\n\
        if (pv_pivot.x != 0.0 || pv_pivot.y != 0.0) {\n\
        \x20   if (xf_neutral) {\n\
        \x20       xf_ox = params.offset_x + pv_pivot.x * (1.0 - xf_sx);\n\
        \x20       xf_oy = params.offset_y + pv_pivot.y * (1.0 - xf_sy);\n\
        \x20   } else {\n\
        \x20       xf_ox = params.offset_x + pv_pivot.x\n\
        \x20           - (pv_pivot.x * xf_sx + pv_pivot.y * params.skew_x);\n\
        \x20       xf_oy = params.offset_y + pv_pivot.y\n\
        \x20           - (pv_pivot.y * xf_sy + pv_pivot.x * params.skew_y);\n\
        \x20   }\n\
        }\n\
        var xf_full = vec2<f32>(\n\
            xf_p.x * xf_sx + xf_ox,\n\
            xf_p.y * xf_sy + xf_oy);\n\
        if (!xf_neutral) {\n\
        \x20   xf_full = vec2<f32>(\n\
        \x20       xf_p.x * xf_sx + xf_p.y * params.skew_x + xf_ox,\n\
        \x20       xf_p.y * xf_sy + xf_p.x * params.skew_y + xf_oy);\n\
        }\n\
        write_P(i, vec2<f32>(\n\
            xf_p.x + (xf_full.x - xf_p.x) * xf_f,\n\
            xf_p.y + (xf_full.y - xf_p.y) * xf_f));\n"
    ),
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWriteExisting,
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
    ],
    params: &[
        "scale",
        "uniform",
        "scale_y",
        "offset_x",
        "offset_y",
        "pivot_mode",
        "pivot_x",
        "pivot_y",
        SKEW_X,
        SKEW_Y,
    ],
    count_law: None,
    variant_by_param: None,
    // ⭐⭐ **Sem recusa: os TRÊS modos correm no dispositivo** (ciclo 3, W1).
    // A recusa que estava aqui dizia que o centroide *«é uma REDUÇÃO sobre o
    // stream e não um mapa por elemento»* e nomeava a própria cura — *«o canal
    // `reduce -> broadcast -> map` que os deformadores usam é o que a
    // levantaria»*. Esse canal já tinha shipado (GPU/M5) e o `motion.spherize`
    // já media o centroide com ele; a recusa sobreviveu ao dia em que deixou de
    // ser verdade. Ver [`REDUCES`].
    applicable: None,
};

pub(crate) static REDUCES: &[ReduceSpec] = ph2d_nodegraph::pivot::CENTROID_REDUCES;
