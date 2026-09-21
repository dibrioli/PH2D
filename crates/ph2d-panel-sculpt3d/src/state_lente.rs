//! ⭐⭐⭐ **A LENTE, do lado do PAINEL** — convergente ou paralela.
//!
//! # ⛔⛔ Por que ela é escrita duas vezes
//!
//! Este painel é UI e **não conhece o renderizador** (ele não arrasta o `wgpu`), logo o conceito
//! vive aqui e em [`ph2d_mesh_render::Lens`]. É exactamente a mesma divisão que a [`super::LightMode`]
//! já paga, e o que a torna honesta é o **gate de ida-e-volta** da crate da família — a única que
//! vê os dois lados: *toda lente do device resolve para um modo do painel e volta ao mesmo, e todo
//! modo do painel é alcançável*.
//!
//! ⚠️ **A ORDEM é a tag**, como em toda fileira segmentada desta casa: a posição em [`Lens::ALL`] é
//! o que o despacho lê, logo uma lente nova entra **no fim**.

/// **O que o olho faz com o que está longe** — ver o `//!`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LensMode {
    /// Raios que convergem: o que um escultor espera, e o valor de fábrica.
    #[default]
    Perspective,
    /// Raios paralelos: o tamanho na tela não depende da distância — a vista de CAD, e a que um
    /// sprite quer (o report do dono de 2026-09-21).
    Ortho,
}

impl LensMode {
    /// A fonte da contagem — o pintor e o despacho derivam a fileira daqui.
    pub const ALL: [Self; 2] = [Self::Perspective, Self::Ortho];

    /// A chave de i18n do rótulo.
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Perspective => "panel.sculpt3d.lens.perspective",
            Self::Ortho => "panel.sculpt3d.lens.ortho",
        }
    }

    /// A posição em [`Self::ALL`] — o que a fileira segmentada acende.
    #[must_use]
    pub fn option_index(self) -> usize {
        Self::ALL.iter().position(|l| *l == self).unwrap_or(0)
    }

    /// A inversa. Fora da faixa devolve o valor de fábrica — *um índice que a fileira não pinta
    /// não tem resposta melhor do que a de sempre.*
    #[must_use]
    pub fn from_option_index(i: usize) -> Self {
        Self::ALL.get(i).copied().unwrap_or_default()
    }
}
