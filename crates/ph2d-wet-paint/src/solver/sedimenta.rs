//! **O pigmento DESCE mais que a água** (W1g, a opção «Heavy Edge»): com a tela inclinada, a fracção
//! `a·|ĝ_eixo|` do pigmento em suspensão de cada célula passa à vizinha de baixo (por eixo, a montante
//! → a jusante), pesada por quanto a água das duas se toca: `min(f_c, f_d) / f_c`. Onde a vizinha está
//! seca (`f_d = 0`) nada passa — o pigmento acumula-se na última água da orla de baixo e assenta lá.
//!
//! ⚠️ **Não move ÁGUA** e não tem decisão por célula: o peso é contínuo no filme (a W1f recusou toda a
//! trava célula a célula na entrada do seco, 05 §4b). Conservativo (gather: quem recebe lê o que a
//! fonte dá), independente de ordem, e `a = 0` ou `|g| = 0` não toca num byte.
use crate::colorops::km_weighted_mean_color;
use crate::grid::{AdvCell, Grid, span_x_of};
use crate::par::{self, Rows};

/// O tecto da fracção por passo: a soma dos dois eixos fica `≤ 0,45·√2 < 1` (a célula nunca dá o que
/// não tem).
pub const SEDIMENTA_MAX: f64 = 0.45;

/// A ligação de água entre a célula (filme `fc`) e a vizinha (`fd`): `0` com uma das duas seca.
#[inline]
fn liga(fc: f32, fd: f32) -> f64 {
    if fc <= 0.0 {
        return 0.0;
    }
    f64::from(fc.min(fd).max(0.0)) / f64::from(fc)
}

fn sentido(v: f64, n: f64) -> i32 {
    if v > 1e-12 * n {
        1
    } else if v < -1e-12 * n {
        -1
    } else {
        0
    }
}

/// A geometria de um passo: as fracções por eixo e o sentido de cada eixo.
#[derive(Clone, Copy)]
struct Passo {
    ax: f64,
    ay: f64,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    s: usize,
}

impl Passo {
    #[inline]
    fn dentro(&self, x: i32, y: i32) -> bool {
        x >= 1 && x <= self.w && y >= 1 && y <= self.h
    }

    #[inline]
    fn at(&self, x: i32, y: i32) -> usize {
        x as usize + y as usize * self.s
    }

    /// O que a célula (x, y) dá a cada vizinha de baixo, em fracção da sua massa.
    #[inline]
    fn da(&self, film: &[f32], x: i32, y: i32) -> (f64, f64) {
        let i = self.at(x, y);
        let fx = if self.dx != 0 && self.dentro(x + self.dx, y) {
            self.ax * liga(film[i], film[self.at(x + self.dx, y)])
        } else {
            0.0
        };
        let fy = if self.dy != 0 && self.dentro(x, y + self.dy) {
            self.ay * liga(film[i], film[self.at(x, y + self.dy)])
        } else {
            0.0
        };
        (fx, fy)
    }
}

/// Um passo: `a` = a fracção por passo ao longo de `ĝ`, `(gx, gy)` a gravidade do passo.
pub fn sedimenta(g: &mut Grid, a: f64, gx: f64, gy: f64, km: bool) {
    let n = (gx * gx + gy * gy).sqrt();
    if a <= 0.0 || n <= 0.0 || g.by0 > g.by1 {
        return;
    }
    let a = a.min(SEDIMENTA_MAX);
    let p = Passo {
        ax: a * (gx / n).abs(),
        ay: a * (gy / n).abs(),
        dx: sentido(gx, n),
        dy: sentido(gy, n),
        w: g.w as i32,
        h: g.h as i32,
        s: g.s,
    };
    g.scratch.ensure(g.cells);
    let s = g.s;
    let rows = (g.by1 - g.by0 + 1).max(0) as usize;
    let span = (g.bx1 - g.bx0 + 1).max(0) as usize;
    let mode = Rows::pick(rows, span, par::MIN_CELLS_ADVECT);
    let (by0, by1) = (g.by0, g.by1);
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    {
        let Grid {
            scratch,
            film,
            susp,
            susp_rgb,
            row_lo,
            row_hi,
            spans_enabled,
            bx0,
            bx1,
            ..
        } = &mut *g;
        let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
        let (film, susp, rgb): (&[f32], &[f32], &[[f32; 3]]) = (film, susp, susp_rgb);
        let (row_lo, row_hi): (&[i32], &[i32]) = (row_lo, row_hi);
        par::walk_rows(mode, &mut scratch.dst[band.clone()], s, |ri, row| {
            let y = by0 + ri as i32;
            let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
            let mut cores = [0.0f64; 12];
            let mut pesos = [0.0f64; 4];
            let mut km_out = [0.0f64; 3];
            for x in lo..=hi {
                let i = p.at(x, y);
                let (ox, oy) = p.da(film, x, y);
                let fica = f64::from(susp[i]) * (1.0 - ox - oy);
                // O que entra: das vizinhas de CIMA (a montante), a fracção que elas dão a esta.
                let mut ent = [(rgb[i], fica), ([0.0; 3], 0.0), ([0.0; 3], 0.0)];
                if p.dx != 0 && p.dentro(x - p.dx, y) {
                    let j = p.at(x - p.dx, y);
                    ent[1] = (rgb[j], f64::from(susp[j]) * p.da(film, x - p.dx, y).0);
                }
                if p.dy != 0 && p.dentro(x, y - p.dy) {
                    let j = p.at(x, y - p.dy);
                    ent[2] = (rgb[j], f64::from(susp[j]) * p.da(film, x, y - p.dy).1);
                }
                let total = fica + ent[1].1 + ent[2].1;
                let mut c = rgb[i];
                if ent[1].1 + ent[2].1 > 0.0 && total > 0.0 {
                    let inv = 1.0 / total;
                    let out = if km {
                        for (k, (cc, w)) in ent.iter().enumerate() {
                            cores[3 * k..3 * k + 3].copy_from_slice(&cc.map(f64::from));
                            pesos[k] = *w;
                        }
                        km_weighted_mean_color(&cores, &pesos, 3, inv, &mut km_out);
                        km_out
                    } else {
                        let mut o = [0.0; 3];
                        for (cc, w) in ent {
                            for (o, v) in o.iter_mut().zip(cc) {
                                *o += f64::from(v) * w;
                            }
                        }
                        o.map(|v| v * inv)
                    };
                    c = out.map(|v| v as f32);
                }
                row[x as usize] = AdvCell {
                    film: film[i],
                    susp: total as f32,
                    rgb: c,
                };
            }
        });
    }
    let Grid {
        scratch,
        susp,
        susp_rgb,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let dst: &[AdvCell] = &scratch.dst;
    par::walk_rows2(
        mode,
        &mut susp[band.clone()],
        &mut susp_rgb[band],
        s,
        |ri, sr, cr| {
            let y = by0 + ri as i32;
            let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
            if lo > hi {
                return;
            }
            let base = y as usize * s;
            for x in lo as usize..=hi as usize {
                sr[x] = dst[base + x].susp;
                cr[x] = dst[base + x].rgb;
            }
        },
    );
}
