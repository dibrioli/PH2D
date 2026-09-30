//! **O Rewet redissolve a tinta MOLHADA da sessão** (report do dono, 2026-09-29: *«o Rewet não
//! afeta a mancha de tinta quando a tinta está molhada»*; medido `2,6×` mais fraco que sobre a seca —
//! doc 44 §3).
//!
//! Sobre tinta SECA o Rewet do composite faz duas coisas com a tinta de baixo: DISSOLVE-a (a cor
//! dela, borrada pelo raio do Spread e pesada pela presença, tinge a lavagem) e MISTURA-a com o
//! pigmento novo pela lei da água. Ele lê só a base SECA da sessão, de propósito (a tinta molhada
//! vizinha seria contada duas vezes — o rectângulo que clareia, Enio 2026-07-09). A tinta molhada
//! vive no plano da cor da sessão; é aí que a mesma água a encontra, no depósito deste traço:
//!
//! * **dissolve** — o parceiro da mistura de cada texel deixa de ser só a tinta que estava NELE e
//!   passa a ser a da vizinhança dissolvida: o plano da sessão de ANTES deste traço, borrado pelo
//!   MESMO raio do Spread ([`raio_da_agua`], a porta que o composite também lê) e pela MESMA soma
//!   (`box_blur4`, presença e as três cores pesadas por ela), levado a `água` do caminho;
//! * **mistura** — pela lei da água ([`super::watercolor_mistura::deposita`] →
//!   [`super::watercolor_mistura::alvo_sobre_seco`]).
//!
//! ⭐ **Não depende da cadência:** o plano de ANTES do traço não muda durante ele (sem Smudge), logo o
//! borrado de um texel é o mesmo em qualquer dab que o calcule — e o depósito recompõe o texel a
//! partir dele em vez de o acumular. ⚠️ Com o Smudge ligado o `antes` é arrastado, e o borrado lê o
//! arrastado do fim do lote anterior.

use super::watercolor_field::box_blur4;
use super::watercolor_mistura::PlanosDaMistura;

/// **O raio da água** (px): o Spread do pincel, o mesmo número que o composite usa para dissolver a
/// tinta seca. Uma porta, dois leitores — escrito duas vezes, o molhado e o seco dissolveriam a
/// distâncias diferentes no dia em que um deles mudasse o tecto.
pub(super) fn raio_da_agua(brush: &ph2d_painter_brush::BrushSpec) -> usize {
    brush.edge_spread.round().clamp(0.0, 48.0) as usize // LITERAL-PX-OK: tecto histórico do Spread do composite
}

/// A tinta de antes do traço, dissolvida na vizinhança de um lote de dabs.
pub(super) struct Dissolucao {
    x0: usize,
    y0: usize,
    w: usize,
    /// Presença e as três cores pesadas por ela, borradas.
    campos: [Vec<f32>; 4],
    agua: f32,
}

impl Dissolucao {
    /// O parceiro da mistura no texel `(x, y)`: a tinta que lá estava (`antes`, recto) levada a
    /// `água` do caminho até à tinta dissolvida da vizinhança — em pré-multiplicado, porque o papel
    /// do plano é `0,0,0,0` e em recto a média com ele ESCURECE.
    pub(super) fn parceiro(&self, antes: [u8; 4], x: usize, y: usize) -> [u8; 4] {
        let i = (y - self.y0) * self.w + (x - self.x0);
        let p = self.campos[0][i].clamp(0.0, 1.0);
        let aa = f32::from(antes[3]) / 255.0;
        let na = aa + (p - aa) * self.agua;
        if na <= 0.0 {
            return [antes[0], antes[1], antes[2], 0];
        }
        let mut out = [0u8; 4];
        for c in 0..3 {
            let pd = f32::from(antes[c]) * aa;
            let ps = self.campos[c + 1][i]; // já pesada pela presença
            out[c] = ((pd + (ps - pd) * self.agua) / na)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
        out[3] = (na * 255.0).round().clamp(0.0, 255.0) as u8;
        out
    }
}

impl PlanosDaMistura {
    /// Dissolve a tinta de ANTES do traço à volta da caixa `[x0, y0, x1, y1)` de um lote de dabs: o
    /// plano de antes é o `antes` onde este traço já fotografou e o próprio `buf` onde ainda não
    /// tocou (ali o plano não mudou). Borrado por `raio`, na caixa alargada por ele.
    pub(super) fn dissolve(
        &self,
        buf: &[u8],
        (fw, fh): (usize, usize),
        [x0, y0, x1, y1]: [usize; 4],
        raio: usize,
        agua: f32,
    ) -> Dissolucao {
        let (dx0, dy0) = (x0.saturating_sub(raio), y0.saturating_sub(raio));
        let (dx1, dy1) = ((x1 + raio).min(fw), (y1 + raio).min(fh));
        let (w, h) = (dx1.saturating_sub(dx0), dy1.saturating_sub(dy0));
        let mut campos: [Vec<f32>; 4] = std::array::from_fn(|_| vec![0.0; w * h]);
        for y in dy0..dy1 {
            for x in dx0..dx1 {
                let g = y * fw + x;
                let px = if self.capturado[g] {
                    &self.antes[g * 4..g * 4 + 4]
                } else {
                    &buf[g * 4..g * 4 + 4]
                };
                let p = f32::from(px[3]) / 255.0;
                let i = (y - dy0) * w + (x - dx0);
                campos[0][i] = p;
                for c in 0..3 {
                    campos[c + 1][i] = f32::from(px[c]) * p;
                }
            }
        }
        let [a, b, c, d] = &campos;
        Dissolucao {
            x0: dx0,
            y0: dy0,
            w,
            campos: box_blur4([a, b, c, d], w, h, raio),
            agua,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Dissolucao;

    /// Diluir tinta com PAPEL muda-lhe o ALFA e nunca a COR — é o que o pré-multiplicado garante.
    /// ⚠️ A fixtura do produto não o via: ali o papel é `0,0,0,0` e a faixa é opaca, e nos dois a
    /// mistura recta e a pré-multiplicada coincidem; só um texel de alfa PARCIAL as separa (a orla
    /// anti-serrilhada de um traço), e em recto ele saía quase o DOBRO mais claro.
    #[test]
    fn diluir_com_papel_muda_o_alfa_e_nao_a_cor() {
        let papel = Dissolucao {
            x0: 0,
            y0: 0,
            w: 1,
            campos: [vec![0.0], vec![0.0], vec![0.0], vec![0.0]],
            agua: 0.5,
        };
        let meia = [60, 110, 240, 128];
        let p = papel.parceiro(meia, 0, 0);
        assert!(
            p[..3]
                .iter()
                .zip(&meia[..3])
                .all(|(a, b)| a.abs_diff(*b) <= 1),
            "a cor mudou ao diluir com papel: {p:?} contra {meia:?}"
        );
        assert!(
            p[3].abs_diff(64) <= 1,
            "o alfa tem de ir a metade: lê {}",
            p[3]
        );
    }
}
