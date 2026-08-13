//! **O que UM cook produz neste quadro** — o alvo da bomba.
//!
//! Extraído do `lib.rs` no teto de LOC do HR-18, pela costura que já estava lá:
//! o `lib.rs` roda o RELÓGIO (`MotionCookPump`) e o `lower.rs` responde *com o que
//! um stream cozido se parece na tela*; este arquivo responde *para QUEM este cook
//! está cozinhando* — as duas metades do MESMO caminho, e a razão de elas serem
//! uma só está no doc abaixo.

use ph2d_nodegraph::graph::NodeId;

/// What one pump cook produces this frame — the two consumers of the SAME
/// forward/scrub tick march + `pre` feedback, kept as one path so a "fast
/// preview" boundary cook can never drift from the sink cook
/// ([[feedback_two_doors_to_the_same_question_diverge]]).
pub(crate) enum CookTarget<'a> {
    /// The render sinks, lowered into `instances` (the CPU render path).
    Sinks {
        sinks: &'a [NodeId],
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
    },
    /// Each named node's raw output stream, stored in `boundary_streams` (the
    /// CPU→GPU boundary: the lowering is the GPU's job now, so the CPU stops at
    /// the stream).
    Boundaries(&'a [NodeId]),
}

impl CookTarget<'_> {
    /// Is there work to cook? Drives the once-per-frame `pre`-feedback advance
    /// (a boundary node is always work; an empty sink list is not).
    pub(crate) fn has_work(&self) -> bool {
        match self {
            CookTarget::Sinks { sinks, .. } => !sinks.is_empty(),
            CookTarget::Boundaries(nodes) => !nodes.is_empty(),
        }
    }
}
