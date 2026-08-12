#![allow(clippy::doc_markdown)]
//! **A MESMA LEI, DITA AO DISPOSITIVO** — o kernel do `sim.spawn` e a lei de contagem da janela.
//!
//! Mora num irmão porque o `lib.rs` cruzou o teto de LOC ao ganhar a probabilidade, e porque o
//! corte é honesto: o pai responde *o que este nó FAZ* (manifesto, `eval`, registro) e este
//! arquivo responde *como a MESMA resposta é dita ao device*. Nenhuma aritmética é reescrita
//! aqui — `born_in` e [`survives`](super::survives) são as do pai, chamadas por esta lei de
//! contagem, que é precisamente o que impede as duas metades de discordarem sobre quantos
//! elementos a janela tem.

use super::{PULSE_COL, born_in, survives};
use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, GpuKernel, ID_WRAP, ROWS_COL, SourceWindow,
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
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        var sp_id: u32 = (params.window_first + i) % 16777216u;\n\
        if (params.probability < 1.0) {\n\
        \x20   var sp_k: u32 = 0u;\n\
        \x20   var sp_seen: u32 = 0u;\n\
        \x20   loop {\n\
        \x20       if (sp_k >= 256u) { break; }\n\
        \x20       let sp_c = (params.window_first + sp_k) % 16777216u;\n\
        \x20       if (sp_rand01(u32(max(params.seed, 0.0)), sp_c, 11u) < params.probability) {\n\
        \x20           if (sp_seen == i) { sp_id = sp_c; break; }\n\
        \x20           sp_seen = sp_seen + 1u;\n\
        \x20       }\n\
        \x20       sp_k = sp_k + 1u;\n\
        \x20   }\n\
        }\n\
        var sp_row: u32 = 0u;\n\
        if (params.window_src_n > 0u) {\n\
        \x20   if (params.scatter >= 0.5) {\n\
        \x20       let sp_draw = sp_rand01(u32(max(params.seed, 0.0)), sp_id, 7u);\n\
        \x20       sp_row = u32(sp_draw * f32(params.window_src_n)) % params.window_src_n;\n\
        \x20   } else {\n\
        \x20       sp_row = sp_id % params.window_src_n;\n\
        \x20   }\n\
        }\n\
        write_cp_rows(i, f32(sp_row));\n\
        write_id(i, f32(sp_id));\n",
    wgsl_lib: "\
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
        }\n",
    bindings: &[
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
        // **The pulse port's refusal** (ADR-0127 D3). This kernel has no lane for an event
        // column, and a pulse-born element is born at the row that FIRED — arithmetic the
        // device cannot reach without a prefix scan it was never given. Absent column ⇒ the
        // plan claims the node exactly as it always has (the device path this wave ships is
        // byte-identical); present ⇒ the frame recedes to the CPU, which is where the pulse
        // family already lives. The alternative is a device answer with every pulse-birth
        // silently MISSING, and nothing on screen to say so.
        ColumnBinding {
            column: PULSE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::RefuseIfPresent,
            identity: [0.0; 4],
            port: 1,
        },
    ],
    params: &["scatter", "seed", "probability"],
    // The SAME `born_in` the CPU `eval` runs, in the same `f64` — this is why
    // `CountLawCtx` carries `dt` (ADR-0136). The window's `first` is wrapped
    // here, once, in integer arithmetic (the emitter's rule: the kernel is TOLD
    // the window, never re-derives it).
    //
    // ⚠️ **Com a probabilidade, `first` deixa de ser o id da saída 0 e passa a ser a ORIGEM DA
    // BUSCA** — a janela vira esparsa, e o kernel acha o `i`-ésimo sobrevivente a partir daqui.
    // O `count` é quantos sobrevivem, contados pela MESMA [`survives`] que o `eval` chama, sobre
    // o MESMO id envolvido. Duas contas discordando aqui não dariam erro: dariam uma janela com
    // elementos que o kernel não sabe preencher.
    //
    // ⚠️ **O wrap é o `ID_WRAP` e não o `span` do `eval`**, porque esta lei só corre quando o
    // dispositivo aceita o nó — e ele RECUSA com a porta `pulse` fiada (a binding
    // `RefuseIfPresent` abaixo), que é exatamente o caso em que o `span` seria a metade.
    count_law: Some(|c| {
        let rate = (c.param)("rate") as f64;
        let seed = (c.param)("seed").max(0.0) as u32;
        let probability = (c.param)("probability");
        let born = born_in(rate, c.playhead, c.dt);
        let first = born.start % ID_WRAP;
        let count = born
            .map(|k| k % ID_WRAP)
            .filter(|id| survives(*id, seed, probability))
            .count();
        SourceWindow {
            count,
            first,
            age_first: 0.0,
        }
    }),
    variant_by_param: None,
    applicable: None,
};
