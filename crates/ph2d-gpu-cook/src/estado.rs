//! **O ESTADO DO SEQUENCIADOR** — o que a [`GpuCook`] GUARDA entre quadros.
//!
//! ⚠️ Irmão do `lib.rs` pelo tecto de LOC (HR-18) e por ASSUNTO: ali está o que o sequenciador
//! FAZ (uma função de ~450 linhas, o `cook`), aqui o que ele TEM — o pool, as caches de pipeline,
//! o estado da simulação entre tiques, e os valores dos fios deste quadro.
//!
//! ⚠️ **Os campos são `pub(crate)` porque o tipo saiu da raiz.** Um campo privado é visível ao
//! módulo que o declara e aos descendentes dele; com a struct na raiz, todo módulo da crate os
//! lia. *Mover um tipo muda quem o pode ler, e isso é o que este `pub(crate)` compra de volta.*

use crate::{
    BufferPool, CachedPipeline, GpuCheckpointRing, GpuInstances, GpuStream, grid, plan, reduce,
    reduce_stage, shape, stream_op, tap, voronoi,
};
use ph2d_nodegraph::graph::NodeId;
use std::collections::BTreeMap;

/// The sequencer. Owns the buffer pool, the pipeline caches and the
/// persistent instance output; reuse ONE across frames (like the CPU pump).
#[derive(Default)]
pub struct GpuCook {
    pub(crate) pool: BufferPool,
    /// Kernel pipelines keyed by `(node type, column-presence signature)`.
    pub(crate) kernel_pipelines: BTreeMap<(u64, u64), CachedPipeline>,
    /// Lowering pipelines keyed by the 6-column presence signature.
    pub(crate) lower_pipelines: BTreeMap<u64, CachedPipeline>,
    /// Per-stage uniform buffers (index = stage position; last = lowering).
    pub(crate) uniforms: Vec<wgpu::Buffer>,
    /// The persistent instance output (grow-only, like `InstanceBuffer`).
    pub(crate) instances: Option<GpuInstances>,
    /// **Last tick's output** of each node that feeds a `pre` edge — the GPU
    /// mirror of `Cook::prev_outputs`, populated at the end of every cook by
    /// the same rule as `Cook::advance_tick_scoped` (ADR-0127 D1).
    ///
    /// This IS the simulation state, and holding it costs a refcount: a
    /// `GpuStream` is `Arc<wgpu::Buffer>` columns, so "last tick's output" is
    /// literally the buffers the last tick wrote, and [`BufferPool::reclaim`]
    /// skips anything still referenced. No readback, no copy, no barrier —
    /// the ping-pong falls out of the fact that state was always a column.
    pub(crate) prev: BTreeMap<NodeId, GpuStream>,
    /// ⭐⭐⭐ **O VALOR DE CADA PARAM DIRIGIDO neste quadro** (doc 110 §3). Quem o enche é o
    /// chamador, que tem o cozedor da CPU em mão; o sequenciador não coze nada.
    ///
    /// ⚠️ **Vazio = a lei de antes desta wave**: nenhum nó com fio é encenado (`plan::eligible`),
    /// logo um chamador que se esqueça de o encher perde o dispositivo — nunca desenha errado.
    pub(crate) driven: plan::DrivenParams,
    /// What the host knows about the last cook — element counts and column sets
    /// per staged node, for the graph panel (see [`shape::CookShape`]).
    pub(crate) shape: shape::CookShape,
    /// Gate-only stream retention ([`debug_read`]); off in production, where it
    /// would pin every intermediate against [`BufferPool::reclaim`].
    pub(crate) debug_retain: bool,
    pub(crate) debug_streams: BTreeMap<NodeId, GpuStream>,
    /// Last cook's output streams, held for the frame-path [`tap`]. **Cleared at
    /// the top of every cook, BEFORE [`BufferPool::reclaim`]** — holding a
    /// `GpuStream` is a refcount on its buffers, so a hold that outlived the
    /// frame would defeat the pool exactly like `debug_streams` does. Held only
    /// across the window in which the buffers are alive anyway, it costs nothing.
    ///
    /// Populated unconditionally rather than behind a flag: the tap is a
    /// *frame-path* facility (0,5% of a 60 fps frame, measured), and a flag would
    /// mean the panel's first frame after enabling it shows the previous cook.
    pub(crate) tap_streams: BTreeMap<NodeId, GpuStream>,
    /// The tap's compute pipeline, built on first use and reused. `Option` and
    /// not built in `new()` because `GpuCook` is `Default` and has no device.
    pub(crate) tap_pipeline: Option<tap::TapPipeline>,
    /// The fixed tick [`Self::prev`] belongs to — the GPU sim's own clock,
    /// mirroring `MotionCookPump::last_cooked_tick`. A sequential cook owes one
    /// step per tick, so the caller needs to know which one it last took; a
    /// stateless plan never reads this.
    pub(crate) last_tick: Option<u64>,
    /// The playhead of the last cook — the GPU mirror of `Cook::prev_playhead`,
    /// and computed into a count law's `dt` by **the same expression** the CPU's
    /// `EvalCtx::dt` uses (`map_or(0.0, |p| playhead - p)`), so a birth law
    /// (`sim.spawn`, ADR-0136) counts the same births on both sides. `None`
    /// after a seed (`dt = 0` — nothing is born on a tick with no history);
    /// restored through the scrub ring like the CPU checkpoint restores its
    /// `prev_playhead`.
    pub(crate) last_playhead: Option<f64>,
    /// The backwards-scrub ring (D5): past states, held by refcount, on the
    /// device. See [`ring`].
    pub(crate) ring: GpuCheckpointRing,
    /// A **live edit** invalidated the sim: the next [`Self::rewind_for`] seeds AT
    /// the tick it is asked for instead of anchoring at 0. See
    /// [`Self::reseed_from_next_tick`].
    pub(crate) reseed: bool,
    /// The spatial-grid service (ADR-0140 D2), built on first use like the tap
    /// pipeline — `Option` because `GpuCook` is `Default` and has no device.
    pub(crate) grid: Option<grid::Grid>,
    /// Transients of THIS cook's grid builds (scan scratch + cursors). Cleared at
    /// the top of every cook, like [`Self::tap_streams`]; wgpu keeps the buffers
    /// alive across the submit even as the next cook drops them.
    pub(crate) grid_scratch: grid::GridScratch,
    /// The grid output buffers (`starts`/`sorted`) a kernel pass binds, held for
    /// the same window — one per grid-bearing stage this cook.
    pub(crate) grid_hold: Vec<grid::GridBuffers>,
    /// The whole-stream reduction service (the deformer channel), built on first
    /// use like the grid — `Option` because `GpuCook` is `Default`.
    pub(crate) reduce: Option<reduce::Reduce>,
    /// The reduce map passes' N-sized scratch, held until this cook's submit —
    /// the same window as [`Self::grid_hold`], for the same reason. Pooled
    /// column buffers are `Arc`, the per-pass uniforms are owned, so both shapes
    /// are held (mirroring `stream_op_hold` / `stream_op_hold_bufs`).
    pub(crate) reduce_hold: Vec<wgpu::Buffer>,
    pub(crate) reduce_hold_bufs: Vec<std::sync::Arc<wgpu::Buffer>>,
    /// The 4-byte reduction results a kernel pass binds, held for the same
    /// window — one set per reducing stage this cook.
    pub(crate) reduce_results_hold: Vec<reduce_stage::ReduceResults>,
    /// The LUT tables a kernel pass samples (A1-gpu), held for the same window —
    /// filled from a text param and bound like the reduction results. One flat
    /// Vec across the cook, cleared at the top like the others.
    pub(crate) lut_hold: Vec<wgpu::Buffer>,
    /// The structural stream-op pipelines (ADR-0136), built on first use like
    /// the tap and the grid — `Option` because `GpuCook` is `Default`.
    pub(crate) stream_op_pipes: Option<stream_op::StreamOpPipes>,
    /// Stream-op **and algorithm** transients that must outlive THIS cook's
    /// final submit (the post-compaction gathers' uniforms and rows buffers;
    /// the voronoi passes' uniforms, ADR-0139). Cleared at the top of every
    /// cook, like [`Self::grid_hold`].
    pub(crate) stream_op_hold: Vec<wgpu::Buffer>,
    pub(crate) stream_op_hold_bufs: Vec<std::sync::Arc<wgpu::Buffer>>,
    /// The engine-algorithm pipelines (ADR-0139), built on first use like the
    /// stream ops — `Option` because `GpuCook` is `Default`.
    pub(crate) voronoi_pipes: Option<voronoi::VoronoiPipes>,
    /// The texture-run partition of the last cook's instance buffer, so the
    /// renderer can draw a `source.object` graph by binding the object's texture
    /// per run — computed CPU-side from the boundary, no readback (see
    /// [`tex_runs`]). Empty for a non-object graph. Persistent, like
    /// [`Self::instances`].
    pub(crate) tex_runs: Vec<ph2d_render::GpuTexRun>,
    /// ⭐⭐⭐ **A COSTURA QUE NÃO MUDOU NÃO SE ENVIA OUTRA VEZ** (ciclo 8, W1 — doc 113 §6).
    ///
    /// Por nó de fronteira: o stream da CPU que foi enviado e o `GpuStream` que ele virou. Um
    /// quadro cuja costura PARTILHA as alocações com esta
    /// ([`ph2d_nodegraph::attr::Stream::shares_storage_with`]) reutiliza o envio — uma tabela, um
    /// texto ou uma forma parada era copiada para a placa sessenta vezes por segundo.
    ///
    /// ⚠️ **Guardar o stream da CPU ao lado é o que torna a comparação SEGURA**: ele segura os
    /// `Arc` das colunas, e uma alocação que não pode ser libertada não pode ser reutilizada noutro
    /// sítio com o mesmo endereço.
    ///
    /// ⚠️ **E segurar o `GpuStream` é o que o protege do [`BufferPool::reclaim`]** — pela mesma
    /// propriedade que já mantém [`Self::prev`] vivo: o pool só recicla o que mais ninguém
    /// referencia.
    pub(crate) sent_boundaries: BTreeMap<NodeId, (ph2d_nodegraph::attr::Stream, GpuStream)>,
    /// Quantas costuras foram ENVIADAS e quantas foram reconhecidas como a mesma — o instrumento
    /// da cura acima, pela mesma razão do contador do `Cook::set_external`: o ganho não muda o
    /// comportamento, então sem contador nenhum gate o distingue de não o fazer.
    pub(crate) boundary_uploads: u64,
    pub(crate) boundary_reuses: u64,
}
