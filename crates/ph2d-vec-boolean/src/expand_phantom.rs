//! **O caçador de FANTASMAS** do Offset Round — módulo irmão de [`super`] (teto de LOC).
//!
//! Só o Round passa por aqui: o Bevel e o Miter montam a banda por peças de área positiva
//! ([`super::band`]), e uma banda sem laço invertido não deixa refugo a caçar.

use kurbo::{BezPath, PathEl, Point};
use linesweeper::FillRule as LsFillRule;

use super::{Region, end_of, flat_lines};

/// Remove os contornos **FANTASMAS** — o refugo do stroke com caneta maior que o laço.
///
/// Com `2|d|` rivalizando o tamanho do próprio laço, o contorno interno da banda
/// ([`penned_outline`]) degenera num laço auto-cruzado cujo winding vira ruído, e o refugo
/// atravessa o sweep: **encolher** (`base ∖ banda`) devolvia uma ILHA no meio de uma forma
/// que devia ANIQUILAR, e **crescer** (`base ∪ banda`) ganhava um FURO fantasma no miolo.
/// Medido no donut do smoke (2026-07-20): `d=−4` com Round/Bevel devolvia 12 verts de área
/// 2,52 onde o Miter (corretamente) devolvia nada — a forma *crescia de volta* conforme o
/// artista encolhia mais; e `d=+4` inflava a área de 19,8 (a resposta exata, que o Miter dá)
/// para 30,7. É o mecanismo do report *"muda para round mas não muda para Miter e Bevel"*:
/// no `d` extremo cada join produzia um refugo DIFERENTE (nada · fantasma · o MESMO
/// fantasma), então uns cliques "mudavam" e outros não.
///
/// O discriminador é **DISTÂNCIA ao laço fonte**: toda borda legítima de um offset ROUND contém
/// pontos a `|d|` do laço (aresta offsetada e arco, os dois a exatamente `|d|`). O teste é o
/// **MÁXIMO** do contorno, nunca o mínimo. Um fantasma vive no MIOLO: todo ponto dele fica aquém.
///
/// ⛔ **Isto já valeu para o Bevel, e a promessa era falsa** (prompt 49, W2): *"as pontas da corda
/// estão a `|d|`, o máximo a absolve"* só vale enquanto as cordas vizinhas não se cortam. Na
/// estrela do gate `the_offset_sweep_matches_the_table` cortam-se para dentro com `d ≥ 0,44`
/// (sobra o pentágono das cinco cordas dos vales) e para fora com `d ≥ 1,64` — a borda inteira é
/// MEIO de corda, abaixo de `|d|`, e a resposta certa era apagada. O Bevel e o Miter montam hoje a
/// banda por peças ([`super::band`]) e não passam por aqui.
///
/// Caminho comum = **zero custo**: nada descartado devolve a região INTACTA (sem re-sweep);
/// o re-sweep só roda no caso degenerado que hoje devolve geometria errada.
pub(super) fn drop_phantoms(region: Region, loop_bez: &BezPath, d: f64) -> Option<Region> {
    let segs = flat_segments(loop_bez);
    let floor = d.abs() * 0.99;
    let is_real = |contour: &BezPath| -> bool {
        let mut max_d2 = 0.0_f64;
        for el in flat_lines(contour).elements() {
            if let Some(p) = end_of(el) {
                max_d2 = max_d2.max(dist2_to_segs(p, &segs));
                if max_d2 >= floor * floor {
                    return true;
                }
            }
        }
        false
    };
    let mut kept = BezPath::new();
    let mut dropped = false;
    for group in &region.groups {
        let Some((outer, holes)) = group.split_first() else {
            continue;
        };
        if !is_real(outer) {
            dropped = true; // o grupo inteiro é refugo — os furos dele morrem junto
            continue;
        }
        kept.extend(outer.iter());
        for hole in holes {
            if is_real(hole) {
                kept.extend(hole.iter());
            } else {
                dropped = true;
            }
        }
    }
    if !dropped {
        return Some(region);
    }
    // Contornos sobreviventes já saíram orientados do sweep — NonZero os recompõe.
    Region::of(&kept, LsFillRule::NonZero)
}

/// O laço achatado como segmentos de reta — a régua do [`drop_phantoms`].
fn flat_segments(bez: &BezPath) -> Vec<kurbo::Line> {
    let flat = flat_lines(bez);
    let mut segs = Vec::new();
    let (mut start, mut prev) = (Point::ZERO, Point::ZERO);
    for el in flat.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                start = p;
                prev = p;
            }
            PathEl::LineTo(p) => {
                segs.push(kurbo::Line::new(prev, p));
                prev = p;
            }
            PathEl::ClosePath => {
                segs.push(kurbo::Line::new(prev, start));
                prev = start;
            }
            // `flat_lines` só emite MoveTo/LineTo/ClosePath.
            _ => {}
        }
    }
    segs
}

/// Distância AO QUADRADO de `p` ao segmento mais próximo (quadrado: o comparador do
/// [`drop_phantoms`] só precisa ordenar, e a raiz por-amostra seria transcendental à toa).
fn dist2_to_segs(p: Point, segs: &[kurbo::Line]) -> f64 {
    let mut best = f64::INFINITY;
    for s in segs {
        let v = s.p1 - s.p0;
        let len2 = v.hypot2();
        let t = if len2 <= 0.0 {
            0.0
        } else {
            ((p - s.p0).dot(v) / len2).clamp(0.0, 1.0)
        };
        let q = s.p0 + v * t;
        best = best.min((p - q).hypot2());
    }
    best
}
