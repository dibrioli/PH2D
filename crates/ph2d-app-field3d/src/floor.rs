//! ⭐⭐⭐ **O CHÃO DO MODO RENDER, ANCORADO** — a metade da `W4` que é da app (`docs/Render3d/07`).
//!
//! A lei do chão (onde o raio o toca, quanto escurece, onde fica o ponto mais baixo) vive na
//! [`ph2d_field_render::ground`]. O que esta porta decide é **quando** a altura é lida.
//!
//! # ⭐ Lida UMA vez, e fica
//!
//! A régua da `W4` é *um objecto a `0`, `1` e `10 cm` do chão dá três sombras diferentes*. Um chão
//! que seguisse o ponto mais baixo a cada quadro subiria com a peça, e levantá-la nunca a separaria
//! da sombra. ⇒ a altura é lida quando o modo Render **liga** e fica até ele voltar a ligar — o que
//! o KeyShot chama *pousar no chão*: um gesto, não uma lei contínua.
//!
//! ⚠️ **Uma peça que desça abaixo do chão não o empurra**: o chão é invisível e não tapa nada, logo
//! a parte de baixo dela continua a ver-se inteira, e voltar a ligar o Render pousa-a de novo.
//!
//! ⚠️ **A âncora é CACHE, não vista** ([`crate::view::View::of`]): o módulo que re-arma lê-a de novo.

use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_render::Ground;

/// ⭐ **O chão deste pedido** — `None` fora do modo Render, e num documento sem geometria.
///
/// A primeira chamada com o Render ligado lê o ponto mais baixo do documento **real** (nunca o
/// contorno engrossado do quadro de movimento, que desce e sobe com a mão) e guarda-o em `ancora`.
pub(crate) fn anchored(
    ancora: &mut Option<f32>,
    shading: crate::shading::Shading,
    doc: &FieldDoc,
    reg: &Registry,
) -> Option<Ground> {
    if shading != crate::shading::Shading::Render {
        return None;
    }
    if ancora.is_none() {
        *ancora = ph2d_field_render::lowest_point(doc, reg);
    }
    ancora.map(|height| Ground { height })
}

#[cfg(test)]
#[path = "floor_tests.rs"]
mod tests;
