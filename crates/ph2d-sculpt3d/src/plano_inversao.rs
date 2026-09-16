//! **DUAS LEIS PARA A MESMA TECLA** — o que a inversão faz ao
//! [`crate::Verb::Plane`] (`SPEC_pincel_de_plano.md` §5).
//!
//! ⭐⭐ **É um SELECTOR e não um segundo modificador**, e a diferença é
//! observável: com um selector o artista escolhe *o que o `Ctrl` vai fazer* e
//! depois carrega; com dois modificadores ele teria de carregar em dois. A
//! referência escolheu o selector, e a limitação que isso traz é **declarada
//! pelos autores dela em público**: não há como ter um pincel que *muda de modo*
//! ao ser invertido, porque o modo é um só.
//!
//! ⚠️ **A segunda lei é EXACTA, e é a medição mais limpa da espec:** inverter um
//! pincel de `altura 1 / profundidade 0` no modo *trocar* dá saída
//! **byte-idêntica** (`0,000e+00`) à do mesmo pincel **não invertido** com
//! `altura 0 / profundidade 1`. ⇒ *não é «parecido com trocar os tectos»: é
//! trocar os tectos.* E com `altura = profundidade` ele é um **no-op**, também ao
//! bit.

/// O que a inversão faz — ver o cabeçalho.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlanoInversao {
    /// **AFASTAR do plano** — a força troca de sinal e os vértices fogem do
    /// plano em vez de se aproximarem. É o de fábrica, e é o que a família
    /// inteira desta casa já faz com o `Ctrl`.
    ///
    /// ⚠️ Medido: dá saída **diferente** da base (`7,0e-02`), ou seja é uma lei e
    /// não um no-op disfarçado.
    #[default]
    Afastar,
    /// **TROCAR OS TECTOS** — a altura e a profundidade trocam de papel, e a
    /// força **não muda**.
    ///
    /// ⭐ Com isto o artista tem *aparar* e *encher* na mesma mão: um pincel
    /// nascido a aparar (`altura 1 / profundidade 0`) enche enquanto o `Ctrl`
    /// estiver em baixo. É a razão de existir do selector.
    TrocarTectos,
}

impl PlanoInversao {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 2] = [Self::Afastar, Self::TrocarTectos];

    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Afastar => "Push Away",
            Self::TrocarTectos => "Swap Limits",
        }
    }

    /// **O PAR DE TECTOS que este dab vai usar**, já com a inversão resolvida.
    ///
    /// ⚠️ **Porta ÚNICA com dois chamadores** (a construção da pegada e a
    /// bancada), pela razão que esta casa já pagou três vezes: escrita duas
    /// vezes, a troca ficaria a valer num sítio e não no outro — e o defeito
    /// seria **mudo**, porque um pincel simétrico (`altura = profundidade`) lê
    /// igual nas duas leituras.
    #[must_use]
    pub fn tectos(self, invertido: bool, altura: f32, profundidade: f32) -> (f32, f32) {
        if invertido && self == Self::TrocarTectos {
            (profundidade, altura)
        } else {
            (altura, profundidade)
        }
    }

    /// **O SINAL da translação** — `−1` afasta do plano, `+1` aproxima.
    ///
    /// ⚠️ **Só o [`Self::Afastar`] o move**, e é o que separa as duas leis: no
    /// modo *trocar* a força **não muda** (espec §5), então inverter ali com o
    /// sinal trocado daria as duas leis ao mesmo tempo — e a saída não seria
    /// nenhuma delas.
    #[must_use]
    pub fn sinal(self, invertido: bool) -> f32 {
        if invertido && self == Self::Afastar {
            -1.0
        } else {
            1.0
        }
    }
}
