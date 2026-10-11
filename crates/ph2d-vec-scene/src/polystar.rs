//! **Polígono e estrela com os parâmetros da F10** — e a espiral com divergência e raio interno.
//! Módulo irmão de [`crate::shapes`] (que guarda as primitivas de sempre: com os campos novos a
//! zero, cada função daqui DEVOLVE a de lá, byte a byte — um save anterior cozinha igual).
//! Doc, oráculos e recusas: `docs/UI_New_and_Simple/vetor_nova_ui/15_mais_parametros_das_formas.md`.
//!
//! As leis, cada uma do seu oráculo:
//! - **Smooth** (o *Rounded* do Inkscape, corrido em `oraculo_parametros/`): o vértice fica liso,
//!   alças na direcção vizinho-anterior → vizinho-seguinte, de comprimento `smooth × a corda`;
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

/// O polígono / a estrela de [`PolyStar`] inscrito na elipse `rx`/`ry`, 1.ª ponta às 12h.
#[must_use]
pub fn polystar(center: [f64; 2], rx: f64, ry: f64, s: &PolyStar) -> VecPath {
    let plain = s.twist_deg == 0.0 && s.curve == 0.0 && s.smooth == 0.0;
    if plain && s.style == CornerStyle::Round {
        return match s.inner {
            None => crate::regular_polygon_rounded(center, rx, ry, s.n, s.radii[0]),
            Some(k) => crate::star_rounded(center, rx, ry, s.n, k, s.radii[0], s.radii[1]),
        };
    }
    let unit = unit_vertices(s);
    let m = unit.len();
    // As alças de cada aresta, no círculo: o liso (tangentes) + a envergadura (normal).
    let mut out_h = vec![[0.0; 2]; m];
    let mut in_h = vec![[0.0; 2]; m];
    for i in 0..m {
        let (a, b) = (unit[i], unit[(i + 1) % m]);
        let d = [b[0] - a[0], b[1] - a[1]];
        let len = d[0].hypot(d[1]);
        if len <= 1e-12 {
            continue;
        }
        let (ta, tb) = (tangent(&unit, i), tangent(&unit, (i + 1) % m));
        let k = s.smooth * len;
        // A envergadura: o meio da cúbica anda `h` pela normal de FORA quando as duas alças
        // andam `4h/3`; para dentro, nunca além de 90 % da distância da corda ao centro.
        let normal = [d[1] / len, -d[0] / len];
        let reach = (a[0] * normal[0] + a[1] * normal[1]).abs();
        let h = (s.curve * len * 0.5).max(-0.9 * reach);
        let bend = [normal[0] * h * 4.0 / 3.0, normal[1] * h * 4.0 / 3.0];
        out_h[i] = [ta[0] * k + bend[0], ta[1] * k + bend[1]];
        in_h[(i + 1) % m] = [-tb[0] * k + bend[0], -tb[1] * k + bend[1]];
    }
    let to_world = |p: [f64; 2]| [center[0] + rx * p[0], center[1] + ry * p[1]];
    let lisa = s.smooth != 0.0;
    let verts: Vec<VecVertex> = (0..m)
        .map(|i| {
            let p = unit[i];
            VecVertex {
                anchor: to_world(p),
                in_handle: to_world([p[0] + in_h[i][0], p[1] + in_h[i][1]]),
                out_handle: to_world([p[0] + out_h[i][0], p[1] + out_h[i][1]]),
                kind: if lisa {
                    VertexKind::Smooth
                } else {
                    VertexKind::Corner
                },
                corner_radius: if s.inner.is_some() && i % 2 == 1 {
                    s.radii[1]
                } else {
                    s.radii[0]
                },
            }
        })
        .collect();
    let straight = s.curve == 0.0 && !lisa;
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

/// A direcção do liso no vértice `i`: do vizinho ANTERIOR para o SEGUINTE — medido no Inkscape
/// (a estrela torcida a 15°: a alça da ponta a 15,015°, não a 0° da perpendicular ao raio; sem
/// torção as duas coincidem por simetria).
fn tangent(unit: &[[f64; 2]], i: usize) -> [f64; 2] {
    let m = unit.len();
    let (a, b) = (unit[(i + m - 1) % m], unit[(i + 1) % m]);
    let d = [b[0] - a[0], b[1] - a[1]];
    let l = d[0].hypot(d[1]);
    if l <= 1e-12 {
        return [0.0, 0.0];
    }
    [d[0] / l, d[1] / l]
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
/// cortado leva as voltas de dentro, como no Inkscape. Os dois a zero = [`crate::spiral`], byte a
/// byte. Cúbicas de Hermite com a tangente ANALÍTICA, como a de sempre.
#[must_use]
pub fn spiral_ex(
    center: [f64; 2],
    rx: f64,
    ry: f64,
    turns: u32,
    expansion: f64,
    inner: f64,
) -> VecPath {
    if expansion == 0.0 && inner <= 0.0 {
        return crate::spiral(center, rx, ry, turns);
    }
    let t = turns.clamp(1, crate::MAX_SPIRAL_TURNS);
    let e = (1.0 + expansion).max(0.05);
    let total = std::f64::consts::TAU * f64::from(t);
    let start = std::f64::consts::FRAC_PI_2;
    // O parâmetro da curva: `t = u^q`. Com `e < 1` a lei `r = t^e` tem derivada infinita no centro
    // e uma cúbica não a segue (medido: 0,0135 da lei a `e = 0,5`); andar pelo RAIO (`q = 1/e`,
    // `r = u`) deixa tudo liso, e o ângulo é que acelera — daí `q×` mais passos.
    let q = if e < 1.0 { 1.0 / e } else { 1.0 };
    let u0 = inner.clamp(0.0, 0.95).powf(1.0 / q);
    let steps = t as usize * SPIRAL_STEPS_PER_TURN * q.ceil() as usize;
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
