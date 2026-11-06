//! A curva de volume de uma camada, por pontos.

use serde::{Deserialize, Serialize};

/// Um ponto da curva. `exp` diz como se chega a ele a partir do anterior: em curva exponencial, se os dois
/// valores forem positivos; senão, em reta.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvPoint {
    pub t: f32,
    pub v: f32,
    pub exp: bool,
}

/// Sem pontos, o volume é 1 do começo ao fim. Antes do primeiro ponto vale o primeiro; depois do último, o último.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    pub points: Vec<EnvPoint>,
}

/// −60 dB: o valor em que a queda de um `t60` termina.
const MENOS_60_DB: f32 = 0.001;

impl Envelope {
    /// Subida em reta, queda de 60 dB em `t60` e um fechamento em reta até o silêncio, para não haver estalo.
    #[must_use]
    pub fn hit(attack: f32, t60: f32, close: f32) -> Self {
        let pico = attack + t60;
        Self {
            points: vec![
                EnvPoint {
                    t: 0.0,
                    v: 0.0,
                    exp: false,
                },
                EnvPoint {
                    t: attack,
                    v: 1.0,
                    exp: false,
                },
                EnvPoint {
                    t: pico,
                    v: MENOS_60_DB,
                    exp: true,
                },
                EnvPoint {
                    t: pico + close,
                    v: 0.0,
                    exp: false,
                },
            ],
        }
    }
}
