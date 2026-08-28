//! **UM POPOVER NASCE NA BANDA DOS PAINÉIS, NUNCA NA BORDA DA JANELA.**
//!
//! Repro do smoke de 2026-08-02 (*"selecionar None é mais difícil e só funciona se clicar na parte
//! de baixo do nome"*), medido no FRAME REAL: com oitenta e uma linhas o picker de token não cabe
//! em lado nenhum, então o clamp toma *o lado com mais espaço* — e contra a JANELA isso o encostava
//! no topo dela. A linha de SOLTAR nascia em `y ∈ [2, 30]`: **2 px da borda**, por cima da barra de
//! ferramentas (`(14, 14, 1338, 64)`) e 64 px ACIMA do painel a que pertence (`inspector.y = 94`).
//!
//! ⚠️ **O `HitIndex` resolvia o clique ali** — o popover regista por último, e o gesto real
//! (Down+Up pelo `dispatch_pointer` da shell) devolvia `Click` em TODO `y` da linha. Logo o gate
//! NÃO pode ser sobre quem ganha o hit: ele é sobre ONDE a linha nasce. Quem come aqueles pixels
//! vive fora do nosso índice — `panel_at` responde `None` sobre a barra de topo, e a faixa de borda
//! da janela é do gestor de janelas —, e a cura não é disputar aquela faixa: é não nascer lá.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::TokenBindings;
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use std::sync::Once;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: HERO_VIEWPORT_W,
    h: HERO_VIEWPORT_H,
};

fn hero_with_vector_panel() -> HeroScreen {
    static INIT: Once = Once::new();
    ph2d_editor_core::test_support::ensure_panel_registry();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<VectorPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(VectorPanel::ID, true);
    hero
}

fn paint_frame(hero: &mut HeroScreen) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(hero, VIEWPORT, &mut scene, &mut text);
}

/// A linha de SOLTAR do picker de token, com o painel rolado a `scroll` (o que move o chip pela
/// tela — e é a POSIÇÃO DO CHIP que decide para que lado o popover abre).
fn first_row_at(scroll: f32) -> (Rect, HeroScreen) {
    let mut hero = hero_with_vector_panel();
    hero.store.set_panel_scroll(ids::VECTOR_PANEL, scroll);
    paint_frame(&mut hero);
    match hero
        .store
        .get_mut(ph2d_panel_vector::ids::VECTOR_TOKEN_FILL)
    {
        Some(InteractiveState::Dropdown { open, .. }) => *open = true,
        _ => panic!("o chip de token nao esta' registado como Dropdown (scroll={scroll})"),
    }
    paint_frame(&mut hero);
    let row = hero
        .hit_index
        .rect_for(ph2d_panel_vector::ids::vector_token_option_id(0, 0))
        .unwrap_or_else(|| panic!("a linha de soltar nao foi registada (scroll={scroll})"));
    (row, hero)
}

/// **A primeira linha nasce dentro da banda de chrome — abaixo da barra e sobre o painel.**
///
/// O sweep de scroll é parte do gate: a posição do chip decide o lado para que o popover abre, e
/// uma única posição passaria por sorte.
#[test]
fn the_first_row_of_the_token_picker_lands_on_the_panels_band() {
    state::set_stroke_present(Some(true));
    state::set_token_bindings(Some(TokenBindings::default()));
    let layout = HeroLayout::for_viewport(VIEWPORT);
    let band = layout.popover_region();
    let bar_bottom = layout.top_bar.y + layout.top_bar.h;

    for scroll in [0.0_f32, 200.0, 400.0, 600.0, 800.0] {
        let (row, hero) = first_row_at(scroll);
        assert!(
            row.y >= bar_bottom,
            "scroll={scroll}: a linha de SOLTAR nasceu em y={} — sobre a barra de topo (que acaba \
             em {bar_bottom}). Foi assim que ela ficou a 2 px da borda da janela.",
            row.y
        );
        assert!(
            row.y >= band.y && row.y + row.h <= band.y + band.h,
            "scroll={scroll}: a linha ({:?}) saiu da banda de chrome ({band:?})",
            row
        );
        // A pergunta do PRODUTO: a linha aterra sobre uma superfície de painel, e não sobre um
        // sítio que só a janela conhece. É esta que `panel_at` respondia `None`.
        let (cx, cy) = (row.x + row.w * 0.5, row.y + row.h * 0.5);
        assert!(
            hero.store.panel_at(cx, cy).is_some(),
            "scroll={scroll}: o centro da linha de SOLTAR ({cx}, {cy}) nao esta' sobre painel \
             nenhum — e quem pergunta *isto esta' sobre um painel?* nao tem por que concordar com \
             o hit-index"
        );
    }
    state::set_token_bindings(None);
}

/// **A porta é UMA** — nenhum popover deste painel clampa contra a janela.
///
/// ⚠️ Arch-gate porque o defeito é de GEOMETRIA e nasce no sítio da chamada: são cinco popovers
/// (token, mistura de filtro, ponta de traço, categoria do catálogo, fonte) e o sexto nasce com o
/// defeito se a resposta for escrita de novo. Um gate por popover cobriria os cinco que alguém se
/// lembrou de listar.
#[test]
fn no_popover_in_this_panel_clamps_against_the_window() {
    const FILES: [(&str, &str); 5] = [
        ("paint_tokens.rs", include_str!("../src/paint_tokens.rs")),
        (
            "paint_filters_blend.rs",
            include_str!("../src/paint_filters_blend.rs"),
        ),
        ("paint_markers.rs", include_str!("../src/paint_markers.rs")),
        ("paint_catalog.rs", include_str!("../src/paint_catalog.rs")),
        ("font_dropdown.rs", include_str!("../src/font_dropdown.rs")),
    ];
    for (name, src) in FILES {
        // Controle positivo: o scanner esta' a olhar para um ficheiro que de facto clampa.
        assert!(
            src.contains("popover_rect_clamped("),
            "{name}: o gate deixou de ver a chamada que julga (ficheiro renomeado?)"
        );
        assert!(
            !src.contains("popover_rect_clamped(chip, ctx.viewport)")
                && !src.contains("popover_rect_clamped(chip_rect, ctx.viewport)"),
            "{name}: clampa contra a JANELA — a lista encosta na borda de cima e sai de cima do \
             painel dela. A regiao e' `ctx.layout.popover_region()`."
        );
        assert!(
            src.contains("ctx.layout.popover_region()"),
            "{name}: o popover tem de perguntar a regiao ao layout"
        );
    }
}
