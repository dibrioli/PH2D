//! **A BANDA do Offset por PEÇAS** — módulo irmão de [`super`] (teto de LOC).
//!
//! A banda de um laço é o traço da fronteira com largura `2d`: um retângulo por aresta e, em cada
//! vértice, a peça da junção no lado de FORA da curva (o triângulo do Bevel, o losango do Miter —
//! que cai em triângulo além do `miter_limit`). É a definição do SVG, a que o `resvg` pinta.
//!
//! # ⛔ Por que não o contorno do stroker da kurbo
//!
//! O stroker devolve UM traçado por lado. Onde o traçado se dobra (aresta mais curta que `d`,
//! quina reentrante, duas paredes a menos de `2d`), os laços invertidos dele CANCELAM parte dos
//! retângulos sob `NonZero` — o winding do contorno cru dá `0` dentro de um retângulo. Medido
//! (prompt 49, W2): a estrela do gate saía a `0,6 %` da verdade em `d = 1,64` e a `27 %` em
//! `d = 3,9` (e o caçador de fantasmas acabava a apagá-la); no catálogo de 47 formas, a seta a
//! `10,5 %`, o balão a `9,4 %`, o raio a `9,6 %` em `d = 2`. Aqui cada peça entra **orientada com
//! área positiva**, então o `NonZero` do sweep É a união — e sem laço invertido não há fantasma.
//!
//! # O lado que não muda a resposta só entra se cruzar
//!
//! O resultado é `base ∪ banda` (crescer) ou `base ∖ banda` (encolher), então a metade da banda
//! que fica INTEIRA dentro da base (ao crescer) ou inteira fora dela (ao encolher) não muda a
//! resposta — e é ela que mais se sobrepõe às vizinhas e encarece o sweep. Uma peça desse lado só
//! entra se alguma aresta da base CRUZAR a borda dela ou um vértice da base cair dentro
//! ([`crosses_out`], conservador: na dúvida entra). O lado útil vai em CORRIDAS de polígono
//! simples ([`push_band`]). A resposta é a da banda inteira (gate
//! `the_side_filter_never_changes_the_answer`, catálogo inteiro). Custo medido em release no
//! catálogo de 47 formas, Bevel e Miter, `d ∈ ±{0,05 … 2}` na caixa 2×2, intercalado no mesmo
//! processo: **1,0× o stroker no total** (uma peça por aresta e por vértice custava 2,3×); as
//! formas lisas e convexas ficam mais baratas (elipse 0,6×), as de muitos vales mais caras
//! (nuvem 1,4×, lua 1,75×, faixa 2,3× — quase só em `d` da ordem da forma).
//!
//! O Round continua no stroker: a junção é um arco, e lá a estrela bate a verdade em 100/100.
//!
//! Origem: o estudo `docs/Vector Module/Estudos/ESTUDO_vectorcraft_comparado_2026-10-11.md` §3 e o
//! prompt 49 (W2). Nenhum código portado.

use kurbo::{BezPath, PathEl, Point, Vec2};
use ph2d_vec_scene::LineJoin;

/// A banda de `flat` (a fronteira da base: polilinhas fechadas, só retas) com meia-largura
/// `|d|`, para `base ∪ banda` (`d > 0`) ou `base ∖ banda` (`d < 0`), como peças de área positiva.
pub(super) fn pieces(flat: &BezPath, d: f64, join: LineJoin, miter_limit: f64) -> BezPath {
    pieces_with(flat, d, join, miter_limit, false)
}

/// [`pieces`] com o filtro de lado desligável (`keep_all`) — a banda INTEIRA é o controle do
/// teste que prova que o filtro não muda a resposta.
fn pieces_with(
    flat: &BezPath,
    d: f64,
    join: LineJoin,
    miter_limit: f64,
    keep_all: bool,
) -> BezPath {
    let polys = polylines(flat);
    let edges: Vec<(Point, Point)> = polys
        .iter()
        .flat_map(|p| (0..p.len()).map(move |i| (p[i], p[(i + 1) % p.len()])))
        .collect();
    let mut out = BezPath::new();
    for poly in &polys {
        push_band(&mut out, poly, &edges, d, (join, miter_limit), keep_all);
    }
    out
}

/// As polilinhas fechadas de `flat`, sem vértice repetido.
fn polylines(flat: &BezPath) -> Vec<Vec<Point>> {
    let mut all: Vec<Vec<Point>> = Vec::new();
    let mut cur: Vec<Point> = Vec::new();
    let mut flush = |cur: &mut Vec<Point>| {
        while cur.len() > 1 && cur.first() == cur.last() {
            cur.pop();
        }
        if cur.len() >= 2 {
            all.push(std::mem::take(cur));
        }
        cur.clear();
    };
    for el in flat.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                flush(&mut cur);
                cur.push(p);
            }
            PathEl::LineTo(p) => {
                if cur.last() != Some(&p) {
                    cur.push(p);
                }
            }
            PathEl::ClosePath => flush(&mut cur),
            // O chamador achata antes; uma curva aqui seria um defeito dele.
            PathEl::QuadTo(..) | PathEl::CurveTo(..) => {
                debug_assert!(false, "a banda por peças só come retas");
            }
        }
    }
    flush(&mut cur);
    all
}

fn left(u: Vec2) -> Vec2 {
    Vec2::new(-u.y, u.x)
}

fn unit(a: Point, b: Point) -> Vec2 {
    (b - a) / (b - a).hypot()
}

/// Para que lado da fronteira o laço tem o EXTERIOR: `+1` se à esquerda de quem o percorre.
/// Pelo sinal da área — o sweep orienta furo ao contrário do contorno de fora, e a regra serve
/// aos dois.
fn exterior_is_left(poly: &[Point]) -> f64 {
    let n = poly.len();
    let twice: f64 = (0..n)
        .map(|i| poly[i].to_vec2().cross(poly[(i + 1) % n].to_vec2()))
        .sum();
    -twice.signum()
}

/// A junção no vértice `v` entre as direções `u0 → u1`: o lado de FORA da curva (em
/// «esquerdas») e a peça, `[v, p0, (bico), p1]`. `None` se as arestas forem colineares.
fn join_piece(
    v: Point,
    u0: Vec2,
    u1: Vec2,
    h: f64,
    (join, miter_limit): (LineJoin, f64),
) -> Option<(f64, Vec<Point>)> {
    let cross = u0.cross(u1);
    if cross == 0.0 {
        return None;
    }
    // O lado de FORA da curva — independente do sentido do laço.
    let s = -cross.signum();
    let (n0, n1) = (left(u0) * s, left(u1) * s);
    let (p0, p1) = (v + n0 * h, v + n1 * h);
    let bis = n0 + n1;
    // cos(φ/2) = |n0 + n1| / 2; o bico do miter fica a h / cos(φ/2) do vértice.
    let half_cos = bis.hypot() / 2.0;
    let miters = join == LineJoin::Miter && half_cos > 0.0 && 1.0 / half_cos <= miter_limit;
    let piece = if miters {
        vec![v, p0, v + bis / bis.hypot() * (h / half_cos), p1]
    } else {
        vec![v, p0, p1]
    };
    Some((s, piece))
}

/// Curva acumulada máxima de uma CORRIDA (ver [`push_band`]): com até 90° a tira de uma corrida
/// convexa não tem como voltar sobre si.
const RUN_TURN: f64 = std::f64::consts::FRAC_PI_2;

/// A banda de UM laço.
///
/// O lado útil (`useful`) vai em **corridas**: arestas seguidas cujos vértices dobram PARA o
/// lado útil (a junção cai nele e a tira só se abre) viram UM polígono simples — a cadeia da
/// base e, de volta, a cadeia deslocada com as junções. É o que o stroker faz onde está certo,
/// sem o lado que se dobra; e é o que poupa ao sweep as arestas coincidentes entre retângulo e
/// triângulo vizinhos (medido: 3–6× mais lento com uma peça por aresta e por vértice). A corrida
/// parte num vértice que dobra para o OUTRO lado (lá as tiras vizinhas cruzam-se, e o sweep
/// resolve) ou quando a curva acumulada passaria de [`RUN_TURN`]. O lado inútil vai peça a peça,
/// cada uma só se cruzar para fora ([`crosses_out`]).
fn push_band(
    out: &mut BezPath,
    poly: &[Point],
    edges: &[(Point, Point)],
    d: f64,
    style: (LineJoin, f64),
    keep_all: bool,
) {
    let n = poly.len();
    let h = d.abs();
    // O lado que MUDA a resposta: o exterior ao crescer, o interior ao encolher (em «esquerdas»).
    let useful = exterior_is_left(poly) * d.signum();
    let dir: Vec<Vec2> = (0..n).map(|i| unit(poly[i], poly[(i + 1) % n])).collect();
    // O lado inútil, peça a peça.
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        let nn = left(dir[i]) * (h * useful);
        let other = [a, b, b - nn, a - nn];
        if keep_all || crosses_out(&other, &[a, b], edges) {
            push_positive(out, &other);
        }
        if keep_all {
            push_positive(out, &[a, b, b + nn, a + nn]);
        }
    }
    // A junção de cada vértice: no lado útil entra na corrida (`bends`); no outro, peça a peça.
    let mut joins: Vec<Option<Vec<Point>>> = vec![None; n];
    let mut bends = vec![false; n];
    if n >= 3 {
        for i in 0..n {
            let Some((s, piece)) = join_piece(poly[i], dir[(i + n - 1) % n], dir[i], h, style)
            else {
                bends[i] = dir[(i + n - 1) % n].dot(dir[i]) > 0.0; // colinear segue a corrida
                continue;
            };
            if keep_all || (s != useful && crosses_out(&piece, &[poly[i]], edges)) {
                push_positive(out, &piece);
            } else if s == useful {
                bends[i] = true;
                joins[i] = Some(piece);
            }
        }
    }
    if keep_all {
        return;
    }
    // As corridas do lado útil: começam num vértice que NÃO dobra para ele (ou no 0, se todos
    // dobram — um laço convexo inteiro), e partem também pela curva acumulada.
    let start = (0..n).find(|&i| !bends[i]).unwrap_or(0);
    let mut k = 0;
    while k < n {
        let mut chain: Vec<Point> = Vec::new();
        let mut base: Vec<Point> = vec![poly[(start + k) % n]];
        let mut turn = 0.0;
        loop {
            let e = (start + k) % n;
            let (a, b) = (poly[e], poly[(e + 1) % n]);
            let nn = left(dir[e]) * (h * useful);
            chain.extend([a + nn, b + nn]);
            base.push(b);
            k += 1;
            let v = (e + 1) % n;
            let phi = dir[e].dot(dir[v]).clamp(-1.0, 1.0).acos();
            if k >= n || !bends[v] || turn + phi > RUN_TURN {
                // O vértice que parte a corrida leva a sua junção como peça própria.
                if let Some(piece) = joins[v].take() {
                    push_positive(out, &piece);
                }
                break;
            }
            turn += phi;
            if let Some(piece) = joins[v].take() {
                // [v, p0, (bico), p1] — p0/p1 já estão na cadeia; só o bico entra.
                if piece.len() == 4 {
                    chain.push(piece[2]);
                }
            }
        }
        chain.reverse();
        base.extend(chain);
        push_positive(out, &base);
    }
}

/// A peça `q` (convexa, ancorada na fronteira em `anchor`) sai do lado onde nasceu? — alguma
/// aresta da base cruza a borda dela fora da âncora, ou um vértice da base cai dentro dela.
/// Conservador: encostar conta como cruzar (na dúvida a peça entra, e entrar nunca erra).
fn crosses_out(q: &[Point], anchor: &[Point], edges: &[(Point, Point)]) -> bool {
    let (lo, hi) = bounds(q);
    let scale = (hi - lo).hypot().max(f64::MIN_POSITIVE);
    let eps = scale * 1e-9;
    let at_anchor = |p: Point| anchor.iter().any(|&a| (p - a).hypot() <= eps);
    let qa = signed2(q);
    for &(p, r) in edges {
        if p.x.max(r.x) < lo.x - eps
            || p.x.min(r.x) > hi.x + eps
            || p.y.max(r.y) < lo.y - eps
            || p.y.min(r.y) > hi.y + eps
        {
            continue;
        }
        if !at_anchor(p) && strictly_inside(q, qa, p, eps) {
            return true;
        }
        for k in 0..q.len() {
            let (s0, s1) = (q[k], q[(k + 1) % q.len()]);
            // A aresta de âncora da peça (a própria aresta da base) não é borda a cruzar.
            if anchor.len() == 2 && at_anchor(s0) && at_anchor(s1) {
                continue;
            }
            if let Some(x) = hit(s0, s1, p, r, eps)
                && !at_anchor(x)
            {
                return true;
            }
        }
    }
    false
}

fn bounds(q: &[Point]) -> (Point, Point) {
    q.iter().fold((q[0], q[0]), |(lo, hi), p| {
        (
            Point::new(lo.x.min(p.x), lo.y.min(p.y)),
            Point::new(hi.x.max(p.x), hi.y.max(p.y)),
        )
    })
}

fn signed2(q: &[Point]) -> f64 {
    (0..q.len())
        .map(|i| q[i].to_vec2().cross(q[(i + 1) % q.len()].to_vec2()))
        .sum()
}

/// `p` dentro do polígono CONVEXO `q`, a mais de `eps` de toda borda.
fn strictly_inside(q: &[Point], qa: f64, p: Point, eps: f64) -> bool {
    (0..q.len()).all(|k| {
        let (a, b) = (q[k], q[(k + 1) % q.len()]);
        let e = b - a;
        let len = e.hypot();
        len > 0.0 && e.cross(p - a) * qa.signum() / len > eps
    })
}

/// O ponto onde os segmentos `[a,b]` e `[c,e]` se tocam (com folga `eps`), se tocam.
fn hit(a: Point, b: Point, c: Point, e: Point, eps: f64) -> Option<Point> {
    let (r, s) = (b - a, e - c);
    let den = r.cross(s);
    let (rl, sl) = (r.hypot(), s.hypot());
    if rl == 0.0 || sl == 0.0 {
        return None;
    }
    if den.abs() <= f64::EPSILON * rl * sl {
        // Paralelos: tocam se colineares e sobrepostos — devolve um ponto comum.
        if (c - a).cross(r).abs() / rl > eps {
            return None;
        }
        let t0 = (c - a).dot(r) / (rl * rl);
        let t1 = (e - a).dot(r) / (rl * rl);
        let (lo, hi) = (t0.min(t1).max(0.0), t0.max(t1).min(1.0));
        let tol = eps / rl;
        return (lo <= hi + tol).then(|| a + r * ((lo + hi) / 2.0).clamp(0.0, 1.0));
    }
    let t = (c - a).cross(s) / den;
    let u = (c - a).cross(r) / den;
    let (tt, tu) = (eps / rl, eps / sl);
    ((-tt..=1.0 + tt).contains(&t) && (-tu..=1.0 + tu).contains(&u)).then(|| a + r * t)
}

/// Emite `pts` como um laço fechado de área POSITIVA (e nada, se a área for nula).
fn push_positive(out: &mut BezPath, pts: &[Point]) {
    let area = signed2(pts);
    if area == 0.0 {
        return;
    }
    let mut it: Box<dyn Iterator<Item = &Point>> = if area > 0.0 {
        Box::new(pts.iter())
    } else {
        Box::new(pts.iter().rev())
    };
    if let Some(&first) = it.next() {
        out.move_to(first);
        for &p in it {
            out.line_to(p);
        }
        out.close_path();
    }
}

#[cfg(test)]
#[path = "expand_band_tests.rs"]
mod tests;
