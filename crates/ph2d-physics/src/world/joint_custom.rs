//! **O joint que o artista descreve por EIXO** — o vocabulário do `Custom`.
//!
//! Todo tipo do kit é um `GenericJoint` do rapier com uma configuração de eixos
//! fixa: um Pin trava os dois lineares e deixa o angular; um Slider trava `LinY`
//! e o angular e deixa `LinX`; uma solda trava os três. O `Custom` é o mesmo
//! motor com a configuração **autorada** — e é por isso que ele não traz
//! geometria nova nenhuma: ele expõe o que o construtor sempre teve.
//!
//! ⚠️ **Um eixo tem TRÊS estados, não dois** (o modelo do Unreal: *Free /
//! Limited / Locked*). Um booleano `travado?` colapsaria *livre* e *limitado*
//! num só, e a diferença entre eles é o que faz um batente existir — no rapier
//! são coisas separadas (`locked_axes` × `limits`), e um limite num eixo travado
//! é um número que ninguém lê.

/// O que este grau de liberdade pode fazer.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum AxisMode {
    /// Livre para sempre.
    #[default]
    Free,
    /// Livre entre dois batentes.
    Limited,
    /// Preso.
    Locked,
}

/// Um grau de liberdade descrito.
///
/// `min`/`max` só são lidos em [`AxisMode::Limited`] — a mesma política do
/// `limits_enabled` do componente, e pelo mesmo motivo: um número deixado para
/// trás por uma troca de modo não pode seguir em vigor em silêncio.
#[derive(Copy, Clone, Debug, PartialEq)]
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

/// Qual dos três graus de liberdade um número nomeia.
///
/// ⚠️ **Enum e não índice** — os `[AxisSpec; 3]` abaixo são indexados por ele
/// (`as usize`), e a alternativa era um `usize` cru que todo leitor teria de
/// lembrar de não passar de 2.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum CustomAxis {
    #[default]
    X = 0,
    Y = 1,
    Rotation = 2,
}

impl CustomAxis {
    /// Os três, na ordem em que o painel os pinta e o array os guarda.
    pub const ALL: [CustomAxis; 3] = [CustomAxis::X, CustomAxis::Y, CustomAxis::Rotation];

    /// O eixo do rapier que este nomeia — a porta única da tradução.
    #[must_use]
    pub fn rapier(self) -> rapier2d::dynamics::JointAxis {
        use rapier2d::dynamics::JointAxis;
        match self {
            CustomAxis::X => JointAxis::LinX,
            CustomAxis::Y => JointAxis::LinY,
            CustomAxis::Rotation => JointAxis::AngX,
        }
    }

    /// O bit da máscara de travamento.
    #[must_use]
    pub fn mask(self) -> rapier2d::dynamics::JointAxesMask {
        use rapier2d::dynamics::JointAxesMask;
        match self {
            CustomAxis::X => JointAxesMask::LIN_X,
            CustomAxis::Y => JointAxesMask::LIN_Y,
            CustomAxis::Rotation => JointAxesMask::ANG_X,
        }
    }

    /// **Este eixo é um comprimento?** — a pergunta de UNIDADE, e a razão de
    /// `motor_in_metres` deixar de ser uma propriedade do TIPO num Custom: aqui
    /// ela é do EIXO que o artista escolheu para o motor.
    #[must_use]
    pub fn in_metres(self) -> bool {
        !matches!(self, CustomAxis::Rotation)
    }

    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        match tag {
            1 => CustomAxis::Y,
            2 => CustomAxis::Rotation,
            _ => CustomAxis::X,
        }
    }
}

/// A descrição inteira de um [`super::JointKind::Custom`].
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct CustomDesc {
    /// Indexado por [`CustomAxis`].
    pub axes: [AxisSpec; 3],
    /// Em qual grau de liberdade o motor age.
    ///
    /// ⚠️ **Autorado, nunca inferido.** A alternativa tentadora — *"o motor
    /// dirige o primeiro eixo não travado"* — é mágica: o artista muda um modo
    /// de eixo por outra razão e o motor troca de lugar sem nada dizer.
    pub motor_axis: CustomAxis,
}
