//! **A tangente HONESTA nas quinas** — módulo irmão de [`super`] (teto de LOC).
//!
//! A banda Bevel/Miter ([`super::band`]) mede cada junção pelas direções das RETAS vizinhas do
//! vértice, porque o Offset corre sobre a forma achatada. A tolerância do achatamento limita a
//! POSIÇÃO; numa quina onde chega uma curva, a reta vizinha erra a TANGENTE de `θ`, e a junção
//! erra `|d|·θ`. Na cúspide da cauda do balão (`SpeechRect`, `d = 0,4`) eram ~15° de cada lado:
//! um bevel de 150° no lugar do de 180°, uma cunha de `0,04` (BUGS #39). Aqui cada curva que
//! encosta numa quina é partida perto dela até a reta vizinha errar no máximo `θ`.

use kurbo::{BezPath, CubicBez, ParamCurve, PathEl, PathSeg, Point, Vec2};

/// Junto a uma quina, nenhuma curva desce abaixo deste parâmetro: com `2⁻⁴⁰` o pedaço já é do
/// tamanho do erro do `f64` sobre a forma.
const MAX_HALVINGS: u32 = 40;

/// A direção com que `seg` sai de `t = 0` (`end = false`) ou chega a `t = 1` (`end = true`),
/// saltando handles colapsados. `None` num segmento degenerado.
fn tangent(seg: PathSeg, end: bool) -> Option<Vec2> {
    let pts: Vec<Point> = match seg {
        PathSeg::Line(l) => vec![l.p0, l.p1],
        PathSeg::Quad(q) => vec![q.p0, q.p1, q.p2],
        PathSeg::Cubic(c) => vec![c.p0, c.p1, c.p2, c.p3],
    };
    let v = if end {
        let p = *pts.last()?;
        pts.iter()
            .rev()
            .skip(1)
            .map(|&q| p - q)
            .find(|v| v.hypot2() > 0.0)?
    } else {
        let p = pts[0];
        pts.iter()
            .skip(1)
            .map(|&q| q - p)
            .find(|v| v.hypot2() > 0.0)?
    };
    Some(v / v.hypot())
}

fn angle(a: Vec2, b: Vec2) -> f64 {
    a.cross(b).atan2(a.dot(b)).abs()
}

/// O maior `t = 2⁻ᵏ` (medido a partir da ponta `end`) em que a corda até à ponta erra a tangente
/// dela em no máximo `theta`.
fn honest_t(c: CubicBez, end: bool, theta: f64) -> Option<f64> {
    let tan = tangent(PathSeg::Cubic(c), end)?;
    let tip = if end { c.p3 } else { c.p0 };
    (1..=MAX_HALVINGS)
        .map(|k| 0.5f64.powi(k as i32))
        .find(|&s| {
            let q = c.eval(if end { 1.0 - s } else { s });
            let chord = if end { tip - q } else { q - tip };
            chord.hypot2() > 0.0 && angle(chord, tan) <= theta
        })
}

/// `bez` com cada curva que encosta numa QUINA (tangente que salta mais de `theta`) partida
/// junto dela — ver o doc do módulo. Retas e vértices lisos passam intactos.
pub(super) fn split_at_corners(bez: &BezPath, theta: f64) -> BezPath {
    let mut out = BezPath::new();
    for sub in subpaths(bez) {
        let n = sub.len();
        let Some(first) = sub.first() else { continue };
        out.move_to(first.start());
        for i in 0..n {
            let seg = sub[i];
            let PathSeg::Cubic(c) = (match seg {
                PathSeg::Quad(q) => PathSeg::Cubic(q.raise()),
                s => s,
            }) else {
                out.line_to(seg.end());
                continue;
            };
            let corner = |a: PathSeg, a_end: bool, b: PathSeg, b_end: bool| matches!((tangent(a, a_end), tangent(b, b_end)), (Some(u), Some(v)) if angle(u, v) > theta);
            let (prev, next) = (sub[(i + n - 1) % n], sub[(i + 1) % n]);
            let t0 = corner(prev, true, seg, false)
                .then(|| honest_t(c, false, theta))
                .flatten()
                .unwrap_or(0.0);
            let t1 = corner(seg, true, next, false)
                .then(|| honest_t(c, true, theta))
                .flatten()
                .map_or(1.0, |s| 1.0 - s);
            let mut cuts = vec![0.0, t0, t1, 1.0];
            cuts.dedup();
            for w in cuts.windows(2).filter(|w| w[1] > w[0]) {
                let p = c.subsegment(w[0]..w[1]);
                out.curve_to(p.p1, p.p2, p.p3);
            }
        }
        out.close_path();
    }
    out
}

/// Os sub-caminhos FECHADOS de `bez` como segmentos (o fecho implícito incluído). Chegam do
/// sweep, que não emite segmento de comprimento zero.
fn subpaths(bez: &BezPath) -> Vec<Vec<PathSeg>> {
    let mut all: Vec<Vec<PathSeg>> = Vec::new();
    let mut cur = BezPath::new();
    let mut flush = |cur: &mut BezPath| {
        if cur.elements().len() > 1 {
            cur.close_path();
            all.push(cur.segments().collect());
        }
        *cur = BezPath::new();
    };
    for el in bez.elements() {
        match *el {
            PathEl::MoveTo(_) => {
                flush(&mut cur);
                cur.push(*el);
            }
            PathEl::ClosePath => flush(&mut cur),
            _ => cur.push(*el),
        }
    }
    flush(&mut cur);
    all
}
