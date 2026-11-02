//! **O NÚMERO do smart guide** — quanto vale a distância que a guia desenha
//! (plano 25 §9, o último item da tabela da W6).
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `snap_labels.rs`.

use ph2d_editor_core::LengthDisplay;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};

/// Largura MÍNIMA desta ficha. Comporta `-1234.5 px`, e o teto **é uma consequência**,
/// não um palpite: o valor e a resolução são medidos na MESMA unidade de display,
/// então a razão entre eles é o comprimento do segmento **em pixels de TELA** —
/// e uma guia que a tela não mostra não recebe número. Com a tela em ~2000 px,
/// dígitos antes + depois da vírgula ficam em ~4 mais o sinal e o ponto.
///
/// ⚠️ Ela sobrevive à passagem para [`ph2d_editor_core::readout`] como **piso**, e é isso que torna a
/// migração byte-idêntica: o texto mais largo que esta ficha pode escrever mede menos que 60 px,
/// então a ficha continua a ter exactamente a largura que tinha. Há gate que o MEDE.
const CHIP_MIN_W_PX: f32 = 60.0; // LITERAL-PX-OK: ficha do rótulo (chrome)

/// Desenha as fichas de distância das guias de snap ativas neste frame.
///
/// ⚠️ **Depois do traço das guias, e é a mesma ordem (e o mesmo motivo) do
/// overlay de dimensões do Line e do readout de joint:** a `VectorScene` tem de
/// estar livre para o renderizador de texto. E a ficha **cobre** o tracejado sob
/// ela de propósito — o número É aquele segmento, então lê melhor ocupando-o do
/// que pairando ao lado, onde teria de escolher um dos dois lados sem critério.
pub fn draw(
    guides: &[ph2d_vec_render::Guide],
    cam: Affine,
    px_per_world: f64,
    display: LengthDisplay,
    theme: Theme,
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
) {
    let labels = ph2d_vec_render::snap_labels(guides, cam);
    if labels.is_empty() {
        return;
    }
    for l in labels {
        let text = format!(
            "{} {}",
            display.text_at_zoom(l.world_len, px_per_world),
            display.suffix()
        );
        ph2d_editor_core::readout::paint_chip(
            text_system,
            scene,
            &text,
            [l.at[0] as f32, l.at[1] as f32],
            CHIP_MIN_W_PX,
            theme,
        );
    }
}
