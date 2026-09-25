//! ⭐⭐ **AS ESCOLHAS COM NOME DESTE PAINEL PASSAM PELA PORTA DA CASA — e respondem onde são pintadas.**
//!
//! ⛔⛔ Até 2026-09-24 o *Neighborhood*, o *Orientation*/*Offset* do hex, o *Parity* e o *Layer* eram
//! montados à mão, com o nome SEMPRE por cima (`paint_text` + o grupo a toda a largura) — a forma que
//! o dono reprovou no Inspector (*«Label acima do campo numérico! Muito ruim!»*) — e as peças não se
//! declaravam grupo, logo o censo de comandos lia `20` botões onde o painel oferece `2` comandos.
//!
//! ⚠️ **A forma que a porta escolhe não é deste gate** (ao lado do nome quando cabe, PALETA quando
//! não — lei da [`ph2d_editor_core::property_row::paint_choice_row`]). Este gate afirma as duas
//! coisas que são DESTE painel: *elas passam pela porta* e *o clique no centro de cada peça chega ao
//! estado do canvas*.

use ph2d_editor_core::grid_snap::ids as g;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_grid::square::SquareNeighborhood;
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
fn as_escolhas_com_nome_passam_pela_porta() {
    let mut host = MockPanelHost::with_panel::<GridSnapPanel>();
    let mut state = GridSnapPanelState;
    let (_, pintadas) = ph2d_editor_core::property_row::escolha::medindo(|| {
        host.paint::<GridSnapPanel>(&mut state, VIEWPORT);
    });
    // A grelha de omissão é a `Square`: pinta o *Neighborhood* dela e o *Layer* do *Display*.
    for (nome, primeira) in [
        ("Neighborhood", g::GS_CFG_NEIGHBORHOOD_4),
        ("Layer", g::GS_LAYER_IN_FRONT),
    ] {
        assert!(
            pintadas
                .iter()
                .any(|e| e.primeira == primeira && e.pecas == 2),
            "a escolha `{nome}` nao passou pela porta `paint_choice_row` — voltou a ser montada a \
             mao (o nome por cima e as pecas soltas no censo). Pintadas: {pintadas:?}"
        );
    }
}

#[test]
fn cada_peca_responde_onde_e_pintada() {
    let mut host = MockPanelHost::with_panel::<GridSnapPanel>();
    let mut state = GridSnapPanelState;
    let painted = host.paint::<GridSnapPanel>(&mut state, VIEWPORT);
    let centro = |id| {
        let r = painted
            .iter()
            .rev()
            .find(|(pid, _)| *pid == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{id:?} nunca foi pintado"));
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    };

    // ⚠️ O CONTROLO: os valores de omissão são os opostos dos que os cliques pedem, senão as
    //    asserções passariam sem clique nenhum.
    assert_eq!(
        host.grid_snap_state().square_cfg.neighborhood,
        SquareNeighborhood::Von4
    );
    assert!(host.grid_snap_state().grid_in_front);

    for id in [g::GS_CFG_NEIGHBORHOOD_8, g::GS_LAYER_BEHIND] {
        let (cx, cy) = centro(id);
        assert_eq!(
            host.hit_at(cx, cy),
            Some(id),
            "{id:?} e pintado mas outra coisa e dona do centro dele"
        );
        for ev in host.click_at(cx, cy) {
            let _ = host.apply_panel_event::<GridSnapPanel>(&mut state, ev);
        }
    }
    assert_eq!(
        host.grid_snap_state().square_cfg.neighborhood,
        SquareNeighborhood::Moore8,
        "o clique real no `Moore8` nao chegou ao estado da grelha"
    );
    assert!(
        !host.grid_snap_state().grid_in_front,
        "o clique real no `Behind` nao chegou ao estado da grelha"
    );
}
