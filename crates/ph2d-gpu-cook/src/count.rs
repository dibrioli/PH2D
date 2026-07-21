//! **How wide is this stage?** — the engine's count law, asked in one place.
//!
//! Dispatch size is host-side, so before a stage can run the sequencer must
//! answer "how many elements does this node emit?". For most nodes the answer is
//! *"as many as port 0"* — the output rides its base input. But that is a LAW,
//! not a fact of the machine, and until this module existed it was the only law
//! the engine knew, hard-coded at the one call site with a generator's window as
//! the single special case.
//!
//! That hard-coding was a latent divergence: `value.lfo` unconnected computes
//! `n = input(0).count().max(1)` on the CPU — one global oscillation — while the
//! GPU sized it by port 0, got `0`, and **skipped the stage entirely**. Nothing
//! could produce a length-1 VALUE field on the device, so the whole `value.*`
//! family was unreachable the moment a consumer of one appeared. It never fired
//! because no consumer had a kernel yet, and the node's own parity gate connects
//! its input and so never visits the case.
//!
//! So the question gets one door ([`stage_window`]) and kernels that need a law
//! other than the default state it in the same expression their `eval` uses.
//!
//! **The count and the identity window are ONE answer.** A generator reports
//! both (`n(t)` and where the alive id window begins, ADR-0130); asking twice
//! would let a stage dispatch `n` elements while telling the kernel about a
//! window of a different size, which no gate would notice because both numbers
//! are separately plausible.

use crate::GpuStream;
use crate::plan::resolve_param;
use ph2d_nodegraph::gpu::{CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;

/// The stage's window: how many elements it emits, and — for a generator — the
/// identity window those elements carry.
///
/// `inputs` are the resolved input streams in port order (an unconnected port is
/// the empty stream, exactly like the CPU's `ctx.input(p)`).
pub(crate) fn stage_window(
    kernel: &GpuKernel,
    graph: &Graph,
    node: NodeId,
    manifest: &'static NodeManifest,
    inputs: &[GpuStream],
    playhead: f64,
    dt: f64,
) -> SourceWindow {
    let Some(law) = kernel.count_law else {
        // The default law: the output rides port 0 (`ColumnBinding::port`), so it
        // is exactly as long. `of_count` and not a bare number — a transformer
        // inherits its elements, so it has no spawn window of its own to report.
        return SourceWindow::of_count(inputs.first().map_or(0, |s| s.count) as usize);
    };
    let counts: Vec<u32> = inputs.iter().map(|s| s.count).collect();
    let param = |name: &str| resolve_param(graph, node, manifest, name);
    law(&CountLawCtx {
        inputs: &counts,
        param: &param,
        playhead,
        dt,
    })
}
