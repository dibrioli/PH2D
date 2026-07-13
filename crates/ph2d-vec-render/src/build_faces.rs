//! O realce das faces do **Shape Builder** — módulo irmão (LOC cap).
//!
//! Sem isto o Shape Builder é adivinhação: o artista arrasta o cursor sobre um emaranhado
//! de formas sobrepostas e só descobre o que pegou depois de soltar. **O realce é a
//! feature** — o resto é a booleana que já existia.
//!
//! Duas camadas, e a distinção entre elas carrega significado:
//!
//! - **Face sob o cursor** (o hover): um véu fino. Diz "é ESTA que você vai pegar".
//! - **Faces já pintadas** (as marcadas): um véu mais forte, e uma borda. Dizem "estas já
//!   são suas". Elas persistem enquanto o botão está apertado, para que o artista veja o
//!   caminho que já percorreu — sem isso, num arrasto longo, ele perde a conta.
//!
//! **Subtrair pinta com outra cor.** Um gesto que APAGA e um que UNE não podem parecer a
//! mesma coisa: o artista solta o botão e descobre o que fez. A cor é a única coisa que ele
//! olha durante o arrasto.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vec_scene::VecPath;
use ph2d_vector::{Affine, Brush, Color as VelloColor, Stroke, VectorScene};

/// Opacidade do véu da face sob o cursor (ela ainda não é sua).
const HOVER_ALPHA: f32 = 0.22;
/// Opacidade do véu de uma face já pintada.
const MARKED_ALPHA: f32 = 0.45;
/// Espessura da borda de uma face pintada, em pixels de tela.
const EDGE_PX: f64 = 1.5;

/// Desenha o realce. `hover` = a face sob o cursor; `marked` = as já pintadas. Tudo em
/// MUNDO (a shell já assou); `transform` leva mundo→tela.
///
/// `subtract` troca a cor: o gesto que apaga não pode parecer o que une.
pub fn draw_build_faces(
    hover: Option<&VecPath>,
    marked: &[VecPath],
    subtract: bool,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    if hover.is_none() && marked.is_empty() {
        return;
    }
    // UNIR = o acento (é o que se ganha). SUBTRAIR = o token de perigo (é o que se perde).
    // Sem essa distinção o artista arrasta às cegas e só descobre no release.
    let base = if subtract {
        ColorToken::Danger
    } else {
        ColorToken::Accent
    };
    let c = base.resolve(theme);
    let tint = |a: f32| VelloColor::from_rgba8(c.r, c.g, c.b, (a * 255.0) as u8);

    for m in marked {
        let bp = crate::build_bezpath(m);
        target.inner_mut().fill(
            crate::fill_rule(m),
            transform,
            &Brush::Solid(tint(MARKED_ALPHA)),
            None,
            &bp,
        );
        // A borda: uma face pintada tem contorno, e é o que a separa da vizinha quando duas
        // se encostam (dois véus adjacentes da mesma cor viram um borrão só).
        target.inner_mut().stroke(
            &Stroke::new(EDGE_PX),
            transform,
            &Brush::Solid(tint(1.0)),
            None,
            &bp,
        );
    }
    if let Some(h) = hover {
        target.inner_mut().fill(
            crate::fill_rule(h),
            transform,
            &Brush::Solid(tint(HOVER_ALPHA)),
            None,
            &crate::build_bezpath(h),
        );
    }
}
