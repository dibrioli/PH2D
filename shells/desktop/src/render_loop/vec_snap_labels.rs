//! **O NÚMERO do smart guide** — quanto vale a distância que a guia desenha
//! (plano 25 §9, o último item da tabela da W6).
//!
//! O segmento tracejado entre o ponto que encaixou e o ponto em que ele encaixou
//! já dizia *com o quê* a forma alinhou. O que faltava era **quanto** — a
//! pergunta que faz de um encaixe uma medição, e a razão de a W6 se chamar
//! PRECISÃO.
//!
//! # Três donos, e nenhum deles é este arquivo
//!
//! - **quais guias merecem número e onde ele pousa** — [`ph2d_vec_render::snap_labels`],
//!   geometria pura, sem noção de unidade;
//! - **que número, com que casas** — [`ph2d_editor::LengthDisplay`], a porta única
//!   que a régua e o painel de Grid Snap também usam;
//! - **como uma ficha é desenhada** — aqui, e só aqui.
//!
//! Este arquivo existe porque desenhar exige `TextSystem` e a cena do frame, que
//! nenhum dos dois outros alcança; ele não decide nada que os outros decidem.
//!
//! # O sufixo
//!
//! ⚠️ A régua imprime o número **NU** e este imprime **com sufixo** (`px`/`m`), e
//! a diferença não é gosto: uma régua é entendida pela faixa graduada em que ela
//! vive, e esta ficha **paira sobre a arte** sem eixo nenhum ao lado que a
//! explique. É também o que torna a unidade ativa visível sem abrir o menu.

use ph2d_editor::LengthDisplay;
use ph2d_editor::paint::{fill_rounded_rect, paint_text_centered, resolve};
use ph2d_editor::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, VectorScene};

/// Corpo do número, px. Igual ao rótulo da régua: as duas superfícies dizem
/// comprimento de mundo, e um corpo diferente as faria parecer dois sistemas.
const LABEL_PX: f32 = 9.0; // LITERAL-PX-OK: corpo do rótulo (chrome), espelha `ruler::LABEL_PX`
/// Meia-largura da ficha. Comporta `-1234.5 px`, e o teto **é uma consequência**,
/// não um palpite: o valor e a resolução são medidos na MESMA unidade de display,
/// então a razão entre eles é o comprimento do segmento **em pixels de TELA** —
/// e uma guia que a tela não mostra não recebe número. Com a tela em ~2000 px,
/// dígitos antes + depois da vírgula ficam em ~4 mais o sinal e o ponto.
const CHIP_HALF_W_PX: f32 = 30.0; // LITERAL-PX-OK: ficha do rótulo (chrome)
/// Meia-altura da ficha — uma linha de texto mais o respiro.
const CHIP_HALF_H_PX: f32 = 8.0; // LITERAL-PX-OK: ficha do rótulo (chrome)
/// Raio do canto da ficha.
const CHIP_RADIUS_PX: f32 = 3.0; // LITERAL-PX-OK: ficha do rótulo (chrome)

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
    let bg = resolve(ColorToken::Bg1, theme);
    let fg = resolve(ColorToken::Text1, theme);
    for l in labels {
        let text = format!(
            "{} {}",
            display.text_at_zoom(l.world_len, px_per_world),
            display.suffix()
        );
        let rect = Rect::new(
            l.at[0] as f32 - CHIP_HALF_W_PX,
            l.at[1] as f32 - CHIP_HALF_H_PX,
            CHIP_HALF_W_PX * 2.0,
            CHIP_HALF_H_PX * 2.0,
        );
        fill_rounded_rect(scene, rect, CHIP_RADIUS_PX, bg);
        paint_text_centered(text_system, scene, &text, rect, LABEL_PX, fg);
    }
}
