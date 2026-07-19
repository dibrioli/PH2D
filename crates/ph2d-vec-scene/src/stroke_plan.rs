//! **O que um traço DESENHA** — a porta única.
//!
//! Um `StrokeSpec` não é uma caneta só. Com pontas (arrowheads) o traço vira TRÊS coisas: a
//! linha **encurtada** para caber nas pontas, e uma ponta em cada extremo — cheia
//! (preenchida) ou vazada (traçada com a mesma caneta, mas **sempre sólida**: o tracejado é
//! da LINHA, a ponta é um símbolo).
//!
//! Essa receita existia **uma vez, dentro do renderer**. Isso bastava enquanto desenhar era
//! a única coisa que se fazia com um traço. Deixou de bastar quando o **Outline Stroke**
//! (`ph2d_vec_boolean::outline_stroke`) passou a converter um traço em forma: se ele
//! reescrevesse a receita, as duas responderiam *"o que este traço desenha?"* separadamente,
//! e o dia em que uma mudasse — um recuo de ponta, um `marker_scale` — a outra ficaria
//! calada e errada. O artista clicaria em Outline Stroke e receberia uma forma que **não é a
//! que estava na tela**, sem erro nenhum. [[feedback_two_doors_to_the_same_question_diverge]]
//!
//! Então a receita mora aqui e tem dois consumidores: quem **pinta** e quem **assa**.
//!
//! ⚠️ **O caso comum não paga nada.** 99% dos paths não têm ponta, e aí o plano é uma peça
//! só cujo caminho é `Cow::Borrowed` — o próprio path, sem cópia (o mesmo truque do
//! [`crate::VecPath::cooked`]). Só quem tem ponta constrói geometria.

use std::borrow::Cow;

use crate::{StrokeSpec, VecPath, stroke_head, trim_path};

/// Uma peça do que o traço desenha.
#[derive(Clone, Debug, PartialEq)]
pub enum StrokePiece<'a> {
    /// **A linha**, traçada com a caneta do estilo: largura · cap · join · dash.
    Line { path: Cow<'a, VecPath> },
    /// **Uma ponta vazada**, traçada com uma caneta SIMPLES na largura crua — sem dash
    /// (o tracejado é da linha; a ponta é um símbolo) e sem o cap/join do estilo
    /// (engrossar o risco fecharia um losango vazado).
    Symbol { path: VecPath },
    /// **Uma ponta cheia**, preenchida com a cor do traço (seta, losango, bolinha).
    Fill { path: VecPath },
}

/// O que `path` desenha sob `s`, na ordem em que desenha (a linha primeiro, as pontas por
/// cima). Vazio é possível e correto: uma linha mais curta que os recuos somados de duas
/// pontas gordas não tem linha nenhuma — só as pontas.
#[must_use]
pub fn stroke_plan<'a>(path: &'a VecPath, s: &StrokeSpec) -> Vec<StrokePiece<'a>> {
    let mut out = Vec::new();
    // Sem pontas nada é encurtado — e `Cow::Borrowed` é o que torna o caso comum gratuito.
    let line = if s.has_markers() {
        trim_path(
            path,
            s.marker_start.inset(s.marker_scale) * s.width,
            s.marker_end.inset(s.marker_scale) * s.width,
        )
        .map(Cow::Owned)
    } else {
        Some(Cow::Borrowed(path))
    };
    if let Some(path) = line {
        out.push(StrokePiece::Line { path });
    }
    for at_start in [true, false] {
        let Some((marker, geo)) = stroke_head(path, s, at_start) else {
            continue;
        };
        out.push(if marker.is_filled() {
            StrokePiece::Fill { path: geo }
        } else {
            StrokePiece::Symbol { path: geo }
        });
    }
    out
}

#[cfg(test)]
#[path = "stroke_plan_tests.rs"]
mod tests;
