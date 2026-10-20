//! ⭐ **A PONTA DESFIADA DAS CERDAS** (J2, `docs/Painter/Pinceis/03_MOTORES.md` §7 c; o knob
//! [`ph2d_wet_paint::tuning::Knob::BristleTipRagged`]). O pincel chato do Rebelle acaba o traço RECTO e DESFIADO (o
//! recto de 200 px: curva −0,02 · desfiado 6,5 px; a 40 px 0,00 · 1,6 — o desfiado ≈ 3,5 % do diâmetro, a todo
//! tamanho); o nosso acabava liso (desfiado 0,7). Cada FIO da W2 ([`ph2d_wet_paint::bristles_fios`]) é um pelo com o
//! seu DESVIO ao longo do traço ([`FiosNoDab::comprimento_do_fio`], o mesmo o traço inteiro): a pegada dele é a do
//! dab DESLOCADA desse desvio. Deslocar não muda quanto cada célula recebe no meio do traço — só as duas pontas
//! saem fio a fio. O RECTO a toda a largura é a lâmina RECTANGULAR (`dab_quina` 1 no preset do Oil Bristle).
//!
//! ⛔ Recusas MEDIDAS (03_MOTORES §7 c): cortar cada dab recto no referencial do TRAÇO (perpendicular à tangente) —
//! a meia-corda da lâmina fixa muda com o rumo, e a escada da curva do W3b subia a 2,62 (tecto 2,55); cada fio com o
//! seu COMPRIMENTO (um corte por fio) — massa diferente fio a fio, a escada da curva 2,68; a pegada em caixa no
//! referencial do traço, 3,4; a ESPESSURA da lâmina cortada a 0,6 do eixo menor (o além do Rebelle, 21/12 px a 200) —
//! tira pigmento por área: a rampa de pressão 0,66 → 0,59 de cobertura, o azul sobre o vermelho seco deixa ver o
//! vermelho (0,54 → 0,70) e a distância à origem na matriz 0,81 → 1,01.

use ph2d_wet_paint::bristles_fios::FiosNoDab;
use ph2d_wet_paint::brush::CellFn;

/// O desvio de um fio de desvio `1` em fracção da meia-corda da pegada ao longo do traço (pelo centro). MEDIDO
/// (03_MOTORES §7 c): o desfiado do recto de 200 px do Oil Bristle com o `bristleTipRagged` 0,7 · 0,9 lê 5,3–6,0 ·
/// 6,6–7,6 px (o Rebelle 6,4–6,5).
pub(super) const ALCANCE: f64 = 0.5;

/// A ponta das cerdas de UM dab ([`Self::nova`]).
pub(super) struct PontaDasCerdas {
    fios: FiosNoDab,
    rho: f64,
    /// O centro do dab (células).
    centro: [f64; 2],
    px_cel: f64,
    /// O desvio de um fio de desvio `1` (px): [`ALCANCE`] da meia-corda.
    meia: f64,
}

impl PontaDasCerdas {
    /// A ponta do dab de centro `centro` (células) com a silhueta `sil`, ou `None` sem a lei (`rho ≤ 0`) ou sem os
    /// fios — o dab de sempre, ao bit. `alcance_px` = até onde a silhueta pode ir do centro.
    pub(super) fn nova(
        fios: Option<FiosNoDab>,
        rho: f64,
        centro: [f64; 2],
        px_cel: f64,
        alcance_px: f64,
        sil: CellFn<'_>,
    ) -> Option<Self> {
        let fios = fios.filter(|_| rho > 0.0)?;
        let n = fios.normal();
        let t = [n[1], -n[0]];
        // A meia-corda pelo centro ao longo do traço: a silhueta é convexa, então a bissecção acha a fronteira.
        let dentro = |s: f64| {
            let c = |k: usize| (centro[k] + t[k] * s / px_cel).round() as i32;
            sil(c(0), c(1)) > 0.0
        };
        let (mut a, mut b) = (0.0, alcance_px.max(px_cel));
        for _ in 0..16 {
            let m = 0.5 * (a + b);
            if dentro(m) {
                a = m;
            } else {
                b = m;
            }
        }
        Some(Self {
            fios,
            rho,
            centro,
            px_cel,
            meia: ALCANCE * a.max(0.5 * px_cel),
        })
    }

    /// A silhueta `sil` na célula `(cx, cy)` com a ponta das cerdas: a do dab deslocada ao longo do traço pelo desvio
    /// do fio da célula — SUB-CÉLULA (a bilinear das quatro células em volta do ponto deslocado): o desvio em px não
    /// pode depender do tamanho da célula (arredondado a células inteiras, a 2 px por célula o pente lia 0,44 da
    /// altura a 1 px no lugar de 0,5).
    pub(super) fn recorta(&self, sil: CellFn<'_>, cx: i32, cy: i32) -> f64 {
        let d = [
            (f64::from(cx) - self.centro[0]) * self.px_cel,
            (f64::from(cy) - self.centro[1]) * self.px_cel,
        ];
        let n = self.fios.normal();
        let fio = self.fios.fio_no_ponto(d);
        let desvio = self.meia * (FiosNoDab::comprimento_do_fio(fio, self.rho) - 1.0);
        let t = [n[1], -n[0]];
        let q = [
            f64::from(cx) - t[0] * desvio / self.px_cel,
            f64::from(cy) - t[1] * desvio / self.px_cel,
        ];
        let (x0, y0) = (q[0].floor(), q[1].floor());
        let (fx, fy) = (q[0] - x0, q[1] - y0);
        let (x0, y0) = (x0 as i32, y0 as i32);
        let s = |x: i32, y: i32, w: f64| if w > 0.0 { w * sil(x, y) } else { 0.0 };
        s(x0, y0, (1.0 - fx) * (1.0 - fy))
            + s(x0 + 1, y0, fx * (1.0 - fy))
            + s(x0, y0 + 1, (1.0 - fx) * fy)
            + s(x0 + 1, y0 + 1, fx * fy)
    }
}
