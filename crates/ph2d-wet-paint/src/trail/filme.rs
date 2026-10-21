//! ⭐ **O FILME do traço** ([`crate::tuning::Knob::StrokeFilm`], J4 aguarela — `docs/Painter/Pinceis/exames/aguarela.md`
//! «Motor J4»): o pincel de lavagem deixa UMA camada de tinta por onde passa, como o Rebelle — cada célula fica com o
//! MÁXIMO que um dab do traço lhe trouxe, nunca a SOMA dos dabs que a cruzaram.
//!
//! Medido (sonda `familias::aguarela::sonda`): somar os dabs faz o miolo do traço — o recto de 200 px do Mop é a soma
//! de 44 discos a 8 células, e cada célula lê o tapete das cerdas no desvio ao centro do dab (a mesma linha do tapete
//! ao longo do traço inteiro): listras 0,065 e um padrão de período 8 a ±20 % logo no depósito; o perfil através é a
//! CORDA do disco (o domo: ombros 0,77 do centro); as voltas da espiral e o início do traço somam mais. O máximo é
//! plano por construção: a corda, a fase do tapete e a sobreposição deixam de contar.
//!
//! A espessura do filme é a do miolo da soma de antes (`2r ÷ passo` dabs), para o tom do preset não mudar de escala.
//! `0` = a soma de sempre, ao bit (o campo nem existe).

/// O máximo por célula do traço vivo (em unidades de depósito × dabs), da grade inteira; preguiçoso.
#[derive(Clone, Default)]
pub(crate) struct Filme {
    campo: Vec<f32>,
    /// O passo de uma linha da grade do campo.
    s: usize,
    /// As células escritas neste traço (`x1 < x0` = nenhuma): o que [`Self::limpa`] zera.
    caixa: (i32, i32, i32, i32),
}

impl Filme {
    /// O campo do tamanho da grade (`cells`), a zeros se ainda não existia.
    pub(crate) fn garante(&mut self, cells: usize, s: usize) {
        if self.campo.len() != cells || self.s != s {
            self.campo = vec![0.0; cells];
            self.s = s;
            self.caixa = (0, 0, -1, -1);
        }
    }

    /// Um traço novo: zera o que o anterior escreveu.
    pub(crate) fn limpa(&mut self) {
        let (x0, y0, x1, y1) = self.caixa;
        if x1 >= x0 && !self.campo.is_empty() {
            for y in y0..=y1 {
                let o = y as usize * self.s;
                self.campo[o + x0 as usize..=o + x1 as usize].fill(0.0);
            }
        }
        self.caixa = (0, 0, -1, -1);
    }

    /// As linhas `y0..=y1` do campo (stride `s`), uma fatia cada — o depósito por linhas parte-as como parte o `wet`.
    pub(crate) fn linhas(&mut self, (x0, y0, x1, y1): (i32, i32, i32, i32)) -> std::slice::ChunksMut<'_, f32> {
        let s = self.s;
        self.caixa = if self.caixa.2 < self.caixa.0 {
            (x0, y0, x1, y1)
        } else {
            (self.caixa.0.min(x0), self.caixa.1.min(y0), self.caixa.2.max(x1), self.caixa.3.max(y1))
        };
        self.campo[y0 as usize * s..(y1 as usize + 1) * s].chunks_mut(s)
    }
}

/// O que um dab ACRESCENTA a uma célula do filme: `alvo` acima do máximo guardado (e o máximo sobe), ou nada.
#[inline]
#[must_use]
pub(crate) fn acrescenta(guardado: &mut f32, alvo: f64) -> f64 {
    let m = f64::from(*guardado);
    if alvo <= m {
        return 0.0;
    }
    *guardado = alvo as f32;
    alvo - m
}

/// Quantos dabs o miolo de um traço recto empilha: `2r ÷ passo`, com o passo a corda entre dois dabs da lane (`0` no
/// primeiro dab: o sub-passo da cápsula) e nunca menos de uma célula.
#[inline]
#[must_use]
pub(crate) fn dabs_no_miolo(r: f64, corda: f64, capsula: f64) -> f64 {
    let passo = if corda > 0.0 { corda } else { capsula * 2.0 * r };
    (2.0 * r / passo.max(1.0)).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::{Filme, acrescenta, dabs_no_miolo};

    /// ⭐ O filme guarda o MÁXIMO: um dab mais fraco por cima não acrescenta nada, um mais forte só a diferença.
    #[test]
    fn o_filme_acrescenta_so_o_que_passa_do_maximo() {
        let mut m = 0.0f32;
        assert_eq!(acrescenta(&mut m, 4.0), 4.0);
        assert_eq!(acrescenta(&mut m, 3.0), 0.0);
        assert_eq!(acrescenta(&mut m, 6.0), 2.0);
        assert_eq!(m, 6.0);
    }

    /// ⭐ Um traço novo começa sem filme: o anterior é zerado onde escreveu (e só lá).
    #[test]
    fn o_traco_novo_comeca_sem_filme() {
        let mut f = Filme::default();
        f.garante(10 * 8, 10);
        for l in f.linhas((2, 3, 5, 4)) {
            l[2..=5].fill(7.0);
        }
        f.limpa();
        assert!(f.campo.iter().all(|&v| v == 0.0));
        assert_eq!(f.caixa.2 < f.caixa.0, true);
    }

    /// O miolo de um recto empilha `2r ÷ passo` dabs; o 1.º dab (sem corda) lê o sub-passo da cápsula; nunca menos de 1.
    #[test]
    fn a_espessura_e_a_do_miolo_da_soma() {
        assert_eq!(dabs_no_miolo(100.0, 8.0, 0.025), 25.0);
        assert_eq!(dabs_no_miolo(100.0, 0.0, 0.025), 40.0);
        assert_eq!(dabs_no_miolo(2.0, 0.5, 0.025), 4.0);
        assert_eq!(dabs_no_miolo(0.3, 0.0, 0.025), 1.0);
    }
}
