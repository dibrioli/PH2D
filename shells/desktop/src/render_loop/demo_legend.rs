//! **AS FICHAS DA CENA DE SMOKE** — o passe que desenha a legenda publicada por
//! [`crate::motion_demo_legend`].
//!
//! É chrome, pela mesma razão da decoração da folha ao lado: ela diz o que aquilo É, não como se
//! parece, e não pode entrar em bake nenhum. A ficha é a mesma do readout da casa
//! ([`ph2d_editor::readout::paint_chip`]) — corpo, altura, raio e cores saem de lá, e não daqui.
//!
//! ⚠️ **Em pixels de TELA, e não em metros.** Uma legenda tem de ser igualmente legível com o
//! canvas perto ou longe; um rótulo em unidades de mundo encolheria justamente quando o artista
//! se afasta para ver a grelha inteira, que é quando ele mais precisa dela.

use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, Point, VectorScene};

use crate::motion_demo_legend::Caption;

/// A ficha da legenda cresce com o texto — não há largura herdada a preservar.
const NO_MIN_W_PX: f32 = 0.0;

/// Desenha uma ficha por legenda, no ponto de tela em que a âncora de mundo caiu.
///
/// ⚠️ **Depois de tudo o que a cena desenha**, como o rótulo do smart guide e a decoração da
/// folha: a `VectorScene` tem de estar livre para o renderizador de texto, e a ficha **tapa** o
/// que estiver por baixo de propósito — é a lei escrita no `paint_chip`.
pub(crate) fn draw(
    captions: &[Caption],
    cam: Affine,
    theme: Theme,
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
) {
    for c in captions {
        let p = cam * Point::new(f64::from(c.world[0]), f64::from(c.world[1]));
        ph2d_editor::readout::paint_chip(
            text_system,
            scene,
            &c.text,
            [p.x as f32, p.y as f32],
            NO_MIN_W_PX,
            theme,
        );
    }
}
