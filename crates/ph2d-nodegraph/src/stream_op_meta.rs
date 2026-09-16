//! The **structural stream operations** side channel (ADR-0136) — the
//! count-changing family's contract, a sibling of the kernel metadata in
//! [`crate::gpu`] and re-exported there (`ph2d_nodegraph::gpu::StreamOp`).
//! Split out at the workspace LOC cap; the semantics are unchanged.

use crate::gpu::{CountLawCtx, GpuKernel, ReduceSpec};
use crate::port::Dim;

/// The reserved column a [`StreamOp::Compact`] predicate kernel writes its
/// verdict to: `1.0` keep, `0.0` drop, per element. An ordinary `f32` column so
/// the predicate is a PLAIN [`GpuKernel`] (same codegen, same uniform layout,
/// same pipeline cache) — the sequencer pops it off the predicate's output and
/// feeds it to the scan; it never reaches a downstream stream.
pub const KEEP_FLAG_COL: &str = "cp_keep";

/// The reserved column a [`StreamOp::SourceRows`] kernel writes: the TEMPLATE
/// row output element `i` is born from, as an `f32` value cast (exact for every
/// row below [`ID_WRAP`], which is also the id model's own ceiling — a template
/// wider than 2²⁴ is refused before dispatch). The sequencer gathers every
/// remaining template column at these rows and drops this one.
pub const ROWS_COL: &str = "cp_rows";

/// A node whose output stream is **structurally** different from its input —
/// filtered, minted from gathered rows, concatenated, or projected down to one
/// named column — declared on the side like a kernel (ADR-0126, ADR-0136). The
/// per-element MAP stays [`GpuKernel`]; these are the other four verbs of a
/// stream engine, and the machinery for each lives once in the sequencer.
///
/// The count of a [`Self::Compact`] output is a fact of the DATA, so it is
/// resolved on the device and read back at the compaction seam (8 bytes — the
/// bounded kind of readback; ADR-0136 §2). Every other variant's count is
/// host-computable (a sum, a count law, port 0).
#[derive(Copy, Clone, Debug)]
pub enum StreamOp {
    /// **Order-preserving filter** (`sim.lifetime`, `motion.cull`): before the
    /// node's own kernel runs, `predicate` — a plain [`GpuKernel`] writing
    /// [`KEEP_FLAG_COL`] — is dispatched over input `port`, the flags are
    /// exclusive-scanned, survivors scatter their row into a dense rows buffer,
    /// and every column of `port`'s stream is gathered at those rows. The node's
    /// registered kernel then runs on the COMPACTED stream (`sim.lifetime`'s
    /// `life` writer is an ordinary per-element kernel there); a passthrough
    /// kernel (`motion.cull`) means the compaction IS the node.
    Compact {
        /// The input port whose stream is filtered.
        port: usize,
        /// The predicate kernel: reads its declared bindings, writes
        /// [`KEEP_FLAG_COL`] (`1.0` keep / `0.0` drop) per element.
        predicate: GpuKernel,
    },
    /// **Birth by template gather** (`sim.spawn`): the node's registered kernel
    /// (whose `count_law` sizes the dispatch — this is why [`CountLawCtx`] has
    /// `dt`) writes [`ROWS_COL`] plus its own columns (`id`); afterwards the
    /// sequencer gathers every OTHER column of input `port` at those rows, so a
    /// newborn inherits whatever the template carries without the kernel
    /// enumerating it. No readback: the count is the count law's.
    SourceRows {
        /// The port carrying the template stream.
        port: usize,
    },
    /// **Concatenation** (`motion.combine`): the listed input ports, in order,
    /// laid end to end — column union, an input lacking a column (or carrying it
    /// at a different dim, the CPU's variant rule) contributes zeros. Pure
    /// `copy_buffer_to_buffer` + `clear_buffer`; no shader, no readback. Empty
    /// inputs are skipped exactly like the CPU's non-empty snapshot rule.
    Concat {
        /// The ports concatenated, in port order.
        ports: &'static [usize],
    },
    /// **Named-column projection** (`value.attribute`): the column's NAME is a
    /// TEXT param — dynamic, so inexpressible as a static [`ColumnBinding`] —
    /// resolved by the sequencer against the input stream's column map at cook
    /// time. Scalar mode is a buffer copy, length mode a fixed `vec2`-magnitude
    /// kernel, a missing/mistyped name a zero-fill: the CPU's exact ladder.
    Project {
        /// The text param naming the column (`value.attribute::ATTR_KEY`).
        text_param: &'static str,
        /// The f32 param selecting the mode (0 scalar · 1 vec2 length).
        mode_param: &'static str,
    },
    /// ⭐ **O ECO QUE SE CARREGA** (ciclo 7, W1d — o `motion.trail`): cada tique o nó é
    /// `filtrar(estado) ++ vivo`, e só DEPOIS o kernel do nó corre sobre a junção.
    ///
    /// 1. o `predicate` corre sobre a porta `state` CRUA (com as `predicate_reduces`
    ///    dobradas antes dele, sobre o mesmo estado cru — *há algum eco na faixa `1..s`?* é
    ///    uma pergunta sobre o estado INTEIRO) e o estado é compactado como num
    ///    [`Self::Compact`];
    /// 2. os sobreviventes e a porta `live` são JUNTADOS (sobreviventes primeiro, como a
    ///    CPU), com as [`ConcatFill`] a dizer a identidade de cada coluna que um dos lados
    ///    não tem — ⚠️ o [`Self::Concat`] enche de ZEROS, e um `size`/`tint` a zero apaga o
    ///    eco;
    /// 3. o kernel registado corre sobre a junção (porta 0 = a junção); a contagem é a da
    ///    junção, e o corpo sabe onde acabam os carregados por um uniform DERIVADO
    ///    (`DerivedUniform`) sobre as contagens CRUAS.
    ///
    /// ⚠️ **E há um caso em que a CPU devolve a entrada VIVA tal e qual** (o rastro de um
    /// eco só): `identity` responde-o sobre as contagens cruas, e então o estágio corre o
    /// `identity_kernel` sobre a porta `live` em vez de tudo o resto — que é um
    /// `PASSTHROUGH` ou o que a CPU ainda escreve por cima (a coluna do modo).
    Carry {
        /// A porta do estado (o `pre` do próprio nó).
        state_port: usize,
        /// A porta viva. ⚠️ Tem de ser a `0`: a junção passa a ser a base do kernel.
        live_port: usize,
        /// Escreve [`KEEP_FLAG_COL`] sobre a porta `state` crua.
        predicate: GpuKernel,
        /// As reduções que o predicado lê, dobradas sobre as entradas CRUAS.
        predicate_reduces: &'static [ReduceSpec],
        /// A identidade de cada coluna na junção — as que não estão aqui enchem a zero.
        fills: &'static [ConcatFill],
        /// `true` quando a CPU devolve a porta viva tal e qual (sobre as contagens cruas).
        identity: fn(&CountLawCtx<'_>) -> bool,
        /// O que corre sobre a porta viva nesse caso.
        identity_kernel: GpuKernel,
    },
}

/// **A identidade de UMA coluna numa junção** ([`StreamOp::Carry`]) — o `default_for` da CPU.
///
/// Cada faixa da coluna leva `value[faixa]`; `only_dim` restringe a regra a uma dimensão (o
/// `uv_rect` é o atlas inteiro só quando é `Vec4` — noutra dimensão a CPU enche a zero).
#[derive(Copy, Clone, Debug)]
pub struct ConcatFill {
    /// O nome da coluna.
    pub column: &'static str,
    /// O valor de cada faixa.
    pub value: [f32; 4],
    /// Só nesta dimensão (`None` = em todas).
    pub only_dim: Option<Dim>,
}

impl ConcatFill {
    /// A identidade da coluna `name` com a dimensão `dim` — `[0; 4]` quando nenhuma regra a
    /// nomeia.
    #[must_use]
    pub fn of(fills: &[ConcatFill], name: &str, dim: Dim) -> [f32; 4] {
        fills
            .iter()
            .find(|f| f.column == name && f.only_dim.is_none_or(|d| d == dim))
            .map_or([0.0; 4], |f| f.value)
    }
}
