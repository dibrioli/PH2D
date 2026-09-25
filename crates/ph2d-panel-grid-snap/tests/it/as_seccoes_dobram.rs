//! ⭐⭐ **AS QUATRO SECÇÕES DO GRID DOBRAM** — ordem do dono, 2026-09-24, com foto: *«no Grid as
//! seções não fecham. Isso deve ser corrigido»*.
//!
//! Até esse dia os títulos (*Grid Kind* · *Target* · *Display* · *Inspect*) eram texto pintado à mão
//! (ou um `SectionHeader` com id `NodeId(0)`, o da *Inspect*): nenhum registava hit-rect, e um
//! clique neles não fazia nada. O gate é o gesto REAL — pintar, carregar no centro do rect que o
//! painel registou, e ver a secção dobrar.
//!
//! ⚠️ **Um host por cabeçalho**: dobrar um encolhe o painel e os de baixo sobem, então reutilizar o
//! mesmo host mediria rects que a dobra anterior invalidou (a lição do irmão da Física).

use ph2d_editor_core::zones::Rect;
use ph2d_panel_grid_snap::GridSnapPanel;
use ph2d_panel_grid_snap::state::GridSnapPanelState;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

#[test]
fn cada_seccao_dobra_ao_clique_no_titulo() {
    let cabecalhos = [
        ("Grid Kind", ph2d_panel_grid_snap::ids::GS_SEC_KIND),
        ("Target", ph2d_panel_grid_snap::ids::GS_SEC_TARGET),
        ("Display", ph2d_panel_grid_snap::ids::GS_SEC_DISPLAY),
        (
            "Inspect",
            ph2d_editor_core::grid_snap::ids::GS_INSPECT_HEADER,
        ),
    ];
    for (nome, id) in cabecalhos {
        let mut host = MockPanelHost::with_panel::<GridSnapPanel>();
        let mut state = GridSnapPanelState;
        let painted = host.paint::<GridSnapPanel>(&mut state, VIEWPORT);
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| *pid == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| {
                panic!("o titulo `{nome}` nao registou hit-rect — nao se pode clicar")
            });
        assert!(
            !host.store().is_collapsed(id),
            "fixture: `{nome}` ja' nascia dobrada, entao este caso nao mede a dobra"
        );
        for ev in host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5) {
            let _ = host.apply_panel_event::<GridSnapPanel>(&mut state, ev);
        }
        assert!(
            host.store().is_collapsed(id),
            "SECCAO QUE NAO FECHA: o titulo `{nome}` recebe o clique e a seccao nao dobra — o \
             report do dono volta"
        );
    }
}
