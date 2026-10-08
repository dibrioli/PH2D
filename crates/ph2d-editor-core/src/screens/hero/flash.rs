//! **O PISCAR DA PALETA** (F4, D10) — o `Ctrl+K` que ensina: escolher um comando faz piscar o botão
//! onde ele MORA, para quem conhecia a UI antiga a achar na nova (o *Feature Finder* do Affinity).
//!
//! ⚠️ **O estado guarda o COMANDO, não o botão** ([`WidgetStore::flash`]): cada pintor pergunta se o
//! seu alvo é uma PORTA para ele — o próprio botão, o `⋯` da fila que o guarda, ou o pulldown da
//! área que o contém. Um botão escondido atrás do `⋯` não pisca no vazio.

use crate::ids;
use crate::interaction::WidgetStore;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme};

/// A intensidade do anel em `slot` agora, se `slot` é uma porta para o comando a piscar.
#[must_use]
pub fn level_at(store: &WidgetStore, steady: bool, slot: NodeId) -> Option<f32> {
    let (id, level) = store.flashing(steady)?;
    // O pulldown que continha o comando NA ESCOLHA — e é ELE que vai para o `⋯` quando a fila não
    // cabe.
    let pill = store.flash_pill();
    let behind_more = store
        .tool_overflow()
        .iter()
        .any(|e| e.node_id().is_some_and(|n| n == id || Some(n) == pill));
    let home = slot == id
        || Some(slot) == pill
        || (slot == ids::TOOL_BAR_OVERFLOW && behind_more);
    home.then_some(level)
}

/// O anel sobre `rect`, na cor do anel de foco e com a intensidade do piscar.
///
/// ⚠️ **Não pela porta da moldura** (`stroke_frame`): no tema moderno ela desenha o estilo da tabela
/// de estados e IGNORA a cor e a intensidade — o anel nunca aparecia (a foto da cena 10, 08/10).
pub fn paint_ring(scene: &mut ph2d_vector::VectorScene, theme: Theme, rect: Rect, level: f32) {
    let colour = crate::paint::resolve(ColorToken::FocusRing, theme).with_alpha(level);
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    // FRAME-RAW-OK: o PISCAR da paleta (F4) — um realce passageiro POR CIMA do botão, não a moldura dele
    crate::paint::stroke_rounded_rect(scene, rect, radius, StrokeToken::Heavy.px(), colour);
}

/// O anel de `slot`, se ele é uma porta para o comando a piscar.
pub fn ring_if_home(
    scene: &mut ph2d_vector::VectorScene,
    store: &WidgetStore,
    steady: bool,
    theme: Theme,
    slot: NodeId,
    rect: Rect,
) {
    if let Some(level) = level_at(store, steady, slot) {
        paint_ring(scene, theme, rect, level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **O anel DESENHA em todo tema** — a 1.ª redacção ia pela porta da moldura, que no tema
    /// moderno ignora a cor e a intensidade: o estado piscava e a tela não (a foto da cena 10).
    #[test]
    fn the_ring_paints_in_every_theme() {
        for theme in Theme::ALL {
            let mut scene = ph2d_vector::VectorScene::new();
            paint_ring(&mut scene, theme, Rect::new(10.0, 10.0, 40.0, 24.0), 1.0);
            assert!(
                !scene.inner().encoding().is_empty(),
                "{theme:?}: o anel do piscar não desenhou nada"
            );
        }
    }
}
