//! **QUICKSHAPE** — a lei pura: um traço de mão → a forma perfeita que ele queria ser (F7, D10).
//!
//! O gatilho (segurar no fim do traço) mora em [`crate::quickshape_hold`]; aqui só a pergunta
//! *«que forma é esta?»*, sobre as amostras cruas do [`crate::Pencil`].
//!
//! A lei foi MEDIDA, não escrita: o Xournal++ 1.3.8 correu por script sobre 65 traços nossos
//! ([`oraculo_quickshape/00_resultados.md`](../../../docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/00_resultados.md)),
//! e onde o produto do dono (o QuickShape do Procreate) manda mais — elipses, polígonos de 5+
//! lados, polilinhas abertas, cantos arredondados — manda ele. Cada constante abaixo traz a FAIXA
//! em que todas as fixtures continuam certas (varrida uma a uma) e de onde vem a fronteira.
//!
//! O caminho, numa frase: reamostra por comprimento, parte em lados por Ramer-Douglas-Peucker,
//! funde viragens pequenas, absorve lados curtos nas quinas, e se TODOS os lados são rectos é um
//! polígono (linha, polilinha, triângulo, rectângulo, …); senão, se fecha uma volta, é a elipse dos
//! momentos de área da região desenhada (exacta para uma elipse perfeita); senão não é forma.

use ph2d_vec_scene::{VecPath, VecVertex};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

type P2 = [f64; 2];

/// Tamanho mínimo (diagonal da caixa do traço), em **px de tela**. O oráculo recusa abaixo de 40
/// unidades de página = **94 px** à escala do corpus (`× 0,42491`). Faixa medida: a linha de
/// 60 px e o rectângulo 70×50 (diagonal 86 px) são crus, o círculo de raio 40 (diagonal 113 px) é
/// reconhecido ⇒ `(86, 113]`. É também o lado mínimo: um lado mais curto é QUINA (o rectângulo
/// 500×60 do oráculo é cru porque os lados de 60 px não são lados).
pub const MIN_SIZE_PX: f64 = 94.0;
/// Amostras da reamostragem por comprimento (o desenho não depende da cadência do rato).
const RESAMPLE: usize = 256;
/// Tolerância do RDP, em fracção da diagonal. Faixa válida `0,01..0,10`.
const RDP_OF_DIAG: f64 = 0.04;
/// Viragem abaixo da qual dois lados são UM (graus). Faixa `5..41`; o octógono vira 45°.
const MERGE_TURN_DEG: f64 = 25.0;
/// Flecha máxima de um lado RECTO, em fracção do comprimento dele. Faixa `0,04..0,075`: o
/// oráculo endireita uma flecha de 3,3 % (`linha_arco_leve`); um lado de círculo dá ≥ 7,7 %.
const BOW_MAX: f64 = 0.055;
/// A flecha mede-se sem os 10 % de cada ponta (a quina arredondada da mão). Faixa `0,10..0,15`.
const BOW_TRIM: f64 = 0.10;
/// O ajuste da recta de cada lado ignora 15 % de cada ponta, pela mesma razão.
const FIT_TRIM: f64 = 0.15;
/// Folga de fecho, em fracção do comprimento. Faixa `0,05..0,07`: o oráculo fecha 4,4 % (círculo
/// aberto 40 px) e recusa 7,6 % (rectângulo aberto 90 px).
const CLOSE_OF_LENGTH: f64 = 0.06;
/// A ponta procura a outra nos 10 % iniciais/finais do traço (o traço que PASSA do início).
const CLOSE_WINDOW: f64 = 0.10;
/// Desvio máximo do ângulo de quina a 90° para ser rectângulo (graus). Oráculo: o paralelogramo
/// de 6° vira rectângulo, o de 10° não ⇒ `(6, 10)`. Fora disto é quadrilátero como desenhado.
const RIGHT_TOL_DEG: f64 = 8.0;
/// Endireitar aos eixos (graus). Oráculo: linha e rectângulo rodados 4° saem alinhados, 6° não.
const AXIS_SNAP_DEG: f64 = 5.0;
/// Resíduo radial rms máximo de uma elipse (fracção do raio). Faixa `0,03..0,10`.
const ELLIPSE_RMS_MAX: f64 = 0.04;
/// Uma elipse com eixos nesta razão é um CÍRCULO. Faixa `1,05..1,50` (a elipse 3:2 fica elipse).
const ROUND_RATIO: f64 = 1.10;
/// A viragem total de uma elipse é uma volta ± isto (graus). Faixa `40..180`.
const TURN_TOL_DEG: f64 = 90.0;
/// O degrau do `regular` (o 2.º dedo do Procreate = `Shift`): ângulos em múltiplos de 15°.
pub const REGULAR_STEP_DEG: f64 = 15.0;

/// A forma reconhecida, em MUNDO.
#[derive(Clone, Debug, PartialEq)]
pub enum QuickShape {
    /// Uma recta de `a` (onde o traço começou) a `b`.
    Line { a: P2, b: P2 },
    /// Cantos em ordem de desenho; `closed` = polígono, senão polilinha aberta.
    Polygon { corners: Vec<P2>, closed: bool },
    /// `rot` em radianos, do eixo x para o semi-eixo `rx`.
    Ellipse {
        centre: P2,
        rx: f64,
        ry: f64,
        rot: f64,
    },
}

/// **A lei.** `px_to_world` converte as fronteiras de tela; `regular` = quadrado, círculo,
/// triângulo equilátero, polígono regular, linha a 15°.
#[must_use]
pub fn recognise(samples: &[P2], px_to_world: f64, regular: bool) -> Option<QuickShape> {
    if samples.len() < 3 {
        return None;
    }
    let min = MIN_SIZE_PX * px_to_world;
    let (lo, hi) = samples
        .iter()
        .fold(([f64::MAX; 2], [f64::MIN; 2]), |(l, h), p| {
            (
                [l[0].min(p[0]), l[1].min(p[1])],
                [h[0].max(p[0]), h[1].max(p[1])],
            )
        });
    let diag = dist(lo, hi);
    let length: f64 = samples.windows(2).map(|w| dist(w[0], w[1])).sum();
    if diag < min || length <= 0.0 {
        return None;
    }
    let closed = gap(samples, length) <= CLOSE_OF_LENGTH * length;
    let r = resample(samples, RESAMPLE);
    let mut zones: Vec<Zone> = rdp(&r, RDP_OF_DIAG * diag)
        .into_iter()
        .map(|i| (i, i))
        .collect();
    merge_small_turns(&r, &mut zones);
    absorb_short_sides(&r, &mut zones, min);
    polygonal(samples, &r, &zones, min, closed, regular)
        .or_else(|| closed.then(|| ellipse(samples, &r, regular)).flatten())
}

impl QuickShape {
    /// O ponto que fica parado quando a mão, sem soltar, escala e roda a forma: o INÍCIO de uma
    /// linha ou polilinha, o centro de uma forma fechada.
    #[must_use]
    pub fn pivot(&self) -> P2 {
        match self {
            Self::Line { a, .. } => *a,
            Self::Polygon {
                corners,
                closed: false,
            } => corners.first().copied().unwrap_or_default(),
            Self::Polygon { corners, .. } => centroid(corners),
            Self::Ellipse { centre, .. } => *centre,
        }
    }

    /// A semelhança à volta de `pivot`: escala `scale`, roda `rot` radianos.
    #[must_use]
    pub fn transformed(&self, pivot: P2, scale: f64, rot: f64) -> Self {
        let m = |p: P2| add(pivot, mul(rotate(sub(p, pivot), rot), scale));
        match self {
            Self::Line { a, b } => Self::Line { a: m(*a), b: m(*b) },
            Self::Polygon { corners, closed } => Self::Polygon {
                corners: corners.iter().map(|&p| m(p)).collect(),
                closed: *closed,
            },
            Self::Ellipse {
                centre,
                rx,
                ry,
                rot: r0,
            } => Self::Ellipse {
                centre: m(*centre),
                rx: rx * scale,
                ry: ry * scale,
                rot: r0 + rot,
            },
        }
    }

    /// O caminho com nós editáveis (quinas; a elipse com as quatro alças canónicas), sem estilo.
    #[must_use]
    pub fn to_path(&self) -> VecPath {
        match self {
            Self::Line { a, b } => corners_path(&[*a, *b], false),
            Self::Polygon { corners, closed } => corners_path(corners, *closed),
            Self::Ellipse {
                centre,
                rx,
                ry,
                rot,
            } => {
                let mut p = ph2d_vec_scene::ellipse(*centre, *rx, *ry);
                let turn = |q: P2| add(*centre, rotate(sub(q, *centre), *rot));
                for v in &mut p.verts {
                    v.anchor = turn(v.anchor);
                    v.in_handle = turn(v.in_handle);
                    v.out_handle = turn(v.out_handle);
                }
                p
            }
        }
    }
}

fn corners_path(corners: &[P2], closed: bool) -> VecPath {
    VecPath {
        verts: corners.iter().map(|&p| VecVertex::corner(p)).collect(),
        closed,
        ..VecPath::default()
    }
}

// ── o caminho poligonal ─────────────────────────────────────────────────────────────────────────

/// Uma QUINA, como a faixa de `r` que ela ocupa: um ponto do RDP, ou um lado curto absorvido (o
/// chanfro, a curva de uma quina arredondada, o gancho na ponta). Os lados correm ENTRE faixas — sem
/// isto metade do chanfro entrava em cada lado vizinho e lia-se como flecha.
type Zone = (usize, usize);

/// Uma recta ajustada: um ponto dela e a direcção unitária.
#[derive(Clone, Copy)]
struct Fit {
    c: P2,
    d: P2,
}

fn polygonal(
    samples: &[P2],
    r: &[P2],
    zones: &[Zone],
    min: f64,
    closed: bool,
    regular: bool,
) -> Option<QuickShape> {
    // Uma quina maior que o tamanho mínimo já não é quina: as faixas encadeadas de um S escondiam
    // as curvas dele e liam-se como uma linha.
    if zones.iter().any(|&(a, b)| dist(r[a], r[b]) > min) {
        return None;
    }
    let sides: Vec<(usize, usize)> = zones.windows(2).map(|w| (w[0].1, w[1].0)).collect();
    if sides.is_empty()
        || sides
            .iter()
            .any(|&(a, b)| b <= a || bow(&r[a..=b]) > BOW_MAX)
    {
        return None;
    }
    let mut fits: Vec<Fit> = sides
        .iter()
        .map(|&(a, b)| trimmed_fit(&r[a..=b]))
        .collect::<Option<_>>()?;
    // Fechado a começar a meio de um lado (ou a passar do início): o último lado e o primeiro são
    // o MESMO lado.
    if closed
        && fits.len() > 3
        && turn(fits[fits.len() - 1].d, fits[0].d) < MERGE_TURN_DEG.to_radians()
    {
        let (la, lb) = sides[sides.len() - 1];
        let (fa, fb) = sides[0];
        let mut q = r[la..=lb].to_vec();
        q.extend_from_slice(&r[fa..=fb]);
        fits[0] = trimmed_fit(&q)?;
        fits.pop();
    }
    match (closed, fits.len()) {
        (false, 1) => Some(line(samples, &r[sides[0].0..=sides[0].1], regular)),
        (false, _) => Some(polyline(samples, &mut fits, regular)),
        (true, k) if k >= 3 => Some(closed_polygon(&fits, regular)),
        _ => None,
    }
}

/// A recta ajusta-se ao CORPO (`body`, sem os ganchos das pontas, que são quinas absorvidas); as
/// pontas são as projecções extremas de TODAS as amostras (o oráculo estende a linha até à ponta do
/// gancho).
fn line(samples: &[P2], body: &[P2], regular: bool) -> QuickShape {
    let Some(Fit { c, d }) = tls(body) else {
        return QuickShape::Line {
            a: samples[0],
            b: samples[samples.len() - 1],
        };
    };
    let mut d = if regular {
        unit(snap(angle(d), REGULAR_STEP_DEG.to_radians()))
    } else {
        axis_snap(d)
    };
    if dot(d, sub(samples[samples.len() - 1], samples[0])) < 0.0 {
        d = mul(d, -1.0);
    }
    let (t0, t1) = samples.iter().fold((f64::MAX, f64::MIN), |(lo, hi), &p| {
        let t = dot(sub(p, c), d);
        (lo.min(t), hi.max(t))
    });
    QuickShape::Line {
        a: add(c, mul(d, t0)),
        b: add(c, mul(d, t1)),
    }
}

fn polyline(samples: &[P2], fits: &mut [Fit], regular: bool) -> QuickShape {
    if regular {
        for f in fits.iter_mut() {
            f.d = unit(snap(angle(f.d), REGULAR_STEP_DEG.to_radians()));
        }
    }
    let k = fits.len();
    let mut corners = vec![project(samples[0], fits[0])];
    corners.extend((1..k).map(|j| intersect(fits[j - 1], fits[j])));
    corners.push(project(samples[samples.len() - 1], fits[k - 1]));
    QuickShape::Polygon {
        corners,
        closed: false,
    }
}

fn closed_polygon(fits: &[Fit], regular: bool) -> QuickShape {
    let k = fits.len();
    let corners: Vec<P2> = (0..k)
        .map(|j| intersect(fits[(j + k - 1) % k], fits[j]))
        .collect();
    let corners = if regular {
        regularise(&corners)
    } else if k == 4
        && (0..4).all(|j| {
            (turn(fits[(j + 3) % 4].d, fits[j].d) - FRAC_PI_2).abs() < RIGHT_TOL_DEG.to_radians()
        })
    {
        square_up(fits)
    } else {
        corners
    };
    QuickShape::Polygon {
        corners,
        closed: true,
    }
}

/// Os quatro lados passam a dois pares ortogonais, na rotação média do traço (média circular de
/// `4θ`, que não distingue os quatro lados), cada um pelo centróide do seu lado.
fn square_up(fits: &[Fit]) -> Vec<P2> {
    let (s, c) = fits.iter().fold((0.0, 0.0), |(s, c), f| {
        let a = 4.0 * angle(f.d);
        (s + a.sin(), c + a.cos())
    });
    let m = angle(axis_snap(unit(f64::atan2(s, c) / 4.0)));
    let sq: Vec<Fit> = fits
        .iter()
        .map(|f| Fit {
            c: f.c,
            d: unit(m + snap(angle(f.d) - m, FRAC_PI_2)),
        })
        .collect();
    (0..4).map(|j| intersect(sq[(j + 3) % 4], sq[j])).collect()
}

/// O polígono regular de `k` lados mais perto dos cantos: mesmo centro, raio médio, a fase média,
/// o sentido de desenho; e endireitado aos eixos como o rectângulo.
fn regularise(corners: &[P2]) -> Vec<P2> {
    let k = corners.len();
    let c = centroid(corners);
    let radius = corners.iter().map(|&p| dist(p, c)).sum::<f64>() / k as f64;
    let area: f64 = (0..k)
        .map(|i| cross(corners[i], corners[(i + 1) % k]))
        .sum();
    let step = TAU / k as f64 * if area < 0.0 { -1.0 } else { 1.0 };
    let (s, co) = corners
        .iter()
        .enumerate()
        .fold((0.0, 0.0), |(s, co), (i, &p)| {
            let a = angle(sub(p, c)) - i as f64 * step;
            (s + a.sin(), co + a.cos())
        });
    let mut phase = f64::atan2(s, co);
    let at = |phase: f64, i: usize| add(c, mul(unit(phase + i as f64 * step), radius));
    let side = angle(sub(at(phase, 1), at(phase, 0)));
    phase += angle(axis_snap(unit(side))) - side;
    (0..k).map(|i| at(phase, i)).collect()
}

// ── a elipse ────────────────────────────────────────────────────────────────────────────────────

fn ellipse(samples: &[P2], r: &[P2], regular: bool) -> Option<QuickShape> {
    let total: f64 = (1..r.len() - 1)
        .map(|i| wrap_pi(angle(sub(r[i + 1], r[i])) - angle(sub(r[i], r[i - 1]))))
        .sum();
    if (total.abs() - TAU).abs() > TURN_TOL_DEG.to_radians() {
        return None;
    }
    let (centre, a, b, rot) = area_moments(samples)?;
    let rms = (samples
        .iter()
        .map(|&p| {
            let q = rotate(sub(p, centre), -rot);
            ((q[0] / a).hypot(q[1] / b) - 1.0).powi(2)
        })
        .sum::<f64>()
        / samples.len() as f64)
        .sqrt();
    if rms > ELLIPSE_RMS_MAX {
        return None;
    }
    if regular || a / b <= ROUND_RATIO {
        let radius = samples.iter().map(|&p| dist(p, centre)).sum::<f64>() / samples.len() as f64;
        return Some(QuickShape::Ellipse {
            centre,
            rx: radius,
            ry: radius,
            rot: 0.0,
        });
    }
    Some(QuickShape::Ellipse {
        centre,
        rx: a,
        ry: b,
        rot,
    })
}

/// A elipse com os mesmos momentos de área que a região que o traço fecha (Green sobre o
/// polígono das amostras): uma elipse cheia de semi-eixos `a ≥ b` tem covariância `diag(a², b²)/4`.
fn area_moments(p: &[P2]) -> Option<(P2, f64, f64, f64)> {
    let n = p.len();
    let (mut a2, mut cx, mut cy, mut ixx, mut iyy, mut ixy) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..n {
        let ([x0, y0], [x1, y1]) = (p[i], p[(i + 1) % n]);
        let k = x0 * y1 - x1 * y0;
        a2 += k;
        cx += (x0 + x1) * k;
        cy += (y0 + y1) * k;
        ixx += (x0 * x0 + x0 * x1 + x1 * x1) * k;
        iyy += (y0 * y0 + y0 * y1 + y1 * y1) * k;
        ixy += (x0 * y1 + 2.0 * x0 * y0 + 2.0 * x1 * y1 + x1 * y0) * k;
    }
    let area = a2 / 2.0;
    if area.abs() <= f64::MIN_POSITIVE {
        return None;
    }
    let c = [cx / (6.0 * area), cy / (6.0 * area)];
    let sxx = ixx / (12.0 * area) - c[0] * c[0];
    let syy = iyy / (12.0 * area) - c[1] * c[1];
    let sxy = ixy / (24.0 * area) - c[0] * c[1];
    let (mid, half) = ((sxx + syy) / 2.0, ((sxx - syy) / 2.0).hypot(sxy));
    let (l1, l2) = (mid + half, mid - half);
    (l2 > 0.0).then(|| {
        (
            c,
            2.0 * l1.sqrt(),
            2.0 * l2.sqrt(),
            0.5 * f64::atan2(2.0 * sxy, sxx - syy),
        )
    })
}

// ── o traço partido em lados ────────────────────────────────────────────────────────────────────

/// A menor distância entre uma ponta e a janela inicial/final da outra.
fn gap(p: &[P2], length: f64) -> f64 {
    let mut cum = 0.0;
    let mut head = vec![p[0]];
    let mut tail = Vec::new();
    for w in p.windows(2) {
        cum += dist(w[0], w[1]);
        if cum <= CLOSE_WINDOW * length {
            head.push(w[1]);
        }
        if cum >= (1.0 - CLOSE_WINDOW) * length {
            tail.push(w[1]);
        }
    }
    let last = p[p.len() - 1];
    let a = head.iter().map(|&h| dist(h, last)).fold(f64::MAX, f64::min);
    let b = tail.iter().map(|&t| dist(t, p[0])).fold(f64::MAX, f64::min);
    a.min(b)
}

fn resample(p: &[P2], n: usize) -> Vec<P2> {
    let mut cum = vec![0.0];
    for w in p.windows(2) {
        cum.push(cum[cum.len() - 1] + dist(w[0], w[1]));
    }
    let total = cum[cum.len() - 1];
    let mut out = Vec::with_capacity(n);
    let mut j = 0;
    for i in 0..n {
        let t = total * i as f64 / (n - 1) as f64;
        while j + 2 < cum.len() && cum[j + 1] < t {
            j += 1;
        }
        let seg = cum[j + 1] - cum[j];
        let u = if seg > 0.0 {
            ((t - cum[j]) / seg).clamp(0.0, 1.0)
        } else {
            0.0
        };
        out.push(add(p[j], mul(sub(p[j + 1], p[j]), u)));
    }
    out
}

/// Ramer-Douglas-Peucker, devolvendo os ÍNDICES que ficam (iterativo, pilha explícita).
fn rdp(r: &[P2], tol: f64) -> Vec<usize> {
    let mut keep = vec![false; r.len()];
    keep[0] = true;
    keep[r.len() - 1] = true;
    let mut stack = vec![(0, r.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let worst = ((a + 1)..b)
            .map(|i| (seg_dist(r[i], r[a], r[b]), i))
            .fold((0.0, a), |m, x| if x.0 > m.0 { x } else { m });
        if worst.0 > tol {
            keep[worst.1] = true;
            stack.push((a, worst.1));
            stack.push((worst.1, b));
        }
    }
    (0..r.len()).filter(|&i| keep[i]).collect()
}

/// Funde, menor primeiro, as quinas que viram menos que [`MERGE_TURN_DEG`].
fn merge_small_turns(r: &[P2], z: &mut Vec<Zone>) {
    while z.len() > 2 {
        // Com SENTIDO (segmentos, não rectas): um bico de 160° não é uma viragem de 20°.
        let (t, j) = (1..z.len() - 1)
            .map(|j| {
                let a = sub(r[z[j].0], r[z[j - 1].1]);
                let b = sub(r[z[j + 1].0], r[z[j].1]);
                (wrap_pi(angle(b) - angle(a)).abs(), j)
            })
            .fold((f64::MAX, 0), |m, x| if x.0 < m.0 { x } else { m });
        if t >= MERGE_TURN_DEG.to_radians() {
            return;
        }
        z.remove(j);
    }
}

/// Um lado mais curto que `min` é parte de uma quina (ou um gancho na ponta): as duas faixas que o
/// fecham passam a UMA, que o contém.
fn absorb_short_sides(r: &[P2], z: &mut Vec<Zone>, min: f64) {
    while z.len() > 2 {
        let (len, j) = (0..z.len() - 1)
            .map(|j| (dist(r[z[j].1], r[z[j + 1].0]), j))
            .fold((f64::MAX, 0), |m, x| if x.0 < m.0 { x } else { m });
        if len >= min {
            return;
        }
        z[j].1 = z[j + 1].1;
        z.remove(j + 1);
        merge_small_turns(r, z);
    }
}

/// A flecha de um lado: o termo quadrático do resíduo à recta, em fracção do comprimento.
fn bow(q: &[P2]) -> f64 {
    let k = (q.len() as f64 * BOW_TRIM) as usize;
    let q = &q[k..q.len() - k];
    let Some(f) = tls(q) else { return 0.0 };
    let t: Vec<f64> = q.iter().map(|&p| dot(sub(p, f.c), f.d)).collect();
    let (lo, hi) = t
        .iter()
        .fold((f64::MAX, f64::MIN), |(l, h), &x| (l.min(x), h.max(x)));
    let half = (hi - lo) / 2.0;
    if half <= f64::MIN_POSITIVE {
        return 0.0;
    }
    // Mínimos quadrados de `e = α u² + β u + γ`, `u ∈ [-1, 1]`: as equações normais 3×3.
    let mut m = [[0.0; 3]; 3];
    let mut v = [0.0; 3];
    for (&p, &ti) in q.iter().zip(&t) {
        let u = (ti - (hi + lo) / 2.0) / half;
        let row = [u * u, u, 1.0];
        let e = cross(f.d, sub(p, f.c));
        for i in 0..3 {
            v[i] += row[i] * e;
            for j in 0..3 {
                m[i][j] += row[i] * row[j];
            }
        }
    }
    solve3(m, v).map_or(0.0, |s| s[0].abs() / (2.0 * half))
}

fn solve3(m: [[f64; 3]; 3], v: [f64; 3]) -> Option<[f64; 3]> {
    let det = |a: [[f64; 3]; 3]| {
        a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
    };
    let d = det(m);
    if d.abs() <= f64::MIN_POSITIVE {
        return None;
    }
    let col = |c: usize| {
        let mut a = m;
        for (row, &x) in a.iter_mut().zip(&v) {
            row[c] = x;
        }
        det(a) / d
    };
    Some([col(0), col(1), col(2)])
}

fn trimmed_fit(q: &[P2]) -> Option<Fit> {
    let k = ((q.len() as f64 * FIT_TRIM) as usize).max(1);
    if q.len() > 2 * k + 1 {
        tls(&q[k..q.len() - k])
    } else {
        tls(q)
    }
}

/// Mínimos quadrados TOTAIS (a recta que minimiza a distância perpendicular).
fn tls(q: &[P2]) -> Option<Fit> {
    if q.len() < 2 {
        return None;
    }
    let c = centroid(q);
    let (sxx, syy, sxy) = q.iter().fold((0.0, 0.0, 0.0), |(a, b, s), &p| {
        let [x, y] = sub(p, c);
        (a + x * x, b + y * y, s + x * y)
    });
    (sxx + syy > 0.0).then(|| Fit {
        c,
        d: unit(0.5 * f64::atan2(2.0 * sxy, sxx - syy)),
    })
}

fn intersect(f: Fit, g: Fit) -> P2 {
    let den = cross(f.d, g.d);
    if den.abs() <= 1e-9 {
        return mul(add(f.c, g.c), 0.5);
    }
    add(f.c, mul(f.d, cross(sub(g.c, f.c), g.d) / den))
}

fn project(p: P2, f: Fit) -> P2 {
    add(f.c, mul(f.d, dot(sub(p, f.c), f.d)))
}

fn axis_snap(d: P2) -> P2 {
    let a = angle(d);
    let q = snap(a, FRAC_PI_2);
    if (a - q).abs() <= AXIS_SNAP_DEG.to_radians() {
        unit(q)
    } else {
        d
    }
}

// ── geometria ───────────────────────────────────────────────────────────────────────────────────

/// O ângulo, arredondado ao múltiplo de `step` mais perto.
#[must_use]
pub fn snap(a: f64, step: f64) -> f64 {
    (a / step).round() * step
}

/// A viragem entre duas RECTAS (direcção sem sentido, `mod π`), em `[0, π/2]`.
fn turn(a: P2, b: P2) -> f64 {
    let t = wrap_pi(angle(b) - angle(a)).abs();
    t.min(PI - t)
}

fn wrap_pi(a: f64) -> f64 {
    a - TAU * ((a + PI) / TAU).floor()
}

fn centroid(q: &[P2]) -> P2 {
    let s = q.iter().fold([0.0, 0.0], |s, &p| add(s, p));
    mul(s, 1.0 / q.len().max(1) as f64)
}

fn seg_dist(p: P2, a: P2, b: P2) -> f64 {
    let v = sub(b, a);
    let vv = dot(v, v);
    let t = if vv > 0.0 {
        (dot(sub(p, a), v) / vv).clamp(0.0, 1.0)
    } else {
        0.0
    };
    dist(p, add(a, mul(v, t)))
}

fn rotate(p: P2, a: f64) -> P2 {
    let (s, c) = a.sin_cos();
    [p[0] * c - p[1] * s, p[0] * s + p[1] * c]
}
fn unit(a: f64) -> P2 {
    let (s, c) = a.sin_cos();
    [c, s]
}
fn angle(v: P2) -> f64 {
    v[1].atan2(v[0])
}
fn add(a: P2, b: P2) -> P2 {
    [a[0] + b[0], a[1] + b[1]]
}
fn sub(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
fn mul(a: P2, s: f64) -> P2 {
    [a[0] * s, a[1] * s]
}
fn dot(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn cross(a: P2, b: P2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn dist(a: P2, b: P2) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

#[cfg(test)]
#[path = "quickshape_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "quickshape_pencil_tests.rs"]
mod pencil_tests;
