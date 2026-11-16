//! Como uma alça do desenho lê o cursor (o valor na unidade do cartão).

use crate::MOTOR_PREVIEW_S;

/// Como uma alça lê o cursor.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Geo {
    /// Ao longo de `dir` (no ecrã) a partir de `origin`, em metros (`scale` px/m).
    Linear {
        origin: (f32, f32),
        dir: (f32, f32),
        scale: f32,
    },
    /// O ângulo em volta de `centre`, em graus, a contar de `zero` (rad, no ecrã com y para cima).
    Angular { centre: (f32, f32), zero: f32 },
    /// O motor que gira: o ângulo como em `Angular`, lido como o giro em [`MOTOR_PREVIEW_S`] (°/s). Sem
    /// tecto: a espiral dá as voltas que o número pedir, e o arrasto conta-as ([`Geo::value_near`]).
    Spin { centre: (f32, f32), zero: f32 },
}

impl Geo {
    /// O valor (na unidade do cartão) que o cursor em `(x, y)` diz.
    #[must_use]
    pub fn value_at(self, x: f32, y: f32) -> f32 {
        match self {
            Self::Linear { origin, dir, scale } => {
                ((x - origin.0) * dir.0 + (y - origin.1) * dir.1) / scale.max(f32::EPSILON)
            }
            Self::Angular { centre, zero } => {
                let a = libm::atan2f(centre.1 - y, x - centre.0) - zero;
                let a = libm::atan2f(libm::sinf(a), libm::cosf(a));
                a.to_degrees()
            }
            Self::Spin { centre, zero } => {
                Self::Angular { centre, zero }.value_at(x, y) / MOTOR_PREVIEW_S
            }
        }
    }

    /// O valor em `(x, y)` mais perto de `prev`: no `Spin` o ângulo do cursor ganha as voltas inteiras que o
    /// deixam junto do giro de antes — o arrasto que dá a volta à roda passa de ±180° sem saltar.
    #[must_use]
    pub fn value_near(self, x: f32, y: f32, prev: f32) -> f32 {
        let Self::Spin { .. } = self else {
            return self.value_at(x, y);
        };
        let raw = self.value_at(x, y) * MOTOR_PREVIEW_S;
        const TURN: f32 = 360.0; // LITERAL-PX-OK: uma volta, em graus
        let turns = ((prev * MOTOR_PREVIEW_S - raw) / TURN).round();
        (raw + turns * TURN) / MOTOR_PREVIEW_S
    }
}
