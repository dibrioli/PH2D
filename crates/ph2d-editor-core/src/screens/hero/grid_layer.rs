//! ⭐⭐⭐ **A CAMADA DA GRADE — à frente dos objectos, ou ATRÁS deles de verdade.**
//!
//! ⛔⛔ **Report do dono, 2026-09-24:** *«seção Display, Behind deixa o grid mais discreto mas não
//! atrás dos objetos. corrija»*. Até esse dia a grade era pintada SEMPRE na cena do chrome, que o
//! compositor põe por cima do `game_rt` — e o `Behind` era uma APROXIMAÇÃO escrita por extenso no
//! `paint.rs` (*«halving the grid's effective opacity … reads as "the grid is farther"»*,
//! a opacidade multiplicada por `0,4`), com o caminho verdadeiro deixado como *TODO follow-up*: *«real "behind"
//! rendering needs a second Vello intermediate + a 3-layer compositor»*.
//!
//! ⭐ **O caminho verdadeiro já existia quando o TODO foi escrito, com outro nome:** as FAIXAS DE
//! DESENHO do ADR-0154 Fase 2 montam o quadro num acumulador do mundo (`WorldRt`), de trás para a
//! frente — o fundo, as faixas de baixo, os sprites, as de cima — e o compositor lê o acumulador.
//! ⇒ com o `Behind`, a grade é a PRIMEIRA coisa depois do fundo, e a shell força esse modo.
//!
//! ⚠️ **UMA porta, duas saídas** — a decisão de «onde» vive aqui e só aqui:
//! - [`paint_in_chrome`] pinta a grade na cena do chrome **só quando ela está à FRENTE** (o caminho
//!   de sempre, byte a byte, e sem o `× 0,4`);
//! - [`paint_behind`] pinta-a numa cena PRÓPRIA **só quando ela está ATRÁS**, e devolve `true` —
//!   é esse `true` que manda a shell montar o quadro em camadas.
//!
//! As duas pedem a MESMA vista (a do canvas do layout) e o MESMO pintor, logo não podem divergir.

use crate::screens::HeroScreen;
use crate::screens::layout::HeroLayout;
use ph2d_vector::VectorScene;

/// A grade está ligada e o anfitrião publicou uma câmera?
fn live(hero: &HeroScreen) -> bool {
    hero.view.grid_visible && hero.grid.view.is_some()
}

/// ⭐ **A grade vai ATRÁS dos objectos neste quadro?** — a pergunta que a shell faz para decidir se
/// monta o quadro em camadas.
#[must_use]
pub fn is_behind(hero: &HeroScreen) -> bool {
    live(hero) && !hero.grid.snap_state.grid_in_front
}

fn paint_grid(hero: &HeroScreen, layout: &HeroLayout, scene: &mut VectorScene) {
    let Some(view) = hero.grid.view else {
        return;
    };
    let view = crate::grid::GridView {
        canvas: layout.canvas,
        ..view
    };
    crate::grid_snap::render::paint(scene, &view, &hero.grid.snap_state, hero.theme);
}

/// Pinta a grade na cena do CHROME — só quando ela está à FRENTE dos objectos.
pub fn paint_in_chrome(hero: &HeroScreen, layout: &HeroLayout, scene: &mut VectorScene) {
    if live(hero) && hero.grid.snap_state.grid_in_front {
        paint_grid(hero, layout, scene);
    }
}

/// Pinta a grade numa cena PRÓPRIA, para ir ATRÁS dos objectos. A cena é limpa sempre; devolve
/// `true` quando pintou (o `Behind` está ligado e há layout deste quadro).
///
/// ⚠️ Usa o `last_layout` que o [`crate::screens::paint_hero_screen`] deste quadro publicou — o
/// mesmo canvas que a grade da frente usaria.
pub fn paint_behind(hero: &HeroScreen, scene: &mut VectorScene) -> bool {
    scene.reset();
    let Some(layout) = hero.last_layout.as_ref() else {
        return false;
    };
    if !is_behind(hero) {
        return false;
    }
    paint_grid(hero, layout, scene);
    true
}

#[cfg(test)]
#[path = "grid_layer_tests.rs"]
mod tests;
