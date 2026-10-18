//! **O RELEVO de cada fio** (W3, prompt 49; doc 49_medicao). Os fios do óleo do Rebelle não só pintam: uns
//! CONSTROEM a tinta (Add impasto, cristas) e outros ESCAVAM-na (Remove impasto, sulcos — o Bristle Flat dele
//! tem `particle_thin_remove_impasto_strength` 0,65), e os fios escuros da foto são a sombra deles. Aqui cada
//! fio da W2 ([`crate::bristles_fios`]) deixa na tinta a altura do seu ganho (a pressão própria e a mão leve),
//! com o sinal do seu tipo; a altura é o campo [`crate::grid::Grid::altura`], que a luz do Impasto acende.
//! [`Relevo::NEUTRO`] (altura `0`) = o hospedeiro nem chama a lei.

use crate::bristles_fios::Ponto;

const SEMENTE_TIPO: u32 = 0x6f69_0006;

/// Quanto um dab puxa a altura de uma célula para o alvo do fio, por unidade de cobertura da silhueta: o
/// pincel penteia a tinta, e o que fica é o que os últimos pelos deixaram (com o Spacing dos óleos cada
/// célula leva dezenas de dabs; a meia taxa o alvo chega em poucos).
pub const TAXA: f64 = 0.5;

/// Os knobs do relevo de um pincel (os do mapa `engine` do preset).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Relevo {
    /// A altura (px) que um fio de ganho 1 deixa (`0` = nenhum relevo).
    pub altura_px: f64,
    /// A fracção dos fios que escavam (`0` = todos constroem, `1` = todos escavam).
    pub escava: f64,
}

impl Relevo {
    pub const NEUTRO: Relevo = Relevo {
        altura_px: 0.0,
        escava: 0.5,
    };

    /// Sem altura: a lei de antes.
    #[must_use]
    pub fn neutro(&self) -> bool {
        self.altura_px == 0.0
    }

    /// O fio `fio` escava — o tipo é do fio (determinístico, a mesma faixa o traço inteiro).
    #[must_use]
    pub fn escava(&self, fio: i32) -> bool {
        crate::rng::hash2(fio, 0, SEMENTE_TIPO) < self.escava
    }

    /// A altura (px) que o fio do ponto `p` deixa sob o `contacto` (`0..1`, o raio do dab ÷ o nominal): o ganho
    /// dele vezes o contacto² (a mão leve deixa uma película mais fina E sulca menos fundo; medido no doc
    /// 49_medicao: o leve da rampa 6,61 → 6,32 sem mexer no forte), para cima se constrói e para baixo se escava.
    #[must_use]
    pub fn alvo(&self, p: &Ponto, contacto: f64) -> f64 {
        self.alvo_do_fio(p.fio, p.ganho, contacto)
    }

    /// [`Self::alvo`] do fio `fio` de ganho `ganho` (o hospedeiro tabela-o por fio, uma vez por dab).
    #[must_use]
    pub fn alvo_do_fio(&self, fio: i32, ganho: f64, contacto: f64) -> f64 {
        let c = contacto.clamp(0.0, 1.0);
        let h = self.altura_px * ganho * c * c;
        if self.escava(fio) { -h } else { h }
    }
}

/// O passo de uma célula: a altura `h` puxada para o `alvo` pela cobertura `w` (`0..1`) da silhueta do dab.
#[must_use]
pub fn penteia(h: f32, alvo: f64, w: f64) -> f32 {
    let k = (TAXA * w).clamp(0.0, 1.0);
    (f64::from(h) + k * (alvo - f64::from(h))) as f32
}

#[cfg(test)]
#[path = "bristles_relevo_tests.rs"]
mod tests;
