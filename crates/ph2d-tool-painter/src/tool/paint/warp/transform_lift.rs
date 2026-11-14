//! B06.A4 — o levante do Transform sem passar a tela inteira. Restrito a uma seleção, o laço corre só
//! na caixa dela (fora, a máscara é zero: patch transparente, base = origem, cobertura toda na base);
//! a camada inteira não copia o patch (ele É a origem: `a·1 = a`) e só a base zera o alfa. Faixas
//! de linhas disjuntas, a régua 2^17 da cor; oráculo serial em `transform_lift_tests.rs`.

use super::super::Region;
use std::sync::Arc;

type Caixa = Option<(u32, u32, u32, u32)>;

/// A caixa dos texels com máscara > 0 (`None` = nenhum). Além do fim da máscara conta zero (como o
/// `selection_coverage_at`).
pub(in crate::tool::paint) fn mask_bbox(mask: &[u8], w: u32, h: u32) -> Option<Region> {
    let wu = w as usize;
    let mut cx: Caixa = None;
    for y in 0..h {
        let a = (y as usize * wu).min(mask.len());
        let row = &mask[a..(a + wu).min(mask.len())];
        if row.iter().fold(0u8, |o, &v| o | v) == 0 {
            continue;
        }
        let x0 = row.iter().position(|&v| v != 0).unwrap_or(0) as u32;
        let x1 = row.iter().rposition(|&v| v != 0).unwrap_or(0) as u32;
        cx = juntar(cx, Some((x0, y, x1, y)));
    }
    regiao(cx)
}

/// `(patch, base, src)` do levante da cor.
type Levante = (Arc<Vec<u8>>, Arc<Vec<u8>>, Region);

/// `(patch, base, src)` do levante da cor; `None` quando nada sobe. `sel` = (cobertura, caixa dela).
pub(super) fn lift_colour(
    cur: &Arc<Vec<u8>>,
    sel: Option<(&[u8], Region)>,
    w: u32,
    h: u32,
) -> Option<Levante> {
    let n = (w as usize) * (h as usize);
    let wu = w as usize;
    let mut base = cur.to_vec();
    let Some((mask, sb)) = sel else {
        // Camada inteira: `m = 1` em todo texel ⇒ o patch é a origem byte a byte; a base fica sem alfa.
        let mut nada = vec![(); n];
        let caixas = faixas2(&mut base, &mut nada, 4, 1, wu, 0, h, w, |y0, b, _| {
            let mut cx: Caixa = None;
            for ry in 0..(b.len() / (4 * wu)) as u32 {
                let y = y0 + ry;
                for x in 0..w {
                    let i = (ry as usize * wu + x as usize) * 4;
                    if b[i + 3] > 0 {
                        cx = juntar(cx, Some((x, y, x, y)));
                    }
                    b[i + 3] = 0;
                }
            }
            cx
        });
        let src = regiao(caixas.into_iter().fold(None, juntar))?;
        return Some((Arc::clone(cur), Arc::new(base), src));
    };
    let mut patch = vec![0u8; n * 4];
    let cur = &cur[..];
    let caixas = faixas2(
        &mut patch,
        &mut base,
        4,
        4,
        wu,
        sb.y,
        sb.h,
        sb.w,
        |y0, p, b| {
            let mut cx: Caixa = None;
            for ry in 0..(p.len() / (4 * wu)) as u32 {
                let y = y0 + ry;
                for x in sb.x..sb.x + sb.w {
                    let m = f32::from(mask.get((y * w + x) as usize).copied().unwrap_or(0)) / 255.0;
                    if m <= 0.0 {
                        continue;
                    }
                    let gi = ((y * w + x) as usize) * 4;
                    let i = (ry as usize * wu + x as usize) * 4;
                    let a = f32::from(cur[gi + 3]);
                    p[i] = cur[gi];
                    p[i + 1] = cur[gi + 1];
                    p[i + 2] = cur[gi + 2];
                    p[i + 3] = (a * m).round() as u8;
                    b[i + 3] = (a * (1.0 - m)).round() as u8;
                    if p[i + 3] > 0 {
                        cx = juntar(cx, Some((x, y, x, y)));
                    }
                }
            }
            cx
        },
    );
    let src = regiao(caixas.into_iter().fold(None, juntar))?;
    Some((Arc::new(patch), Arc::new(base), src))
}

/// A partição da cobertura do corpo pela MESMA máscara que partiu o alfa: patch `c·m`, base `c·(1−m)`.
pub(super) fn split_covers(
    covers: &Arc<Vec<u8>>,
    sel: Option<(&[u8], Region)>,
    w: u32,
) -> (Arc<Vec<u8>>, Option<Arc<Vec<u8>>>) {
    let Some((mask, sb)) = sel else {
        return (Arc::clone(covers), None);
    };
    let wu = w as usize;
    let mut cp = vec![0u8; covers.len()];
    let mut cb = covers.to_vec();
    let covers = &covers[..];
    faixas2(&mut cp, &mut cb, 1, 1, wu, sb.y, sb.h, sb.w, |y0, p, b| {
        for ry in 0..(p.len() / wu) as u32 {
            let y = y0 + ry;
            for x in sb.x..sb.x + sb.w {
                let gi = (y * w + x) as usize;
                let i = ry as usize * wu + x as usize;
                let mk = f32::from(mask.get(gi).copied().unwrap_or(0)) / 255.0;
                let c = f32::from(covers[gi]);
                p[i] = (c * mk).round().clamp(0.0, 255.0) as u8;
                b[i] = (c * (1.0 - mk)).round().clamp(0.0, 255.0) as u8;
            }
        }
    });
    (Arc::new(cp), Some(Arc::new(cb)))
}

fn juntar(a: Caixa, b: Caixa) -> Caixa {
    match (a, b) {
        (Some(a), Some(b)) => Some((a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))),
        (a, None) => a,
        (None, b) => b,
    }
}

fn regiao(c: Caixa) -> Option<Region> {
    c.map(|(x0, y0, x1, y1)| Region {
        x: x0,
        y: y0,
        w: x1 - x0 + 1,
        h: y1 - y0 + 1,
    })
}

/// `f(y0, linhas_a, linhas_b)` sobre as linhas `[y0, y0+rows)` de dois planos (`ka`/`kb` elementos por
/// texel, largura `wu`), em faixas disjuntas: o mesmo bit em qualquer nº de fios. Um plano só passa
/// um `[(); n]` como segundo (ZST: nada aloca).
#[allow(clippy::too_many_arguments)]
fn faixas2<A: Send, B: Send, R: Send>(
    a: &mut [A],
    b: &mut [B],
    ka: usize,
    kb: usize,
    wu: usize,
    y0: u32,
    rows: u32,
    cols: u32,
    f: impl Fn(u32, &mut [A], &mut [B]) -> R + Sync,
) -> Vec<R> {
    if rows == 0 || wu == 0 {
        return Vec::new();
    }
    let (ra, rb) = (ka * wu, kb * wu);
    let (s, e) = (y0 as usize, (y0 + rows) as usize);
    let (a, b) = (&mut a[s * ra..e * ra], &mut b[s * rb..e * rb]);
    let rpb = (rows as usize).div_ceil(n_faixas(rows, cols)).max(1);
    let jobs: Vec<_> = a.chunks_mut(rpb * ra).zip(b.chunks_mut(rpb * rb)).collect();
    let y_de = |i: usize| y0 + (i * rpb) as u32;
    if jobs.len() <= 1 {
        return jobs
            .into_iter()
            .enumerate()
            .map(|(i, (ca, cb))| f(y_de(i), ca, cb))
            .collect();
    }
    use rayon::prelude::*;
    jobs.into_par_iter()
        .enumerate()
        .map(|(i, (ca, cb))| f(y_de(i), ca, cb))
        .collect()
}

/// A régua da cor (`for_dirty_rows_parallel`): série abaixo de 2^17 texels.
fn n_faixas(rows: u32, cols: u32) -> usize {
    #[cfg(test)]
    if let Some(n) = tests::faixas_forcadas() {
        return n;
    }
    crate::compositor::parallel_threads(rows as usize * cols as usize, rows as usize)
}

#[cfg(test)]
#[path = "transform_lift_tests.rs"]
mod tests;
