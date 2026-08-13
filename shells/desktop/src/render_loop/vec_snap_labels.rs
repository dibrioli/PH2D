//! **O NÚMERO do smart guide** — quanto vale a distância que a guia desenha
//! (plano 25 §9, o último item da tabela da W6).
//!
//! O segmento tracejado entre o ponto que encaixou e o ponto em que ele encaixou
//! já dizia *com o quê* a forma alinhou. O que faltava era **quanto** — a
//! pergunta que faz de um encaixe uma medição, e a razão de a W6 se chamar
//! PRECISÃO.
//!
//! # Três donos, e NENHUM deles é este arquivo
//!
//! - **quais guias merecem número e onde ele pousa** — [`ph2d_vec_render::snap_labels`],
//!   geometria pura, sem noção de unidade;
//! - **que número, com que casas** — [`ph2d_editor::LengthDisplay`], a porta única
//!   que a régua e o painel de Grid Snap também usam;
//! - **como uma ficha é desenhada** — [`ph2d_editor::readout`], desde a C3 do estudo da UI viva.
//!
//! ⚠️ O terceiro item dizia *«aqui, e só aqui»* até esta ficha ter deixado de ser a única: o corpo,
//! a altura, o raio e as cores mudaram-se para a porta comum, e o que sobrou aqui é a largura
//! MÍNIMA — o único número que é desta ficha e de mais nenhuma. Este arquivo existe porque desenhar
//! exige `TextSystem` e a cena do frame; ele não decide nada que os outros decidem.
//!
//! # O sufixo
//!
//! ⚠️ A régua imprime o número **NU** e este imprime **com sufixo** (`px`/`m`), e
//! a diferença não é gosto: uma régua é entendida pela faixa graduada em que ela
//! vive, e esta ficha **paira sobre a arte** sem eixo nenhum ao lado que a
//! explique. É também o que torna a unidade ativa visível sem abrir o menu.

use ph2d_editor::LengthDisplay;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};

/// Largura MÍNIMA desta ficha. Comporta `-1234.5 px`, e o teto **é uma consequência**,
/// não um palpite: o valor e a resolução são medidos na MESMA unidade de display,
/// então a razão entre eles é o comprimento do segmento **em pixels de TELA** —
/// e uma guia que a tela não mostra não recebe número. Com a tela em ~2000 px,
/// dígitos antes + depois da vírgula ficam em ~4 mais o sinal e o ponto.
///
/// ⚠️ Ela sobrevive à passagem para [`ph2d_editor::readout`] como **piso**, e é isso que torna a
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
pub(crate) fn draw(
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
        ph2d_editor::readout::paint_chip(
            text_system,
            scene,
            &text,
            [l.at[0] as f32, l.at[1] as f32],
            CHIP_MIN_W_PX,
            theme,
        );
    }
}
