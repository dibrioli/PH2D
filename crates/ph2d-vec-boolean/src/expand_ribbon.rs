//! **A FITA do Power Stroke** — módulo irmão de [`crate::expand`] (teto de LOC).
//!
//! O traço de largura VARIÁVEL não é uma pilha de discos: é uma fita de dois trilhos a
//! `±w(s)/2` da linha de centro, fechada por tampas nas pontas. O porquê está no doc-comment do
//! [`power_stroke`] — foi o "traço rugoso" que o smoke reprovou (2026-07-20) que o decidiu.
//!
//! Aqui mora só o que MOLDA a fita; quem a regulariza (o sweep, a [`super::Region`]) e quem
//! limpa as lascas continua no pai, porque essas duas respostas são compartilhadas com o
//! Offset — duas cópias divergiriam.

use kurbo::{BezPath, PathEl, Point, Vec2};
use linesweeper::FillRule as LsFillRule;
use ph2d_vec_scene::{LineCap, StrokeSpec, VecPath, WidthStops};

use super::{MIN_TOL, Region, drop_slivers, ink_style, tolerance};
use crate::{Closing, to_bez_with};

/// **Power Stroke** — o traço com largura VARIÁVEL, assado em forma preenchida.
///
/// # A largura variável é uma FITA de dois trilhos, não uma pilha de discos
///
/// A primeira versão fatiava o arco e unia um disco por fatia. Convergia, mas cada disco tem
/// **ponta redonda**, e a união de discos de larguras vizinhas deixa um FESTÃO na borda — o
/// "traço rugoso" que o smoke reprovou (`2026-07-20`). A borda de um traço de largura
/// variável não é uma sucessão de tampas: é uma curva contínua a `w(s)/2` da linha de centro.
///
/// Então o motor é o clássico do Inkscape/Illustrator: achata a linha de centro num polígono
/// fino e, em cada ponto, desloca por `±w(s)/2` na NORMAL — dois **trilhos**, ligados por
/// tampas nas pontas, formando UM contorno. A borda passa a seguir `w(s)` de forma lisa; não
/// há festão porque não há tampa entre amostras. As cúspides e auto-cruzamentos (onde a
/// curvatura aperta mais que a largura) somem no sweep, como sempre — o [`Region::of`]
/// regulariza a fita UMA vez, e é isso que dispensa o offsetter analítico que a pesquisa do
/// módulo diz que ninguém tem.
///
/// É também mais barato: um polígono e um sweep, no lugar de 64 traçados-e-sweeps.
///
/// Devolve vazio sem traço, com perfil UNIFORME (aí o comando é o [`outline_stroke`], e ter
/// dois botões para a mesma saída seria pior que ter um), ou se o sweep falhar.
#[must_use]
pub fn power_stroke(path: &VecPath, profile: &WidthStops) -> Vec<VecPath> {
    let Some(s) = path.stroke.filter(|_| !profile.is_uniform()) else {
        return Vec::new();
    };
    let cooked = path.cooked();
    let mut ink = BezPath::new();
    for c in 0..cooked.contour_count() {
        let Some((verts, closed)) = cooked.contour(c) else {
            continue;
        };
        let piece = VecPath {
            verts: verts.to_vec(),
            closed,
            ..VecPath::default()
        };
        ribbon_into(
            &mut ink,
            &to_bez_with(&piece, Closing::AsDrawn),
            &s,
            profile,
            closed,
        );
    }
    match Region::of(&ink, LsFillRule::NonZero).filter(|r| !r.is_empty()) {
        Some(acc) => drop_slivers(acc.into_paths(&ink_style(&s))),
        None => Vec::new(),
    }
}

/// Amostras mínimas ao longo do arco. A geometria já foi capturada pelo achatamento; isto é a
/// densidade em que o PERFIL de largura é lido — e ele varia mesmo onde a linha é RETA (o
/// achatamento não subdivide uma reta, então uma linha reta voltava com dois pontos só e o
/// meio grosso do perfil nunca era amostrado). 128 é folgado para o `smoothstep`, que é liso.
const RIBBON_SAMPLES: usize = 128;

/// Achata `bez` (um contorno) num polígono fino, o **densifica por arco** para amostrar o
/// perfil de largura, e devolve os pontos + o `t` de ARCO normalizado de cada um (`0` no
/// começo, `1` no fim). `None` se degenerar.
///
/// Por arco e não por parâmetro de Bézier: é a unidade em que o [`WidthStops`] mora, a mesma
/// do Zig Zag e do Blend — duas formas que se veem iguais têm de se comportar igual.
fn flatten_arc(bez: &BezPath, closed: bool) -> Option<(Vec<Point>, Vec<f64>)> {
    let tol = tolerance(bez);
    let mut raw: Vec<Point> = Vec::new();
    kurbo::flatten(bez.elements().iter().copied(), tol, |el| match el {
        PathEl::MoveTo(p) if raw.is_empty() => raw.push(p),
        PathEl::LineTo(p) => raw.push(p),
        _ => {}
    });
    // Amostras coincidentes (o achatamento repete um ponto numa tangente afiada) quebram a
    // normal por diferença — funde. No fechado, o último ponto volta ao primeiro.
    raw.dedup_by(|a, b| (*a - *b).hypot() <= MIN_TOL);
    if closed && raw.len() >= 2 && (raw[0] - raw[raw.len() - 1]).hypot() <= MIN_TOL {
        raw.pop();
    }
    if raw.len() < 2 {
        return None;
    }
    // Comprimento total do arco (o fechado inclui o segmento de volta ao início).
    let n = raw.len();
    let seg_count = if closed { n } else { n - 1 };
    let total: f64 = (0..seg_count)
        .map(|i| (raw[(i + 1) % n] - raw[i]).hypot())
        .sum();
    if total <= MIN_TOL {
        return None;
    }
    // Densifica: nenhum vão maior que `step`. Preserva os pontos do achatamento (a geometria) e
    // insere amostras nos trechos longos (a variação de largura).
    #[allow(clippy::cast_precision_loss)]
    let step = total / RIBBON_SAMPLES as f64;
    let mut pts = Vec::with_capacity(RIBBON_SAMPLES + n);
    let mut arc = Vec::with_capacity(RIBBON_SAMPLES + n);
    let mut acc = 0.0;
    for i in 0..seg_count {
        let (a, b) = (raw[i], raw[(i + 1) % n]);
        let seg = (b - a).hypot();
        pts.push(a);
        arc.push(acc / total);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let subs = (seg / step).ceil() as usize;
        for k in 1..subs {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f64 / subs as f64;
            pts.push(a.lerp(b, f));
            arc.push((acc + seg * f) / total);
        }
        acc += seg;
    }
    if !closed {
        pts.push(raw[n - 1]);
        arc.push(1.0);
    }
    Some((pts, arc))
}

/// A normal unitária (perpendicular à tangente) no ponto `i` de `pts`. Cíclica quando
/// `closed`, forward/backward nas pontas quando aberto.
fn normal_at(pts: &[Point], i: usize, closed: bool) -> Vec2 {
    let n = pts.len();
    let tangent = if closed {
        pts[(i + 1) % n] - pts[(i + n - 1) % n]
    } else if i == 0 {
        pts[1] - pts[0]
    } else if i == n - 1 {
        pts[n - 1] - pts[n - 2]
    } else {
        pts[i + 1] - pts[i - 1]
    };
    let len = tangent.hypot();
    if len <= MIN_TOL {
        return Vec2::new(0.0, 0.0);
    }
    let t = tangent / len;
    Vec2::new(-t.y, t.x)
}

/// **A fita** de `bez`: acumula em `ink` o contorno cru (pré-sweep) dos dois trilhos a
/// `±w(s)/2` da linha de centro, **um quadrilátero por segmento**.
///
/// Por quadrilátero, e não um polígono só: onde a curvatura aperta mais que a largura os
/// trilhos se CRUZAM, e um polígono único auto-intersectante confunde o winding do `NonZero`
/// (medido: a senoide adversária saía em 3 peças). Os quads vizinhos partilham a aresta
/// `[esquerda_{i+1}, direita_{i+1}]` — estão COLADOS por ela, então a união é conexa mesmo
/// quando um quad se torce. A borda externa continua sendo os trilhos (lisa, sem festão), e um
/// contorno FECHADO vira um anel de graça (os quads ladrilham a fita; o miolo fica sem
/// cobertura = o furo).
fn ribbon_into(
    ink: &mut BezPath,
    bez: &BezPath,
    s: &StrokeSpec,
    profile: &WidthStops,
    closed: bool,
) {
    let Some((pts, arc)) = flatten_arc(bez, closed) else {
        return;
    };
    let half = |i: usize| 0.5 * s.width * profile.at(arc[i]).max(0.0);
    let rail = |i: usize, sign: f64| pts[i] + normal_at(&pts, i, closed) * (sign * half(i));
    let n = pts.len();
    let seg_count = if closed { n } else { n - 1 };
    for i in 0..seg_count {
        let j = (i + 1) % n;
        push_loop(
            ink,
            [rail(i, 1.0), rail(j, 1.0), rail(j, -1.0), rail(i, -1.0)].into_iter(),
        );
    }
    if !closed {
        cap_loop(
            ink,
            pts[n - 1],
            half(n - 1),
            normal_at(&pts, n - 1, false),
            s.cap,
            true,
        );
        cap_loop(
            ink,
            pts[0],
            half(0),
            normal_at(&pts, 0, false),
            s.cap,
            false,
        );
    }
}

/// Emite `pts` como um subpath fechado (poligonal) em `ink`.
fn push_loop(ink: &mut BezPath, mut pts: impl Iterator<Item = Point>) {
    if let Some(first) = pts.next() {
        ink.move_to(first);
        for p in pts {
            ink.line_to(p);
        }
        ink.close_path();
    }
}

/// A **tampa** da ponta, como um contorno fechado próprio a unir com o último quad: liga o
/// trilho de um lado ao do outro em torno do centro `c` com meia-largura `h`. `forward` = a
/// tampa do FIM (bojo para +tangente). `normal` é a normal do centro naquele extremo.
fn cap_loop(ink: &mut BezPath, c: Point, h: f64, normal: Vec2, cap: LineCap, forward: bool) {
    if h <= MIN_TOL {
        return; // ponta afilada num bico — nada a arredondar
    }
    // A direção do bojo é ±tangente = ∓ a perpendicular da normal (a normal aponta para o
    // trilho esquerdo; girá-la −90° dá a tangente para a frente).
    let dir = Vec2::new(normal.y, -normal.x) * if forward { 1.0 } else { -1.0 };
    let mut poly: Vec<Point> = Vec::with_capacity(16);
    // O diâmetro (a aresta que o quad já toca) + o miolo do arco/quadrado à frente.
    poly.push(c + normal * h);
    match cap {
        // Butt: o quad já tem a aresta reta na ponta — não há tampa a somar.
        LineCap::Butt => return,
        // Square: dois cantos projetados `h` à frente.
        LineCap::Square => {
            poly.push(c + normal * h + dir * h);
            poly.push(c - normal * h + dir * h);
        }
        // Round: semicírculo de raio `h`, amostrado. A borda macia da caligrafia.
        LineCap::Round => {
            const SEGS: usize = 12;
            for k in 1..SEGS {
                #[allow(clippy::cast_precision_loss)]
                let theta = std::f64::consts::PI * (k as f64) / (SEGS as f64);
                poly.push(c + (normal * theta.cos() + dir * theta.sin()) * h);
            }
        }
    }
    poly.push(c - normal * h);
    push_loop(ink, poly.into_iter());
}
