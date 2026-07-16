//! ADR-0114 W6 — **o pick de TRAÇO** do Edit Mode, módulo-irmão de `flip_select` (cap de
//! LOC do HR-18). É o gêmeo do `flip_select_points` (que faz o pick de PONTO): dado um
//! ponto em coords da ARTE, qual traço está sob ele.
//!
//! Uma porta só: o `flip_select` **re-exporta** o [`stroke_at`], então quem seleciona
//! continua chamando `flip_select::stroke_at` e não existe um 2º hit-test de traço.

use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke};
use ph2d_vec_scene::Xform;

/// Raio mínimo de pick, em px de TELA. Uma linha de 1 px tem de ser clicável sem que o
/// usuário mire no pixel — é a mesma folga que o gizmo usa para pegar a arte.
const MIN_PICK_PX: f32 = 5.0; // LITERAL-PX-OK: folga de pick, nao metrica de design

/// O traço sob o ponto `local`, se houver — **o de cima primeiro** (a ordem de z é a
/// ordem da lista, fundo → topo; então a varredura é de trás para a frente).
///
/// `px_to_world` converte px de tela em unidades de MUNDO e `w2l` desce de mundo para o
/// espaço LOCAL do objeto — a mesma conversão que o balde faz (`flip_fill::boundaries`):
/// a espessura do traço é absoluta em px de tela (brush absoluto, Enio 2026-07-11)
/// enquanto os pontos são unidades de documento, e é por isso que o raio de pick
/// **acompanha o zoom**: aproximar a câmera não pode exigir mira mais fina.
#[must_use]
pub(crate) fn stroke_at(
    drawing: &FlipDrawing,
    local: Vec2,
    px_to_world: f32,
    w2l: &Xform,
) -> Option<usize> {
    // px de TELA → unidade LOCAL (o `mean_scale` do objeto é o último degrau).
    let px_to_local = px_to_world * w2l.mean_scale() as f32;
    drawing
        .strokes
        .iter()
        .enumerate()
        .rev() // o de CIMA primeiro
        .find(|(_, s)| hits(s, local, px_to_local))
        .map(|(i, _)| i)
}

/// O ponto `local` pega este traço? (Tinta OU preenchimento.)
fn hits(s: &FlipStroke, p: Vec2, px_to_local: f32) -> bool {
    // (a) O INTERIOR do preenchimento — inclusive o de uma região (`hide_stroke`), que
    //     não tem linha nenhuma para se aproximar. Os buracos não pegam: clicar no furo
    //     de um "O" é clicar no que está ATRÁS dele.
    if s.fill.is_some()
        && crate::flip_fill::ring_contains(s.positions(), p)
        && !s
            .holes
            .iter()
            .any(|h| crate::flip_fill::ring_contains(h, p))
    {
        return true;
    }
    // (b) A TINTA: a meia-espessura do traço (px de tela → local, como o balde), com um
    //     piso para que uma linha fina não exija mira de pixel. Uma região não tem tinta.
    if s.hide_stroke {
        return false;
    }
    let pos = s.positions();
    let widths = s.widths();
    let reach = |i: usize| -> f32 {
        let half = widths.get(i).copied().unwrap_or(0.0) * 0.5;
        (half.max(MIN_PICK_PX) * px_to_local).max(f32::EPSILON)
    };
    if pos.len() == 1 {
        let d = p - pos[0];
        return d.x * d.x + d.y * d.y <= reach(0) * reach(0);
    }
    // `segments()` — a porta única (inclui a COSTURA de um traço fechado, que é o que o
    // render desenha). Iterar `positions().windows(2)` aqui perdia a última aresta: um
    // triângulo tinha 3 linhas na tela e 2 clicáveis.
    s.segments().any(|(i, a, b)| {
        let r = reach(i);
        seg_dist2(p, a, b) <= r * r
    })
}

/// Distância² de `p` ao segmento `a`→`b`.
fn seg_dist2(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len2 = ab.x * ab.x + ab.y * ab.y;
    let t = if len2 > 0.0 {
        (((p - a).x * ab.x + (p - a).y * ab.y) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let c = a + ab * t;
    let d = p - c;
    d.x * d.x + d.y * d.y
}
