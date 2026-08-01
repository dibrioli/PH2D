//! **Os espelhos de UI do TRAÇO** — `StrokeCap`/`StrokeJoin` e as duas direções de conversão
//! contra `ph2d_vec_scene::{LineCap, LineJoin}`. Irmão de [`super::params`] pelo teto de 700 LOC,
//! cortado por assunto: ali moram as faixas e os mapeamentos de slider, aqui os **tipos** que a UI
//! usa para falar do traço.
//!
//! ⚠️ **As duas direções moram juntas de propósito.** Uma conversão com dois donos é a que diverge
//! no dia em que uma variante entra — e a shell tinha metade dela num `match` solto.

/// UI-facing line cap / join — espelho de `ph2d_vec_scene::{LineCap, LineJoin}`.
///
/// ⚠️ **A justificativa antiga deste espelho era FALSA e foi corrigida em 2026-08-01**: ela dizia
/// *"the tool crate doesn't dep `ph2d-vec-scene`"*, e a tool depende dela desde que `Marker`,
/// `ShapeKind` e `StrokeAlign` passaram a viajar no snapshot como o tipo do DOCUMENTO (é o que o
/// `params_snapshot` explica: um espelho a mais só cria uma tabela para manter em dia).
///
/// Os espelhos FICAM — tirá-los é churn pelo painel inteiro sem ganho —, mas as duas direções
/// passam a morar aqui, em `From`, em vez de num `match` solto na shell. Uma conversão com dois
/// donos é a que diverge quando uma variante entra.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StrokeCap {
    #[default]
    Butt,
    Round,
    Square,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StrokeJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

impl From<ph2d_vec_scene::LineCap> for StrokeCap {
    fn from(c: ph2d_vec_scene::LineCap) -> Self {
        match c {
            ph2d_vec_scene::LineCap::Butt => Self::Butt,
            ph2d_vec_scene::LineCap::Round => Self::Round,
            ph2d_vec_scene::LineCap::Square => Self::Square,
        }
    }
}
impl From<StrokeCap> for ph2d_vec_scene::LineCap {
    fn from(c: StrokeCap) -> Self {
        match c {
            StrokeCap::Butt => Self::Butt,
            StrokeCap::Round => Self::Round,
            StrokeCap::Square => Self::Square,
        }
    }
}
impl From<ph2d_vec_scene::LineJoin> for StrokeJoin {
    fn from(j: ph2d_vec_scene::LineJoin) -> Self {
        match j {
            ph2d_vec_scene::LineJoin::Miter => Self::Miter,
            ph2d_vec_scene::LineJoin::Round => Self::Round,
            ph2d_vec_scene::LineJoin::Bevel => Self::Bevel,
        }
    }
}
impl From<StrokeJoin> for ph2d_vec_scene::LineJoin {
    fn from(j: StrokeJoin) -> Self {
        match j {
            StrokeJoin::Miter => Self::Miter,
            StrokeJoin::Round => Self::Round,
            StrokeJoin::Bevel => Self::Bevel,
        }
    }
}
