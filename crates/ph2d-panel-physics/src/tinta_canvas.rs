//! ⭐ **A tinta do contorno no canvas é a cor da CAMADA** (dono 07/11, GEF02 onda 9d, ficha
//! `f1_physics_static_shape.md`): a mesma cor da aba e do Inspector (`ColorToken::layer`). O estado
//! (sólida · sensor vazio/aceso · atravessável · solta) muda o traço e a opacidade, nunca a matiz.
//!
//! ⭐ **O TIPO do corpo lê-se pelo traço** (dono 07/11, onda 9g: *«Static grosso, Dynamic fino e
//! Kinematic pontilhado - pode ser assim»*). O sensor tem traços LONGOS, a solta traços curtos e
//! espaçados; a peça de um corpo composto herda o tipo do corpo dono (`governing_kind`).
//! Mora no painel (e não no overlay da `ph2d-app-physics`) porque a paleta das camadas é dele e a
//! crate do overlay está no tecto de linhas (onda 9g).

use ph2d_physics_ecs::{BodyKind, Collider};
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme};
use ph2d_vector::{Color, Stroke};

/// O estado de uma forma, que decide o padrão e a opacidade da cor da sua camada.
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

/// O que um contorno pinta: uma cor fixa (as anotações) ou a cor da camada sob uma marca e o tipo
/// do corpo dono (`None` = solta).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tinta {
    Fixa([f32; 4]),
    Camada {
        camada: u8,
        marca: Marca,
        corpo: Option<BodyKind>,
    },
    /// ⭐ Uma linha de sensor (o traço fino de [`SONDA_PX`]). Com `camada` (o `RaySensor` do objecto, emenda 4, dono
    /// 08/11) a matiz é a da camada e o estado fica na opacidade `rgba[3]` (parado · nada · viu), com a sombra do
    /// contorno; sem ela (os sensores do personagem, o caminho) é a cor fixa `rgba`, sem sombra.
    Sonda {
        camada: Option<u8>,
        rgba: [f32; 4],
    },
}

/// A espessura de uma linha de sensor, mais fina que o contorno (`OUTLINE_PX` da `ph2d-app-physics`).
pub const SONDA_PX: f64 = 1.25; // LITERAL-PX-OK: chrome de overlay

impl PartialEq<[f32; 4]> for Tinta {
    fn eq(&self, rgba: &[f32; 4]) -> bool {
        *self == Tinta::Fixa(*rgba)
    }
}

const ALFA_SOLIDA: f32 = 0.85; // LITERAL-PX-OK: opacidade do contorno
const ALFA_SENSOR_VAZIO: f32 = 0.6; // LITERAL-PX-OK: opacidade do contorno
const ALFA_ATRAVESSAVEL: f32 = 0.35; // LITERAL-PX-OK: opacidade do contorno
const ALFA_SOLTA: f32 = 0.45; // LITERAL-PX-OK: opacidade do contorno

/// A espessura do traço pelo tipo do corpo: Static grosso, Dynamic e Kinematic finos.
#[must_use]
pub fn espessura(corpo: Option<BodyKind>) -> f64 {
    let t = match corpo {
        Some(BodyKind::Static) => StrokeToken::Heavy,
        _ => StrokeToken::Default,
    };
    f64::from(t.px())
}

/// O padrão `[traço, vão]` em px de tela; `None` = contínuo. Sensor: traços longos; Kinematic:
/// pontos (o traço do tamanho da espessura); solta: traços curtos e espaçados.
#[must_use]
pub fn padrao(marca: Marca, corpo: Option<BodyKind>) -> Option<[f64; 2]> {
    let px = |s: Spacing| f64::from(s.px());
    match (marca, corpo) {
        (Marca::SensorVazio | Marca::SensorAceso, _) => Some([px(Spacing::Lg), px(Spacing::Sm)]),
        (Marca::Solta, _) => Some([px(Spacing::Xs), px(Spacing::Md)]),
        (_, Some(BodyKind::Kinematic)) => Some([espessura(corpo), px(Spacing::Xs)]),
        _ => None,
    }
}

/// A marca de uma forma: atravessável antes de sensor (uma one-way nunca é sensor), solta sem corpo.
#[must_use]
pub fn marca(col: &Collider, corpo: Option<BodyKind>, aceso: bool, atravessavel: bool) -> Tinta {
    let marca = match (corpo.is_some(), atravessavel, col.is_sensor, aceso) {
        (false, ..) => Marca::Solta,
        (true, true, ..) => Marca::Atravessavel,
        (true, false, true, true) => Marca::SensorAceso,
        (true, false, true, false) => Marca::SensorVazio,
        (true, false, false, _) => Marca::Solida,
    };
    Tinta::Camada {
        camada: col.layer,
        marca,
        corpo,
    }
}

/// O traço e a cor finais, com a cor da camada resolvida pelo tema.
#[must_use]
pub fn tinta_do_contorno(tinta: Tinta, theme: Theme) -> (Stroke, Color) {
    let (camada, marca, corpo) = match tinta {
        Tinta::Fixa(rgba) => {
            return (
                Stroke::new(f64::from(StrokeToken::Default.px())),
                Color::new(rgba),
            );
        }
        Tinta::Camada {
            camada,
            marca,
            corpo,
        } => (camada, marca, corpo),
        Tinta::Sonda { camada, rgba } => {
            let cor = camada.map_or(Color::new(rgba), |j| {
                let mut c =
                    ph2d_editor_core::paint::resolve(ColorToken::layer(usize::from(j)), theme);
                c.components[3] *= rgba[3];
                c
            });
            return (Stroke::new(SONDA_PX), cor);
        }
    };
    let mut c = ph2d_editor_core::paint::resolve(ColorToken::layer(usize::from(camada)), theme);
    let alfa = match marca {
        Marca::Solida => ALFA_SOLIDA,
        Marca::SensorVazio => ALFA_SENSOR_VAZIO,
        Marca::SensorAceso => 1.0,
        Marca::Atravessavel => ALFA_ATRAVESSAVEL,
        Marca::Solta => ALFA_SOLTA,
    };
    let mut traco = Stroke::new(espessura(corpo));
    if let Some(p) = padrao(marca, corpo) {
        traco = traco.with_dashes(0.0, p);
    }
    c.components[3] *= alfa;
    (traco, c)
}

const ALFA_SOMBRA: f32 = ph2d_tokens::OUTLINE_SHADOW_ALPHA;

/// ⭐ **A sombra escura por baixo do contorno da camada** (dono 07/11, GEF02 fecho: *«sombra escura
/// por baixo dos contornos do canvas, SIM»*): o contorno laranja do Crate sumia sobre a arte laranja.
/// Mais larga `Thin` de cada lado, o MESMO padrão (o pontilhado continua ponto), a cor escura do
/// `BgScrim` (escura em todos os temas) sob a opacidade da marca. `None` nas anotações de cor fixa.
#[must_use]
pub fn sombra_do_contorno(tinta: Tinta, theme: Theme) -> Option<(Stroke, Color)> {
    if matches!(tinta, Tinta::Fixa(_) | Tinta::Sonda { camada: None, .. }) {
        return None;
    }
    let (traco, cor) = tinta_do_contorno(tinta, theme);
    let mut sombra = Stroke::new(traco.width + 2.0 * f64::from(StrokeToken::Thin.px()));
    if traco.dash_pattern.len() == 2 {
        sombra = sombra.with_dashes(0.0, [traco.dash_pattern[0], traco.dash_pattern[1]]);
    }
    let mut c = ph2d_editor_core::paint::resolve(ColorToken::BgScrim, theme);
    c.components[3] = ALFA_SOMBRA * cor.components[3];
    Some((sombra, c))
}
