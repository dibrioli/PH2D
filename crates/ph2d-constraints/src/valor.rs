//! **Valor segue valor** (W5, [`docs/BlenderConstraints/06_W5_resto.md`](../../../docs/BlenderConstraints/06_W5_resto.md)
//! §3.1): um número do alvo, faixa → faixa e com mola opcional, num número do dono. A fonte e o
//! destino são um canal da pose ou QUALQUER campo animável (o endereço do `ph2d_ecs::FieldKey`); quem
//! lê e escreve o mundo é a fase — aqui só a lei.

use serde::{Deserialize, Serialize};

use crate::{Canal, Mix, Pose2, wrap_pi};

/// O endereço de um número: um canal da pose, ou um campo animável pelo endereço durável da casa
/// (`familia`, `campo`, `i`, `j` do `ph2d_ecs::FieldKey` — esta crate é folha e não vê o ECS).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Prop {
    Pose(Canal),
    Campo([u16; 4]),
}

impl Default for Prop {
    fn default() -> Self {
        Self::Pose(Canal::X)
    }
}

/// Os knobs da mola (os da [`crate::Spring`]): segundos de uma oscilação e o ressalto.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mola {
    pub duracao: f64,
    pub ressalto: f64,
}

impl Default for Mola {
    fn default() -> Self {
        Self {
            duracao: 0.5,
            ressalto: 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Valor {
    pub fonte: Prop,
    pub de_faixa: [f64; 2],
    pub destino: Prop,
    pub para_faixa: [f64; 2],
    /// Fora da faixa de entrada continua a recta; sem isto, apara nas bordas.
    pub extrapola: bool,
    /// `Add` soma ao valor de agora (a escala da pose multiplica).
    pub mix: Mix,
    /// `None` = segue no mesmo quadro.
    pub mola: Option<Mola>,
}

impl Default for Valor {
    fn default() -> Self {
        Self {
            fonte: Prop::Pose(Canal::Y),
            de_faixa: [0.0, 1.0],
            destino: Prop::Pose(Canal::Rot),
            para_faixa: [0.0, std::f64::consts::FRAC_PI_2],
            extrapola: false,
            mix: Mix::Replace,
            mola: Some(Mola::default()),
        }
    }
}

/// ⭐ **A lei da faixa** (a do Transformation e a do Valor): `v` na faixa `de` vai para a faixa
/// `para`; sem `extrapola` apara nas bordas. Uma faixa de entrada nula é um degrau em `de[0]`.
#[must_use]
pub fn faixa(v: f64, de: [f64; 2], para: [f64; 2], extrapola: bool) -> f64 {
    let [a, b] = de;
    let t = if b == a {
        if v >= a { 1.0 } else { 0.0 }
    } else {
        (v - a) / (b - a)
    };
    let t = if extrapola { t } else { t.clamp(0.0, 1.0) };
    let [c, d] = para;
    c + t * (d - c)
}

impl Valor {
    /// O valor da fonte na pose do alvo (a rotação embrulhada, como no Transformation); `None` para
    /// um campo, que a fase lê.
    #[must_use]
    pub fn da_pose(&self, alvo: Pose2) -> Option<f64> {
        match self.fonte {
            Prop::Pose(Canal::Rot) => Some(wrap_pi(alvo.rot)),
            Prop::Pose(c) => Some(c.of(alvo)),
            Prop::Campo(_) => None,
        }
    }

    /// A fonte `v` levada à faixa do destino.
    #[must_use]
    pub fn mapeia(&self, v: f64) -> f64 {
        faixa(v, self.de_faixa, self.para_faixa, self.extrapola)
    }

    /// O destino com a saída `out` composta sobre o valor de `agora`, à força `forca`.
    #[must_use]
    pub fn compoe(&self, agora: f64, out: f64, forca: f64) -> f64 {
        let k = forca.clamp(0.0, 1.0);
        let escala = matches!(self.destino, Prop::Pose(Canal::Sx | Canal::Sy));
        let alvo = match (self.mix, escala) {
            (Mix::Replace, _) => out,
            (Mix::Add, true) => agora * out,
            (Mix::Add, false) => agora + out,
        };
        agora + (alvo - agora) * k
    }

    /// A pose do dono com a saída `out` no canal de destino (nada se o destino é um campo).
    #[must_use]
    pub fn na_pose(&self, dono: Pose2, out: f64, forca: f64) -> Pose2 {
        let Prop::Pose(c) = self.destino else {
            return dono;
        };
        let mut p = dono;
        c.set(&mut p, self.compoe(c.of(dono), out, forca));
        p
    }
}
