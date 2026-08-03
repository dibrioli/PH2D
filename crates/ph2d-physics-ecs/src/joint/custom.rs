//! **A configuração de eixos que o artista escreve** — o vocabulário AUTORADO
//! do [`super::JointKind::Custom`].
//!
//! Irmão do `kind.rs` e do `properties.rs`, e o corte é o mesmo que o arquivo
//! pai já desenhava: aqui *o que um grau de liberdade pode fazer*, lá *que
//! espécie de restrição é esta*.
//!
//! ⚠️ **Estes tipos são o par serde do `ph2d_physics::joint_custom`**, e a
//! duplicação é a MESMA que o `JointKind` e o `MotorMode` já têm, pelo mesmo
//! motivo: o que viaja no arquivo de projeto é este lado, e o wrapper fica
//! plain-data. A tradução acontece **uma vez**, no `bridge::joint_desc` — e é
//! por ela ser um `match` exaustivo que um estado novo não pode nascer sem
//! resposta dos dois lados.

use serde::{Deserialize, Serialize};

/// O que este grau de liberdade pode fazer. **Append-only** — o postcard
/// codifica o discriminante posicionalmente.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisMode {
    /// Livre para sempre.
    #[default]
    Free,
    /// Livre entre dois batentes.
    Limited,
    /// Preso.
    Locked,
}

impl AxisMode {
    /// Os três, na ordem em que o painel os pinta.
    pub const ALL: [AxisMode; 3] = [AxisMode::Free, AxisMode::Limited, AxisMode::Locked];

    /// A tag do segmented ↔ o modo, as duas direções no mesmo lugar para que uma
    /// discordância seja visível — o idioma do `BodyKind::tag`.
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            AxisMode::Free => 0,
            AxisMode::Limited => 1,
            AxisMode::Locked => 2,
        }
    }

    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        match tag {
            1 => AxisMode::Limited,
            2 => AxisMode::Locked,
            _ => AxisMode::Free,
        }
    }

    /// A chave i18n do rótulo — a string fica com o painel (HR-15).
    #[must_use]
    pub fn i18n_suffix(self) -> &'static str {
        match self {
            AxisMode::Free => "free",
            AxisMode::Limited => "limited",
            AxisMode::Locked => "locked",
        }
    }
}

/// Qual dos três graus de liberdade um número nomeia.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomAxis {
    #[default]
    X,
    Y,
    Rotation,
}

impl CustomAxis {
    /// Os três, na ordem em que o array os guarda e o painel os pinta.
    pub const ALL: [CustomAxis; 3] = [CustomAxis::X, CustomAxis::Y, CustomAxis::Rotation];

    #[must_use]
    pub fn index(self) -> usize {
        match self {
            CustomAxis::X => 0,
            CustomAxis::Y => 1,
            CustomAxis::Rotation => 2,
        }
    }

    #[must_use]
    pub fn tag(self) -> u8 {
        self.index() as u8
    }

    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        match tag {
            1 => CustomAxis::Y,
            2 => CustomAxis::Rotation,
            _ => CustomAxis::X,
        }
    }

    /// **Este eixo é um comprimento?** A pergunta de UNIDADE, e a razão de
    /// `motor_in_metres` deixar de ser propriedade do TIPO num Custom.
    #[must_use]
    pub fn in_metres(self) -> bool {
        !matches!(self, CustomAxis::Rotation)
    }

    /// A chave i18n do rótulo.
    #[must_use]
    pub fn i18n_suffix(self) -> &'static str {
        match self {
            CustomAxis::X => "x",
            CustomAxis::Y => "y",
            CustomAxis::Rotation => "rotation",
        }
    }
}

/// Um grau de liberdade descrito.
///
/// `min`/`max` só são LIDOS em [`AxisMode::Limited`] — a política do
/// `limits_enabled` do componente, e pelo mesmo motivo: um número deixado para
/// trás por uma troca de modo não pode seguir em vigor em silêncio.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AxisSpec {
    pub mode: AxisMode,
    /// Radianos no eixo de rotação, metros nos dois lineares.
    pub min: f32,
    pub max: f32,
}

impl Default for AxisSpec {
    fn default() -> Self {
        Self {
            mode: AxisMode::Free,
            min: -1.0,
            max: 1.0,
        }
    }
}

/// A configuração inteira de um [`super::JointKind::Custom`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CustomAxes {
    /// Indexado por [`CustomAxis::index`].
    pub axes: [AxisSpec; 3],
    /// Em qual grau de liberdade o motor age.
    ///
    /// ⚠️ **Autorado, nunca inferido.** *"O motor dirige o primeiro eixo não
    /// travado"* é mágica: o artista muda um modo por outra razão e o motor
    /// troca de lugar sem nada dizer.
    pub motor_axis: CustomAxis,
}

impl CustomAxes {
    /// O eixo `a`, por nome.
    #[must_use]
    pub fn axis(&self, a: CustomAxis) -> AxisSpec {
        self.axes[a.index()]
    }

    /// Mutável, idem — a porta única de escrita, para nenhum chamador indexar o
    /// array cru e passar de 2.
    pub fn axis_mut(&mut self, a: CustomAxis) -> &mut AxisSpec {
        &mut self.axes[a.index()]
    }

    /// **Este Custom pode publicar uma reação ANGULAR?**
    ///
    /// O rapier não publica nada de um eixo que não restringe, então um limiar de
    /// torque só é alcançável se o eixo de rotação estiver travado ou limitado —
    /// a mesma pergunta que a solda mole respondeu para o `breaks_on_torque` do
    /// componente, aqui feita ao EIXO em vez de ao tipo.
    #[must_use]
    pub fn constrains_rotation(&self) -> bool {
        self.axis(CustomAxis::Rotation).mode != AxisMode::Free
    }
}
