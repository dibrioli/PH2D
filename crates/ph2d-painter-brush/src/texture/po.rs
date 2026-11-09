//! ⭐ **O PÓ POUSADO** (dono 09/11, o Pastel: «enche o papel paulatinamente em várias repassadas»): a tinta que a
//! camada já tinha no pen-down enche o dente do papel. O Height lê o papel a `t + fill·pó·(1 − t)`
//! ([`super::GrainLaw::papel_com_po`]): o pó é QUANTO do papel à volta já está coberto — o alfa de antes do traço
//! desfocado à escala de um vale ([`RAIO_DA_CAIXA`], três caixas ≈ gaussiana σ 5,5 px) — imutável durante o traço
//! (determinístico entre bandas, sem corrida com o que o próprio traço pousa). Medido 10/11: o desfoque de ~1 px só
//! engrossava os grãos já pintados (manchas de borda dura, os vales vazios); à escala do vale, os vales sobem juntos.

/// O plano do pó (`0..=255` por texel da tela, linhas de `largura`).
#[derive(Clone, Copy, Debug)]
pub struct PoDoPapel<'a> {
    /// O alfa desfocado da camada no pen-down.
    pub plano: &'a [u8],
    /// A largura da tela (texels).
    pub largura: usize,
}

impl PoDoPapel<'_> {
    /// O pó no texel `(px, py)` (`0..1`); fora da tela, nenhum.
    #[must_use]
    #[expect(clippy::cast_sign_loss, reason = "px, py ≥ 0 conferidos")]
    pub fn em(&self, px: i64, py: i64) -> f32 {
        if px < 0 || py < 0 || px as usize >= self.largura {
            return 0.0;
        }
        self.plano
            .get(py as usize * self.largura + px as usize)
            .map_or(0.0, |&v| f32::from(v) / 255.0)
    }
}

/// O raio de cada uma das três caixas do desfoque do pó (px): σ = √(3·((2r + 1)² − 1)/12) ≈ 5,5 px — a escala de um
/// vale do papel do Pastel (Noise 32 px de 3 oitavas).
pub const RAIO_DA_CAIXA: usize = 5;

/// A média em caixa de raio `r` de `src` para `dst` (o mesmo tamanho), com a borda RENORMALIZADA (a caixa cortada à
/// linha divide pelo que contém). Puro sobre uma linha — o chamador reparte as linhas entre os núcleos.
pub fn caixa(src: &[f32], dst: &mut [f32], r: usize) {
    let n = src.len();
    if n == 0 || dst.len() != n {
        return;
    }
    let mut soma: f32 = src[..=r.min(n - 1)].iter().sum();
    let mut lo = 0usize; // primeiro índice dentro da caixa
    let mut hi = r.min(n - 1); // último índice dentro da caixa
    for (x, d) in dst.iter_mut().enumerate() {
        *d = soma / (hi - lo + 1) as f32;
        if x + r + 1 < n {
            hi += 1;
            soma += src[hi];
        }
        if x >= r {
            soma -= src[lo];
            lo += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A caixa conserva a constante (a borda renormalizada não escurece), espalha um impulso por `2r + 1` e conserva a
    /// massa longe da borda; o pó lê `0` fora da tela.
    #[test]
    fn a_caixa_e_a_media_e_o_po_le_a_tela() {
        let mut out = vec![0.0; 20];
        caixa(&[0.7; 20], &mut out, 5);
        assert!(out.iter().all(|v| (v - 0.7).abs() < 1e-5), "{out:?}");
        let mut imp = vec![0.0; 30];
        imp[10] = 11.0;
        let mut out = vec![0.0; 30];
        caixa(&imp, &mut out, 5);
        assert!(
            out[5..=15].iter().all(|v| (v - 1.0).abs() < 1e-5),
            "{out:?}"
        );
        assert_eq!(out[4], 0.0);
        assert_eq!(out[16], 0.0);
        let plano = [255u8, 0, 0, 51];
        let p = PoDoPapel {
            plano: &plano,
            largura: 2,
        };
        assert_eq!(p.em(-1, 0), 0.0);
        assert_eq!(p.em(2, 0), 0.0);
        assert_eq!(p.em(0, 0), 1.0);
        assert!((p.em(1, 1) - 0.2).abs() < 1e-6);
        assert_eq!(p.em(0, 2), 0.0);
    }
}
