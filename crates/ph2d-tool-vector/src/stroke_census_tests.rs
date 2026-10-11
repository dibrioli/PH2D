//! ⭐ **O censo do TRAÇO sobre a geometria cozida** (F12 A, 11/10): toda forma do catálogo × cada
//! campo varrido na faixa dele (e a proporção da caixa), com um traço grosso. Procura as três
//! espécies das fotos do dono: (a) a junta que SALTA entre bico e chanfro ao variar um campo,
//! (b) o laço / a cúspide na geometria cozida, (c) o bico fora da caixa visual.
//! Doc: `docs/UI_New_and_Simple/vetor_nova_ui/19_o_traco_das_formas.md`.

use crate::shapes::{FieldUnit, SHAPES, ShapeDesc};
use ph2d_vec_scene::{ShapeKind, ShapeValues, cook};
use ph2d_vector::{BezPath, ParamCurve, PathEl, PathSeg, Point, Vec2};

/// Caixa de 400 px (4 mundo) e traço de 30 px: a razão das fotos do dono (~7,5 %).
const BOX: f64 = 4.0;
const PX: f64 = 0.01;
const SAMPLES: usize = 81;

/// Uma amostra: o valor do campo varrido e o que a geometria dela mostra.
#[derive(Clone, Debug, Default)]
pub(crate) struct Probe {
    /// Juntas (quinas entre segmentos) e quantas passam do limite do bico.
    pub joins: usize,
    pub over_limit: usize,
    /// Quinas com volta > 179° — a ida-e-volta (cúspide entre segmentos).
    pub reversals: usize,
    /// Cúspides DENTRO de um segmento (a tangente vira ao contrário).
    pub inner_cusps: usize,
    /// Cruzamentos LOCAIS do contorno (laços pequenos).
    pub local_loops: usize,
    /// GANCHOS: numa QUINA, a ponta de um troço vira mais de 30° dentro de meia largura do traço (a
    /// junta lê a tangente da ponta, e o traço grosso desenha o «X» da foto 1 do dono).
    pub hooks: usize,
}

/// A largura do traço do censo (mundo): 7,5 % da caixa, a das fotos.
pub(crate) const WIDTH: f64 = 0.3;

fn hooked(seg: &PathSeg, at_end: bool) -> bool {
    let (Some(t), o) = (
        tangent_end(*seg, at_end),
        if at_end { seg.end() } else { seg.start() },
    ) else {
        return false;
    };
    let mut prev = o;
    for i in 1..=256 {
        let u = i as f64 / 256.0;
        let q = seg.eval(if at_end { 1.0 - u } else { u });
        let d = if at_end { o - q } else { q - o };
        if d.hypot() >= 0.5 * WIDTH {
            return t.dot(d / d.hypot()) < 30f64.to_radians().cos();
        }
        prev = q;
    }
    let _ = prev;
    false
}

fn tangent_end(seg: PathSeg, at_end: bool) -> Option<Vec2> {
    let pts: Vec<Point> = match seg {
        PathSeg::Line(l) => vec![l.p0, l.p1],
        PathSeg::Quad(q) => vec![q.p0, q.p1, q.p2],
        PathSeg::Cubic(c) => vec![c.p0, c.p1, c.p2, c.p3],
    };
    let n = pts.len();
    let pairs: Vec<(Point, Point)> = if at_end {
        (0..n - 1).rev().map(|i| (pts[i], pts[n - 1])).collect()
    } else {
        (1..n).map(|i| (pts[0], pts[i])).collect()
    };
    pairs
        .into_iter()
        .map(|(a, b)| b - a)
        .find(|d| d.hypot() > 1e-9)
        .map(|d| d / d.hypot())
}

fn subpaths(p: &BezPath) -> Vec<(Vec<PathSeg>, bool)> {
    let mut out = Vec::new();
    let mut cur = BezPath::new();
    let flush = |cur: &mut BezPath, out: &mut Vec<(Vec<PathSeg>, bool)>| {
        if cur.elements().len() > 1 {
            let closed = matches!(cur.elements().last(), Some(PathEl::ClosePath));
            out.push((cur.segments().collect(), closed));
        }
        *cur = BezPath::new();
    };
    for el in p.elements() {
        if matches!(el, PathEl::MoveTo(_)) {
            flush(&mut cur, &mut out);
        }
        cur.push(*el);
        if matches!(el, PathEl::ClosePath) {
            flush(&mut cur, &mut out);
        }
    }
    flush(&mut cur, &mut out);
    out
}

fn seg_len(s: &PathSeg) -> f64 {
    (s.end() - s.start()).hypot().max(match s {
        PathSeg::Cubic(c) => (c.p1 - c.p0).hypot() + (c.p2 - c.p1).hypot() + (c.p3 - c.p2).hypot(),
        PathSeg::Quad(q) => (q.p1 - q.p0).hypot() + (q.p2 - q.p1).hypot(),
        PathSeg::Line(_) => 0.0,
    })
}

/// O censo de uma geometria.
pub(crate) fn probe(p: &BezPath, miter_limit: f64) -> Probe {
    let mut r = Probe::default();
    // A volta a partir da qual o bico passa do limite: 1/cos(φ/2) > L.
    let phi_limit = 2.0 * (1.0 / miter_limit).acos();
    for (segs, closed) in subpaths(p) {
        let segs: Vec<PathSeg> = segs.into_iter().filter(|s| seg_len(s) > 1e-9).collect();
        let n = segs.len();
        if n == 0 {
            continue;
        }
        let joins = if closed { n } else { n - 1 };
        for k in 0..joins {
            let (a, b) = (segs[k], segs[(k + 1) % n]);
            let (Some(t1), Some(t2)) = (tangent_end(a, true), tangent_end(b, false)) else {
                continue;
            };
            let phi = t1.dot(t2).clamp(-1.0, 1.0).acos();
            if phi < 1f64.to_radians() {
                continue;
            }
            r.joins += 1;
            if hooked(&a, true) || hooked(&b, false) {
                r.hooks += 1;
            }
            if phi > phi_limit {
                r.over_limit += 1;
            }
            if phi > 179f64.to_radians() {
                r.reversals += 1;
            }
        }
        for s in &segs {
            if let PathSeg::Cubic(_) = s {
                let mut prev: Option<Vec2> = None;
                for i in 0..64 {
                    let v = s.eval((i + 1) as f64 / 64.0) - s.eval(i as f64 / 64.0);
                    if v.hypot() < 1e-12 {
                        continue;
                    }
                    if let Some(pv) = prev
                        && pv.dot(v) < -0.5 * pv.hypot() * v.hypot()
                    {
                        r.inner_cusps += 1;
                        break;
                    }
                    prev = Some(v);
                }
            }
        }
        // Laços locais: o polígono aplanado cruza-se a menos de 24 arestas de distância.
        let mut poly: Vec<Point> = Vec::new();
        for s in &segs {
            for i in 0..16 {
                poly.push(s.eval(i as f64 / 16.0));
            }
        }
        if !closed {
            poly.push(segs[n - 1].end());
        }
        let m = poly.len();
        let edges = if closed { m } else { m - 1 };
        for i in 0..edges {
            for j in i + 2..(i + 24).min(edges) {
                if closed && (j + 1) % m == i {
                    continue;
                }
                if cross(poly[i], poly[(i + 1) % m], poly[j], poly[(j + 1) % m]) {
                    r.local_loops += 1;
                }
            }
        }
    }
    r
}

fn cross(a: Point, b: Point, c: Point, d: Point) -> bool {
    let o = |p: Point, q: Point, r: Point| (q - p).cross(r - p);
    let (d1, d2, d3, d4) = (o(a, b, c), o(a, b, d), o(c, d, a), o(c, d, b));
    d1 * d2 < -1e-18 && d3 * d4 < -1e-18
}

/// Uma linha do censo: a forma, o campo (ou a proporção da caixa), e a sequência das amostras.
pub(crate) struct Row {
    pub shape: &'static str,
    pub field: String,
    pub probes: Vec<(f64, Probe)>,
}

fn path_of(kind: ShapeKind, v: &ShapeValues, aspect: f64) -> BezPath {
    let b = [BOX, BOX * aspect];
    let p = cook(kind, [0.0, 0.0], b, v);
    ph2d_vec_render::build_bezpath(&p.cooked())
}

fn sweep(d: &ShapeDesc, i: usize) -> Vec<f64> {
    let f = &d.fields[i];
    match f.unit {
        FieldUnit::Count | FieldUnit::Choice(_) => {
            let (lo, hi) = (f.min.round() as i64, f.max.min(f.min + 24.0).round() as i64);
            (lo..=hi).map(|x| x as f64).collect()
        }
        _ => (0..SAMPLES)
            .map(|s| f.min + (f.max - f.min) * s as f64 / (SAMPLES - 1) as f64)
            .collect(),
    }
}

pub(crate) fn census(miter_limit: f64) -> Vec<Row> {
    let mut rows = Vec::new();
    for d in SHAPES {
        let base = d.kind.defaults();
        for (i, f) in d.fields.iter().enumerate() {
            let probes = sweep(d, i)
                .into_iter()
                .map(|x| {
                    let mut v = base;
                    v[i] = if f.unit == FieldUnit::Px { x * PX } else { x };
                    (x, probe(&path_of(d.kind, &v, 1.0), miter_limit))
                })
                .collect();
            rows.push(Row {
                shape: d.label,
                field: f.label.to_string(),
                probes,
            });
        }
        let probes = (0..SAMPLES)
            .map(|s| {
                let aspect = 0.25 * 16f64.powf(s as f64 / (SAMPLES - 1) as f64);
                (aspect, probe(&path_of(d.kind, &base, aspect), miter_limit))
            })
            .collect();
        rows.push(Row {
            shape: d.label,
            field: "aspect".to_string(),
            probes,
        });
    }
    rows
}

/// Quantas vezes o número de bicos cortados MUDA entre amostras vizinhas com as mesmas juntas.
pub(crate) fn jumps(r: &Row) -> usize {
    r.probes
        .windows(2)
        .filter(|w| w[0].1.joins == w[1].1.joins && w[0].1.over_limit != w[1].1.over_limit)
        .count()
}

#[test]
#[ignore = "sonda: imprime o censo (F12 A)"]
fn print_the_stroke_census() {
    let l = ph2d_vec_scene::MITER_LIMIT;
    for r in census(l) {
        let j = jumps(&r);
        let rev = r.probes.iter().filter(|p| p.1.reversals > 0).count();
        let cus = r.probes.iter().filter(|p| p.1.inner_cusps > 0).count();
        let lo = r.probes.iter().filter(|p| p.1.local_loops > 0).count();
        let hk = r.probes.iter().filter(|p| p.1.hooks > 0).count();
        if j + rev + cus + lo + hk == 0 {
            continue;
        }
        let at = |pred: &dyn Fn(&Probe) -> bool| {
            let xs: Vec<f64> = r
                .probes
                .iter()
                .filter(|p| pred(&p.1))
                .map(|p| p.0)
                .collect();
            match (xs.first(), xs.last()) {
                (Some(a), Some(b)) => format!("[{a:.3}..{b:.3}]"),
                _ => String::new(),
            }
        };
        println!(
            "{:<10} {:<12} jumps={j:<2} rev={rev:<3}{} cusps={cus:<3}{} loops={lo:<3}{} hooks={hk}{}",
            r.shape,
            r.field,
            at(&|p| p.reversals > 0),
            at(&|p| p.inner_cusps > 0),
            at(&|p| p.local_loops > 0),
            at(&|p| p.hooks > 0),
        );
    }
}

/// Os PARES de campos (grelha grossa): os defeitos que só um par de ajustes acorda.
pub(crate) fn census_pairs(miter_limit: f64, grid: usize) -> Vec<(String, f64, f64, Probe)> {
    let mut out = Vec::new();
    let axis = |d: &ShapeDesc, i: usize| -> Vec<f64> {
        let all = sweep(d, i);
        let step = (all.len() / grid).max(1);
        all.into_iter().step_by(step).collect()
    };
    for d in SHAPES {
        let base = d.kind.defaults();
        let world = |i: usize, x: f64| {
            if d.fields[i].unit == FieldUnit::Px {
                x * PX
            } else {
                x
            }
        };
        for i in 0..d.fields.len() {
            for j in i + 1..d.fields.len() {
                for x in axis(d, i) {
                    for y in axis(d, j) {
                        let mut v = base;
                        v[i] = world(i, x);
                        v[j] = world(j, y);
                        let p = probe(&path_of(d.kind, &v, 1.0), miter_limit);
                        let name =
                            format!("{} {}×{}", d.label, d.fields[i].label, d.fields[j].label);
                        out.push((name, x, y, p));
                    }
                }
            }
        }
    }
    out
}

#[test]
#[ignore = "sonda: imprime o censo dos PARES (F12 A)"]
fn print_the_pair_census() {
    let rows = census_pairs(ph2d_vec_scene::MITER_LIMIT, 10);
    let mut seen: std::collections::BTreeMap<String, (usize, usize, usize, String)> =
        std::collections::BTreeMap::new();
    for (name, x, y, p) in &rows {
        let e = seen.entry(name.clone()).or_default();
        if p.local_loops > 0 {
            e.0 += 1;
            if e.3.len() < 80 {
                e.3 += &format!("({x:.2},{y:.2}) ");
            }
        }
        if p.inner_cusps > 0 || p.hooks > 0 {
            e.1 += 1;
        }
        if p.reversals > 0 {
            e.2 += 1;
        }
    }
    for (name, (lo, cu, rev, at)) in seen {
        if lo + cu > 0 {
            println!("{name:<36} loops={lo:<4} cusps|hooks={cu:<4} rev={rev:<4} {at}");
        }
    }
}

fn pen() -> ph2d_vector::Stroke {
    ph2d_vector::Stroke::new(WIDTH)
        .with_join(ph2d_vector::Join::Miter)
        .with_miter_limit(ph2d_vec_scene::MITER_LIMIT)
}

/// ⭐ (b) **Nenhum laço em forma nenhuma**, campo a campo e na proporção da caixa; e o polígono e a
/// estrela sem gancho (as fotos 1–3 do dono). A cauda de ganchos noutras formas: doc 19 §5.
#[test]
fn no_cooked_shape_loops_and_the_polystar_has_no_hook() {
    for r in census(ph2d_vec_scene::MITER_LIMIT) {
        for (x, p) in &r.probes {
            assert_eq!(p.local_loops, 0, "{} {} = {x}: laço", r.shape, r.field);
            if matches!(r.shape, "Poly" | "Star") {
                assert_eq!(p.hooks, 0, "{} {} = {x}: gancho", r.shape, r.field);
            }
        }
    }
    for (name, x, y, p) in census_pairs(ph2d_vec_scene::MITER_LIMIT, 10) {
        if name.starts_with("Poly ") {
            assert!(p.local_loops + p.hooks == 0, "{name} ({x}, {y}): {p:?}");
        }
    }
}

/// ⭐ (a) **Nenhum bico cai no chanfro**: em toda forma × campo, cada junta além do limite tem
/// tinta na linha de corte (o corte existe e soma com o corpo — sem furo).
#[test]
fn every_sharp_join_of_every_shape_is_cut_not_bevelled() {
    let mut cuts = 0;
    for d in SHAPES {
        let base = d.kind.defaults();
        for (i, f) in d.fields.iter().enumerate() {
            for x in sweep(d, i).into_iter().step_by(4) {
                let mut v = base;
                v[i] = if f.unit == FieldUnit::Px { x * PX } else { x };
                let bp = path_of(d.kind, &v, 1.0);
                let out = ph2d_vec_render::miter_clip::stroke_outline(&bp, &pen(), 1e-4);
                for c in ph2d_vec_render::miter_clip::patches(&bp, &pen()) {
                    let [_, a2, b2, _] = c.quad;
                    let tip = Point::new((a2[0] + b2[0]) * 0.5, (a2[1] + b2[1]) * 0.5);
                    let back = Point::new(c.probe[0], c.probe[1]) - tip;
                    let inside = tip + back * (1e-3 / back.hypot().max(1e-12));
                    assert_ne!(
                        ph2d_vector::Shape::winding(&out, inside),
                        0,
                        "{} {} = {x}: o corte não tem tinta",
                        d.label,
                        f.label
                    );
                    cuts += 1;
                }
            }
        }
    }
    assert!(
        cuts > 50,
        "o censo tem de achar quinas além do limite: {cuts}"
    );
}

/// A régua do `resvg`: a 1.ª linha de píxeis com mais de metade de tinta em alguma coluna (a
/// cobertura por amostragem 16 × 16 do contorno NonZero).
fn first_inked_row(out: &BezPath) -> f64 {
    for row in 0..300 {
        for col in 130..170 {
            let mut hit = 0;
            for i in 0..16 {
                for j in 0..16 {
                    let q = Point::new(
                        f64::from(col) + (f64::from(i) + 0.5) / 16.0,
                        f64::from(row) + (f64::from(j) + 0.5) / 16.0,
                    );
                    hit += i32::from(ph2d_vector::Shape::winding(out, q) != 0);
                }
            }
            if hit > 128 {
                return f64::from(row);
            }
        }
    }
    f64::NAN
}

/// ⭐ O ORÁCULO: o topo do traço de uma quina, ângulo a ângulo, bate com o `resvg` (`miter-clip`,
/// `docs/UI_New_and_Simple/vetor_nova_ui/oraculo_traco/`), ±1 px da medida dele.
#[test]
fn the_cut_tip_matches_the_resvg_miter_clip() {
    let raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/UI_New_and_Simple/vetor_nova_ui/oraculo_traco/fixtures/miter_clip_resvg.json"
    ))
    .expect("o fixture do resvg");
    let fx: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let mut n = 0;
    for e in fx["experiencias"].as_array().unwrap() {
        let a = e["interior_deg"].as_f64().unwrap();
        let want = e["miter_clip_top_y"].as_f64().unwrap();
        let h = (a / 2.0).to_radians();
        let mut bp = BezPath::new();
        bp.move_to((150.0 - 200.0 * h.sin(), 60.0 + 200.0 * h.cos()));
        bp.line_to((150.0, 60.0));
        bp.line_to((150.0 + 200.0 * h.sin(), 60.0 + 200.0 * h.cos()));
        let pen = ph2d_vector::Stroke::new(24.0)
            .with_join(ph2d_vector::Join::Miter)
            .with_miter_limit(4.0);
        let out = ph2d_vec_render::miter_clip::stroke_outline(&bp, &pen, 1e-3);
        let top = first_inked_row(&out);
        assert!((top - want).abs() <= 1.0, "{a}°: nós {top}, resvg {want}");
        n += 1;
    }
    assert_eq!(n, 10);
}
