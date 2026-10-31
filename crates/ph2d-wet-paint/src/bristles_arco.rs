//! ⭐ **O ARCO dos tufos** (31/10, missão «superar o Rebelle»; o Fan Brush): num leque real as cerdas abrem-se num
//! ARCO — as pontas formam uma curva, a meio do arco elas tocam o papel primeiro e as laterais só com pressão. A lei
//! lê o ponto no referencial da PEGADA (a do dab, com a rotação da ponta: `dabRotation` fixa · segue o traço · a
//! inclinação, CAP-01) — `s` ao longo do eixo MAIOR (a largura do leque, `−1..1`) e `t` ao longo do menor (`−1..1`
//! na borda da elipse):
//!
//! - a coluna `s` do leque é uma faixa centrada em `centro(s) = a·(s² − ½)` com meia-espessura `1 − a/2`, em unidades
//!   da meia-altura LOCAL da elipse (`√(1 − s²)`) — a meio (`s = 0`) a faixa sobe para `−a/2`, nas pontas desce para
//!   `+a/2`: a pegada é um CRESCENTE, os centros dos tufos ficam na curva e cada coluna guarda a mesma fracção dela
//!   (arrastado com a mão inteira, o leque pinta a largura toda);
//! - o tufo `s` precisa da pressão `a·s²` para tocar: a pressão leve só pousa o meio, a inteira pousa o leque todo.
//!
//! [`Arco::NEUTRO`] (`a = 0`) = o hospedeiro nem chama a lei: a pegada de sempre, ao bit.

/// A orla (na meia-altura local da coluna) em que a faixa de um tufo passa de cheia a vazia — sem degrau de célula.
pub const ORLA: f64 = 0.1;

/// A pressão (fracção) em que um tufo passa de fora do papel a tocá-lo inteiro.
pub const ORLA_DA_PRESSAO: f64 = 0.1;

/// O arco dos tufos de um pincel (o knob `tuftArc` do mapa `engine` do preset).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arco {
    /// A curvatura `a` (`0..1`): `0` = a pegada de sempre.
    pub curvatura: f64,
}

impl Arco {
    pub const NEUTRO: Arco = Arco { curvatura: 0.0 };

    /// Sem curvatura: a lei de antes.
    #[must_use]
    pub fn neutro(&self) -> bool {
        self.curvatura == 0.0
    }

    fn a(&self) -> f64 {
        self.curvatura.clamp(0.0, 1.0)
    }

    /// O centro da faixa do tufo em `s` (na meia-altura local da coluna).
    #[must_use]
    pub fn centro(&self, s: f64) -> f64 {
        self.a() * (s * s - 0.5)
    }

    /// A meia-espessura da faixa de cada tufo (na meia-altura local da coluna).
    #[must_use]
    pub fn meia_espessura(&self) -> f64 {
        1.0 - 0.5 * self.a()
    }

    /// Quanto o tufo em `s` toca o papel à `pressao` (`0..1`): `1` quando ela passa `a·s²`, `0` uma
    /// [`ORLA_DA_PRESSAO`] abaixo.
    #[must_use]
    pub fn toca(&self, s: f64, pressao: f64) -> f64 {
        ((pressao.clamp(0.0, 1.0) - self.a() * s * s) / ORLA_DA_PRESSAO + 1.0).clamp(0.0, 1.0)
    }

    /// O peso `0..1` do ponto `f = [s, t]` (o referencial da pegada) à `pressao`: a faixa do tufo × o contacto dele.
    #[must_use]
    pub fn no_ponto(&self, f: [f64; 2], pressao: f64) -> f64 {
        let [s, t] = f;
        let s = s.clamp(-1.0, 1.0);
        let tau = t / (1.0 - s * s).max(1e-6).sqrt();
        let faixa =
            ((self.meia_espessura() - (tau - self.centro(s)).abs()) / ORLA + 0.5).clamp(0.0, 1.0);
        faixa * self.toca(s, pressao)
    }

    /// ⭐ **A pressão que o TAMANHO leu** neste dab: o `contacto` (o raio do dab ÷ o nominal) desfeito pela lei do
    /// `pressureSize` (o raio na pressão 0 é `1 − ps`). Sem resposta do tamanho (`ps = 0`) a mão conta inteira.
    #[must_use]
    pub fn pressao_do_contacto(contacto: f64, pressure_size: f64) -> f64 {
        if pressure_size <= 0.0 {
            return 1.0;
        }
        (1.0 - (1.0 - contacto) / pressure_size).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
#[path = "bristles_arco_tests.rs"]
mod tests;
