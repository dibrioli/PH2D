//! **A ROTAÇÃO** — para onde o objecto olha.
//!
//! ⚠️ **Ela lê a direcção de MUNDO**, depois do viewpoint (ver o cabeçalho da
//! crate): o desenho está no ecrã, não no tabuleiro. Num tabuleiro isométrico a
//! seta `→` manda o corpo para nordeste, e é para nordeste que ele tem de olhar.
//!
//! ⚠️ **Parado, ele fica onde estava.** Sem intenção não há direcção — *direcção
//! ausente não é o ângulo zero*, e um corpo que voltasse a apontar para leste ao
//! largar a tecla seria um defeito que só se vê a jogar.

use crate::{Vec2, normalize};

/// **Para onde o objecto olha.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RotationMode {
    /// Não roda — o default, e o que uma sprite vista de cima com quatro poses
    /// desenhadas quer.
    #[default]
    None,
    /// Vira para onde anda, por qualquer ângulo.
    ToMovement,
    /// Vira para o rumo cardeal mais perto.
    Snap90,
    /// Vira para um dos oito rumos.
    Snap45,
}

impl RotationMode {
    /// Todos, na ordem do painel. ⛔ **A fonte da contagem.**
    pub const ALL: [Self; 4] = [Self::None, Self::ToMovement, Self::Snap90, Self::Snap45];

    /// O rótulo que o painel pinta.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "Don't Rotate",
            Self::ToMovement => "Face Movement",
            Self::Snap90 => "Face 4 Directions",
            Self::Snap45 => "Face 8 Directions",
        }
    }

    /// De quantos em quantos graus o ALVO encaixa — `None` é livre.
    #[must_use]
    pub const fn snap_deg(self) -> Option<f32> {
        match self {
            Self::None | Self::ToMovement => None,
            Self::Snap90 => Some(90.0),
            Self::Snap45 => Some(45.0),
        }
    }

    /// Se este modo lê a velocidade de viragem.
    ///
    /// ⭐ O painel esconde a linha em [`RotationMode::None`] — *um knob que o modo
    /// não sabe ler não se mostra*.
    #[must_use]
    pub const fn reads_speed(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// **A porta**: ângulo de agora + direcção de mundo ⇒ ângulo novo, em radianos.
///
/// `speed_deg` é graus por segundo; ⚠️ **zero é instantâneo**, a mesma convenção
/// das rampas do [`crate::intent`].
#[must_use]
pub fn rotate_toward(atual: f32, dir_mundo: Vec2, mode: RotationMode, speed_deg: f32, dt: f32) -> f32 {
    if matches!(mode, RotationMode::None) {
        return atual;
    }
    let Some(dir) = normalize(dir_mundo) else {
        return atual;
    };
    // `libm` pelo motivo de sempre: este ângulo é gravado no `Transform` e o
    // `physics_ecs_c9` compara-o entre três sistemas operacionais.
    let mut alvo = libm::atan2f(dir[1], dir[0]);
    if let Some(passo) = mode.snap_deg() {
        let passo_rad = passo.to_radians();
        alvo = libm::roundf(alvo / passo_rad) * passo_rad;
    }
    let delta = arco_curto(alvo - atual);
    if !(speed_deg.is_finite() && speed_deg > 0.0) || !dt.is_finite() || dt <= 0.0 {
        return atual + delta;
    }
    let maximo = speed_deg.to_radians() * dt;
    if delta.abs() <= maximo {
        atual + delta
    } else {
        atual + maximo * delta.signum()
    }
}

/// Dobra um ângulo para `(−π, π]` — *virar 350° é virar −10°*.
///
/// ⚠️ Sem isto o corpo dá a volta pelo lado longo sempre que o alvo cruza `±π`, e
/// o sintoma (*«ele gira ao contrário de vez em quando»*) é do tipo que se lê como
/// defeito de física.
fn arco_curto(mut a: f32) -> f32 {
    const TAU: f32 = core::f32::consts::TAU;
    while a > core::f32::consts::PI {
        a -= TAU;
    }
    while a <= -core::f32::consts::PI {
        a += TAU;
    }
    a
}
