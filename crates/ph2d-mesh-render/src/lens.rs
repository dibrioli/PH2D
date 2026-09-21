//! ⭐⭐⭐ **A LENTE da escultura** — convergente ou paralela.
//!
//! # ⛔ Ela NÃO é um desenho novo: é o vocabulário que esta casa já decidiu
//!
//! O modelador implícito shipa a mesma escolha desde a W15 dele, com o mesmo nome
//! ([`ph2d_field_render::Lens`]), a mesma tecla (`Numpad5`, a do Blender) e a mesma LEI — *as duas
//! lentes coincidem exactamente no plano do alvo*. O que muda entre os dois módulos é a câmera que
//! a consome: lá a orientação é um quaternion e a extensão é um campo (`half_extent`); aqui é
//! `yaw`/`pitch` e a extensão **deriva** de [`crate::Camera3d::distance`] e do `fov_y`
//! ([`crate::Camera3d::view_height`]).
//!
//! ⛔⛔ **E por isso este enum não tem o `half_fov` que o do modelador carrega.** Lá a variante
//! convergente **possui** a abertura; aqui ela já é um campo da câmera, e duplicá-la na variante
//! daria duas respostas a *«qual é a abertura?»* — a segunda ficaria velha no dia em que alguém
//! mexesse no campo. *Herda-se a escolha e a tecla, nunca a forma de a guardar.*
//!
//! # ⚠️ Porque a paralela é precisa aqui, e não é gosto
//!
//! O report do dono (2026-09-21) é de um sprite: *«Só temos a visão em perspectiva em sculpt. Não
//! temos Ortográfica. Precisamos de ambas.»* Uma peça que vai virar **arte 2D** é enquadrada com a
//! lente que não faz a distância mentir sobre o tamanho — é a vista de CAD, e é a que todo
//! pipeline de sprite-a-partir-de-3D usa.

/// ⭐ **O que o olho faz com o que está longe.**
///
/// ⚠️ **A ordem de [`Self::ALL`] é a ordem em que o painel as oferece, e o índice É a tag** — a
/// mesma lei que o `SignalVerb::ALL` desta casa paga: uma variante nova entra **no fim**, senão
/// toda escolha já gravada muda de significado em silêncio.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lens {
    /// Raios que **convergem** num olho — o que um escultor espera, e o valor de fábrica.
    #[default]
    Perspective,
    /// Raios **paralelos**: o tamanho na tela não depende da distância.
    Ortho,
}

impl Lens {
    /// ⚠️ **A FONTE da contagem** — o painel e o teclado derivam a fileira daqui, nunca de uma
    /// lista escrita à mão ao lado.
    pub const ALL: [Self; 2] = [Self::Perspective, Self::Ortho];

    /// A chave de i18n do rótulo.
    ///
    /// ⛔ **Uma chave e não o texto**: o app é em inglês e a tabela é quem o diz (HR-15).
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Perspective => "panel.sculpt3d.lens.perspective",
            Self::Ortho => "panel.sculpt3d.lens.ortho",
        }
    }

    /// A posição em [`Self::ALL`] — o que um selector segmentado guarda.
    #[must_use]
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|l| *l == self).unwrap_or(0)
    }

    /// A inversa de [`Self::index`]. Fora da faixa devolve o valor de fábrica.
    #[must_use]
    pub fn from_index(i: usize) -> Self {
        Self::ALL.get(i).copied().unwrap_or_default()
    }

    /// **A OUTRA** — o que a tecla alterna.
    ///
    /// ⚠️ **Derivada de [`Self::ALL`] e não de um `match` de dois braços**, e a razão é a terceira
    /// lente: um `match` escrito à mão continuaria a compilar e passaria a saltar uma delas.
    #[must_use]
    pub fn other(self) -> Self {
        Self::from_index((self.index() + 1) % Self::ALL.len())
    }
}

#[cfg(test)]
#[path = "lens_tests.rs"]
mod tests;
