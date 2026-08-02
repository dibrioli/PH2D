//! **A ETIQUETA de uma moldura** — o nome que se lê acima do canto superior-esquerdo dela.
//!
//! Enio, 2026-08-01: *"precisamos de uma pequena label no topo esquerdo dos frames para
//! identificarmos quais são os frames"* (com o Figma ao lado, onde toda prancheta traz o nome
//! por cima). Sem ela uma moldura é indistinguível de um retângulo qualquer: as duas desenham a
//! mesma silhueta, e a única diferença — o recorte — só se vê quando há conteúdo a transbordar.
//!
//! # Três decisões, e todas são sobre o que a etiqueta NÃO é
//!
//! 1. **Ela é chrome de TELA, ancorado no MUNDO.** A posição segue o pan/zoom (é a moldura que ela
//!    nomeia), mas o tamanho não: `LABEL_PX` é altura de fonte em pixels de tela, constante. Uma
//!    etiqueta que escalasse com o zoom seria ilegível em todo zoom menos um — e é o mesmo erro
//!    que já custou um borrão ao realce do Flip, onde a espessura subiu pelo afim do mundo.
//! 2. **Ela é DESENHO, nunca alvo.** Não há região de hit: clicar o nome para selecionar a moldura
//!    é o idioma do Figma, e aqui custaria uma faixa de canvas que engole o pen-down de toda
//!    ferramenta logo acima de cada moldura — exatamente o preço que a régua mediu para justificar
//!    viver só com o Vector em mãos. A moldura já se seleciona clicando nela.
//! 3. **O topo é o do MUNDO Y-up**, então o canto é `(min_x, max_y)` — e a etiqueta sobe na TELA,
//!    onde `+y` desce. Trocar um destes sinais põe o nome dentro da moldura, sobre a arte.

use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

use crate::grid::{GridView, world_bounds, world_to_screen_x, world_to_screen_y};
use crate::paint::{paint_text, resolve};

/// Altura da fonte da etiqueta, em pixels de TELA. Pequena de propósito: ela identifica, não
/// compete com a arte.
pub const LABEL_PX: f32 = 11.0;

/// Folga entre a base da etiqueta e a borda de cima da moldura, em pixels de tela. Sem ela o nome
/// encosta na silhueta e passa a ler como parte do desenho.
pub const LABEL_GAP_PX: f32 = 4.0;

/// Teto de largura do nome desenhado — um nome comprido é cortado em vez de atravessar a tela.
pub const LABEL_MAX_W_PX: f32 = 240.0;

/// Uma moldura a nomear: o canto superior-esquerdo dela **em MUNDO**, o nome, e se ela está
/// selecionada.
///
/// ⚠️ Plain data publicada pela shell (o padrão do [`crate::gizmo::PointGizmoView`]): o
/// `editor-core` não alcança a cena vetorial nem o ECS, e não deve — se alcançasse, haveria duas
/// respostas para *"quais molduras existem, e como se chamam?"*.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameLabel {
    /// `(min_x, max_y)` em MUNDO — o canto que a tela mostra em cima e à esquerda.
    pub world_top_left: [f64; 2],
    pub name: String,
    pub selected: bool,
}

/// Onde a etiqueta pousa na TELA, dado o canto de mundo. Função pura — é ela que os gates medem,
/// e é ela que o pintor usa (uma segunda projeção desenharia o nome longe da moldura).
#[must_use]
pub fn label_origin(view: &GridView, world_top_left: [f64; 2]) -> (f32, f32) {
    let (bounds, _) = world_bounds(view);
    let x = world_to_screen_x(world_top_left[0] as f32, &bounds, view);
    let y = world_to_screen_y(world_top_left[1] as f32, &bounds, view);
    // A etiqueta SOBE na tela: em coordenada de tela `+y` desce, então subir é subtrair. O `LABEL_PX`
    // entra junto porque `paint_text` ancora pelo TOPO do texto — sem ele a folga seria medida do
    // topo da letra e o nome encostaria na moldura.
    (x, y - LABEL_GAP_PX - LABEL_PX)
}

/// Desenha as etiquetas. Nada é registado para hit — ver a decisão 2 no topo do módulo.
pub fn paint_frame_labels(
    scene: &mut VectorScene,
    view: &GridView,
    labels: &[FrameLabel],
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let idle = resolve(ColorToken::Text2, theme);
    let hot = resolve(ColorToken::Accent, theme);
    for l in labels {
        let (x, y) = label_origin(view, l.world_top_left);
        // Fora do canvas não se desenha: uma moldura panada para fora da tela deixaria o nome
        // colado na borda, apontando para nada.
        if !view.canvas.contains(x, y) {
            continue;
        }
        paint_text(
            text_system,
            scene,
            &l.name,
            x,
            y,
            LABEL_PX,
            LABEL_MAX_W_PX,
            if l.selected { hot } else { idle },
        );
    }
}

#[cfg(test)]
#[path = "frame_label_tests.rs"]
mod tests;
