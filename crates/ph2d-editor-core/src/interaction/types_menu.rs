//! **Que menu cada coisa abre** — os tipos que o dispatch usa para escolher a TABELA.
//!
//! Split de `types.rs` sob o teto de 700 LOC, e uma unidade por direito próprio: um
//! `TimelineHitKind` diz *o que está sob o cursor*, e isto diz *que menu isso merece*.
//! São perguntas diferentes, e a segunda é a que cresce quando uma família de track
//! ganha uma ação que as outras não têm.

/// **Que família de track esta row é**, e portanto que menu o botão direito abre.
///
/// Vive aqui e não no `ph2d-timeline` porque é uma pergunta de INTERAÇÃO: o dispatch
/// precisa dela para escolher a tabela, e ele não conhece `PropKind` (nem deve).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackMenuKind {
    /// Rotation, Scale, Opacity, Time Remap — só as ações comuns.
    Plain,
    /// `TranslationX`/`Y`: o modo de eixos SEPARADOS, que pode virar trajetória.
    Axis,
    /// `Position`: a trajetória, que pode virar eixos separados — e que tem
    /// auto-orient.
    Path,
}
