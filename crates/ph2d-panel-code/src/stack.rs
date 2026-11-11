//! ⭐ **A pilha da aba de scripts** (auditoria de layout 11/11, janela 1280×720): o código em cima,
//! depois o quadro do depurador, a Bancada e a faixa de baixo. Cada cartão diz a altura que quer e o
//! mínimo a que encolhe ([`Want`]); quando a aba não chega, encolhem pela ordem — a Bancada, o
//! depurador, a faixa — até ao mínimo de cada um, para o código guardar [`CODE_LEAST_ROWS`] linhas.
//! Antes cada cartão tirava o seu terço sem olhar para os outros: o código ia a zero e a Bancada,
//! posta de baixo, caía por cima do depurador, posto de cima.

use ph2d_tokens::{Spacing, row_pitch_px};

/// Quantas linhas o código guarda, por pequena que seja a aba (o que os cartões deixam abaixo disto
/// sai deles, não do código).
pub const CODE_LEAST_ROWS: usize = 4;

/// A altura que um cartão quer e o mínimo a que encolhe (`h == 0` = o cartão não está).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Want {
    pub h: f32,
    pub least: f32,
}

/// O mínimo do cartão do código: a busca (`find_h`, 0 = fechada), [`CODE_LEAST_ROWS`] linhas e o
/// respiro do poço.
#[must_use]
pub fn code_least(find_h: f32) -> f32 {
    find_h + row_pitch_px() * CODE_LEAST_ROWS as f32 + Spacing::Sm.px() * 2.0
}

/// As alturas de `wants` (pela ordem em que encolhem) numa aba de altura `avail` que deixa
/// `code_least` ao código; cada cartão presente custa também um `gap`. Sem espaço nem para os
/// mínimos, ficam os mínimos (o código fica com o resto, nunca negativo).
#[must_use]
pub fn fit<const N: usize>(avail: f32, code_least: f32, gap: f32, wants: [Want; N]) -> [f32; N] {
    let mut hs = wants.map(|w| w.h.max(0.0));
    let used: f32 = hs.iter().filter(|h| **h > 0.0).map(|h| h + gap).sum();
    let mut over = used - (avail - code_least).max(0.0);
    for (h, w) in hs.iter_mut().zip(wants) {
        if over <= 0.0 {
            break;
        }
        let cut = over.min((*h - w.least.min(*h)).max(0.0));
        *h -= cut;
        over -= cut;
    }
    hs
}
