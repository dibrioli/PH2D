//! ⭐ **O BORRÃO DA ORLA MOLHADA** (bug #58, 4.ª volta; o *Wet Edges* do Procreate, *«soften and blur the edges of
//! your brushstrokes to mimic pigment bleeding into paper»*): a máscara visível do plano, pré-multiplicada, borrada
//! por um gaussiano separável de `σ = molhada · W` (isotrópico; o alcance [`raio`] `= ⌈3σ⌉` entra na margem do lote e
//! na folga da janela, logo o que sangra para o papel está sempre dentro do que o lote recompõe).
//! `TracoComposto::compoe_molhado` mistura-o com o plano pelo peso `e` da orla — o miolo fica intacto. Com `σ = W/3`
//! (o alcance dentro de `W`) o Glaze Round de fábrica (`W` 7, borda macia de ~20 px) mudava ≤ 4/255 a Wet 0,8: o
//! botão lia-se morto.
//!
//! ⚠️ **A janela não muda um bit:** cada soma corre os mesmos toques pela mesma ordem; um toque fora da janela está
//! fora da TELA (a janela tem a folga `raio + 1`) e vale zero.

use super::Region;
use rayon::prelude::*;

/// A partir de quantos texels o borrão corre no pool (o limiar da orla).
const EM_PARALELO_DESDE: usize = 16_384;

/// O `σ` (px) do borrão para a molhada `m` e a largura `w` da orla.
#[must_use]
pub(super) fn sigma(m: f32, w: f32) -> f32 {
    m.clamp(0.0, 1.0) * w
}

/// O alcance (px) do borrão: o raio do núcleo, `⌈3σ⌉`.
#[must_use]
pub(super) fn raio(m: f32, w: f32) -> u32 {
    (3.0 * sigma(m, w)).ceil().max(0.0) as u32
}

/// O núcleo gaussiano normalizado de `σ`, com raio `⌈3σ⌉`.
fn nucleo(sigma: f32) -> Vec<f32> {
    let raio = (3.0 * sigma).ceil().max(0.0) as i32;
    let mut g: Vec<f32> = (-raio..=raio)
        .map(|k| {
            if sigma > 0.0 {
                (-(k * k) as f32 / (2.0 * sigma * sigma)).exp()
            } else {
                f32::from(u8::from(k == 0))
            }
        })
        .collect();
    let soma: f32 = g.iter().sum();
    g.iter_mut().for_each(|x| *x /= soma);
    g
}

/// ⭐ **O borrão em `r`** (linha a linha, `r.w × r.h`): `pre` é o plano visível pré-multiplicado na janela `jan`
/// (`jan.w × jan.h`, coordenadas da tela), que contém `r` com a folga da orla; `e` é a orla em `r`.
///
/// Só se borra onde a composição o lê: na orla (`e > 0`) e no papel a menos de um raio da tinta; o resto fica a zero
/// (o fundo do traço tem peso `0`, e o papel longe da tinta borra a zero exacto) — o miolo de um pincel grande não paga.
#[must_use]
pub(super) fn borrao_da_orla(
    pre: &[[f32; 4]],
    jan: Region,
    r: Region,
    sigma: f32,
    e: &[f32],
) -> Vec<[f32; 4]> {
    let (jw, jh) = (jan.w as usize, jan.h as usize);
    debug_assert_eq!(pre.len(), jw * jh);
    let g = nucleo(sigma);
    let raio = (g.len() / 2) as i64;
    let (rw, rh) = (r.w as usize, r.h as usize);
    if rw == 0 || rh == 0 {
        return Vec::new();
    }
    // A passada horizontal nas linhas de `r` alargadas pelo raio (presas à janela), só nas colunas de `r`.
    let (lx0, ly0) = (i64::from(r.x - jan.x), i64::from(r.y - jan.y));
    let hy0 = (ly0 - raio).max(0) as usize;
    let hy1 = ((ly0 + rh as i64 + raio) as usize).min(jh);
    let precisa = precisa(pre, jan, r, raio, e);
    // A coluna de `precisa` acumulada: a passada horizontal só corre onde um texel a menos de um raio a lê.
    let mut col = vec![0u32; (rh + 1) * rw];
    for y in 0..rh {
        for x in 0..rw {
            col[(y + 1) * rw + x] = col[y * rw + x] + u32::from(precisa[y * rw + x]);
        }
    }
    let lida = |hy: usize, dx: usize| {
        let y = hy as i64 - ly0;
        let a = (y - raio).clamp(0, rh as i64) as usize;
        let b = (y + raio + 1).clamp(0, rh as i64) as usize;
        col[b * rw + dx] > col[a * rw + dx]
    };
    let mut hor = vec![[0.0f32; 4]; (hy1 - hy0) * rw];
    let linha_h = |(dy, l): (usize, &mut [[f32; 4]])| {
        let row = &pre[(hy0 + dy) * jw..(hy0 + dy + 1) * jw];
        for (dx, o) in l.iter_mut().enumerate() {
            if !lida(hy0 + dy, dx) {
                continue;
            }
            let cx = lx0 + dx as i64;
            let mut s = [0.0f32; 4];
            for (k, &gk) in g.iter().enumerate() {
                let x = cx + k as i64 - raio;
                if x >= 0 && (x as usize) < jw {
                    let p = row[x as usize];
                    for c in 0..4 {
                        s[c] += gk * p[c];
                    }
                }
            }
            *o = s;
        }
    };
    let em_paralelo = rw * rh >= EM_PARALELO_DESDE;
    por_linha(&mut hor, rw, em_paralelo, linha_h);
    let mut out = vec![[0.0f32; 4]; rw * rh];
    let linha_v = |(dy, l): (usize, &mut [[f32; 4]])| {
        let cy = ly0 + dy as i64;
        for (dx, o) in l.iter_mut().enumerate() {
            if !precisa[dy * rw + dx] {
                continue;
            }
            let mut s = [0.0f32; 4];
            for (k, &gk) in g.iter().enumerate() {
                let y = cy + k as i64 - raio;
                if y >= hy0 as i64 && (y as usize) < hy1 {
                    let p = hor[(y as usize - hy0) * rw + dx];
                    for c in 0..4 {
                        s[c] += gk * p[c];
                    }
                }
            }
            *o = s;
        }
    };
    por_linha(&mut out, rw, em_paralelo, linha_v);
    out
}

/// Os texels de `r` cujo borrão a composição lê: a orla da tinta (`e > 0`) e o papel com tinta a menos de `raio`
/// (a contagem da tinta numa tabela de áreas somadas da janela).
fn precisa(pre: &[[f32; 4]], jan: Region, r: Region, raio: i64, e: &[f32]) -> Vec<bool> {
    let (jw, jh) = (jan.w as usize, jan.h as usize);
    let mut sat = vec![0u32; (jw + 1) * (jh + 1)];
    for y in 0..jh {
        let mut linha = 0u32;
        for x in 0..jw {
            linha += u32::from(pre[y * jw + x][3] > 0.0);
            sat[(y + 1) * (jw + 1) + x + 1] = sat[y * (jw + 1) + x + 1] + linha;
        }
    }
    let (lx0, ly0) = (i64::from(r.x - jan.x), i64::from(r.y - jan.y));
    let presa = |v: i64, n: usize| v.clamp(0, n as i64) as usize;
    let (rw, rh) = (r.w as usize, r.h as usize);
    let mut out = vec![false; rw * rh];
    for dy in 0..rh {
        for dx in 0..rw {
            let (x, y) = (lx0 + dx as i64, ly0 + dy as i64);
            out[dy * rw + dx] = if pre[y as usize * jw + x as usize][3] > 0.0 {
                e[dy * rw + dx] > 0.0
            } else {
                let (x0, x1) = (presa(x - raio, jw), presa(x + raio + 1, jw));
                let (y0, y1) = (presa(y - raio, jh), presa(y + raio + 1, jh));
                let w1 = jw + 1;
                sat[y1 * w1 + x1] + sat[y0 * w1 + x0] > sat[y0 * w1 + x1] + sat[y1 * w1 + x0]
            };
        }
    }
    out
}

/// Corre `f` em cada linha de `buf` (largura `rw`), no pool se `em_paralelo`.
fn por_linha<F>(buf: &mut [[f32; 4]], rw: usize, em_paralelo: bool, f: F)
where
    F: Fn((usize, &mut [[f32; 4]])) + Sync + Send,
{
    if em_paralelo {
        buf.par_chunks_mut(rw).enumerate().for_each(f);
    } else {
        buf.chunks_mut(rw).enumerate().for_each(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **A janela não muda um bit** e o núcleo é normalizado: um ponto borrado soma 1, e qualquer recorte com a
    /// folga dá o que a tela inteira dá.
    #[test]
    fn qualquer_janela_da_o_borrao_da_tela() {
        const L: u32 = 96;
        let tela = Region {
            x: 0,
            y: 0,
            w: L,
            h: L,
        };
        let pre: Vec<[f32; 4]> = (0..L * L)
            .map(|i| {
                let (x, y) = ((i % L) as f32, (i / L) as f32);
                let a = ((30.0 - (x - 40.0).hypot(y - 50.0)) * 0.1).clamp(0.0, 0.6);
                [a * 0.8, a * 0.1, a * 0.2, a]
            })
            .collect();
        let s = sigma(0.8, 9.0);
        // A orla cheia em todo texel: o borrão lê-se em todo lado.
        let um = vec![1.0f32; (L * L) as usize];
        let tudo = borrao_da_orla(&pre, tela, tela, s, &um);
        let mut ponto = vec![[0.0f32; 4]; (L * L) as usize];
        ponto[(48 * L + 48) as usize] = [1.0; 4];
        let soma: f32 = borrao_da_orla(&ponto, tela, tela, s, &um)
            .iter()
            .map(|p| p[3])
            .sum();
        assert!((soma - 1.0).abs() < 1e-5, "o núcleo soma {soma}");
        let folga = raio(0.8, 9.0) + 1;
        for (x, y, w, h) in [(10, 12, 30, 20), (0, 0, 5, 96), (60, 70, 36, 26)] {
            let r = Region { x, y, w, h };
            let x0 = x.saturating_sub(folga);
            let y0 = y.saturating_sub(folga);
            let jan = Region {
                x: x0,
                y: y0,
                w: (x + w + folga).min(L) - x0,
                h: (y + h + folga).min(L) - y0,
            };
            let pj: Vec<[f32; 4]> = (0..jan.h)
                .flat_map(|jy| (0..jan.w).map(move |jx| ((jan.y + jy) * L + jan.x + jx) as usize))
                .map(|i| pre[i])
                .collect();
            let b = borrao_da_orla(&pj, jan, r, s, &vec![1.0; (w * h) as usize]);
            for yy in 0..h {
                for xx in 0..w {
                    let a = b[(yy * w + xx) as usize];
                    let t = tudo[((y + yy) * L + x + xx) as usize];
                    assert!(
                        a.iter().zip(t).all(|(p, q)| p.to_bits() == q.to_bits()),
                        "({xx}, {yy})"
                    );
                }
            }
        }
    }
}
