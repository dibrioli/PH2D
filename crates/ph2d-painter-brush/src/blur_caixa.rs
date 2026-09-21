//! O núcleo de **TRÊS CAIXAS** do blur — o gémeo barato do binomial do irmão [`super::blur`].
//!
//! ⭐ Ele existe por ordem do dono (2026-09-20): *«veja se abaixando a qualidade do blur não fica
//! bem mais leve. Mas só no Blur do composite. O Blur como ferramenta isolada não deve ser
//! modificado.»* — logo ele é pedido por **um** sítio de toda a crate da ferramenta, o laço da
//! pilha, e há censo a afirmá-lo (`o_nucleo_de_caixa_e_do_blur_da_pilha_e_so_dele`).
//!
//! **A lei:** três passagens de caixa por somas correntes aproximam uma gaussiana, e as larguras
//! saem de **CASAMENTO DE VARIÂNCIA** com o binomial que substituem (`σ² = k/2`) — é isso que faz
//! dela a *mesma quantidade de borrão* e não um borrão mais fraco disfarçado de optimização. O
//! custo é `~6` ops/pixel **independente de `k`**, contra `2(2k+1)` taps.
//!
//! **MEDIDO** (`--release`, canvas `1024²`, traço de 720 px, **pareado**, `load 3,4`–`4,0`):
//!
//! | raio | binomial | caixa | ganho p50 (p10..p90) | pior byte | média |
//! |---|---|---|---|---|---|
//! | 12 | `4,03` | `4,09` | **`×0,98`** (`0,95`..`1,00`) | `1` | `0,001` |
//! | 24 | `8,84` | `7,04` | `×1,26` (`1,23`..`1,27`) | `3` | `0,017` |
//! | 48 | `24,21` | `13,31` | `×1,83` (`1,76`..`1,86`) | `1` | `0,001` |
//! | 96 | `84,23` | `26,28` | **`×3,19`** (`3,09`..`3,26`) | `1` | `0,007` |
//!
//! ⛔⛔ **A leitura anterior dizia `×1,21` no raio 12 e estava ERRADA** — ela era `min(B)/min(A)` de
//! corridas SEPARADAS a `load 15`. Pareada, ali não há ganho nenhum: três passagens com avental
//! próprio custam o que um binomial de lado `9` custa, e a caixa só se paga a partir do raio `~24`.
//!
//! ⚠️ Este módulo nasceu de um **CORTE** do `blur.rs` (`822` linhas contra o tecto de `700`), nunca
//! de uma entrada nova no `FILE_OVERAGE_OK`.

use super::blur::src_coord;

/// As três larguras de caixa cuja variância somada mais se aproxima da do binomial de raio `k`.
///
/// Uma caixa de largura ímpar `w` tem variância `(w² − 1)/12`; três delas somam `Σ(wᵢ² − 1)/12`, e
/// o alvo é `σ² = k/2` (a variância do binomial de raio `k`). A largura ideal comum sai de
/// `3(w² − 1)/12 = k/2` ⇒ `w = √(1 + 2k)`, que quase nunca é um ímpar inteiro — por isso as três
/// caixas são **duas larguras vizinhas misturadas**, e a mistura é escolhida por MEDIÇÃO (a que
/// minimiza o erro de variância), nunca por arredondamento.
#[must_use]
pub(crate) fn box_radii(k: usize) -> [usize; 3] {
    #[allow(clippy::cast_precision_loss)]
    let alvo = k as f32 / 2.0;
    let ideal = (1.0 + 2.0 * k as f32).sqrt();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mut wl = ideal.floor() as usize;
    if wl.is_multiple_of(2) {
        wl = wl.saturating_sub(1);
    }
    let wl = wl.max(1);
    let wu = wl + 2;
    let var = |w: usize| {
        #[allow(clippy::cast_precision_loss)]
        let w = w as f32;
        (w * w - 1.0) / 12.0
    };
    let mut melhor = (f32::MAX, 0usize);
    for m in 0..=3usize {
        #[allow(clippy::cast_precision_loss)]
        let v = (3 - m) as f32 * var(wl) + m as f32 * var(wu);
        let erro = (v - alvo).abs();
        if erro < melhor.0 {
            melhor = (erro, m);
        }
    }
    let m = melhor.1;
    std::array::from_fn(|i| if i < m { (wu - 1) / 2 } else { (wl - 1) / 2 })
}

/// Uma passagem de caixa HORIZONTAL por soma corrente: a saída perde `r` de cada lado.
fn caixa_h(src: &[[f32; 4]], w: usize, h: usize, r: usize) -> (Vec<[f32; 4]>, usize) {
    if r == 0 {
        return (src.to_vec(), w);
    }
    let ow = w - 2 * r;
    let lado = 2 * r + 1;
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / lado as f32;
    let mut out = vec![[0f32; 4]; ow * h];
    for j in 0..h {
        let linha = j * w;
        let mut acc = [0f32; 4];
        for i in 0..lado {
            let s = src[linha + i];
            for c in 0..4 {
                acc[c] += s[c];
            }
        }
        for i in 0..ow {
            let o = j * ow + i;
            for c in 0..4 {
                out[o][c] = acc[c] * inv;
            }
            if i + 1 < ow {
                let sai = src[linha + i];
                let entra = src[linha + i + lado];
                for c in 0..4 {
                    acc[c] += entra[c] - sai[c];
                }
            }
        }
    }
    (out, ow)
}

/// Uma passagem de caixa VERTICAL por soma corrente. ⚠️ O acumulador é uma LINHA inteira e desliza
/// para baixo — uma coluna de cada vez leria a memória com passo `w` e pagaria a cache.
fn caixa_v(src: &[[f32; 4]], w: usize, h: usize, r: usize) -> (Vec<[f32; 4]>, usize) {
    if r == 0 {
        return (src.to_vec(), h);
    }
    let oh = h - 2 * r;
    let lado = 2 * r + 1;
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / lado as f32;
    let mut out = vec![[0f32; 4]; w * oh];
    let mut acc = vec![[0f32; 4]; w];
    for j in 0..lado {
        for i in 0..w {
            let s = src[j * w + i];
            for c in 0..4 {
                acc[i][c] += s[c];
            }
        }
    }
    for j in 0..oh {
        for i in 0..w {
            for c in 0..4 {
                out[j * w + i][c] = acc[i][c] * inv;
            }
        }
        if j + 1 < oh {
            for i in 0..w {
                let sai = src[j * w + i];
                let entra = src[(j + lado) * w + i];
                for c in 0..4 {
                    acc[i][c] += entra[c] - sai[c];
                }
            }
        }
    }
    (out, oh)
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub(crate) fn blur_region_caixa(
    buf: &[u8],
    fw: i64,
    fh: i64,
    min_x: i64,
    min_y: i64,
    bw: usize,
    bh: usize,
    k: usize,
    wrap: [bool; 2],
) -> Vec<[f32; 4]> {
    let raios = box_radii(k);
    let r_total: usize = raios.iter().sum();
    let ap_w = bw + 2 * r_total;
    let ap_h = bh + 2 * r_total;
    let mut apron = vec![[0f32; 4]; ap_w * ap_h];
    for j in 0..ap_h {
        let sy = src_coord(min_y + j as i64 - r_total as i64, fh, wrap[1]);
        for i in 0..ap_w {
            let sx = src_coord(min_x + i as i64 - r_total as i64, fw, wrap[0]);
            let si = ((sy * fw + sx) * 4) as usize;
            let a = f32::from(buf[si + 3]);
            let af = a / 255.0;
            apron[j * ap_w + i] = [
                f32::from(buf[si]) * af,
                f32::from(buf[si + 1]) * af,
                f32::from(buf[si + 2]) * af,
                a,
            ];
        }
    }
    // Separável: as três caixas na horizontal, depois as três na vertical.
    let (mut cur, mut w, mut h) = (apron, ap_w, ap_h);
    for r in raios {
        let (n, nw) = caixa_h(&cur, w, h, r);
        cur = n;
        w = nw;
    }
    for r in raios {
        let (n, nh) = caixa_v(&cur, w, h, r);
        cur = n;
        h = nh;
    }
    debug_assert_eq!((w, h), (bw, bh));
    for p in &mut cur {
        let a = p[3];
        let inv = if a > 1e-4 { 255.0 / a } else { 0.0 };
        *p = [p[0] * inv, p[1] * inv, p[2] * inv, a];
    }
    cur
}

/// Blend the blurred region into `buf` over the footprint bbox, weighting each pixel by `weight(i, j)`
/// (the dab mask × strength; `0` skips). `dest = lerp(dest, blurred, w)` per straight channel — the
/// `IMB_BLEND_INTERPOLATE` composite Blender's soften uses. Shared with the canvas-fixed Grain path
/// ([`crate::blur_grain`]), which supplies a per-pixel silhouette × Grain weight closure.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blur::BLUR_KERNEL_MAX;

    /// **A caixa tripla tem a VARIÂNCIA do binomial que ela substitui** — é isso que faz dela a
    /// «mesma quantidade de borrão», e não um borrão mais fraco disfarçado de optimização.
    ///
    /// Um binomial de raio `k` tem `σ² = k/2`; três caixas de larguras `wᵢ` somam `Σ(wᵢ²−1)/12`. O
    /// erro tem de ficar dentro do degrau da própria grelha de larguras (elas são ímpares, logo a
    /// variância só toma valores discretos) — a barra é **meia largura de degrau**, derivada, e não
    /// um epsilon escolhido.
    #[test]
    fn a_caixa_tripla_tem_a_variancia_do_binomial() {
        for k in 1..=BLUR_KERNEL_MAX {
            let r = box_radii(k);
            let var: f32 = r
                .iter()
                .map(|&ri| {
                    let w = (2 * ri + 1) as f32;
                    (w * w - 1.0) / 12.0
                })
                .sum();
            let alvo = k as f32 / 2.0;
            // O degrau: trocar UMA caixa de largura `w` por `w+2` muda a variância em
            // `((w+2)² − w²)/12 = (4w + 4)/12`. Com `w ≈ √(1+2k)`, meia dessas.
            let w = (1.0 + 2.0 * k as f32).sqrt();
            let degrau = (4.0 * w + 4.0) / 12.0;
            assert!(
                (var - alvo).abs() <= degrau * 0.5 + 1e-3,
                "k={k}: variância {var:.3} contra o alvo {alvo:.3} (degrau {degrau:.3})"
            );
        }
    }

    /// **A caixa tripla é uma MÉDIA: sobre um campo constante ela devolve a constante.**
    ///
    /// ⚠️ Esta é a metade que apanha um erro de normalização ou de avental — os dois deixariam a
    /// régua da variância acima VERDE, porque ela mede só as larguras.
    ///
    /// ⛔⛔ **E o ALFA é obrigatório, não é zelo: uma MUTAÇÃO SOBREVIVEU sem ele.** Com a versão que
    /// varria só `0..3`, trocar `1/lado` por `1/(lado − ½)` — um ganho uniforme de `20 %` na média —
    /// passava VERDE, porque este é um borrão em espaço **premultiplicado** e o último passo
    /// un-premultiplica por `255/α`: *um ganho uniforme entra no numerador e no denominador e
    /// divide-se a si próprio*. O canal que não é dividido por nada é o **α**, e é só ele que vê a
    /// normalização. ⇒ *num borrão premultiplicado, um campo constante de RGB não é régua de
    /// normalização nenhuma.*
    #[test]
    fn a_caixa_tripla_preserva_um_campo_constante() {
        let (w, h) = (48u32, 48u32);
        let buf = vec![137u8; (w * h * 4) as usize];
        for k in [1usize, 4, 8, 32] {
            let out = blur_region_caixa(
                &buf,
                i64::from(w),
                i64::from(h),
                8,
                8,
                24,
                24,
                k,
                [false, false],
            );
            for p in &out {
                for v in &p[..3] {
                    assert!(
                        (v - 137.0).abs() < 0.6,
                        "k={k}: a caixa mudou um campo constante para {v:.2}"
                    );
                }
                assert!(
                    (p[3] - 137.0).abs() < 0.6,
                    "k={k}: a caixa não é uma MÉDIA — o alfa saiu {:.2} de 137",
                    p[3]
                );
            }
        }
    }
}
