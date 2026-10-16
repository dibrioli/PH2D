//! ⭐⭐ **A ÁREA DE DESENHO VISÍVEL** — o rectângulo que sobra da janela depois do chrome docado e
//! das réguas, e a **única** resposta a *«onde é que o canvas de facto se vê?»*.
//!
//! ⚠️ **Ela nasceu com o nome de um cliente** (o módulo 3D, que saiu — ADR-0179), e o doc já
//! avisava porquê isso é uma armadilha: *«uma porta com o nome de um dos seus clientes convida à segunda
//! cópia»*. O segundo cliente chegou em 2026-09-07 — o enquadramento da receita ao abrir o *Edit
//! Prefab* —, e a porta mudou-se para um módulo com o nome da **pergunta**.
//!
//! ⚠️ `last_content` **é** a [`ph2d_editor_core::screens::layout::HeroLayout::draw_area`] publicada pelo
//! quadro anterior, já sem as réguas — e sem elas na tela ela é a própria `draw_area`. No primeiro
//! quadro é degenerada, e aí vale a janela, que é o comportamento de sempre.

use ph2d_editor_core::zones::Rect;

/// ⭐ **Onde um ENQUADRAMENTO pode pôr o conteúdo** (F8): a área visível sem a faixa da pílula ⑤ do
/// rodapé, que FLUTUA por cima dela (medido na foto da cena 14: as formas de baixo iam para debaixo
/// dela). A barra ① e a ④ ficam dentro da margem do enquadramento (`float_bar::frame_margin`).
#[must_use]
pub fn frameable(hero: &ph2d_editor_core::screens::hero::HeroScreen, viewport: Rect) -> Rect {
    let mut r = visible(hero, viewport);
    if let Some(l) = hero.last_layout.as_ref()
        && l.bottom_hud.h > 0.0
    {
        let hud_top = hero.escala().rect_to_physical(l.bottom_hud).y;
        r.h = r.h.min(hud_top - r.y).max(0.0);
    }
    r
}

/// ⭐ **A área e a margem de um ENQUADRAMENTO**, px físicos: a [`frameable`] sem a faixa de cima que
/// a ④ deste quadro pede além da margem (`tool_bars::frame_top_margin`), e a margem de sempre.
#[must_use]
pub fn frame_target(
    hero: &ph2d_editor_core::screens::hero::HeroScreen,
    viewport: Rect,
) -> (Rect, f32) {
    use ph2d_editor_core::screens::hero::float_bar;
    let k = hero.escala().factor();
    let margin = float_bar::frame_margin() * k;
    let extra = float_bar::tool_bars::frame_top_margin(hero) * k - margin;
    let mut r = frameable(hero, viewport);
    r.y += extra;
    r.h = (r.h - extra).max(0.0);
    (r, margin)
}

/// A área visível do canvas, ou `viewport` enquanto ela ainda não foi publicada.
#[must_use]
pub fn visible(hero: &ph2d_editor_core::screens::hero::HeroScreen, viewport: Rect) -> Rect {
    let published = hero.last_content;
    if published.w > 0.0 && published.h > 0.0 {
        // ⚠️ Publicada no espaço LÓGICO do chrome; quem pergunta (câmera, palco) vive na janela.
        hero.escala().rect_to_physical(published)
    } else {
        viewport
    }
}

#[cfg(test)]
#[path = "canvas_area_tests.rs"]
mod tests;
