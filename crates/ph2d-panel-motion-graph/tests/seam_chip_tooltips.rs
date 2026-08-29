//! **AS DICAS DOS CHIPS CHEGAM AO STORE** — pela pintura real, não pela tabela.
//!
//! Report do Enio (2026-08-28): *"coloque dicas no hover dos botões posicionados no canto
//! inferior esquerdo do canvas"*. Os nove chips da barra do grafo eram ícones sem nome — só
//! legíveis para quem já sabia o que faziam.
//!
//! ⚠️ **Por que este teste existe ao lado do da tabela.** O irmão em `paint_chrome.rs` prova que
//! a `chip_tooltip` responde por todo chip da `chip_specs` — ele mede a **DECLARAÇÃO**. Este
//! mede o **EXECUTOR**: que o painter de facto chama `set_tooltip` com o id que o hit-index
//! regista. *Um gate sobre a declaração fica verde no dia em que o executor deixa de a ler* — é
//! a forma de gate vazio que a auditoria deste módulo apanhou vinte e quatro vezes em 27/08, e
//! as duas metades juntas são o que a fecha.
//!
//! ⚠️ E o id tem de ser o MESMO que o hover usa: o `paint_hover_tooltip` lê
//! `store.tooltip_for(store.hot_id())`, e o `hot_id` vem do hit-index. Registar a dica noutro id
//! seria uma dica que existe e que ninguém alcança.

use ph2d_editor_core::screens::layout::{CenterSplit, HeroLayout};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_motion_graph::{
    GraphViewSnapshot, MotionGraphPanel, MotionGraphPanelState, set_current_motion_graph,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn layout() -> HeroLayout {
    HeroLayout::for_viewport_split(
        VIEWPORT,
        false,
        ph2d_editor_core::screens::layout::rail_w(),
        CenterSplit::Horizontal {
            t: CenterSplit::T_DEFAULT,
        },
    )
}

/// **Todo chip pintado tem uma dica no store, sob o id que o hover consulta.**
///
/// ⚠️ **O CONTROLE é a contagem**: um grafo vazio ainda desenha a barra, então a varredura tem
/// de encontrar as nove — um laço sobre zero rects passa, e *um zero de «não medido» e um de
/// «tudo certo» são o mesmo byte*.
#[test]
fn every_toolbar_chip_registers_its_tooltip_through_the_real_paint() {
    set_current_motion_graph(Some(GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: Vec::new(),
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }));
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);

    let mut found = 0usize;
    for ordinal in 0..9u16 {
        let id = ph2d_panel_motion_graph::chrome_hit_id_for_tests(ordinal);
        let tip = host
            .store()
            .tooltip_for(id)
            .unwrap_or_else(|| panic!("o chip {ordinal} e' pintado e nao tem dica no store"));
        assert!(!tip.is_empty(), "dica vazia no chip {ordinal}");
        found += 1;
    }
    assert_eq!(found, 9, "os nove chips da barra");

    // ⚠️ E o CONTROLE do próprio id: um ordinal que a barra NÃO desenha não pode ter dica —
    // senão este teste passaria com o painter a registar num espaço de ids qualquer.
    assert!(
        host.store()
            .tooltip_for(ph2d_panel_motion_graph::chrome_hit_id_for_tests(99))
            .is_none(),
        "um ordinal inexistente nao pode ter dica"
    );
}
