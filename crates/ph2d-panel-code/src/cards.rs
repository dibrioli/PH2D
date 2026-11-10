//! ⭐ **As regiões da aba em cartões** (dono 08/11: «tudo fica mais legal dentro de molduras»): a lista,
//! o código com a barra de estado e a faixa Console | Problems são cartões de quinas redondas sobre o
//! chão do painel, com a margem e o vão da aba Collision Layers (`Spacing::Md`) e o raio dela
//! (`Radius::Sm` pela porta do tema). Cada vista pinta o próprio fundo DENTRO do recorte redondo.

use ph2d_editor_core::paint::{fill_rounded_rect, frame_radius, push_rounded_clip};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// A margem entre a aba e os cartões, e o vão entre cartões (a da aba Collision Layers; não é o vão de duas
/// linhas, que é o `control_gap_px`).
#[must_use]
pub fn margin() -> f32 {
    Spacing::Md.px()
}

/// `area` menos a margem de fora em cada lado.
#[must_use]
pub fn inset(area: Rect) -> Rect {
    let g = margin();
    Rect::new(
        area.x + g,
        area.y + g,
        (area.w - g * 2.0).max(0.0),
        (area.h - g * 2.0).max(0.0),
    )
}

/// Os tons do chão e do cartão: os da aba Collision Layers (nos cartões, a lista e o cartão do Kanban).
fn tones() -> (ColorToken, ColorToken) {
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        (ColorToken::KanbanList, ColorToken::KanbanCard)
    } else {
        (ColorToken::PanelBg, ColorToken::Bg1)
    }
}

/// O tom do chão debaixo dos cartões (o degrau abaixo do cartão: cabeçalhos e caixas fundas).
#[must_use]
pub fn ground_tone() -> ColorToken {
    tones().0
}

/// O tom de um cartão (a base da listra de uma tabela dentro dele).
#[must_use]
pub fn card_tone() -> ColorToken {
    tones().1
}

/// O recuo do poço do código dentro do seu cartão.
#[must_use]
pub fn well_pad() -> f32 {
    Spacing::Sm.px()
}

fn radius(theme: &Theme) -> f32 {
    frame_radius(*theme, Radius::Sm.px())
}

/// O chão da aba, por baixo dos cartões.
pub fn floor(scene: &mut VectorScene, area: Rect, theme: &Theme) {
    fill_rounded_rect(
        scene,
        area,
        0.0,
        ph2d_editor_core::paint::resolve(tones().0, *theme),
    );
}

/// Abre um cartão em `r`: o fundo e o recorte redondo. Feche com [`close`].
pub fn open(scene: &mut VectorScene, r: Rect, theme: &Theme) {
    let raio = radius(theme);
    fill_rounded_rect(
        scene,
        r,
        raio,
        ph2d_editor_core::paint::resolve(tones().1, *theme),
    );
    push_rounded_clip(scene, r, raio);
}

pub fn close(scene: &mut VectorScene) {
    scene.pop_layer();
}

/// O contorno de um VALOR mostrado (o número ao vivo no código, a caixa de campo, a dica, o gráfico):
/// não é a moldura de um controlo — é o que separa o número do que está à volta —, por isso não passa
/// pela porta do tema, que num tema moderno o apaga em repouso (smoke do dono 10/11).
pub fn value_outline(scene: &mut VectorScene, r: Rect, radius: f32, ink: ph2d_vector::Color) {
    let w = ph2d_tokens::StrokeToken::Thin.px();
    // FRAME-RAW-OK: contorno de valor, não de controlo; a pele plana apagava-o (dono 10/11).
    ph2d_editor_core::paint::stroke_rounded_rect(scene, r, radius, w, ink);
}
