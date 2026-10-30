//! ⭐ **A LADEIRA DO ÓLEO para a água** (Bug 2 do smoke do dono 30/10: «a água sobe a ladeira do óleo»). O nivelamento
//! corria só pela altura da ÁGUA (`film`) e o capilar pelo dente do papel; a altura do óleo assentado não entrava em
//! passe nenhum e a água espalhava-se por cima dele como por papel liso. A superfície que a água nivela é água + óleo: a
//! do pé da parede corre para longe dela, a que está em cima do óleo desce. Só a tinta ASSENTADA com o Real Oil tem
//! altura (`sett × espessura × Depth·Real Oil`, a massa do campo de `oleo_real`); sem o campo do óleo na grade, `None`
//! e o fluxo de sempre, ao bit.

use crate::grid::Grid;

/// A altura de água (`film`) que uma unidade de massa de óleo assentado ergue. MEDIDO (`diag_a_agua_sobe_a_ladeira_do_oleo`):
/// o miolo de um Dabber tem `~3000` de massa e a água de um Wet Flat `film` `1–4`; a parede tem de ganhar da água.
pub(crate) const FILME_POR_MASSA: f64 = 1.0 / 1000.0;

/// A altura do óleo nas linhas FINAS `r0..=r1` (as que o passe de fluxo amostra), em unidades de `film`.
pub(crate) struct LadeiraDoOleo {
    r0: usize,
    s: usize,
    h: Vec<f32>,
}

impl LadeiraDoOleo {
    /// `None` sem o campo do óleo (nenhum traço com o Real Oil nesta grade).
    pub(crate) fn de(g: &Grid, r0: usize, r1: usize) -> Option<Self> {
        if g.oleo.is_empty() || r1 < r0 {
            return None;
        }
        let s = g.s;
        let r1 = r1.min(g.cells / s - 1);
        let mut h = vec![0f32; (r1 + 1 - r0) * s];
        for y in r0..=r1 {
            for x in 0..s {
                let i = y * s + x;
                let m = f64::from(g.sett[i]);
                if m <= 0.0 {
                    continue;
                }
                let k = crate::grid::oleo_celula::ganho(&g.oleo, i);
                if k > 0.0 {
                    h[(y - r0) * s + x] =
                        (m * crate::grid::espessura::factor(g, i) * k * FILME_POR_MASSA) as f32;
                }
            }
        }
        Some(Self { r0, s, h })
    }

    /// A altura na célula fina `fi` (`0` fora das linhas).
    #[inline]
    pub(crate) fn em(&self, fi: usize) -> f64 {
        fi.checked_sub(self.r0 * self.s)
            .and_then(|k| self.h.get(k))
            .map_or(0.0, |v| f64::from(*v))
    }
}

/// A ladeira das linhas finas que o passe de fluxo das linhas de FLUXO `by0..=by1` amostra — só quando o pincel na mão
/// NÃO tem o Real Oil (`oleo_na_mao`): é a água de um traço novo que não sobe o óleo de antes; o óleo que se pinta corre
/// como sempre (MEDIDO: com a ladeira também nele, as 10 miniaturas Oil/* mudavam 82–230 níveis).
pub(crate) fn do_passe(g: &Grid, by0: i32, by1: i32, oleo_na_mao: bool) -> Option<LadeiraDoOleo> {
    if g.oleo.is_empty() || oleo_na_mao {
        return None;
    }
    let (gh, rf) = (g.h as i32, g.flow.rf);
    let r0 = crate::flow::flow_probe(by0 - 1, gh, rf).max(0) as usize;
    let r1 = crate::flow::flow_probe(by1 + 1, gh, rf).max(0) as usize;
    LadeiraDoOleo::de(g, r0, r1)
}
