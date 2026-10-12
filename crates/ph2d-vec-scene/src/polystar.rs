//! **Polígono e estrela com os parâmetros da F10** — e a espiral com divergência e raio interno.
//! Módulo irmão de [`crate::shapes`] (que guarda as primitivas de sempre: com os campos novos a
//! zero, cada função daqui DEVOLVE a de lá, byte a byte — um save anterior cozinha igual).
//! Doc, oráculos e recusas: `docs/UI_New_and_Simple/vetor_nova_ui/15_mais_parametros_das_formas.md`.
//!
//! As leis, cada uma do seu oráculo:
//! - **Smooth** (o *Rounded* do Inkscape, corrido em `oraculo_parametros/`): o vértice fica liso,
//!   alças na direcção vizinho-anterior → vizinho-seguinte, de comprimento `rounded × a corda`, com
//!   `rounded = smooth × rounded_at_one(m)` (F13: `1` = o redondo; o Inkscape aceita mais e enruga);
//! - **Curve** (o *Curve* do polígono e o *Curved edges* da estrela do Affinity, pelo manual): a
//!   aresta enverga para fora (> 0) ou para dentro (< 0), o meio dela a `curve × meia corda`;
//! - **Twist** (o `arg2` do Inkscape): os vales rodam em relação às pontas;
//! - o **tipo do canto** ([`crate::CornerStyle`]) para os raios de canto.
//!
//! Tudo se constrói no CÍRCULO unitário e só depois se estica para a caixa (afim): a lei é a do
//! Inkscape no círculo, e a elipse é a mesma forma esticada.

use crate::{CornerStyle, VecPath, VecVertex, VertexKind};

/// Os parâmetros de um polígono (`inner = None`) ou de uma estrela.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PolyStar {
    /// Lados (polígono) ou pontas (estrela).
    pub n: u32,
    /// A razão do raio dos vales; `None` = polígono.
    pub inner: Option<f64>,
    /// Graus que os vales rodam (anti-horário).
    pub twist_deg: f64,
    /// A envergadura das arestas (`-1..1`).
    pub curve: f64,
    /// O liso dos vértices (o *Rounded* do Inkscape).
    pub smooth: f64,
    /// Os raios de canto (MUNDO): `[pontas, vales]` (o polígono só usa o 1.º).
    pub radii: [f64; 2],
    pub style: CornerStyle,
}

/// O polígono / a estrela de [`PolyStar`] inscrito na elipse `rx`/`ry`, 1.ª ponta às 12h. Se a
/// torção, a envergadura e o liso juntos cruzam o contorno, os três recuam JUNTOS até ao último
/// desenho sem cruzamento ([`crate::untangle`]).
#[must_use]
pub fn polystar(center: [f64; 2], rx: f64, ry: f64, s: &PolyStar) -> VecPath {
    let k = rounded_at_one(unit_vertices(s).len());
    let at = |t: f64| {
        let p = PolyStar {
            twist_deg: s.twist_deg * t,
            curve: s.curve * t,
            smooth: 0.0,
            ..*s
        };
        let liso = s.smooth.max(0.0) * t;
        smooth_contour(shaped(center, rx, ry, &p), center, [rx, ry], liso, k)
    };
    if s.twist_deg == 0.0 && s.curve == 0.0 && s.smooth <= 0.0 {
        return at(1.0);
    }
    crate::untangle::untangled(s.n.max(1) as usize, at)
}

/// ⭐ **O liso sobre o contorno JÁ FEITO** (F13 — o dono: «Smooth que não aproveita o trabalho dos
/// outros ajustes»): o raio, o tipo do canto, a envergadura e a torção ficam, e o liso amacia as
/// junções. Em `0` nada muda; cada alça roda para a direcção vizinho-anterior → vizinho-seguinte e
/// cresce até `k ×` a corda dela (o *Rounded* do Inkscape), aos poucos; em `1` todo vértice é G1.
/// No espaço da elipse unitária (o do Inkscape): o polígono regular a `1` é o círculo.
fn smooth_contour(mut p: VecPath, c: [f64; 2], r: [f64; 2], s: f64, k: f64) -> VecPath {
    let n = p.verts.len();
    if s <= 0.0 || n < 3 || r[0].abs() < 1e-12 || r[1].abs() < 1e-12 {
        return p;
    }
    let to_u = |q: [f64; 2]| [(q[0] - c[0]) / r[0], (q[1] - c[1]) / r[1]];
    let to_w = |q: [f64; 2]| [c[0] + r[0] * q[0], c[1] + r[1] * q[1]];
    let a: Vec<[f64; 2]> = p.verts.iter().map(|v| to_u(v.anchor)).collect();
    let turn = s.min(1.0);
    let grow = |h: [f64; 2], want: [f64; 2], chord: f64| {
        let l = h[0].hypot(h[1]);
        let hd = if l > 1e-12 {
            [h[0] / l, h[1] / l]
        } else {
            want
        };
        let m = [
            hd[0] + (want[0] - hd[0]) * turn,
            hd[1] + (want[1] - hd[1]) * turn,
        ];
        let lm = m[0].hypot(m[1]);
        let d = if lm > 1e-12 {
            [m[0] / lm, m[1] / lm]
        } else {
            want
        };
        let len = l + (l.max(k * chord) - l) * s;
        [d[0] * len, d[1] * len]
    };
    for i in 0..n {
        let (prev, v, next) = (a[(i + n - 1) % n], a[i], a[(i + 1) % n]);
        let d = [next[0] - prev[0], next[1] - prev[1]];
        let ld = d[0].hypot(d[1]);
        if ld <= 1e-12 {
            continue;
        }
        let dir = [d[0] / ld, d[1] / ld];
        let rel = |h: [f64; 2]| {
            let q = to_u(h);
            [q[0] - v[0], q[1] - v[1]]
        };
        let dist = |q: [f64; 2]| (q[0] - v[0]).hypot(q[1] - v[1]);
        let o = grow(rel(p.verts[i].out_handle), dir, dist(next));
        let b = grow(rel(p.verts[i].in_handle), [-dir[0], -dir[1]], dist(prev));
        let vert = &mut p.verts[i];
        vert.out_handle = to_w([v[0] + o[0], v[1] + o[1]]);
        vert.in_handle = to_w([v[0] + b[0], v[1] + b[1]]);
        // Liso onde as alças ficaram na mesma recta (sempre em `1`; num lado recto, já no 1.º passo).
        let cross = o[0] * b[1] - o[1] * b[0];
        if cross.abs() <= 1e-9 * o[0].hypot(o[1]) * b[0].hypot(b[1])
            && o[0] * b[0] + o[1] * b[1] < 0.0
        {
            vert.kind = VertexKind::Smooth;
        }
    }
    p
}

fn shaped(center: [f64; 2], rx: f64, ry: f64, s: &PolyStar) -> VecPath {
    // O liso não entra aqui: é o `smooth_contour` sobre o que isto devolve.
    let plain = s.twist_deg == 0.0 && s.curve == 0.0;
    if plain && s.style == CornerStyle::Round {
        return match s.inner {
            None => crate::regular_polygon_rounded(center, rx, ry, s.n, s.radii[0]),
            Some(k) => crate::star_rounded(center, rx, ry, s.n, k, s.radii[0], s.radii[1]),
        };
    }
    // Envergadura COM raio de canto (e sem liso): arredonda-se o contorno RECTO (arcos exactos) e
    // enverga-se só o troço recto entre dois cantos. Pela ligação lisa entre duas arestas curvas
    // o canto passava ACIMA do vértice (0,067 num pentágono a envergadura 0,4 e raio 0,15 — o
    // arredondamento engordava a forma): medido em `the_rounding_never_grows_past_the_sharp_corner`.
    if s.curve != 0.0 && s.radii.iter().any(|r| *r > 0.0) {
        let straight = shaped(center, rx, ry, &PolyStar { curve: 0.0, ..*s });
        let sharp: Vec<[f64; 2]> = unit_vertices(s)
            .iter()
            .map(|p| [center[0] + rx * p[0], center[1] + ry * p[1]])
            .collect();
        return bend_between_corners(straight, center, s.curve, &sharp);
    }
    let unit = unit_vertices(s);
    let m = unit.len();
    let radius_of = |i: usize| {
        if s.inner.is_some() && i % 2 == 1 {
            s.radii[1]
        } else {
            s.radii[0]
        }
    };
    let wedge: Vec<f64> = (0..m).map(|i| interior_angle(&unit, i)).collect();
    // As alças de cada aresta, no círculo: a envergadura.
    let mut out_h = vec![[0.0; 2]; m];
    let mut in_h = vec![[0.0; 2]; m];
    for i in 0..m {
        let j = (i + 1) % m;
        let (a, b) = (unit[i], unit[j]);
        let d = [b[0] - a[0], b[1] - a[1]];
        let len = d[0].hypot(d[1]);
        if len <= 1e-12 {
            continue;
        }
        // A envergadura: o meio da cúbica anda `h` pela normal de FORA quando as duas alças
        // andam `4h/3` — sobre a corda a ⅓ (a MESMA lei do [`bend_between_corners`]): a tangente
        // na quina roda `atan(4h/corda)`, a partir de zero. Para dentro, nunca além de 90 % da
        // distância ao centro; e a rotação nunca passa da metade da quina que ela fecha (no limite,
        // uma cúspide — nunca um laço).
        let normal = [d[1] / len, -d[0] / len];
        let reach = (a[0] * normal[0] + a[1] * normal[1]).abs();
        let closing = |w: f64| {
            if s.curve < 0.0 {
                w
            } else {
                std::f64::consts::TAU - w
            }
        };
        let cap = [wedge[i], wedge[j]]
            .iter()
            .map(|w| closing(*w) * 0.5)
            .filter(|half| *half < std::f64::consts::FRAC_PI_2)
            .map(|half| len * 0.25 * half.tan())
            .fold(f64::INFINITY, f64::min);
        let h = (s.curve * len * 0.5).max(-0.9 * reach).clamp(-cap, cap);
        let bend = [normal[0] * h * 4.0 / 3.0, normal[1] * h * 4.0 / 3.0];
        // O troço a ⅓ da corda (abaixo de um passo do campo, `0,01`, entra em proporção).
        let chord = (s.curve.abs() / CURVE_STEP).min(1.0) / 3.0;
        out_h[i] = [d[0] * chord + bend[0], d[1] * chord + bend[1]];
        in_h[j] = [-d[0] * chord + bend[0], -d[1] * chord + bend[1]];
    }
    let to_world = |p: [f64; 2]| [center[0] + rx * p[0], center[1] + ry * p[1]];
    let verts: Vec<VecVertex> = (0..m)
        .map(|i| {
            let p = unit[i];
            VecVertex {
                anchor: to_world(p),
                in_handle: to_world([p[0] + in_h[i][0], p[1] + in_h[i][1]]),
                out_handle: to_world([p[0] + out_h[i][0], p[1] + out_h[i][1]]),
                kind: VertexKind::Corner,
                corner_radius: radius_of(i),
            }
        })
        .collect();
    let straight = s.curve == 0.0;
    if verts.iter().all(|v| v.corner_radius <= 0.0) {
        return closed(clear_radii(verts));
    }
    if straight && s.style == CornerStyle::Round {
        let pts: Vec<[f64; 2]> = verts.iter().map(|v| v.anchor).collect();
        let radii: Vec<f64> = verts.iter().map(|v| v.corner_radius).collect();
        return crate::corners::round_closed_corners(&pts, &radii);
    }
    match crate::corner_live::round_authored_corners_styled(&verts, true, s.style) {
        Some(cooked) => closed(cooked),
        None => closed(clear_radii(verts)),
    }
}

/// Enverga os troços RECTOS (alças nulas ou a ⅓) de um contorno fechado já arredondado: o meio de
/// cada um anda `curve × meia corda` pela normal de FORA (para dentro, nunca além de 90 % da
/// distância ao centro), e as alças vizinhas rodam para a nova tangente — a junção fica lisa.
fn bend_between_corners(
    mut p: VecPath,
    center: [f64; 2],
    curve: f64,
    sharp: &[[f64; 2]],
) -> VecPath {
    let n = p.verts.len();
    // Só enverga o que é LADO do polígono: as paredes do Cutout e o chanfro do Straight ficam rectos
    // (envergados viravam ganchos — o dono, F13).
    let on_a_side = |a: [f64; 2], b: [f64; 2]| {
        let m = sharp.len();
        (0..m).any(|i| {
            let (u, v) = (sharp[i], sharp[(i + 1) % m]);
            let e = [v[0] - u[0], v[1] - u[1]];
            let le = e[0].hypot(e[1]).max(1e-12);
            let off = |q: [f64; 2]| ((q[0] - u[0]) * e[1] - (q[1] - u[1]) * e[0]).abs() / le;
            off(a) < 1e-9 * le.max(1.0) && off(b) < 1e-9 * le.max(1.0)
        })
    };
    // O troço `k → k+1` é um ARCO do canto (nem lado, nem recta): só nele a junção fica G1 com o lado
    // envergado; ao lado de uma parede ou de outro lado fica a quina. Lido ANTES de envergar.
    let arc: Vec<bool> = (0..n)
        .map(|k| {
            let (v, w) = (&p.verts[k], &p.verts[(k + 1) % n]);
            let third = |o: [f64; 2], to: [f64; 2], h: [f64; 2]| {
                let q = [o[0] + (to[0] - o[0]) / 3.0, o[1] + (to[1] - o[1]) / 3.0];
                (h[0] - o[0]).hypot(h[1] - o[1]) < 1e-12 || (h[0] - q[0]).hypot(h[1] - q[1]) < 1e-9
            };
            let line =
                third(v.anchor, w.anchor, v.out_handle) && third(w.anchor, v.anchor, w.in_handle);
            !line && !on_a_side(v.anchor, w.anchor)
        })
        .collect();
    let side: Vec<bool> = (0..n)
        .map(|k| on_a_side(p.verts[k].anchor, p.verts[(k + 1) % n].anchor))
        .collect();
    for k in 0..n {
        let j = (k + 1) % n;
        let (a, b) = (p.verts[k].anchor, p.verts[j].anchor);
        if !side[k] {
            continue;
        }
        let d = [b[0] - a[0], b[1] - a[1]];
        let len = d[0].hypot(d[1]);
        let straight = |h: [f64; 2], o: [f64; 2], t: f64| {
            let q = [o[0] + d[0] * t, o[1] + d[1] * t];
            (h[0] - o[0]).hypot(h[1] - o[1]) < 1e-12 || (h[0] - q[0]).hypot(h[1] - q[1]) < 1e-9
        };
        if len <= 1e-12
            || !straight(p.verts[k].out_handle, a, 1.0 / 3.0)
            || !straight(p.verts[j].in_handle, b, -1.0 / 3.0)
        {
            continue;
        }
        let normal = [d[1] / len, -d[0] / len];
        let reach = ((a[0] - center[0]) * normal[0] + (a[1] - center[1]) * normal[1]).abs();
        let h = (curve * len * 0.5).max(-0.9 * reach) * 4.0 / 3.0;
        let off = [normal[0] * h, normal[1] * h];
        let out = [a[0] + d[0] / 3.0 + off[0], a[1] + d[1] / 3.0 + off[1]];
        let inn = [b[0] - d[0] / 3.0 + off[0], b[1] - d[1] / 3.0 + off[1]];
        p.verts[k].out_handle = out;
        p.verts[j].in_handle = inn;
        // A alça do ARCO vizinho roda para a tangente nova (mesmo comprimento): G1 na junção.
        if arc[(k + n - 1) % n] {
            align_opposite(&mut p.verts[k].in_handle, a, out);
        }
        if arc[j] {
            align_opposite(&mut p.verts[j].out_handle, b, inn);
        }
    }
    p
}

/// Põe `h` (alça de `o`) do lado oposto de `other`, com o comprimento que tinha; nula fica nula.
fn align_opposite(h: &mut [f64; 2], o: [f64; 2], other: [f64; 2]) {
    let l = (h[0] - o[0]).hypot(h[1] - o[1]);
    let t = [o[0] - other[0], o[1] - other[1]];
    let lt = t[0].hypot(t[1]);
    if l > 1e-12 && lt > 1e-12 {
        *h = [o[0] + t[0] / lt * l, o[1] + t[1] / lt * l];
    }
}

/// Os vértices no círculo unitário (Y-para-cima, anti-horário): as pontas no raio 1, os vales no
/// raio `inner` rodados `twist`.
fn unit_vertices(s: &PolyStar) -> Vec<[f64; 2]> {
    let start = std::f64::consts::FRAC_PI_2;
    match s.inner {
        None => {
            let n = s.n.clamp(3, crate::MAX_POLYGON_SIDES) as usize;
            let step = std::f64::consts::TAU / n as f64;
            (0..n)
                .map(|i| {
                    let a = start + step * i as f64;
                    [a.cos(), a.sin()]
                })
                .collect()
        }
        Some(k) => {
            let n = s.n.clamp(3, crate::shapes::MAX_STAR_POINTS) as usize;
            let k = k.clamp(0.05, 0.95);
            let step = std::f64::consts::PI / n as f64;
            let twist = s.twist_deg.to_radians();
            (0..2 * n)
                .map(|i| {
                    let (r, a) = if i % 2 == 0 {
                        (1.0, start + step * i as f64)
                    } else {
                        (k, start + step * i as f64 + twist)
                    };
                    [r * a.cos(), r * a.sin()]
                })
                .collect()
        }
    }
}

/// O *Rounded* do Inkscape que o liso `1` vale num contorno de `m` vértices: a alça do arco do
/// círculo do `m`-ágono regular sobre a corda, `1 / (3·cos²(π/2m))` — o liso nunca passa do redondo.
fn rounded_at_one(m: usize) -> f64 {
    let c = (std::f64::consts::PI / (2.0 * m.max(3) as f64)).cos();
    1.0 / (3.0 * c * c)
}

/// Um passo do campo *Curve* (`shapes_fields::frac`): daí para cima a envergadura é a lei inteira.
const CURVE_STEP: f64 = 0.01;

/// O ângulo INTERNO no vértice `i` (o contorno é anti-horário: o interior fica à esquerda), `0..2π`.
fn interior_angle(unit: &[[f64; 2]], i: usize) -> f64 {
    let m = unit.len();
    let v = unit[i];
    let (p, n) = (unit[(i + m - 1) % m], unit[(i + 1) % m]);
    let (dn, dp) = ([n[0] - v[0], n[1] - v[1]], [p[0] - v[0], p[1] - v[1]]);
    let a = (dn[0] * dp[1] - dn[1] * dp[0]).atan2(dn[0] * dp[0] + dn[1] * dp[1]);
    a.rem_euclid(std::f64::consts::TAU)
}

fn clear_radii(mut verts: Vec<VecVertex>) -> Vec<VecVertex> {
    for v in &mut verts {
        v.corner_radius = 0.0;
    }
    verts
}

fn closed(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// O round-rect por canto ([`crate::rounded_rect_corners`]) com o TIPO do canto: arredondado é a
/// função de sempre, byte a byte; os outros passam pelo motor das quinas vivas com os mesmos raios
/// (os mesmos cortes) e a suavização — que é do ARCO — não se aplica.
#[must_use]
pub fn round_rect_styled(
    a: [f64; 2],
    b: [f64; 2],
    radii: [f64; 4],
    smoothing: f64,
    style: CornerStyle,
) -> VecPath {
    if style == CornerStyle::Round {
        return crate::rounded_rect_corners(a, b, radii, smoothing);
    }
    let (x0, x1) = (a[0].min(b[0]), a[0].max(b[0]));
    let (y0, y1) = (a[1].min(b[1]), a[1].max(b[1]));
    // A volta do `rounded_rect_corners` (BL, BR, TR, TL) e os raios na mesma ordem.
    let [tl, tr, br, bl] = radii;
    let verts: Vec<VecVertex> = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
        .into_iter()
        .zip([bl, br, tr, tl])
        .map(|(p, r)| {
            let mut v = VecVertex::corner(p);
            v.corner_radius = r.max(0.0);
            v
        })
        .collect();
    match crate::corner_live::round_authored_corners_styled(&verts, true, style) {
        Some(cooked) => closed(cooked),
        None => closed(clear_radii(verts)),
    }
}

/// A espiral com **divergência** e **raio interno** (o `expansion` e o `t0` do Inkscape, corridos):
/// `r = t^(1 + expansion)` com `t` de `inner` a 1, o ângulo `θ = 12h + voltas·2π·t` — o miolo
/// cortado leva as voltas de dentro, como no Inkscape. As voltas são FRACCIONÁRIAS (a `revolution`
/// do Inkscape: a ponta de fora segue o ângulo da mão); voltas inteiras com os dois a zero =
/// [`crate::spiral`], byte a byte. Cúbicas de Hermite com a tangente ANALÍTICA, como a de sempre.
#[must_use]
pub fn spiral_ex(
    center: [f64; 2],
    rx: f64,
    ry: f64,
    turns: f64,
    expansion: f64,
    inner: f64,
) -> VecPath {
    let t = turns.clamp(1.0, f64::from(crate::MAX_SPIRAL_TURNS));
    if expansion == 0.0 && inner <= 0.0 && t.fract() == 0.0 {
        return crate::spiral(center, rx, ry, t as u32);
    }
    let e = (1.0 + expansion).max(0.05);
    let total = std::f64::consts::TAU * t;
    let start = std::f64::consts::FRAC_PI_2;
    // O parâmetro da curva: `t = u^q`. Com `e < 1` a lei `r = t^e` tem derivada infinita no centro
    // e uma cúbica não a segue (medido: 0,0135 da lei a `e = 0,5`); andar pelo RAIO (`q = 1/e`,
    // `r = u`) deixa tudo liso, e o ângulo é que acelera — daí `q×` mais passos.
    let q = if e < 1.0 { 1.0 / e } else { 1.0 };
    let u0 = inner.clamp(0.0, 0.95).powf(1.0 / q);
    // Voltas inteiras dão o mesmo número de passos de antes (um save antigo coze igual).
    let steps = ((t * SPIRAL_STEPS_PER_TURN as f64 * q.ceil()).ceil() as usize).max(1);
    let du = (1.0 - u0) / steps as f64;
    let verts = (0..=steps)
        .map(|i| {
            let u = u0 + du * i as f64;
            let tt = u.powf(q);
            let (sn, cs) = (start + total * tt).sin_cos();
            let r = tt.powf(e);
            // d/du de `r` e de `θ`; no centro a que não existe fica nula.
            let dr = if u > 0.0 {
                q * e * u.powf(q * e - 1.0)
            } else {
                0.0
            };
            let da = if u > 0.0 || q == 1.0 {
                total * q * u.powf(q - 1.0)
            } else {
                0.0
            };
            // O alçapão do arco: `(4/3)·tan(Δθ/4)/Δθ` por unidade de passo (→ ⅓ numa recta).
            let dth = da * du;
            let k = if dth > 1e-12 {
                (4.0 / 3.0) * (dth / 4.0).tan() / dth
            } else {
                1.0 / 3.0
            } * du;
            let anchor = [center[0] + rx * r * cs, center[1] + ry * r * sn];
            let (tx, ty) = (
                rx * (dr * cs - r * da * sn) * k,
                ry * (dr * sn + r * da * cs) * k,
            );
            VecVertex::smooth(
                anchor,
                [anchor[0] - tx, anchor[1] - ty],
                [anchor[0] + tx, anchor[1] + ty],
            )
        })
        .collect();
    VecPath {
        verts,
        closed: false,
        ..VecPath::default()
    }
}

/// Cúbicas por volta da espiral divergente — medido contra o Inkscape
/// (`spiral_ex_matches_the_inkscape_divergence_and_inner_radius`).
const SPIRAL_STEPS_PER_TURN: usize = 16;

#[cfg(test)]
#[path = "polystar_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "polystar_smooth_tests.rs"]
mod smooth_tests;
