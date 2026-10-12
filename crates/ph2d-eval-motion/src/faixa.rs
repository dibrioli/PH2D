//! A faixa de um sink nas listas da bomba (irmão do `lib.rs`).

use ph2d_nodegraph::graph::NodeId;

/// ⭐ A faixa de um sink nas duas listas da bomba (ver [`crate::MotionCookPump::faixas`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaixaDoSink {
    /// O nó `motion.output`.
    pub sink: NodeId,
    /// As cópias de sprite dele.
    pub instancias: std::ops::Range<usize>,
    /// As cópias vectoriais dele.
    pub vetores: std::ops::Range<usize>,
    /// As cópias vectoriais dele que o LOD do app passou a ladrilhos, no fim de `instancias` da
    /// lista (vazio até o LOD correr).
    pub ladrilhos: std::ops::Range<usize>,
}
