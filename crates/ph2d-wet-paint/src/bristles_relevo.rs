//! **O RELEVO de cada fio** (W3, prompt 49; W3b, prompt 50 — docs 49_medicao e 50_medicao). Os fios do óleo do
//! Rebelle não só pintam: uns CONSTROEM a tinta (Add impasto, cristas) e outros ESCAVAM-na (Remove impasto, sulcos
//! — o Bristle Flat dele tem `particle_thin_remove_impasto_strength` 0,65), e os fios escuros da foto são a sombra
//! deles. Aqui cada fio da W2 ([`crate::bristles_fios`]) deixa no seu CENTRO a altura do seu ganho (a pressão
//! própria e a mão leve), com o sinal do seu tipo, e entre dois centros a altura vai CONTÍNUA ([`entre`]) — um
//! degrau por fio é uma escada no traço inclinado. A altura é o campo [`crate::grid::Grid::altura`], que o emboss
//! da tinta acende ([`LUZ`]). [`Relevo::NEUTRO`] (altura `0`) = o hospedeiro nem chama a lei.

const SEMENTE_TIPO: u32 = 0x6f69_0006;

/// Quanto um dab puxa a altura de uma célula para o alvo do fio, por unidade de cobertura da silhueta: o
/// pincel penteia a tinta, e o que fica é o que os últimos pelos deixaram (com o Spacing dos óleos cada
/// célula leva dezenas de dabs; a meia taxa o alvo chega em poucos).
pub const TAXA: f64 = 0.5;

/// ⭐ **A luz do relevo** (W3b, doc 50_medicao §2): o cinza (`0..255`) que um declive de 1 px de altura por px soma
/// ao EMBOSS da tinta (`render.rs`, o mesmo termo do gradiente da massa, vezes o `Emboss` da célula): a luz e a
/// sombra do óleo do Rebelle somam o MESMO cinza aos três canais (o eixo da variação da foto é (0,9 · 1 · 0,9)), e
/// a luz do Impasto, que MULTIPLICA a cor, punha-a no eixo do albedo (0,13 · 0,6 · 1) — 3× o contraste para a
/// mesma textura medida pelo canal que mais se afasta do papel.
pub const LUZ: f64 = 20.0;

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

    /// A altura (px) que o fio `fio` de ganho `ganho` deixa no seu centro sob o `contacto` (`0..1`, o raio do
    /// dab ÷ o nominal): o ganho vezes o contacto² (a mão leve deixa uma película mais fina E sulca menos fundo;
    /// doc 49_medicao: o leve da rampa 6,61 → 6,32 sem mexer no forte), para cima se constrói e para baixo se
    /// escava. O hospedeiro tabela-a por fio, uma vez por dab.
    #[must_use]
    pub fn alvo_do_fio(&self, fio: i32, ganho: f64, contacto: f64) -> f64 {
        let c = contacto.clamp(0.0, 1.0);
        let h = self.altura_px * ganho * c * c;
        if self.escava(fio) { -h } else { h }
    }
}

/// ⭐ **A altura entre dois centros de fio** (W3b): de `a` (no centro do 1.º, `t = 0`) a `b` (no do 2.º, `t = 1`)
/// pelo meio-cosseno — contínua e de declive contínuo através das fronteiras, sem passar dos dois (nenhum fio fica
/// mais alto que o seu alvo). O declive máximo é `π/2 · |b − a| ÷ a largura do fio`.
#[must_use]
pub fn entre(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * 0.5 * (1.0 - (std::f64::consts::PI * t.clamp(0.0, 1.0)).cos())
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
