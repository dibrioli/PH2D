//! ⭐ **A tinta do contorno é a cor da CAMADA** (dono 07/11, GEF02 onda 9d, ficha
//! `f1_physics_static_shape.md`): o artista lê no canvas em que camada cada forma está, com a
//! mesma cor da aba e do Inspector (`ColorToken::layer`). O estado (sólida · sensor vazio/aceso ·
//! atravessável · solta) muda só o TRAÇO e a opacidade, nunca a matiz.

use ph2d_physics_ecs::Collider;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Color, Stroke};

use super::outline::OUTLINE_PX;

/// O estado de uma forma, que decide o traço e a opacidade da cor da sua camada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marca {
    Solida,
    SensorVazio,
    /// O mesmo sensor com um corpo dentro: o salto de opacidade É o gatilho a disparar.
    SensorAceso,
    /// A prancha que o player atravessa AGORA: sem isto a descida é invisível.
    Atravessavel,
    /// Sem corpo acima: o solver a ignora; tracejado curto e o mais apagado.
    Solta,
}

/// O que um contorno pinta: uma cor fixa (as anotações) ou a cor da camada sob uma marca.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tinta {
    Fixa([f32; 4]),
    Camada { camada: u8, marca: Marca },
}

impl PartialEq<[f32; 4]> for Tinta {
    fn eq(&self, rgba: &[f32; 4]) -> bool {
        *self == Tinta::Fixa(*rgba)
    }
}

const ALFA_SOLIDA: f32 = 0.85; // LITERAL-COLOR-OK: opacidade do contorno
const ALFA_SENSOR_VAZIO: f32 = 0.6; // LITERAL-COLOR-OK: opacidade do contorno
const ALFA_ATRAVESSAVEL: f32 = 0.35; // LITERAL-COLOR-OK: opacidade do contorno
const ALFA_SOLTA: f32 = 0.45; // LITERAL-COLOR-OK: opacidade do contorno
const SENSOR_DASH_PX: [f64; 2] = [6.0, 4.0]; // LITERAL-PX-OK: chrome de overlay, tracejado de tela
const SOLTA_DASH_PX: [f64; 2] = [2.0, 5.0]; // LITERAL-PX-OK: chrome de overlay, tracejado de tela

/// A marca de uma forma: atravessável antes de sensor (uma one-way nunca é sensor), solta sem corpo.
#[must_use]
pub fn marca(col: &Collider, tem_corpo: bool, aceso: bool, atravessavel: bool) -> Tinta {
    let marca = match (tem_corpo, atravessavel, col.is_sensor, aceso) {
        (false, ..) => Marca::Solta,
        (true, true, ..) => Marca::Atravessavel,
        (true, false, true, true) => Marca::SensorAceso,
        (true, false, true, false) => Marca::SensorVazio,
        (true, false, false, _) => Marca::Solida,
    };
    Tinta::Camada {
        camada: col.layer,
        marca,
    }
}

/// O traço e a cor finais, com a cor da camada resolvida pelo tema.
#[must_use]
pub fn tinta_do_contorno(tinta: Tinta, theme: Theme) -> (Stroke, Color) {
    let (camada, marca) = match tinta {
        Tinta::Fixa(rgba) => return (Stroke::new(OUTLINE_PX), Color::new(rgba)),
        Tinta::Camada { camada, marca } => (camada, marca),
    };
    let mut c = ph2d_editor_core::paint::resolve(ColorToken::layer(usize::from(camada)), theme);
    let (traco, alfa) = match marca {
        Marca::Solida => (Stroke::new(OUTLINE_PX), ALFA_SOLIDA),
        Marca::SensorVazio => (
            Stroke::new(OUTLINE_PX).with_dashes(0.0, SENSOR_DASH_PX),
            ALFA_SENSOR_VAZIO,
        ),
        Marca::SensorAceso => (
            Stroke::new(OUTLINE_PX).with_dashes(0.0, SENSOR_DASH_PX),
            1.0,
        ),
        Marca::Atravessavel => (Stroke::new(OUTLINE_PX), ALFA_ATRAVESSAVEL),
        Marca::Solta => (
            Stroke::new(OUTLINE_PX).with_dashes(0.0, SOLTA_DASH_PX),
            ALFA_SOLTA,
        ),
    };
    c.components[3] *= alfa;
    (traco, c)
}
