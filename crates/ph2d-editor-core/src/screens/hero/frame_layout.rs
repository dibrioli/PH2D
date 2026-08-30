//! **A GEOMETRIA de um quadro do hero** — as bandas de chrome, as colunas ocupadas, e o
//! [`HeroLayout`] que sai delas.
//!
//! ⚠️ **Cortado do `paint.rs` em 2026-08-30 pelo tecto de LOC (708/700), e o corte é por
//! RESPONSABILIDADE:** aquele ficheiro responde *«o que se pinta neste quadro?»* e este responde
//! *«que geometria ele tem?»*. Foi este bloco que cresceu em cada wave desta linha — as bandas, a
//! sonda de colunas ocupadas, a docagem do timeline, as faixas do fundo, a fila de ferramentas —
//! e é ele que continua a crescer enquanto o modelo de áreas avança.

use super::HeroScreen;
use crate::screens::layout::HeroLayout;
use crate::zones::Rect;

/// Resolve o layout deste quadro a partir do estado do hero.
pub(super) fn frame_layout(hero: &HeroScreen, viewport: Rect) -> HeroLayout {
    // Rail width follows the user's Themes-menu rail-button-size
    // preset (Small / Medium / Large; default Small). Switching size
    // shifts Inspector/Hierarchy x-positions accordingly.
    // ⭐ **As medidas do chrome legado, ou ZERO.** *«Sem chrome legado»* não é um modo que o
    // layout conheça — são duas bandas a zero, e a aritmética dele é a de sempre.
    let mut bands = crate::screens::layout::ChromeBands {
        // ⭐ As larguras das colunas são AUTORADAS — o artista arrasta a borda interior delas
        // (`DOCK_SEAM_PX`), e o valor vive no `WidgetStore` como qualquer outra escolha de chrome.
        left_dock_w: hero
            .store
            .dock_width(crate::screens::layout::DockSide::Left),
        right_dock_w: hero
            .store
            .dock_width(crate::screens::layout::DockSide::Right),
        ..crate::screens::layout::ChromeBands::DEFAULT
    };
    if hero.view.legacy_chrome {
        bands.rail_w = hero.store.rail_button_size().rail_width_px();
    } else {
        bands.rail_w = 0.0;
        // ⭐ **A banda de topo fica, e muda de INQUILINO**: a barra de menus ocupa a faixa que os
        // pills ocupavam, e por isso a `F9` TROCA as duas em vez de as empilhar — duas faixas
        // custariam altura permanente ao alvo de 1024 pontos por causa de um interruptor de
        // bissecção.
        bands.top_bar_h = super::menu_bar::MENU_BAR_H;
        // ⭐ E o trilho deita-se: a faixa sai da ÁREA (entre as colunas), não da janela.
        // ⭐ A faixa quebra de linha quando a fila não cabe, então a ALTURA dela sai de uma
        // contagem — e a contagem precisa da largura da área, que não depende da altura. Duas
        // passagens: uma com a faixa a zero, para ler a largura; a definitiva a seguir.
        bands.tool_bar_h = 0.0;
    }
    // Motion Nodes M0.T4: `center_split` is `None` for every non-Motion tool, so
    // this is identical to the legacy layout there; the Motion bridge sets a split
    // while its tool is active.
    // **Quais colunas laterais estão ocupadas** — a área de desenho (e com ela as réguas) cresce
    // para dentro de uma coluna fechada. É o mesmo padrão do `dock_timeline_into_motion` logo
    // abaixo: o layout é uma função pura do que lhe dizem, e ESTE é o sítio que sabe.
    // **Quais colunas laterais estão ocupadas** — perguntado aos rects que os painéis
    // PUBLICARAM no quadro anterior, nunca a uma lista de nomes: são 20 crates a publicar, e a
    // lista de cinco que aqui esteve estava errada exactamente no modo que importava.
    // A sonda serve só para saber ONDE ficam as duas colunas — a geometria delas não depende
    // dos flags —, e o `side_columns` devolve-as ordenadas por `x`, que é o que torna o
    // `mirrored` inofensivo aqui.
    let probe = HeroLayout::for_viewport_bands(
        viewport,
        hero.view.ui_mirrored,
        bands,
        hero.view.center_split,
        crate::screens::layout::DockSides::BOTH,
    );
    let published: Vec<_> = hero.store.panel_rects().collect();
    let (left_col, right_col) = probe.side_columns();
    let docks = crate::screens::layout::DockSides::from_published(left_col, right_col, published);
    if !hero.view.legacy_chrome {
        let flat = HeroLayout::for_viewport_bands(
            viewport,
            hero.view.ui_mirrored,
            bands,
            hero.view.center_split,
            docks,
        );
        let lines = super::tool_bar::tool_bar_lines(
            &hero.store,
            hero.rail_shows_painter_tools(),
            hero.image_edit.mode_on,
            flat.draw_area.w,
        );
        bands.tool_bar_h = super::tool_bar::tool_bar_h(hero.store.rail_button_size(), lines);
    }
    let mut layout = HeroLayout::for_viewport_bands(
        viewport,
        hero.view.ui_mirrored,
        bands,
        hero.view.center_split,
        docks,
    );
    // **The timeline docks INTO the Motion workspace** (W4.T4). Only when both are on screen:
    // otherwise the graph keeps its full band and the timeline keeps its own dock. The condition
    // is read from the panel visibility the bridges already publish — the layout stays a pure
    // function of what it is told, and this is the one place that tells it.
    //
    // Before this, `motion_graph` ran down to the chrome and `timeline` was the bottom strip, so
    // the two occupied the SAME pixels and the timeline (drawn later) painted over the graph.
    if hero.is_panel_visible(super::PANEL_MOTION_GRAPH)
        && hero.is_panel_visible(super::PANEL_TIMELINE)
    {
        layout.dock_timeline_into_motion();
    }
    // ⛔ **As faixas do FUNDO também comem a área de desenho** (auditoria de 2026-08-30): o
    // `timeline` nasce exactamente no `area_x0` e ocupa 240 px no fundo da banda, então a régua
    // da esquerda corria por baixo dele. Depois do `dock_timeline_into_motion`, de propósito —
    // ele MOVE o rect do timeline, e reservar antes reservaria o sítio errado.
    if hero.is_panel_visible(super::PANEL_TIMELINE) {
        layout.reserve_bottom_strip(layout.timeline);
    }
    if hero.is_panel_visible("flip_frames") {
        layout.reserve_bottom_strip(layout.flip_strip);
    }
    layout
}
