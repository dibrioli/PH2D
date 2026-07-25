//! **Hatch** — enche uma forma fechada com linhas paralelas (a hachura do desenho a bico-de-pena).
//!
//! O *Hachure* do Rough.js e o *Hatches* do Inkscape. A abordagem robusta que todos convergem é a
//! **scanline**: segmentos paralelos INDEPENDENTES recortados pela forma, não uma única linha
//! contínua serpenteando (o *Scribble* do Illustrator/AE) — que dobra mal em côncavo. Aqui: roda a
//! forma para as linhas ficarem horizontais, para cada linha (espaçada `spacing`) cruza com TODAS
//! as arestas (contorno + buracos, regra even-odd), pareia os cruzamentos em spans, e cada span
//! vira um sub-contorno ABERTO de 2 vértices. **Cross** roda a família duas vezes (θ e θ+90).
//!
//! # A forma é MANTIDA, as linhas são ADICIONADAS
//!
//! A saída é o caminho ORIGINAL (o outline fechado + seus buracos + o `fill`) com as linhas de
//! hachura APENDADAS aos `subpaths`. O renderer preenche só os contornos FECHADOS (um sub-contorno
//! aberto nunca fura o fill — gate `an_open_contour_never_punches_a_hole_in_the_fill`) e traça
//! TODOS, então o resultado é: forma preenchida + outline + linhas, tudo na cor do traço. Quem quer
//! só a hachura põe `fill = None`.
//!
//! # Contorno ABERTO não tem interior
//!
//! Sem nenhum contorno fechado não há região a encher: o caminho volta intacto. É o mesmo espírito
//! do neutro (a pilha o salta e o `cooked()` devolve `Cow::Borrowed`).

use crate::compound::Contour;
use crate::{VecPath, VecVertex};

/// Abaixo deste espaçamento (em mundo) o efeito é o ponto neutro — um guard, não um teto.
const EPS: f64 = 1e-9;

/// Passos de de Casteljau por segmento ao achatar um contorno em poligonal para o cruzamento.
/// Fixo: a hachura não precisa de precisão sub-pixel na fronteira, e 16 já é liso a olho.
const FLATTEN_STEPS: usize = 16;

/// Teto de linhas de hachura no total — guarda contra um `spacing` minúsculo (save corrompido)
/// virar uma alocação de gigabytes. **Não é o teto do artista** (o slider para muito antes).
const MAX_LINES: usize = 4096;

/// **Os parâmetros do Hatch.** Neutro só em `spacing <= 0` (um guard); com espaçamento positivo o
/// efeito SEMPRE enche — como o Repeater com `count > 1`, ele mostra geometria assim que entra.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HatchSpec {
    /// O ângulo das linhas, em GRAUS.
    pub angle: f64,
    /// O espaçamento entre linhas, em **percentagem da forma** (`100` = a média entre largura e
    /// altura, [`crate::effect::FxCtx`]). Percentagem para o mesmo número dar a mesma densidade
    /// em qualquer escala.
    pub spacing: f64,
    /// `true` = cross-hatch: uma 2ª família a 90° da 1ª.
    pub cross: bool,
}

impl Default for HatchSpec {
    fn default() -> Self {
        // ⚠️ Nasce NEUTRO (`spacing == 0`): a lei do `every_kind_is_born_neutral` (ADR-0132) — um
        // efeito recém-posto na pilha é um no-op byte-idêntico até o artista o configurar (como o
        // Zig Zag nasce com `amplitude == 0`). O ângulo 45° é a forma que ele toma quando o
        // Spacing sobe de 0.
        Self {
            angle: 45.0,
            spacing: 0.0,
            cross: false,
        }
    }
}

impl HatchSpec {
    /// Sem espaçamento positivo não há grade — e o neutro tem de ser um no-op byte-idêntico, senão
    /// a pilha não pode saltá-lo e o `Cow::Borrowed` do `cooked()` morre (ADR-0132).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.spacing <= 0.0
    }
}

/// O ponto de uma cúbica em `t` — de Casteljau, sem transcendental.
fn cubic_point(p0: [f64; 2], c0: [f64; 2], c1: [f64; 2], p1: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let b0 = u * u * u;
    let b1 = 3.0 * u * u * t;
    let b2 = 3.0 * u * t * t;
    let b3 = t * t * t;
    [
        b0 * p0[0] + b1 * c0[0] + b2 * c1[0] + b3 * p1[0],
        b0 * p0[1] + b1 * c0[1] + b2 * c1[1] + b3 * p1[1],
    ]
}

/// Achata um contorno FECHADO numa poligonal (a fronteira que a scanline cruza).
fn flatten_closed(verts: &[VecVertex]) -> Vec<[f64; 2]> {
    let n = verts.len();
    if n < 2 {
        return Vec::new();
    }
    let mut poly = Vec::with_capacity(n * FLATTEN_STEPS);
    for i in 0..n {
        let a = &verts[i];
        let b = &verts[(i + 1) % n];
        for k in 0..FLATTEN_STEPS {
            #[allow(clippy::cast_precision_loss)]
            let t = k as f64 / FLATTEN_STEPS as f64;
            poly.push(cubic_point(a.anchor, a.out_handle, b.in_handle, b.anchor, t));
        }
    }
    poly
}

/// Uma família de linhas horizontais no espaço rodado, recortadas pelos polígonos (even-odd),
/// devolvidas em MUNDO. `(c, s)` = cosseno/seno de `angle` (o eixo da hachura).
fn hatch_family(polys: &[Vec<[f64; 2]>], c: f64, s: f64, spacing: f64, out: &mut Vec<[[f64; 2]; 2]>) {
    // Roda os polígonos por −angle ⇒ as linhas da hachura ficam horizontais (y constante).
    let rot: Vec<Vec<[f64; 2]>> = polys
        .iter()
        .map(|p| p.iter().map(|&[x, y]| [x.mul_add(c, y * s), (-x).mul_add(s, y * c)]).collect())
        .collect();
    let mut ymin = f64::MAX;
    let mut ymax = f64::MIN;
    for p in &rot {
        for &[_, y] in p {
            ymin = ymin.min(y);
            ymax = ymax.max(y);
        }
    }
    if !ymin.is_finite() || ymax - ymin <= EPS {
        return;
    }
    // Alinha as scanlines a uma grade global de `k*spacing` ⇒ a hachura é estável (as linhas não
    // saltam a cada re-cook). Começa na 1ª grade dentro do range.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let k0 = (ymin / spacing).ceil() as i64;
    let mut k = k0;
    let mut xs: Vec<f64> = Vec::new();
    loop {
        #[allow(clippy::cast_precision_loss)]
        let y = k as f64 * spacing;
        if y > ymax || out.len() >= MAX_LINES {
            break;
        }
        xs.clear();
        for p in &rot {
            let m = p.len();
            for i in 0..m {
                let a = p[i];
                let b = p[(i + 1) % m];
                // Regra de cruzamento even-odd meio-aberta (evita contar um vértice 2×).
                if (a[1] > y) != (b[1] > y) {
                    let t = (y - a[1]) / (b[1] - a[1]);
                    xs.push((b[0] - a[0]).mul_add(t, a[0]));
                }
            }
        }
        xs.sort_by(f64::total_cmp);
        // Pareia (x0,x1),(x2,x3),… — cada par é um span DENTRO da forma (buracos já partem o span).
        let mut j = 0;
        while j + 1 < xs.len() {
            let (x0, x1) = (xs[j], xs[j + 1]);
            if x1 - x0 > EPS {
                // Roda de volta para o mundo.
                let p0 = [x0.mul_add(c, -(y * s)), x0.mul_add(s, y * c)];
                let p1 = [x1.mul_add(c, -(y * s)), x1.mul_add(s, y * c)];
                out.push([p0, p1]);
                if out.len() >= MAX_LINES {
                    break;
                }
            }
            j += 2;
        }
        k += 1;
    }
}

/// Um segmento em mundo vira um sub-contorno ABERTO de 2 vértices (quina, sem alças).
fn segment_contour(seg: [[f64; 2]; 2]) -> Contour {
    Contour {
        verts: vec![VecVertex::corner(seg[0]), VecVertex::corner(seg[1])],
        closed: false,
    }
}

/// **Aplica o Hatch ao caminho.** Mantém o caminho original e apende as linhas de hachura aos
/// `subpaths`. Um caminho sem contorno fechado (sem interior) volta intacto.
#[must_use]
pub fn hatch_path(path: &VecPath, spec: &HatchSpec, ref_size: f64) -> VecPath {
    if spec.is_neutral() || ref_size <= EPS {
        return path.clone();
    }
    let spacing = spec.spacing / 100.0 * ref_size;
    if spacing <= EPS {
        return path.clone();
    }
    // Só os contornos FECHADOS definem a região a encher.
    let mut polys: Vec<Vec<[f64; 2]>> = Vec::new();
    if path.closed {
        let p = flatten_closed(&path.verts);
        if p.len() >= 3 {
            polys.push(p);
        }
    }
    for c in &path.subpaths {
        if c.closed {
            let p = flatten_closed(&c.verts);
            if p.len() >= 3 {
                polys.push(p);
            }
        }
    }
    if polys.is_empty() {
        return path.clone(); // sem interior: nada a hachurar.
    }

    let (s, c) = spec.angle.to_radians().sin_cos();
    let mut lines: Vec<[[f64; 2]; 2]> = Vec::new();
    hatch_family(&polys, c, s, spacing, &mut lines);
    if spec.cross {
        // A 2ª família a 90°: (cos, sin) de (angle + 90°) = (−s, c).
        hatch_family(&polys, -s, c, spacing, &mut lines);
    }
    if lines.is_empty() {
        return path.clone();
    }

    let mut out = path.clone();
    out.subpaths.extend(lines.into_iter().map(segment_contour));
    out
}

#[cfg(test)]
#[path = "fx_hatch_tests.rs"]
mod tests;
