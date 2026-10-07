#![allow(clippy::doc_markdown)]
//! **A MESMA LEI, DITA AO DISPOSITIVO** — o kernel do `sim.spawn` e a lei de contagem da janela.
//!
//! Mora num irmão porque o `lib.rs` cruzou o teto de LOC ao ganhar a probabilidade, e porque o
//! corte é honesto: o pai responde *o que este nó FAZ* (manifesto, `eval`, registro) e este
//! arquivo responde *como a MESMA resposta é dita ao device*. Nenhuma aritmética é reescrita
//! aqui — `born_in` e [`survives`](super::survives) são as do pai, chamadas por esta lei de
//! contagem, que é precisamente o que impede as duas metades de discordarem sobre quantos
//! elementos a janela tem.

use super::{LANE_BURST, MAX_PER_TICK, PULSE_COL, PULSE_ID_BASE, born_in, pulse_tick, survives};
use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, CountLawCtx, DerivedUniform, FIRED_ROW_COL, FiredBirth, GpuKernel,
    ID_WRAP, KEEP_FLAG_COL, ROWS_COL, SourceWindow,
};
use ph2d_nodegraph::port::Dim;

/// The GPU kernel (ADR-0136, `StreamOp::SourceRows`): output element `i` IS
/// newborn `window_first + i`. The kernel writes the newborn's identity and the
/// TEMPLATE ROW it is born from ([`ROWS_COL`]); the sequencer then gathers every
/// other template column at those rows — the newborn inherits the whole
/// vocabulary without this kernel enumerating a single column, exactly like the
/// CPU's [`newborns`].
///
/// The id wraps at [`ID_WRAP`] — and so does the CPU's, at the SAME single
/// point (`eval` wraps the ordinal before slotting and stamping), so both sides
/// hash and write the same number. `window_first` arrives already wrapped (the
/// count law's `f64` arithmetic, the emitter's pattern — ADR-0130).
///
/// [`slot`]'s expressions, verbatim: the scatter draw is the same avalanche
/// hash (`sp_hash3`, bit-exact in u32), `u32(draw · n) % n` is Rust's
/// `as usize % n`, round-robin is `id % n`. `rate` is NOT a kernel param — only
/// the count law (host-side, `f64`) reads it.
///
/// ⚠️ **A probabilidade quebra `saída[i] == window_first + i`, e a cura é uma VARREDURA DE
/// POSTO:** com um filtro a janela é esparsa, então o elemento `i` é o `i`-ésimo SOBREVIVENTE a
/// partir de `window_first` — achado pelo laço acima, com a MESMA [`survives`] (mesmo hash,
/// mesma pista 11, mesmo limiar `f32`) que a lei de contagem usou para dizer quantos são.
///
/// ⚠️ **O `256u` é o [`MAX_PER_TICK`], e é o que torna a varredura provadamente suficiente** —
/// `born_in` já capa o span do tique nesse mesmo número, então o `i`-ésimo sobrevivente está
/// dentro dos 256 primeiros candidatos por construção, e o limite do laço não é uma guarda
/// arbitrária: é o teto que a janela já tem. (Literal na string porque `concat!` não alcança uma
/// `const` numérica; há gate pinando os dois no mesmo número.)
///
/// ⚠️ **`probability >= 1` nem entra no laço** — o caminho que o dispositivo shipava é
/// byte-idêntico, exatamente como o `>= 1.0` da [`survives`] do lado da CPU.
/// O corpo da TAXA com o período `$span` — `16777216` sem pulso, `8388608` com ele (o `eval`
/// carva a metade de cima para os nascidos do pulso). Uma macro porque `concat!` não alcança uma
/// `const` numérica; o gate `the_two_rate_spans_are_the_eval_s_spans` pina os dois números.
macro_rules! taxa_wgsl {
    ($span:literal) => {
        concat!(
            "        var sp_id: u32 = (params.window_first + i) % ", $span, "u;\n",
            "        if (params.probability < 1.0) {\n",
            "            var sp_k: u32 = 0u;\n",
            "            var sp_seen: u32 = 0u;\n",
            "            loop {\n",
            "                if (sp_k >= 256u) { break; }\n",
            "                let sp_c = (params.window_first + sp_k) % ", $span, "u;\n",
            "                if (sp_rand01(u32(max(params.seed, 0.0)), sp_c, 11u) < params.probability) {\n",
            "                    if (sp_seen == i) { sp_id = sp_c; break; }\n",
            "                    sp_seen = sp_seen + 1u;\n",
            "                }\n",
            "                sp_k = sp_k + 1u;\n",
            "            }\n",
            "        }\n",
            "        var sp_row: u32 = 0u;\n",
            "        if (params.window_src_n > 0u) {\n",
            "            if (params.scatter >= 0.5) {\n",
            "                let sp_draw = sp_rand01(u32(max(params.seed, 0.0)), sp_id, 7u);\n",
            "                sp_row = u32(sp_draw * f32(params.window_src_n)) % params.window_src_n;\n",
            "            } else {\n",
            "                sp_row = sp_id % params.window_src_n;\n",
            "            }\n",
            "        }\n",
            "        write_cp_rows(i, f32(sp_row));\n",
            "        write_id(i, f32(sp_id));\n",
        )
    };
}

/// O hash e o seno parabólico, gémeos de `ph2d_motion_kit::{hash, trig}` (o do `motion.emitter`).
const SP_LIB: &str = "\
        fn sp_hash3(a: u32, b: u32, lane: u32) -> f32 {\n\
            var h: u32 = a * 0x9e3779b9u + b * 0x85ebca6bu + lane * 0xc2b2ae35u;\n\
            h = h ^ (h >> 16u);\n\
            h = h * 0x7feb352du;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x846ca68bu;\n\
            h = h ^ (h >> 16u);\n\
            return f32(h >> 8u) / f32(16777216u);\n\
        }\n\
        fn sp_rand01(seed: u32, id: u32, lane: u32) -> f32 {\n\
            return sp_hash3(seed, id, lane);\n\
        }\n\
        fn sp_sin_cycles(phase: f32) -> f32 {\n\
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
        }\n";

/// As escritas da taxa: a linha do modelo ([`ROWS_COL`]) e o id.
const TAXA_BINDINGS: &[ColumnBinding] = &[
    ColumnBinding {
        column: ROWS_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    },
    ColumnBinding {
        column: "id",
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    },
];

/// A janela da TAXA com o período `span` — o MESMO `born_in` e a MESMA [`survives`] do `eval`,
/// no mesmo `f64`. ⚠️ Com a probabilidade, `first` é a ORIGEM DA BUSCA: a janela é esparsa e o
/// kernel acha o `i`-ésimo sobrevivente a partir daqui.
fn janela_da_taxa(c: &CountLawCtx<'_>, span: u32) -> SourceWindow {
    let rate = (c.param)("rate") as f64;
    let seed = (c.param)("seed").max(0.0) as u32;
    let probability = (c.param)("probability");
    let born = born_in(rate, c.playhead, c.dt);
    let first = born.start % span;
    let count = born
        .map(|k| k % span)
        .filter(|id| survives(*id, seed, probability))
        .count();
    SourceWindow {
        count,
        first,
        age_first: 0.0,
    }
}

pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: taxa_wgsl!(16777216),
    wgsl_lib: SP_LIB,
    bindings: TAXA_BINDINGS,
    params: &["scatter", "seed", "probability"],
    count_law: Some(|c| janela_da_taxa(c, ID_WRAP)),
    variant_by_param: None,
    applicable: None,
};

/// A TAXA com o pulso ligado: o período é a metade, porque a de cima é dos nascidos do pulso.
pub(crate) const GPU_KERNEL_COM_PULSO: GpuKernel = GpuKernel {
    wgsl: taxa_wgsl!(8388608),
    count_law: Some(|c| janela_da_taxa(c, PULSE_ID_BASE)),
    ..GPU_KERNEL
};

/// **As linhas que DISPARAM** — o `pulse_born` só olha as linhas `0..n` do MODELO (`n` = a
/// contagem da porta 0), e uma linha dispara acima de `0,5`.
const DISPARO: GpuKernel = GpuKernel {
    wgsl: "        write_cp_keep(i, select(0.0, 1.0, read_pulse_pulse(i) > 0.5 && i < u32(params.cp_tpl_n)));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: PULSE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: KEEP_FLAG_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 1,
        },
    ],
    params: &["cp_tpl_n"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// O corpo do nascimento por PULSO: o candidato `k` é a irmã `k % burst` do disparo `k / burst`,
/// com o id `PULSE_ID_BASE + ((base + k) % PULSE_ID_BASE)`; a probabilidade é a MESMA busca de
/// posto da taxa, sobre os `min(F·burst, 256)` candidatos.
macro_rules! pulso_wgsl {
    ($kick:literal) => {
        concat!(
            "        let pb_burst = u32(max(floor(params.burst + 0.5), 0.0));\n",
            "        let pb_cand = min(min(u32(params.cp_fired_n), 256u) * min(pb_burst, 256u), 256u);\n",
            "        var pb_k: u32 = i;\n",
            "        if (params.probability < 1.0) {\n",
            "            var pb_j: u32 = 0u;\n",
            "            var pb_seen: u32 = 0u;\n",
            "            loop {\n",
            "                if (pb_j >= pb_cand) { break; }\n",
            "                let pb_c = 8388608u + ((params.window_first + pb_j) % 8388608u);\n",
            "                if (sp_rand01(u32(max(params.seed, 0.0)), pb_c, 11u) < params.probability) {\n",
            "                    if (pb_seen == i) { pb_k = pb_j; break; }\n",
            "                    pb_seen = pb_seen + 1u;\n",
            "                }\n",
            "                pb_j = pb_j + 1u;\n",
            "            }\n",
            "        }\n",
            "        let pb_id = 8388608u + ((params.window_first + pb_k) % 8388608u);\n",
            "        let pb_row = read_pulse_cp_fired(pb_k / max(pb_burst, 1u));\n",
            "        write_cp_rows(i, pb_row);\n",
            "        write_id(i, f32(pb_id));\n",
            $kick,
        )
    };
}

/// A linha disparada de cada candidato, lida na porta lateral já compactada.
const FIRED_ROW: ColumnBinding = ColumnBinding {
    column: FIRED_ROW_COL,
    dim: Dim::Scalar,
    access: ColumnAccess::SourceRead,
    identity: [0.0; 4],
    port: 1,
};

/// O nascimento por pulso SEM empurrão (`burst_speed = 0`): o `vel` vem do modelo pelo *gather*.
const PULSO: GpuKernel = GpuKernel {
    wgsl: pulso_wgsl!(""),
    wgsl_lib: SP_LIB,
    bindings: &[TAXA_BINDINGS[0], TAXA_BINDINGS[1], FIRED_ROW],
    params: &["seed", "probability", "burst", "cp_fired_n"],
    count_law: Some(janela_do_pulso),
    variant_by_param: Some(|p| {
        if p("burst_speed") == 0.0 {
            &PULSO
        } else {
            &PULSO_COM_EMPURRAO
        }
    }),
    applicable: None,
};

/// O nascimento por pulso COM empurrão — o `burst_kick`: o `vel` do modelo (ou zero, que é o que
/// a CPU materializa) mais `burst_speed` numa direcção sorteada do PRÓPRIO id (pista 23).
const PULSO_COM_EMPURRAO: GpuKernel = GpuKernel {
    wgsl: pulso_wgsl!(
        "        let pb_ph = sp_rand01(u32(max(params.seed, 0.0)), pb_id, 23u);\n\
         let pb_cs = vec2<f32>(sp_sin_cycles(pb_ph + 0.25), sp_sin_cycles(pb_ph));\n\
         write_vel(i, read_template_vel(u32(max(pb_row, 0.0))) + pb_cs * params.burst_speed);\n"
    ),
    bindings: &[
        TAXA_BINDINGS[0],
        TAXA_BINDINGS[1],
        FIRED_ROW,
        ColumnBinding {
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::SourceRead,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["seed", "probability", "burst", "cp_fired_n", "burst_speed"],
    variant_by_param: None,
    ..PULSO
};

/// A janela do PULSO: quantos dos `min(F·burst, 256)` candidatos sobrevivem — `F` é a LARGURA da
/// porta lateral já compactada (as linhas que dispararam), a única coisa que a lei pergunta.
fn janela_do_pulso(c: &CountLawCtx<'_>) -> SourceWindow {
    let burst = (c.param)("burst").round().max(0.0) as u32;
    let n = c.inputs.first().copied().unwrap_or(0);
    let fired = c.inputs.get(1).copied().unwrap_or(0);
    let vazio = SourceWindow {
        count: 0,
        first: 0,
        age_first: 0.0,
    };
    if burst == 0 || n == 0 || c.dt <= 0.0 {
        return vazio;
    }
    let seed = (c.param)("seed").max(0.0) as u32;
    let probability = (c.param)("probability");
    let cand = u64::from(fired)
        .saturating_mul(u64::from(burst))
        .min(u64::from(MAX_PER_TICK));
    let base = pulse_tick(c.playhead, c.dt).wrapping_mul(u64::from(MAX_PER_TICK));
    let id = |j: u64| PULSE_ID_BASE + ((base + j) % u64::from(PULSE_ID_BASE)) as u32;
    let count = (0..cand)
        .filter(|j| survives(id(*j), seed, probability))
        .count();
    SourceWindow {
        count,
        first: (base % u64::from(PULSE_ID_BASE)) as u32,
        ..vazio
    }
}

/// `cp_tpl_n` = a contagem do MODELO (porta 0) · `cp_fired_n` = a LARGURA da porta lateral já
/// compactada.
static DERIVADOS: &[DerivedUniform] = &[
    DerivedUniform {
        param: "cp_tpl_n",
        derive: |c| c.inputs.first().copied().unwrap_or(0) as f32,
    },
    DerivedUniform {
        param: "cp_fired_n",
        derive: |c| c.inputs.get(1).copied().unwrap_or(0) as f32,
    },
];

/// **O nascimento por pulso, dito ao dispositivo** (doc 110 §14.1 (7)).
pub(crate) const NASCIMENTO_POR_PULSO: FiredBirth = FiredBirth {
    port: 1,
    column: PULSE_COL,
    predicate: DISPARO,
    rate_kernel: GPU_KERNEL_COM_PULSO,
    kernel: PULSO,
    derived: DERIVADOS,
};

const _: () = assert!(LANE_BURST == 23, "o WGSL do empurrão usa a pista 23 à mão");
const _: () = assert!(
    PULSE_ID_BASE == 8_388_608,
    "o WGSL do pulso usa 8388608 à mão"
);
