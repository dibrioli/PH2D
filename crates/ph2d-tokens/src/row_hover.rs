//! **O véu do hover de uma linha** (`row-hover`): a camada de estado NEUTRA do tema.
//!
//! Sobre um painel de base fortemente colorida (PlumberRed/Green, Sunset) o véu `mono @ alfa` clareia a base
//! e herda-lhe o matiz, e o pairado deixava de ser mais neutro que a linha escolhida (PlumberGreen: croma
//! 0,084 contra 0,080 depois que o `accent-soft` desceu para ler o texto). Ali o véu passa a cinzento médio
//! com mais alfa: o mesmo contraste legível (≥ 1,3:1), o texto ≥ 4,5:1 e menos matiz da base.
//! Medido em PlumberGreen (g160, alfa 65/255): croma 0,073 · contraste 1,37 · texto 4,77.

use crate::color::{Color, srgb_to_oklch};
use crate::derive::Roles;

/// O croma Oklch da base acima do qual o véu cinzento substitui o `mono` (as bases comuns medem ≤ 0,05).
const BASE_COLORIDA: f64 = 0.06;
/// Quanto do caminho do `mono` ao `mono_inv` o cinzento percorre (255 → 160 no tema escuro).
const MEIO_CINZA: f32 = 0.37;
/// O alfa do véu cinzento, como múltiplo do alfa escrito no `tokens.json` para a família clássica.
const ALFA_MULT: f32 = 1.9;

/// A cor do `row-hover`: `mono @ alfa` (o do `tokens.json` da família clássica, em `escrito`) numa base comum;
/// cinzento médio a `alfa × ALFA_MULT` numa base fortemente colorida.
#[must_use]
pub(crate) fn veil(r: &Roles, escrito: Color) -> Color {
    let alfa = f32::from(escrito.a) / 255.0;
    let b = r.dark_3.color();
    if srgb_to_oklch(b.r, b.g, b.b).1 > BASE_COLORIDA {
        r.mono
            .lerp(r.mono_inv, MEIO_CINZA)
            .with_alpha((alfa * ALFA_MULT).min(1.0))
    } else {
        r.mono.with_alpha(alfa)
    }
}
