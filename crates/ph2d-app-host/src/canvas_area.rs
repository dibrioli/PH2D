//! ⭐⭐ **A ÁREA DE DESENHO VISÍVEL** — o rectângulo que sobra da janela depois do chrome docado e
//! das réguas, e a **única** resposta a *«onde é que o canvas de facto se vê?»*.
//!
//! ⚠️ **Ela nasceu com o nome de um cliente** (`ph2d_viewport3d::layout::area`), e o doc dela já avisava
//! porquê isso é uma armadilha: *«uma porta com o nome de um dos seus clientes convida à segunda
//! cópia»*. O segundo cliente chegou em 2026-09-07 — o enquadramento da receita ao abrir o *Edit
//! Prefab* —, e a porta mudou-se para um módulo com o nome da **pergunta**. O módulo do 3D delega.
//!
//! ⚠️ `last_content` **é** a [`ph2d_editor::screens::layout::HeroLayout::draw_area`] publicada pelo
//! quadro anterior, já sem as réguas — e sem elas na tela ela é a própria `draw_area`. No primeiro
//! quadro é degenerada, e aí vale a janela, que é o comportamento de sempre.

use ph2d_editor::zones::Rect;

/// A área visível do canvas, ou `viewport` enquanto ela ainda não foi publicada.
#[must_use]
pub fn visible(hero: &ph2d_editor::screens::hero::HeroScreen, viewport: Rect) -> Rect {
    let published = hero.last_content;
    if published.w > 0.0 && published.h > 0.0 {
        Rect::new(published.x, published.y, published.w, published.h)
    } else {
        viewport
    }
}
