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

pub use crate::quickshape_geom::snap;
use crate::quickshape_geom::{
    Fit, P2, add, angle, centroid, cross, dist, dot, intersect, mul, project, rdp, resample,
    rotate, solve, sub, tls, turn, unit, wrap_pi,
};
use ph2d_vec_scene::{VecPath, VecVertex};
use std::f64::consts::{FRAC_PI_2, TAU};

/// **As constantes da lei**, num valor só para que a sonda as VARRA no mesmo processo
/// (`quickshape_measure_tests.rs`). [`Law::MEASURED`] é a que corre; cada campo diz a FAIXA em que
/// todas as fixtures e os traços de mão continuam certos, e de onde vem a fronteira.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Law {
    /// Piso do tamanho (diagonal da caixa do traço), px de tela: abaixo disto o tremor da mão manda.
    pub(crate) min_size_px: f64,
    /// Um lado mais curto que o MENOR de `corner_px` (o mínimo do oráculo, 40 un. = 94 px — o
    /// rectângulo 500×60 dele é cru porque os lados de 60 px são quinas) e `corner_of_diag` × a
    /// diagonal (as formas PEQUENAS, ordem do dono 09/10) é QUINA.
    pub(crate) corner_px: f64,
    pub(crate) corner_of_diag: f64,
    /// Tolerância do RDP, fracção da diagonal.
    pub(crate) rdp_of_diag: f64,
    /// Viragem abaixo da qual dois lados são UM (graus); o octógono vira 45°.
    pub(crate) merge_turn_deg: f64,
    /// Flecha máxima de um lado RECTO (fracção do comprimento): o oráculo endireita 3,3 %
    /// (`linha_arco_leve`); um lado de círculo dá ≥ 7,7 %.
    pub(crate) bow_max: f64,
    /// Numa polilinha aberta, a quina interior não passa esta fracção do lado vizinho mais curto.
    pub(crate) open_corner_of_side: f64,
    /// A flecha mede-se sem esta fracção de cada ponta; o ajuste da recta, sem `fit_trim`.
    pub(crate) bow_trim: f64,
    pub(crate) fit_trim: f64,
    /// Folga de fecho (fracção do comprimento) e a janela em que a ponta procura a outra. O oráculo
    /// fecha 4,4 % e recusa 7,6 %; o DONO (09/10: «inclusive com a linha aberta») manda mais: 13 %
    /// fecha sempre o que tem até 10 % em falta, e o U fundo 3:1 (abertura de 14,3 %) só fecha a
    /// partir de 16 % — sonda `measure_open_gaps`.
    pub(crate) close_of_length: f64,
    pub(crate) close_window: f64,
    /// Desvio da quina a 90° para ser rectângulo (graus): oráculo, paralelogramo 6° sim, 10° não.
    pub(crate) right_tol_deg: f64,
    /// Endireitar aos eixos (graus): oráculo, rodado 4° sai alinhado, 6° não.
    pub(crate) axis_snap_deg: f64,
    /// Resíduo radial rms máximo de uma elipse (fracção do raio).
    pub(crate) ellipse_rms_max: f64,
    /// Eixos nesta razão = CÍRCULO (a elipse 3:2 fica elipse).
    pub(crate) round_ratio: f64,
    /// A viragem total de uma elipse é uma volta ± isto (graus).
    pub(crate) turn_tol_deg: f64,
}

impl Law {
    /// Os valores medidos — a faixa de cada um (varrida sozinha) no doc do QuickShape (oráculo
    /// `00_resultados.md` §3d).
    pub(crate) const MEASURED: Self = Self {
        min_size_px: MIN_SIZE_PX,
        corner_px: 94.0,
        corner_of_diag: CORNER_OF_DIAG,
        rdp_of_diag: 0.04,
        merge_turn_deg: 29.0,
        bow_max: 0.065,
        open_corner_of_side: OPEN_CORNER_OF_SIDE,
        bow_trim: 0.0,
        fit_trim: 0.175,
        close_of_length: 0.13,
        close_window: 0.13,
        right_tol_deg: 8.0,
        axis_snap_deg: 5.0,
        ellipse_rms_max: ELLIPSE_RMS_MAX,
        round_ratio: 1.10,
        turn_tol_deg: 150.0,
    };
}

/// O piso de tamanho, px de tela — MEDIDO (sonda `measure_small_shapes`).
pub const MIN_SIZE_PX: f64 = 12.0;
/// Quina relativa — MEDIDA (sonda `measure_the_law_ranges`).
const CORNER_OF_DIAG: f64 = 0.2;
/// Quina interior de polilinha / lado vizinho — MEDIDA (sonda `measure_the_s_curve`): a 0,2 o
/// zigue-zague de quinas arredondadas cai (88 %), a 0,4 o S passa por zigue-zague (2–5 %).
const OPEN_CORNER_OF_SIDE: f64 = 0.3;
/// Resíduo da elipse — MEDIDO (sonda `measure_the_law_ranges`).
const ELLIPSE_RMS_MAX: f64 = 0.075;
/// Amostras da reamostragem por comprimento (o desenho não depende da cadência do rato).
const RESAMPLE: usize = 256;
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
    recognise_with(&Law::MEASURED, samples, px_to_world, regular)
}

pub(crate) fn recognise_with(
    law: &Law,
    samples: &[P2],
    px_to_world: f64,
    regular: bool,
) -> Option<QuickShape> {
    if samples.len() < 3 {
        return None;
    }
    let min = law.min_size_px * px_to_world;
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
    let closed = gap(samples, length, law.close_window) <= law.close_of_length * length;
    let corner = (law.corner_px * px_to_world).min(law.corner_of_diag * diag);
    let r = resample(samples, RESAMPLE);
    let mut zones: Vec<Zone> = rdp(&r, law.rdp_of_diag * diag)
        .into_iter()
        .map(|i| (i, i))
        .collect();
    merge_small_turns(&r, &mut zones, law.merge_turn_deg);
    absorb_short_sides(&r, &mut zones, corner, law.merge_turn_deg);
    polygonal(law, samples, &r, &zones, corner, closed, regular)
        .or_else(|| closed.then(|| ellipse(law, samples, &r, regular)).flatten())
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

fn polygonal(
    law: &Law,
    samples: &[P2],
    r: &[P2],
    zones: &[Zone],
    corner: f64,
    closed: bool,
    regular: bool,
) -> Option<QuickShape> {
    // Uma faixa maior que uma quina já não é quina: as faixas encadeadas de um S escondiam
    // as curvas dele e liam-se como uma linha.
    if zones.iter().any(|&(a, b)| dist(r[a], r[b]) > corner) {
        return None;
    }
    // Numa polilinha ABERTA uma quina interior do tamanho dos lados vizinhos é a crista de um S,
    // não um bico de zigue-zague.
    if !closed && zones.len() > 2 {
        for j in 1..zones.len() - 1 {
            let side = |a: usize, b: usize| dist(r[a], r[b]);
            let adj = side(zones[j - 1].1, zones[j].0).min(side(zones[j].1, zones[j + 1].0));
            if side(zones[j].0, zones[j].1) > law.open_corner_of_side * adj {
                return None;
            }
        }
    }
    let sides: Vec<(usize, usize)> = zones.windows(2).map(|w| (w[0].1, w[1].0)).collect();
    if sides.is_empty()
        || sides
            .iter()
            .any(|&(a, b)| b <= a || bow(&r[a..=b], law.bow_trim) > law.bow_max)
    {
        return None;
    }
    let mut fits: Vec<Fit> = sides
        .iter()
        .map(|&(a, b)| trimmed_fit(&r[a..=b], law.fit_trim))
        .collect::<Option<_>>()?;
    // Fechado a começar a meio de um lado (ou a passar do início): o último lado e o primeiro são
    // o MESMO lado.
    // ⚠️ Com SENTIDO (o último lado tem de CONTINUAR o primeiro): como rectas, os dois lados de uma
    // ponta fina de estrela (< 29°) liam-se paralelos e fundiam-se — a estrela perdia uma ponta.
    let (la, lb) = sides[sides.len() - 1];
    let (fa, fb) = sides[0];
    let (last, first) = (sub(r[lb], r[la]), sub(r[fb], r[fa]));
    if closed
        && fits.len() > 3
        && wrap_pi(angle(first) - angle(last)).abs() < law.merge_turn_deg.to_radians()
    {
        let mut q = r[la..=lb].to_vec();
        q.extend_from_slice(&r[fa..=fb]);
        fits[0] = trimmed_fit(&q, law.fit_trim)?;
        fits.pop();
    }
    match (closed, fits.len()) {
        (false, 1) => Some(line(law, samples, &r[sides[0].0..=sides[0].1], regular)),
        (false, _) => Some(polyline(samples, &mut fits, regular)),
        (true, k) if k >= 3 => Some(closed_polygon(law, &fits, regular)),
        _ => None,
    }
}

/// A recta ajusta-se ao CORPO (`body`, sem os ganchos das pontas, que são quinas absorvidas); as
/// pontas são as projecções extremas de TODAS as amostras (o oráculo estende a linha até à ponta do
/// gancho).
fn line(law: &Law, samples: &[P2], body: &[P2], regular: bool) -> QuickShape {
    let Some(Fit { c, d }) = tls(body) else {
        return QuickShape::Line {
            a: samples[0],
            b: samples[samples.len() - 1],
        };
    };
    let mut d = if regular {
        unit(snap(angle(d), REGULAR_STEP_DEG.to_radians()))
    } else {
        axis_snap(d, law.axis_snap_deg)
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

fn closed_polygon(law: &Law, fits: &[Fit], regular: bool) -> QuickShape {
    let k = fits.len();
    let corners: Vec<P2> = (0..k)
        .map(|j| intersect(fits[(j + k - 1) % k], fits[j]))
        .collect();
    let corners = if regular {
        regularise(&corners, law.axis_snap_deg)
    } else if k == 4
        && (0..4).all(|j| {
            (turn(fits[(j + 3) % 4].d, fits[j].d) - FRAC_PI_2).abs()
                < law.right_tol_deg.to_radians()
        })
    {
        square_up(fits, law.axis_snap_deg)
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
fn square_up(fits: &[Fit], axis_deg: f64) -> Vec<P2> {
    let (s, c) = fits.iter().fold((0.0, 0.0), |(s, c), f| {
        let a = 4.0 * angle(f.d);
        (s + a.sin(), c + a.cos())
    });
    let m = angle(axis_snap(unit(f64::atan2(s, c) / 4.0), axis_deg));
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
fn regularise(corners: &[P2], axis_deg: f64) -> Vec<P2> {
    let k = corners.len();
    let c = centroid(corners);
    let radius = corners.iter().map(|&p| dist(p, c)).sum::<f64>() / k as f64;
    // O passo é o ângulo que o desenho anda de canto a canto à volta do centro: um polígono dá UMA
    // volta (360°/k), uma estrela de 5 pontas dá DUAS (144°) — com 360°/k a estrela virava pentágono.
    let swept: f64 = (0..k)
        .map(|i| wrap_pi(angle(sub(corners[(i + 1) % k], c)) - angle(sub(corners[i], c))))
        .sum();
    let turns = (swept / TAU).round();
    let turns = if turns == 0.0 { 1.0 } else { turns };
    let step = TAU * turns / k as f64;
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
    phase += angle(axis_snap(unit(side), axis_deg)) - side;
    (0..k).map(|i| at(phase, i)).collect()
}

// ── a elipse ────────────────────────────────────────────────────────────────────────────────────

fn ellipse(law: &Law, samples: &[P2], r: &[P2], regular: bool) -> Option<QuickShape> {
    let total: f64 = (1..r.len() - 1)
        .map(|i| wrap_pi(angle(sub(r[i + 1], r[i])) - angle(sub(r[i], r[i - 1]))))
        .sum();
    if (total.abs() - TAU).abs() > law.turn_tol_deg.to_radians() {
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
    if rms > law.ellipse_rms_max {
        return None;
    }
    if regular || a / b <= law.round_ratio {
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
fn gap(p: &[P2], length: f64, window: f64) -> f64 {
    let mut cum = 0.0;
    let mut head = vec![p[0]];
    let mut tail = Vec::new();
    for w in p.windows(2) {
        cum += dist(w[0], w[1]);
        if cum <= window * length {
            head.push(w[1]);
        }
        if cum >= (1.0 - window) * length {
            tail.push(w[1]);
        }
    }
    let last = p[p.len() - 1];
    let a = head.iter().map(|&h| dist(h, last)).fold(f64::MAX, f64::min);
    let b = tail.iter().map(|&t| dist(t, p[0])).fold(f64::MAX, f64::min);
    a.min(b)
}

/// Funde, menor primeiro, as quinas que viram menos que `merge_deg`.
fn merge_small_turns(r: &[P2], z: &mut Vec<Zone>, merge_deg: f64) {
    while z.len() > 2 {
        // Com SENTIDO (segmentos, não rectas): um bico de 160° não é uma viragem de 20°.
        let (t, j) = (1..z.len() - 1)
            .map(|j| {
                let a = sub(r[z[j].0], r[z[j - 1].1]);
                let b = sub(r[z[j + 1].0], r[z[j].1]);
                (wrap_pi(angle(b) - angle(a)).abs(), j)
            })
            .fold((f64::MAX, 0), |m, x| if x.0 < m.0 { x } else { m });
        if t >= merge_deg.to_radians() {
            return;
        }
        z.remove(j);
    }
}

/// Um lado mais curto que `corner` é parte de uma quina (ou um gancho na ponta): as duas faixas que o
/// fecham passam a UMA, que o contém.
fn absorb_short_sides(r: &[P2], z: &mut Vec<Zone>, corner: f64, merge_deg: f64) {
    while z.len() > 2 {
        let (len, j) = (0..z.len() - 1)
            .map(|j| (dist(r[z[j].1], r[z[j + 1].0]), j))
            .fold((f64::MAX, 0), |m, x| if x.0 < m.0 { x } else { m });
        if len >= corner {
            return;
        }
        z[j].1 = z[j + 1].1;
        z.remove(j + 1);
        merge_small_turns(r, z, merge_deg);
    }
}

/// A flecha de um lado: o termo quadrático do resíduo à recta, em fracção do comprimento.
fn bow(q: &[P2], trim: f64) -> f64 {
    let k = (q.len() as f64 * trim) as usize;
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
    solve(m, v).map_or(0.0, |s| s[0].abs() / (2.0 * half))
}

fn trimmed_fit(q: &[P2], trim: f64) -> Option<Fit> {
    let k = ((q.len() as f64 * trim) as usize).max(1);
    if q.len() > 2 * k + 1 {
        tls(&q[k..q.len() - k])
    } else {
        tls(q)
    }
}

fn axis_snap(d: P2, tol_deg: f64) -> P2 {
    let a = angle(d);
    let q = snap(a, FRAC_PI_2);
    if (a - q).abs() <= tol_deg.to_radians() {
        unit(q)
    } else {
        d
    }
}

#[cfg(test)]
#[path = "quickshape_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "quickshape_pencil_tests.rs"]
mod pencil_tests;

#[cfg(test)]
#[path = "quickshape_hand_tests.rs"]
mod hand_tests;

#[cfg(test)]
#[path = "quickshape_measure_tests.rs"]
mod measure_tests;

#[cfg(test)]
#[path = "quickshape_sensitivity_tests.rs"]
mod sensitivity_tests;
