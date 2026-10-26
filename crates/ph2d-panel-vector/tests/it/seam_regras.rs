//! ⭐ **A tool Regras pelo seam do painel** (W4.6, `docs/BlenderConstraints/04_W4_cad.md` §3.7): o pill na coluna ①
//! chega à tool; a ④ mostra SÓ as regras que a escolha permite e o clique vira pedido; a ② mostra o número de uma
//! medida escolhida (nunca de uma REF) e o valor escrito vira pedido.

use ph2d_editor_core::interaction::{BarButton, WidgetEvent};
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_panel_vector::VectorPanel;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::state_cad::{
    self, CadVista, Escolhida, PedidoCad, Possivel, VECTOR_CAD_VALUE,
};
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot, VectorTool};

fn no_modo(mode: DrawMode) {
    ph2d_panel_vector::state::set_current_vector_style(Some(VectorStyleSnapshot {
        mode,
        ..VectorStyleSnapshot::default()
    }));
}

fn ids(grupos: &[Vec<BarButton>]) -> Vec<ph2d_a11y::NodeId> {
    grupos.iter().flatten().map(|b| b.id).collect()
}

#[test]
fn the_rules_tool_sits_in_the_left_column() {
    let mut host = crate::host_aberto();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();
    no_modo(DrawMode::Select);
    let barras = ph2d_panel_vector::bars::tool_bars(host.store(), None, Vec::new());
    assert!(
        ids(&barras.column).contains(&ph2d_tool_vector::ids::VECTOR_MODE_RULES),
        "a coluna ① não tem o botão Rules"
    );
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_RULES),
    );
    assert_eq!(outcome, EventOutcome::Consumed, "o pill Rules morre no painel");
    crate::seam::drain_into_tool(&mut host, &mut tool);
    assert_eq!(tool.mode(), DrawMode::Rules, "o clique não chegou à tool");
}

#[test]
fn a_choice_offers_only_the_rules_it_allows_and_a_click_becomes_a_request() {
    let mut host = crate::host_aberto();
    let mut panel_state = VectorPanelState;
    no_modo(DrawMode::Rules);
    let _ = state_cad::drena();
    state_cad::set_cad_vista(CadVista {
        possiveis: vec![Possivel::Igual, Possivel::Angulo],
        ..CadVista::default()
    });
    let barras = ph2d_panel_vector::bars::tool_bars(host.store(), None, Vec::new());
    assert_eq!(
        ids(&barras.actions),
        vec![Possivel::Igual.id(), Possivel::Angulo.id()],
        "a ④ tem de mostrar só as regras possíveis"
    );
    for p in [Possivel::Igual, Possivel::Angulo] {
        let outcome =
            host.apply_panel_event::<VectorPanel>(&mut panel_state, WidgetEvent::Click(p.id()));
        assert_eq!(outcome, EventOutcome::Consumed, "o botão {p:?} morre no painel");
    }
    assert_eq!(
        state_cad::drena(),
        vec![
            PedidoCad::Junta(Possivel::Igual),
            PedidoCad::Junta(Possivel::Angulo)
        ]
    );
    // Controlo: fora do modo Regras a ④ não é a das regras.
    no_modo(DrawMode::Node);
    let barras = ph2d_panel_vector::bars::tool_bars(host.store(), None, Vec::new());
    assert!(!ids(&barras.actions).contains(&Possivel::Igual.id()));
}

#[test]
fn a_chosen_dimension_shows_its_number_and_the_typed_value_becomes_a_request() {
    let mut host = crate::host_aberto();
    let mut panel_state = VectorPanelState;
    no_modo(DrawMode::Rules);
    let _ = state_cad::drena();
    let tem_numero = |host: &ph2d_ui_testkit::MockPanelHost| {
        ph2d_panel_vector::bars::tool_bars(host.store(), None, Vec::new())
            .row_lead
            .iter()
            .any(|e| matches!(e, ToolRailEntry::Number { id, .. } if *id == VECTOR_CAD_VALUE))
    };
    for (tipo, referencia, esperado) in [
        (Possivel::Distancia, false, true),
        (Possivel::Distancia, true, false),
        (Possivel::Vertical, false, false),
    ] {
        state_cad::set_cad_vista(CadVista {
            escolhida: Some(Escolhida { tipo: Some(tipo), referencia }),
            ..CadVista::default()
        });
        assert_eq!(tem_numero(&host), esperado, "{tipo:?} REF={referencia}");
    }
    state_cad::set_cad_vista(CadVista {
        escolhida: Some(Escolhida {
            tipo: Some(Possivel::Angulo),
            referencia: false,
        }),
        ..CadVista::default()
    });
    ph2d_editor_core::panel::PanelHostInternal::store_mut(&mut host)
        .set_number_value(VECTOR_CAD_VALUE, 110.0);
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(VECTOR_CAD_VALUE),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(state_cad::VECTOR_CAD_DELETE),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        state_cad::drena(),
        vec![PedidoCad::Medida(110.0), PedidoCad::Apaga]
    );
}
