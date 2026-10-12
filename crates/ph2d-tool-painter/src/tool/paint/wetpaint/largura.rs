//! **A LARGURA PROMETIDA (W2)** — o número do painel é a largura que se mede, ± 2 px (decisão do dono,
//! 05 §4.1), em todas as mídias — a largura a ¼ do máximo de cada coluna, a que se vê (`sonda_gestos::
//! largura_que_se_ve`). O motor não a dá por si: ela é uma função da forma, da carga e da água (05 §1.1;
//! a lei medida está no 05 §4b, W2). Por isso
//! cada preset leva a sua tabela MEDIDA raio do motor → largura (`width_table`, gerada pela sonda da
//! fábrica com a água assentada) e esta porta a INVERTE: o traço nasce com o raio do motor cuja largura é
//! `2 · raio` (`dab_spec.rs::authored_spec`). O anel do cursor continua no raio pedido — é ele a promessa.

/// A tabela do **Wet Round** de fábrica, limpa — a lei do pincel que nasce sem preset (o `WetKnobs::DEFAULT`
/// é o Wet Round). É a MESMA do ficheiro (`a_lei_de_omissao_e_a_do_wet_round`); o gerador imprime-a.
pub(crate) const LARGURA_DO_WET_ROUND: [[f64; 2]; 51] = [
    [3.5, 4.0],
    [4.3, 5.0],
    [5.9, 6.0],
    [6.5, 7.0],
    [7.2, 9.0],
    [8.9, 10.0],
    [9.9, 11.0],
    [12.2, 12.0],
    [13.5, 13.0],
    [15.0, 16.0],
    [16.7, 18.0],
    [18.5, 20.0],
    [20.6, 24.0],
    [22.8, 27.0],
    [25.4, 30.0],
    [28.2, 32.0],
    [31.3, 36.0],
    [34.7, 41.0],
    [38.5, 46.0],
    [42.8, 53.0],
    [47.5, 58.0],
    [52.8, 65.0],
    [58.6, 71.0],
    [65.1, 78.0],
    [72.3, 85.0],
    [80.2, 93.0],
    [89.1, 106.0],
    [98.9, 115.0],
    [109.9, 126.0],
    [122.0, 139.0],
    [135.4, 153.0],
    [150.4, 169.0],
    [167.0, 185.0],
    [185.4, 205.0],
    [205.9, 223.0],
    [228.7, 244.0],
    [253.9, 278.0],
    [281.9, 294.0],
    [313.1, 326.0],
    [347.6, 360.0],
    [386.0, 392.0],
    [428.6, 434.0],
    [475.9, 488.0],
    [528.5, 543.0],
    [586.8, 600.0],
    [651.6, 676.0],
    [723.6, 748.0],
    [803.4, 817.0],
    [892.1, 908.0],
    [990.6, 1032.0],
    [1100.0, 1136.0],
];

/// A lei raio do motor → largura medida de um pincel, monótona.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LeiDaLargura {
    /// `(raio do motor, largura)`, ambos a crescer estritamente — em `f64`, os números do ficheiro tal
    /// como vieram (um `f32` alargado escreve-se com 17 algarismos e não volta igual pelo JSON).
    pontos: Vec<(f64, f64)>,
}

impl LeiDaLargura {
    /// A lei de uma tabela medida. Os pontos sem traço (largura 0) e os que não crescem saem: a sonda
    /// lê a mediana das colunas, e um degrau da franja das cerdas pode recuar 1 px. `None` = tabela sem
    /// dois pontos úteis.
    pub(crate) fn de_tabela(t: &[[f64; 2]]) -> Option<Self> {
        let mut pontos: Vec<(f64, f64)> = Vec::with_capacity(t.len());
        for &[r, l] in t {
            if l <= 0.0 || !r.is_finite() || !l.is_finite() {
                continue;
            }
            if pontos.last().is_none_or(|&(r0, l0)| r > r0 && l > l0) {
                pontos.push((r, l));
            }
        }
        (pontos.len() >= 2).then_some(Self { pontos })
    }

    /// A lei de quem MEDE: o raio do motor é `escala ×` o pedido (a «largura» `2 · raio` da tabela).
    pub(crate) fn escala(escala: f32) -> Self {
        let k = f64::from(escala);
        Self {
            pontos: vec![(k, 2.0), (k * 8192.0, 16384.0)],
        }
    }

    /// A lei de um pincel sem tabela: a do Wet Round de fábrica ([`LARGURA_DO_WET_ROUND`]).
    pub(crate) fn de_omissao() -> Self {
        Self::de_tabela(&LARGURA_DO_WET_ROUND).expect("a tabela do Wet Round tem pontos")
    }

    /// Uma impressão da lei (FNV-1a dos bits dos pontos) — a chave do preview pergunta «mudou?».
    pub(crate) fn chave(&self) -> u64 {
        self.pontos
            .iter()
            .flat_map(|&(r, l)| [r.to_bits(), l.to_bits()])
            .flat_map(u64::to_le_bytes)
            .fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
            })
    }

    /// A lei como o preset a guarda.
    pub(crate) fn tabela(&self) -> Vec<[f64; 2]> {
        self.pontos.iter().map(|&(r, l)| [r, l]).collect()
    }

    /// O raio do motor cuja largura medida é `2 · raio`. Fora da tabela, a razão do ponto da ponta.
    pub(crate) fn raio_do_motor(&self, raio: f32) -> f32 {
        self.raio_do_motor_f64(f64::from(raio)) as f32
    }

    fn raio_do_motor_f64(&self, raio: f64) -> f64 {
        let alvo = 2.0 * raio;
        let (r0, l0) = self.pontos[0];
        if alvo <= l0 {
            return r0 * alvo / l0;
        }
        for w in self.pontos.windows(2) {
            let ((ra, la), (rb, lb)) = (w[0], w[1]);
            if alvo <= lb {
                return ra + (rb - ra) * (alvo - la) / (lb - la);
            }
        }
        let (rn, ln) = self.pontos[self.pontos.len() - 1];
        rn * alvo / ln
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverte_a_tabela_e_salta_os_pontos_que_nao_crescem() {
        let lei = LeiDaLargura::de_tabela(&[
            [2.0, 0.0],
            [4.0, 4.0],
            [8.0, 10.0],
            [9.0, 9.0],
            [16.0, 20.0],
        ])
        .unwrap();
        assert_eq!(lei.pontos, vec![(4.0, 4.0), (8.0, 10.0), (16.0, 20.0)]);
        assert_eq!(lei.raio_do_motor(5.0), 8.0);
        assert_eq!(lei.raio_do_motor(7.5), 12.0);
        assert_eq!(lei.raio_do_motor(1.0), 2.0);
        assert_eq!(lei.raio_do_motor(20.0), 32.0);
        assert!(LeiDaLargura::de_tabela(&[[2.0, 0.0], [4.0, 3.0]]).is_none());
    }
}
