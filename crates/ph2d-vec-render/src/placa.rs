//! ⭐⭐⭐ **A FORMA PARA A PLACA** — o que o passe de formas instanciado (`ph2d-shape-gpu`, doc 121
//! do Motion) precisa de uma instância de Motion, montado pelos MESMOS passos que o desenho de hoje.
//!
//! ⚠️ **Não é uma segunda lei do desenho: é a mesma, lida noutra ordem.** O
//! [`crate::instance::draw_shape_instance_tessellated`] desenha um primitivo como *preenchimento
//! com a cor da cópia* + *traço pela porta única do traço* ([`crate::draw_stroke_with`]); aqui os
//! mesmos dois passos devolvem as GEOMETRIAS em vez de as encodar — o mesmo `PathTess`, o mesmo
//! `stroke_plan`, o mesmo `kurbo_stroke` com o mesmo tracejado ajustado. *Uma forma que o passe
//! desenhasse montada por outro caminho seria a forma de outra pessoa.*
//!
//! ⛔ **O que devolve `None` (e fica no Vello, como hoje):** um desenho com TINTA PRÓPRIA
//! (`path.fill` — o vector de documento honra as cores dele, e o passe pinta com a da cópia) e um
//! traço de PADRÃO ou de PINCEL (a tinta não é uma cor).

use ph2d_vec_scene::{StrokePiece, VecPath};
use ph2d_vector::{BezPath, Fill, Stroke};

/// As geometrias de uma forma, prontas para o passe instanciado.
pub struct FormaParaAPlaca {
    /// O preenchimento (com a cor da CÓPIA) e a regra dele.
    pub fill: Option<(BezPath, Fill)>,
    /// As linhas a traçar, cada uma com o estilo dela — a linha principal e o contorno de um
    /// marcador vazado.
    pub tracos: Vec<(BezPath, Stroke)>,
    /// Os marcadores CHEIOS de uma ponta: pintam com a cor do traço.
    pub preenchimentos_do_traco: Vec<BezPath>,
    /// A cor do traço, RGBA de `0..1` não pré-multiplicado — os mesmos bytes que o Vello recebe
    /// (`Color::from_rgba8`).
    pub cor_do_traco: [f32; 4],
}

/// **A forma para a placa**, ou `None` se o passe não a sabe desenhar como o Vello.
#[must_use]
pub fn forma_para_a_placa(path: &VecPath) -> Option<FormaParaAPlaca> {
    if path.fill.is_some() {
        return None;
    }
    let tess = crate::instance::tessellate_shape_instance(path);
    let fill_bp = tess.fill_bp.clone();
    let mut tracos = Vec::new();
    let mut preenchimentos_do_traco = Vec::new();
    let mut cor_do_traco = [0.0; 4];
    if let Some(s) = path.stroke.as_ref() {
        if s.pattern().is_some() || s.brush().is_some() {
            return None;
        }
        let c = s.color();
        cor_do_traco = [
            f32::from(c.r) / 255.0,
            f32::from(c.g) / 255.0,
            f32::from(c.b) / 255.0,
            f32::from(c.a) / 255.0,
        ];
        // O desenho que o traço segue — o `stroke_own` ou, sem ele, o do preenchimento (a mesma
        // escolha do `draw_one_stroke`).
        let linha = tess.stroke_bp.as_ref().or(tess.fill_bp.as_ref())?;
        for peca in ph2d_vec_scene::stroke_plan(path, s) {
            match peca {
                StrokePiece::Line { path: line } => match line {
                    std::borrow::Cow::Borrowed(_) => {
                        tracos.push((linha.clone(), crate::kurbo_stroke(s, tess.dash)));
                    }
                    std::borrow::Cow::Owned(p) => {
                        let dash = ph2d_vec_scene::dash_for(&p, s);
                        tracos.push((crate::build_bezpath(&p), crate::kurbo_stroke(s, dash)));
                    }
                },
                StrokePiece::Symbol { path: geo } => {
                    tracos.push((crate::build_bezpath(&geo), Stroke::new(s.width)));
                }
                StrokePiece::Fill { path: geo } => {
                    preenchimentos_do_traco.push(crate::build_bezpath(&geo));
                }
            }
        }
    }
    Some(FormaParaAPlaca {
        fill: fill_bp.map(|bp| (bp, crate::fill_rule(path))),
        tracos,
        preenchimentos_do_traco,
        cor_do_traco,
    })
}

#[cfg(test)]
#[path = "placa_tests.rs"]
mod tests;
