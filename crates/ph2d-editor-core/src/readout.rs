//! **A FICHA de leitura — o número que aparece onde os olhos já estão.**
//!
//! O estudo da UI viva (§6, C3) pedia que *«o readout que segue a mão»* deixasse de ser uma
//! instância e virasse **regra**. Este módulo é a regra: uma ficha, um corpo, uma cor, uma lei de
//! pouso — e todo gesto que produza um número passa a ter onde o pôr sem inventar a quarta ficha.
//!
//! # O que a medição encontrou antes desta wave
//!
//! Três superfícies desenhavam número sobre a arte, cada uma com a própria aritmética:
//!
//! | quem | corpo | caixa | fundo | ancora em |
//! |---|---|---|---|---|
//! | o rótulo do smart guide (`vec_snap_labels`) | 9 px | 60×16 | sim | o SEGMENTO da guia |
//! | a carga de um joint (`physics_overlay_joint_readout`) | `READOUT_PX` | 110×14 | não | o JOINT |
//! | as dimensões do Line (`painter_bridge_line_overlay`) | 11 px | 80×16 / 24×16 | não | a QUINA |
//!
//! ⚠️ **Nenhuma das três segue a mão** — as três ancoram em GEOMETRIA. E o gesto mais usado do app
//! inteiro — arrastar o gizmo: mover, escalar, girar — **não tinha número nenhum** sobre a tela.
//!
//! # Por que a ficha do gizmo diz um DELTA, e isto não é gosto
//!
//! O Inspector reflecte o `Transform` VIVO **a cada quadro** (`sync.rs`, o braço `else` — ele existe
//! precisamente para o arrasto não tremer), logo o valor **ABSOLUTO já está na tela** durante o
//! gesto. O que não existe em lado nenhum é *quão longe o gesto já foi* ⇒ é isso que a ficha diz.
//! Uma ficha que repetisse o absoluto seria uma segunda cópia de um número que já se lê.
//!
//! # A lei de pouso: ela FOGE do cursor, não escorrega para debaixo dele
//!
//! A ficha pousa abaixo-e-à-direita do cursor (o idioma do Illustrator/Affinity). Junto à borda da
//! tela ela **inverte de lado** em vez de ser espremida contra ela: um clamp puro deslizaria a ficha
//! para **debaixo do ponteiro** exactamente na borda, que é onde o ponteiro tapa o número que se
//! está a ler. Inverter é o que mantém a ficha legível em todo o canvas, e há gate.

use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Color, VectorScene};

use crate::paint::{fill_rounded_rect, paint_text_centered, resolve};
use crate::zones::Rect;

/// Corpo da ficha, px de tela. Herdado do rótulo do smart guide, que o herdou da régua: as três
/// superfícies dizem medida de mundo, e um corpo diferente fá-las-ia parecer três sistemas.
pub const CHIP_FONT_PX: f32 = 9.0; // LITERAL-PX-OK: corpo do rótulo (chrome)
/// Altura da ficha — uma linha de texto mais o respiro.
pub const CHIP_H_PX: f32 = 16.0; // LITERAL-PX-OK: ficha de leitura (chrome)
/// Raio do canto.
pub const CHIP_RADIUS_PX: f32 = 3.0; // LITERAL-PX-OK: ficha de leitura (chrome)
/// Respiro de cada lado do texto.
pub const CHIP_PAD_X_PX: f32 = 6.0; // LITERAL-PX-OK: ficha de leitura (chrome)
/// Folga entre o cursor e a quina mais próxima da ficha. Grande o bastante para o ponteiro do
/// sistema não a tapar, pequena o bastante para o número cair no mesmo golpe de vista.
pub const CURSOR_GAP_PX: f32 = 18.0; // LITERAL-PX-OK: ficha de leitura (chrome)

/// A largura que esta ficha precisa para este texto — nunca menor que `min_w`.
///
/// ⚠️ O `min_w` é o que torna a migração do rótulo do smart guide **byte-idêntica**: ele passa os
/// 60 px que já desenhava, e o texto mais longo que ele pode escrever (`-1234.5 px`) mede menos que
/// isso, então a ficha dele não muda um pixel. Quem não tem largura herdada passa `0.0` e recebe
/// uma ficha do tamanho do que diz.
#[must_use]
pub fn chip_width(text_system: &mut TextSystem, text: &str, min_w: f32) -> f32 {
    let measured = text_system.prefix_width(text, CHIP_FONT_PX) + CHIP_PAD_X_PX * 2.0;
    measured.max(min_w)
}

/// O rectângulo da ficha, dado o CENTRO dela e a largura.
#[must_use]
pub fn chip_rect(center: [f32; 2], w: f32) -> Rect {
    Rect::new(
        center[0] - w * 0.5,
        center[1] - CHIP_H_PX * 0.5,
        w,
        CHIP_H_PX,
    )
}

/// Desenha a ficha centrada em `center`.
///
/// ⚠️ **Ela TAPA o que está por baixo, de propósito.** Um número que paira ao lado do seu assunto
/// tem de escolher um dos lados sem critério; ocupando-o, ele lê-se como sendo aquilo. É a razão
/// que o rótulo do smart guide já dava para cobrir o tracejado, aqui promovida a lei da ficha.
pub fn paint_chip(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    center: [f32; 2],
    min_w: f32,
    theme: Theme,
) {
    let w = chip_width(text_system, text, min_w);
    let rect = chip_rect(center, w);
    let bg: Color = resolve(ColorToken::Bg1, theme);
    let fg: Color = resolve(ColorToken::Text1, theme);
    fill_rounded_rect(scene, rect, CHIP_RADIUS_PX, bg);
    paint_text_centered(text_system, scene, text, rect, CHIP_FONT_PX, fg);
}

/// **Onde a ficha pousa quando ela segue a MÃO** — o centro dela, dado o cursor e o canvas.
///
/// Função pura, e é ela que os gates medem. Abaixo-e-à-direita por omissão; **inverte** de lado
/// quando não caberia; e só então encaixa no canvas, para o caso patológico em que nem invertida
/// cabe (uma tela mais estreita que a ficha).
#[must_use]
pub fn at_cursor(cursor: [f32; 2], w: f32, canvas: Rect) -> [f32; 2] {
    let hw = w * 0.5;
    let hh = CHIP_H_PX * 0.5;
    let mut cx = cursor[0] + CURSOR_GAP_PX + hw;
    let mut cy = cursor[1] + CURSOR_GAP_PX + hh;
    // ⚠️ INVERTER, nunca espremer: um clamp puro põe a ficha debaixo do ponteiro na borda.
    if cx + hw > canvas.x + canvas.w {
        cx = cursor[0] - CURSOR_GAP_PX - hw;
    }
    if cy + hh > canvas.y + canvas.h {
        cy = cursor[1] - CURSOR_GAP_PX - hh;
    }
    let lo_x = canvas.x + hw;
    let hi_x = canvas.x + canvas.w - hw;
    let lo_y = canvas.y + hh;
    let hi_y = canvas.y + canvas.h - hh;
    // `max` antes de `min`: com um canvas mais estreito que a ficha os limites cruzam-se, e
    // esta ordem devolve a borda de cima/esquerda em vez de um número fora dos dois.
    [
        cx.max(lo_x).min(hi_x.max(lo_x)),
        cy.max(lo_y).min(hi_y.max(lo_y)),
    ]
}

#[cfg(test)]
#[path = "readout_tests.rs"]
mod tests;
