//! ⭐ **O par travado** (corrente, cadeado, «Keep proportions»): a proporção é um DADO do cadeado,
//! nunca a conta `novo / velho` sobre os valores vivos. Ela sobrevive a um eixo passar por zero:
//! X 2, Y 1, trava; X → 0 dá Y 0; X → 4 dá Y 2. O sinal faz parte dela (X −2, Y 1, X → 4 dá Y −2).
//!
//! Sem proporção para ler (um eixo, ou os dois, a zero ao travar) ela é 1:1, com o sinal do eixo
//! que o tem. Mudança vinda de fora (Desfazer, gizmo, Reset) entra em [`ParTravado::rever`]: passa a
//! ser a proporção, salvo se tiver um zero. Figma e Rive: `docs/CartoesUI/16_REFERENCIAS_POR_CARTAO.md`.

/// Abaixo disto o valor lê-se zero (o mesmo ε de `format_number`) e não define proporção.
const LE_SE_ZERO: f64 = 1e-6;
/// Folga relativa de «é o mesmo valor»: o documento guarda `f32` e o campo `f64`.
const MESMO_VALOR: f64 = 1e-6;

fn tem_grandeza(v: f64) -> bool {
    v.is_finite() && v.abs() >= LE_SE_ZERO
}

/// A proporção guardada e o par que o cadeado viu por último.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParTravado {
    razao: [f64; 2],
    visto: [f64; 2],
}

impl ParTravado {
    /// O cadeado fecha sobre o par `vivo`.
    #[must_use]
    pub fn ao_ligar(vivo: [f64; 2]) -> Self {
        let razao = match vivo.map(tem_grandeza) {
            [true, true] => vivo,
            [true, false] => [vivo[0].signum(), 1.0],
            [false, true] => [1.0, vivo[1].signum()],
            [false, false] => [1.0, 1.0],
        };
        Self { razao, visto: vivo }
    }

    /// O par que o cadeado viu por último.
    #[must_use]
    pub fn visto(&self) -> [f64; 2] {
        self.visto
    }

    /// O par vivo pode ter mudado por fora; com os dois eixos com grandeza, é a nova proporção.
    pub fn rever(&mut self, vivo: [f64; 2]) {
        let igual = |a: f64, b: f64| (a - b).abs() <= MESMO_VALOR * a.abs().max(b.abs());
        if igual(vivo[0], self.visto[0]) && igual(vivo[1], self.visto[1]) {
            return;
        }
        self.visto = vivo;
        if vivo.into_iter().all(tem_grandeza) {
            self.razao = vivo;
        }
    }

    /// O eixo `eixo` passou a valer `v`: devolve o valor do outro. O `+ 0.0` tira o sinal do zero.
    pub fn segue(&mut self, eixo: usize, v: f64) -> f64 {
        let (i, j) = (eixo.min(1), 1 - eixo.min(1));
        if !v.is_finite() {
            return self.visto[j];
        }
        let outro = v * self.razao[j] / self.razao[i] + 0.0;
        self.visto[i] = v;
        self.visto[j] = outro;
        outro
    }

    /// O espelho troca o sinal de UM eixo sem levar o outro: a proporção troca de sinal com ele,
    /// mesmo com o par a zero, onde o valor vivo não o mostra.
    pub fn espelhou(&mut self, eixo: usize) {
        let i = eixo.min(1);
        self.razao[i] = -self.razao[i];
        self.visto[i] = -self.visto[i];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caminha(t: &mut ParTravado, passos: &[(usize, f64, f64)]) {
        for &(eixo, v, outro) in passos {
            assert_eq!(t.segue(eixo, v), outro, "eixo {eixo} a {v}");
        }
    }

    #[test]
    fn um_eixo_a_zero_nao_perde_a_proporcao() {
        let mut t = ParTravado::ao_ligar([2.0, 1.0]);
        caminha(
            &mut t,
            &[(0, 0.0, 0.0), (0, 4.0, 2.0), (1, 0.0, 0.0), (1, 3.0, 6.0)],
        );
    }

    #[test]
    fn a_travessia_por_zero_leva_o_sinal_e_a_proporcao() {
        let mut t = ParTravado::ao_ligar([2.0, 1.0]);
        caminha(&mut t, &[(0, 1.0, 0.5), (0, 0.0, 0.0), (0, -2.0, -1.0)]);
        // Espelhado por sinal ao travar: a proporção é −2:1.
        let mut e = ParTravado::ao_ligar([-2.0, 1.0]);
        caminha(&mut e, &[(0, 0.0, 0.0), (0, 4.0, -2.0), (1, 3.0, -6.0)]);
    }

    #[test]
    fn sem_proporcao_para_ler_e_um_para_um_com_o_sinal() {
        caminha(&mut ParTravado::ao_ligar([0.0, 0.0]), &[(0, 3.0, 3.0)]);
        caminha(&mut ParTravado::ao_ligar([0.0, 5.0]), &[(1, 7.0, 7.0)]);
        caminha(&mut ParTravado::ao_ligar([0.0, -5.0]), &[(0, 2.0, -2.0)]);
        caminha(&mut ParTravado::ao_ligar([-5.0, 1e-9]), &[(1, 2.0, -2.0)]);
    }

    #[test]
    fn o_outro_eixo_nunca_e_zero_negativo() {
        let mut t = ParTravado::ao_ligar([2.0, -1.0]);
        assert!(t.segue(0, 0.0).is_sign_positive());
        assert!(t.segue(0, -0.0).is_sign_positive());
    }

    #[test]
    fn mudanca_de_fora_vira_proporcao_salvo_com_zero() {
        let mut t = ParTravado::ao_ligar([2.0, 1.0]);
        t.segue(0, 0.0);
        // Com um zero, a proporção fica (o estado da foto do dono: X 0, Y 9).
        t.rever([0.0, 9.0]);
        caminha(&mut t, &[(0, 4.0, 2.0)]);
        // O Desfazer devolve outro par: ele manda.
        t.rever([3.0, 1.0]);
        caminha(&mut t, &[(0, 6.0, 2.0)]);
        // A volta do `f32` do documento não é mudança: a razão de `f64` fica.
        let mut p = ParTravado::ao_ligar([1.0, 3.0]);
        p.segue(0, 0.1);
        let [a, b] = p.visto().map(|v| f64::from(v as f32));
        p.rever([a, b]);
        assert_eq!(p.segue(0, 1.0), 3.0);
    }

    #[test]
    fn o_espelho_troca_o_sinal_mesmo_com_o_par_a_zero() {
        let mut t = ParTravado::ao_ligar([2.0, 1.0]);
        t.segue(0, 0.0);
        t.espelhou(0);
        caminha(&mut t, &[(0, 4.0, -2.0)]);
    }

    #[test]
    fn um_valor_nao_finito_nao_entra() {
        let mut t = ParTravado::ao_ligar([2.0, 1.0]);
        assert_eq!(t.segue(0, f64::NAN), 1.0);
        assert_eq!(t.visto(), [2.0, 1.0]);
    }
}
