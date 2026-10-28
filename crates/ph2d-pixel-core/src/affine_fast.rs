//! **A *Fast Rotation* do Aseprite** (filho de [`super`]): a lei medida no Trial
//! (`oracle_trial_rotation.json`) para o giro e a inclinação fora de 90°.

use super::{Affine, Point, invert3, is_convex, quad_homography};
use crate::sprite::Rect;
use crate::transform::Image;

/// A caixa `[x0, x1) × [y0, y1)` cortada por `clip`; `Err` = vazia (o canto onde cai).
fn clip_box(
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
    clip: Option<Rect>,
) -> Result<(i64, i64, i64, i64), Rect> {
    let Some(c) = clip else {
        return Ok((x0, y0, x1, y1));
    };
    let (cx0, cy0) = (x0.max(i64::from(c.x)), y0.max(i64::from(c.y)));
    let cx1 = x1.min(i64::from(c.x) + i64::from(c.w));
    let cy1 = y1.min(i64::from(c.y) + i64::from(c.h));
    if cx1 <= cx0 || cy1 <= cy0 {
        return Err(Rect::new(cx0 as i32, cy0 as i32, 0, 0));
    }
    Ok((cx0, cy0, cx1, cy1))
}

/// **A *Fast Rotation* do Aseprite** (lei medida no Trial, `oracle_trial_rotation.json`): os
/// quatro cantos da moldura EXACTA (sem assentar) vão ao `floor` da célula; a fonte estica-se
/// sobre esse quadrilátero inteiro e cada célula de saída copia a célula da fonte sob o seu
/// centro — um empate exacto (centro sobre uma aresta da grelha da fonte) resolve-se como se o
/// centro estivesse um nada à ESQUERDA. Paralelogramo (o caso do giro e da inclinação em volta
/// do centro) = aritmética inteira exacta; outro quadrilátero = homografia em `f64`.
pub(super) fn fast_rotation(
    src: &Image,
    t: &Affine,
    transparent: u32,
    clip: Option<Rect>,
) -> Option<(Rect, Image)> {
    let (w, h) = (i64::from(src.w), i64::from(src.h));
    let q = trial_rotation(t, f64::from(src.w), f64::from(src.h))
        .quad(src.w.into(), src.h.into())
        .map(|p| (p.x.floor() as i64, p.y.floor() as i64));
    let x0 = q.iter().map(|p| p.0).min()?;
    let y0 = q.iter().map(|p| p.1).min()?;
    let x1 = q.iter().map(|p| p.0).max()?.max(x0 + 1);
    let y1 = q.iter().map(|p| p.1).max()?.max(y0 + 1);
    let (x0, y0, x1, y1) = match clip_box(x0, y0, x1, y1, clip) {
        Ok(b) => b,
        Err(r) => return Some((r, Image::new(0, 0, src.bpp, 0))),
    };
    let (bw, bh) = ((x1 - x0) as u32, (y1 - y0) as u32);
    let mut out = Image::new(bw, bh, src.bpp, transparent);
    let rect = Rect::new(x0 as i32, y0 as i32, bw, bh);
    let [p0, p1, p2, p3] = q;
    if p1.0 - p0.0 == p2.0 - p3.0 && p1.1 - p0.1 == p2.1 - p3.1 {
        // x = x0 + A·u/w + C·v/h, y = y0 + B·u/w + D·v/h; centros em meia célula (×2).
        let (a, b, c, d) = (p1.0 - p0.0, p1.1 - p0.1, p3.0 - p0.0, p3.1 - p0.1);
        let det = a * d - b * c;
        if det <= 0 {
            return Some((rect, out)); // a moldura cabe numa linha de células: nada a copiar
        }
        let den = 2 * det;
        // floor exacto; no empate, o passo para a esquerda desce se a derivada em x é positiva.
        let cell = |num: i64, dx: i64| {
            let f = num.div_euclid(den);
            if num.rem_euclid(den) == 0 && dx > 0 {
                f - 1
            } else {
                f
            }
        };
        for y in 0..i64::from(bh) {
            let ry = 2 * (y0 + y) + 1 - 2 * p0.1;
            for x in 0..i64::from(bw) {
                let rx = 2 * (x0 + x) + 1 - 2 * p0.0;
                let u = cell(w * (d * rx - c * ry), d);
                let v = cell(h * (a * ry - b * rx), -b);
                if (0..w).contains(&u) && (0..h).contains(&v) {
                    out.set(x as u32, y as u32, src.get(u as u32, v as u32));
                }
            }
        }
    } else {
        let qf = q.map(|(x, y)| Point {
            x: x as f64,
            y: y as f64,
        });
        if !is_convex(&qf) {
            return Some((rect, out));
        }
        let m = invert3(&quad_homography(w as f64, h as f64, &qf));
        for y in 0..i64::from(bh) {
            let oy = (y0 + y) as f64 + 0.5;
            for x in 0..i64::from(bw) {
                let ox = (x0 + x) as f64 + 0.5 - FAST_TIE_LEFT;
                let z = m[6] * ox + m[7] * oy + m[8];
                let sx = ((m[0] * ox + m[1] * oy + m[2]) / z).floor();
                let sy = ((m[3] * ox + m[4] * oy + m[5]) / z).floor();
                if sx >= 0.0 && sy >= 0.0 && sx < w as f64 && sy < h as f64 {
                    out.set(x as u32, y as u32, src.get(sx as u32, sy as u32));
                }
            }
        }
    }
    Some((rect, out))
}

/// O giro como o Trial o guarda: cosseno e seno com 16 bits de fracção e a magnitude arredondada
/// PARA CIMA (0 e ±1 ficam exactos); o centro da fonte fica onde estava. Só decide quando um canto
/// cai a menos de ~1e-5 de um inteiro (o r86 a 53°, −37°, −127° e 143° do oráculo).
fn trial_rotation(t: &Affine, w: f64, h: f64) -> Affine {
    let r = t.a.hypot(t.b);
    if r <= 0.0 {
        return *t;
    }
    let (c, s) = (t.a / r, t.b / r);
    let up = |v: f64| {
        let k = v * TRIAL_TRIG_ONE;
        (if k > 0.0 { k.ceil() } else { k.floor() }) / TRIAL_TRIG_ONE
    };
    let (c2, s2) = (up(c), up(s));
    if c2 == c && s2 == s {
        return *t;
    }
    // Q = R'·Rᵀ leva o giro exacto ao guardado; a inclinação e a escala ficam.
    let (q00, q01) = (c2 * c + s2 * s, c2 * s - s2 * c);
    let (q10, q11) = (s2 * c - c2 * s, s2 * s + c2 * c);
    let lin = Affine {
        a: q00 * t.a + q01 * t.b,
        b: q10 * t.a + q11 * t.b,
        c: q00 * t.c + q01 * t.d,
        d: q10 * t.c + q11 * t.d,
        e: 0.0,
        f: 0.0,
    };
    t.keep_point(&lin, w / 2.0, h / 2.0)
}

/// A unidade do seno/cosseno do Trial (16 bits de fracção; 8..16 bits dão o mesmo no oráculo, 20
/// já não).
const TRIAL_TRIG_ONE: f64 = 65536.0;

/// O «um nada à esquerda» do empate, na homografia (bem abaixo de 1/(2·det) das fontes reais e
/// bem acima do ruído do `f64`).
const FAST_TIE_LEFT: f64 = 1e-9;
