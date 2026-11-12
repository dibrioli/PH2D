//! ⭐ **A tinta do contorno no canvas é a cor da CAMADA** (dono 07/11, GEF02 onda 9d, ficha
//! `f1_physics_static_shape.md`): a mesma cor da aba e do Inspector (`ColorToken::layer`). O estado
//! (sólida · sensor vazio/aceso · atravessável · solta) muda o traço e a opacidade, nunca a matiz.
//!
//! A lei sem cor (`Marca`, `Tinta`, `Tom`, o traço do corpo) mora em `ph2d_shape_handles::contorno`; aqui ficam a cor
//! resolvida pelo tema e a sombra. Mora no painel (e não no overlay da `ph2d-app-physics`) porque a paleta das camadas
//! é dele e a crate do overlay está no tecto de linhas (onda 9g).

use ph2d_shape_handles::contorno::{Marca, SONDA_PX, Tinta, Traco, espessura, padrao};
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{Color, Stroke};

const ALFA_SOLIDA: f32 = 0.85; // LITERAL-PX-OK: opacidade do contorno
const ALFA_SENSOR_VAZIO: f32 = 0.6; // LITERAL-PX-OK: opacidade do contorno
const ALFA_ATRAVESSAVEL: f32 = 0.35; // LITERAL-PX-OK: opacidade do contorno
const ALFA_SOLTA: f32 = 0.45; // LITERAL-PX-OK: opacidade do contorno

/// O traço e a cor finais, com a cor da camada resolvida pelo tema.
#[must_use]
pub fn tinta_do_contorno(tinta: Tinta, theme: Theme) -> (Stroke, Color) {
    let (camada, marca, corpo, tom) = match tinta {
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
            tom,
        } => (camada, marca, corpo, tom),
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
    let t = tom.traco.unwrap_or(Traco {
        largura: espessura(corpo),
        padrao: padrao(marca, corpo),
    });
    let mut traco = Stroke::new(t.largura);
    if let Some(p) = t.padrao {
        traco = traco.with_dashes(0.0, p);
    }
    c.components[3] *= alfa;
    if let Some([r, g, b]) = tom.cor {
        c.components[..3].copy_from_slice(&[r, g, b]);
    }
    c.components[3] *= tom.alfa;
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
