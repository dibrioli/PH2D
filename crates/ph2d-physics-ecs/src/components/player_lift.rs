//! **O ESPELHO SERDE da política de plataforma** (`W-Leave`).
//!
//! ⚠️ **Irmão do [`super::player`] por RESPONSABILIDADE, e o corte é o mesmo que
//! o `CombineRule` já fazia:** a lei mora na crate pura, que **não fala serde**,
//! e o componente carrega o espelho — uma porta de conversão, um discriminante
//! pinado, e nenhuma tabela paralela.

use serde::{Deserialize, Serialize};

/// **O que a plataforma dá ao pulo quando se larga ela** — o espelho serde-nativo
/// do [`ph2d_platformer::PlatformLift`] (`W-Leave`).
///
/// O precedente é o [`super::CombineRule`]: a lei mora na crate pura, que **não
/// fala serde**, e o componente carrega o espelho — uma porta de conversão, um
/// discriminante pinado, e nenhuma tabela paralela.
///
/// **Append-only** — o discriminante É o valor de fio (o postcard codifica
/// posicionalmente) e também o índice do controle segmentado da §14, sem remap.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformLift {
    /// Soma tudo: a altura autorada é medida contra a PLATAFORMA. O mundo de
    /// antes desta wave, e o default do Godot.
    #[default]
    Full,
    /// Soma só o que sobe — um elevador a descer deixa de roubar o pulo.
    UpOnly,
    /// Não soma nada: a altura autorada é sempre medida contra o MUNDO.
    Nothing,
}

impl PlatformLift {
    /// O `u8` com que esta política atravessa a fronteira da UI e volta. Uma
    /// porta, as duas direções.
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            PlatformLift::Full => 0,
            PlatformLift::UpOnly => 1,
            PlatformLift::Nothing => 2,
        }
    }

    /// Recupera a política de um tag. `None` para um tag que nenhuma variante
    /// reivindica — a disciplina do `BodyKind::from_tag`: quem chama decide o
    /// que fazer com um valor que não esperava, em vez de receber um plausível.
    #[must_use]
    pub fn from_tag(tag: u8) -> Option<PlatformLift> {
        match tag {
            0 => Some(PlatformLift::Full),
            1 => Some(PlatformLift::UpOnly),
            2 => Some(PlatformLift::Nothing),
            _ => None,
        }
    }

    /// A política da LEI que este espelho descreve.
    #[must_use]
    pub fn law(self) -> ph2d_platformer::PlatformLift {
        match self {
            PlatformLift::Full => ph2d_platformer::PlatformLift::Full,
            PlatformLift::UpOnly => ph2d_platformer::PlatformLift::UpOnly,
            PlatformLift::Nothing => ph2d_platformer::PlatformLift::Nothing,
        }
    }

    /// O espelho de uma política da lei — a direção inversa da [`Self::law`],
    /// que o `PlatformPlayer::from_config` precisa.
    #[must_use]
    pub fn of_law(law: ph2d_platformer::PlatformLift) -> Self {
        match law {
            ph2d_platformer::PlatformLift::Full => PlatformLift::Full,
            ph2d_platformer::PlatformLift::UpOnly => PlatformLift::UpOnly,
            ph2d_platformer::PlatformLift::Nothing => PlatformLift::Nothing,
        }
    }
}
