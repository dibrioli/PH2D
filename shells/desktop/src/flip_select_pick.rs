//! ADR-0114 W6 — **o pick de TRAÇO** do Edit Mode, módulo-irmão de `flip_select` (cap de
//! LOC do HR-18). É o gêmeo do `flip_select_points` (que faz o pick de PONTO): dado um
//! ponto em coords da ARTE, qual traço está sob ele.
//!
//! Uma porta só: o `flip_select` **re-exporta** o [`stroke_at`], então quem seleciona
//! continua chamando `flip_select::stroke_at` e não existe um 2º hit-test de traço.
//!
//! **§4.B:** o modo Segment precisa de MAIS do que "qual traço" — precisa de *onde* no
//! traço. Em vez de um 2º hit-test (que é exatamente como o BUGS #18 nasceu: quatro donos
//! da mesma pergunta, três errados), quem responde é o mesmo [`hit_on`], e o `stroke_at`
//! passou a ser `hit_at(..).map(si)`. As duas perguntas não podem divergir porque são a
//! mesma função.

use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke};
use ph2d_vec_scene::Xform;

/// Raio mínimo de pick, em px de TELA. Uma linha de 1 px tem de ser clicável sem que o
/// usuário mire no pixel — é a mesma folga que o gizmo usa para pegar a arte.
const MIN_PICK_PX: f32 = 5.0; // LITERAL-PX-OK: folga de pick, nao metrica de design

/// **O que o cursor pegou** num traço (§4.B). O domínio Stroke só quer saber *se* pegou;
/// o Segment quer saber ONDE.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Where {
    /// A **TINTA**, no segmento que começa no ponto `i`, à fração `t ∈ [0,1]` dele (a
    /// convenção de `FlipStroke::segments()` — `i` é o ponto de PARTIDA, e a costura de um
    /// traço fechado é o segmento `n-1`).
    Ink { i: usize, t: f32 },
    /// O traço inteiro, **sem um segmento onde mirar**: o PREENCHIMENTO de uma região (que
    /// não tem tinta — `hide_stroke`), ou um traço de um ponto só. Não existe "um pedaço
    /// do preenchimento": o `fill` é do anel INTEIRO (a regra #4 do módulo — a forma pinta
    /// a si mesma), então quem aponta o miolo apontou a forma, não uma aresta dela.
    Whole,
}

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
    hit_at(drawing, local, px_to_world, w2l).map(|(si, _)| si)
}

/// O mesmo pick, **com o lugar**: `(traço, onde)`. É o que o modo Segment consome.
#[must_use]
pub(crate) fn hit_at(
    drawing: &FlipDrawing,
    local: Vec2,
    px_to_world: f32,
    w2l: &Xform,
) -> Option<(usize, Where)> {
    // px de TELA → unidade LOCAL (o `mean_scale` do objeto é o último degrau).
    let px_to_local = px_to_world * w2l.mean_scale() as f32;
    drawing
        .strokes
        .iter()
        .enumerate()
        .rev() // o de CIMA primeiro
        .find_map(|(i, s)| hit_on(s, local, px_to_local).map(|w| (i, w)))
}

/// O ponto `local` pega este traço, e onde? (Tinta OU preenchimento.)
fn hit_on(s: &FlipStroke, p: Vec2, px_to_local: f32) -> Option<Where> {
    // (a) A TINTA: a meia-espessura do traço (px de tela → local, como o balde), com um
    //     piso para que uma linha fina não exija mira de pixel. Uma região não tem tinta.
    //     Entre os segmentos ao alcance vence o MAIS PRÓXIMO — perto de uma quina duas
    //     arestas alcançam o cursor, e a que ele está apontando é a de menor distância.
    if !s.hide_stroke {
        let pos = s.positions();
        let widths = s.widths();
        let reach = |i: usize| -> f32 {
            let half = widths.get(i).copied().unwrap_or(0.0) * 0.5;
            (half.max(MIN_PICK_PX) * px_to_local).max(f32::EPSILON)
        };
        if pos.len() == 1 {
            let d = p - pos[0];
            if d.x * d.x + d.y * d.y <= reach(0) * reach(0) {
                return Some(Where::Whole); // um ponto não tem segmento onde mirar
            }
        }
        // `segments()` — a porta única (inclui a COSTURA de um traço fechado, que é o que o
        // render desenha). Iterar `positions().windows(2)` aqui perdia a última aresta: um
        // triângulo tinha 3 linhas na tela e 2 clicáveis.
        let best = s
            .segments()
            .filter_map(|(i, a, b)| {
                let (d2, t) = seg_dist2(p, a, b);
                let r = reach(i);
                (d2 <= r * r).then_some((d2, i, t))
            })
            .min_by(|x, y| x.0.total_cmp(&y.0));
        if let Some((_, i, t)) = best {
            return Some(Where::Ink { i, t });
        }
    }
    // (b) O INTERIOR do preenchimento — inclusive o de uma região (`hide_stroke`), que
    //     não tem linha nenhuma para se aproximar. Os buracos não pegam: clicar no furo
    //     de um "O" é clicar no que está ATRÁS dele.
    //
    //     A ordem inverteu em relação ao W6 (a tinta é testada ANTES do fill) e o
    //     `stroke_at` não sente: ele é um OU. Quem sente é o Segment — clicar na BORDA de
    //     uma forma preenchida tem de dar a aresta, não a forma.
    if s.fill.is_some()
        && crate::flip_fill::ring_contains(s.positions(), p)
        && !s
            .holes
            .iter()
            .any(|h| crate::flip_fill::ring_contains(h, p))
    {
        return Some(Where::Whole);
    }
    None
}

/// Distância² de `p` ao segmento `a`→`b`, **e a fração `t`** do pé da perpendicular (o
/// `t` é o que diz de que lado de um corte o clique caiu).
fn seg_dist2(p: Vec2, a: Vec2, b: Vec2) -> (f32, f32) {
    let ab = b - a;
    let len2 = ab.x * ab.x + ab.y * ab.y;
    let t = if len2 > 0.0 {
        (((p - a).x * ab.x + (p - a).y * ab.y) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let c = a + ab * t;
    let d = p - c;
    (d.x * d.x + d.y * d.y, t)
}
