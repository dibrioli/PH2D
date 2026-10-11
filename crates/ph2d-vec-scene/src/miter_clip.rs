//! ⭐ **O bico CORTADO** — o `miter-clip` do SVG 2 (F12 A, 11/10). Uma junta `Miter` cujo bico
//! passaria do [`crate::MITER_LIMIT`] já não cai no chanfro (o salto que o dono fotografou na
//! seta, no coração, na nuvem): o bico é CORTADO por uma recta perpendicular à bissectriz, a
//! `limite × ½w` do vértice. Contínuo em todo ângulo, e nunca mais comprido que o maior bico de
//! antes. Oráculo: o `resvg` (Apache-2.0/MIT), corrido. Doc:
//! `docs/UI_New_and_Simple/vetor_nova_ui/19_o_traco_das_formas.md`.
//!
//! Aqui vive a LEI, em `[f64; 2]` (esta crate não vê a `kurbo`): as juntas de um contorno e o
//! quadrilátero que cada uma acrescenta. Quem expande o traço (o desenho, a caixa visual, a placa,
//! o *Outline Stroke*) junta os quadriláteros ao contorno dele, orientados pelo winding do corpo.

/// Um troço: os quatro pontos de controlo de uma cúbica (a recta é `[p0, p0, p1, p1]`).
pub type Cubic = [[f64; 2]; 4];

/// O quadrilátero `[A, A', B', B]` que o corte acrescenta ao chanfro `A–B`, e o ponto do corpo
/// junto ao chanfro (`probe`) cujo winding diz como orientá-lo.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ClipPatch {
    pub quad: [[f64; 2]; 4],
    pub probe: [f64; 2],
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn unit(v: [f64; 2]) -> Option<[f64; 2]> {
    let l = v[0].hypot(v[1]);
    (l > 1e-12).then(|| [v[0] / l, v[1] / l])
}
fn along(p: [f64; 2], t: [f64; 2], s: f64) -> [f64; 2] {
    [p[0] + t[0] * s, p[1] + t[1] * s]
}

/// O corte de UMA junta em `p` (chega com `t_in`, sai com `t_out`, unitários). `None` = o bico
/// cabe no limite (o stroker desenha-o inteiro) ou não há quina.
#[must_use]
pub fn clip_patch(
    p: [f64; 2],
    t_in: [f64; 2],
    t_out: [f64; 2],
    half_w: f64,
    limit: f64,
) -> Option<ClipPatch> {
    let cos_phi = dot(t_in, t_out).clamp(-1.0, 1.0);
    // 1/cos(φ/2) > limite ⇔ cos φ < 2/limite² − 1.
    if half_w <= 0.0 || cos_phi >= 2.0 / (limit * limit) - 1.0 {
        return None;
    }
    // O lado de FORA da volta; na ida-e-volta (cruz nula) os dois lados dão o mesmo rectângulo.
    let cross = t_in[0] * t_out[1] - t_in[1] * t_out[0];
    let side = if cross >= 0.0 { 1.0 } else { -1.0 };
    let normal = |t: [f64; 2]| [t[1] * side, -t[0] * side];
    let a = along(p, normal(t_in), half_w);
    let b = along(p, normal(t_out), half_w);
    let u = unit(sub(t_in, t_out))?;
    let d = limit * half_w;
    let a2 = along(a, t_in, (d - dot(sub(a, p), u)) / dot(t_in, u));
    let back = [-t_out[0], -t_out[1]];
    let b2 = along(b, back, (d - dot(sub(b, p), u)) / dot(back, u));
    let mid = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    Some(ClipPatch {
        quad: [a, a2, b2, b],
        probe: along(mid, sub(p, mid), 0.25),
    })
}

/// A tangente de chegada (`at_end`) ou de partida de um troço: para o 1.º ponto de controlo
/// distinto, como o stroker a lê.
fn tangent(c: &Cubic, at_end: bool) -> Option<[f64; 2]> {
    let (o, rest) = if at_end {
        (c[3], [c[2], c[1], c[0]])
    } else {
        (c[0], [c[1], c[2], c[3]])
    };
    let scale = 1e-9 * (1.0 + o[0].abs().max(o[1].abs()));
    rest.iter()
        .map(|q| sub(*q, o))
        .find(|d| d[0].hypot(d[1]) > scale)
        .map(|d| {
            let v = unit(d).unwrap_or([1.0, 0.0]);
            if at_end { [-v[0], -v[1]] } else { v }
        })
}

/// Um elemento de caminho, espelho do `PathEl` da `kurbo` (cada leitor traduz o seu).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum El {
    Move([f64; 2]),
    Line([f64; 2]),
    Quad([f64; 2], [f64; 2]),
    Cubic([f64; 2], [f64; 2], [f64; 2]),
    Close,
}

/// Os subcaminhos de `els` como troços cúbicos, com o fecho (a recta de volta, se faltar).
#[must_use]
pub fn subpaths(els: impl IntoIterator<Item = El>) -> Vec<(Vec<Cubic>, bool)> {
    let mut out: Vec<(Vec<Cubic>, bool)> = Vec::new();
    let (mut cur, mut start, mut last) = (Vec::new(), [0.0; 2], [0.0; 2]);
    for el in els {
        let next = match el {
            El::Move(p) => {
                if !cur.is_empty() {
                    out.push((std::mem::take(&mut cur), false));
                }
                (start, last) = (p, p);
                continue;
            }
            El::Line(p) => [last, last, p, p],
            El::Quad(a, p) => [last, a, a, p],
            El::Cubic(a, b, p) => [last, a, b, p],
            El::Close => {
                if last != start {
                    cur.push([last, last, start, start]);
                }
                out.push((std::mem::take(&mut cur), true));
                last = start;
                continue;
            }
        };
        last = next[3];
        cur.push(next);
    }
    if !cur.is_empty() {
        out.push((cur, false));
    }
    out
}

/// O sinal do winding de um polígono simples no seu interior: o da área com sinal (`+` = anti-
/// horário com y para cima — a convenção da `kurbo`, provada em `ph2d-vec-render`).
fn polygon_sign(q: &[[f64; 2]; 4]) -> i32 {
    let a: f64 = (0..4)
        .map(|i| {
            let (p, n) = (q[i], q[(i + 1) % 4]);
            p[0] * n[1] - n[0] * p[1]
        })
        .sum();
    if a > 0.0 {
        1
    } else if a < 0.0 {
        -1
    } else {
        0
    }
}

/// ⭐ **Os quadriláteros no sentido do CORPO** (`body_winding` = o winding do contorno expandido):
/// junto ao chanfro os dois somam-se, e o preenchimento NonZero não mostra costura nem furo.
#[must_use]
pub fn oriented(cuts: &[ClipPatch], body_winding: impl Fn([f64; 2]) -> i32) -> Vec<[[f64; 2]; 4]> {
    cuts.iter()
        .map(|c| {
            let q = c.quad;
            if body_winding(c.probe).signum() * polygon_sign(&q) < 0 {
                [q[3], q[2], q[1], q[0]]
            } else {
                q
            }
        })
        .collect()
}

/// `PH2D_MITER_CLIP=0` volta ao chanfro de antes (o antes × depois do dono); lido uma vez.
#[must_use]
pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_MITER_CLIP").map_or(true, |v| v != "0"))
}

/// ⭐ **Os cortes de um contorno inteiro** — cada subcaminho `(troços, fechado)`, com a junta do
/// fecho quando fechado. Troços degenerados (um ponto) não são quina.
#[must_use]
pub fn clip_patches(subpaths: &[(Vec<Cubic>, bool)], half_w: f64, limit: f64) -> Vec<ClipPatch> {
    let mut out = Vec::new();
    if !enabled() {
        return out;
    }
    for (segs, closed) in subpaths {
        let segs: Vec<&Cubic> = segs
            .iter()
            .filter(|c| tangent(c, false).is_some())
            .collect();
        let n = segs.len();
        let joins = if *closed { n } else { n.saturating_sub(1) };
        for k in 0..joins {
            let (a, b) = (segs[k], segs[(k + 1) % n]);
            if let (Some(t1), Some(t2)) = (tangent(a, true), tangent(b, false))
                && let Some(c) = clip_patch(a[3], t1, t2, half_w, limit)
            {
                out.push(c);
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "miter_clip_tests.rs"]
mod tests;
