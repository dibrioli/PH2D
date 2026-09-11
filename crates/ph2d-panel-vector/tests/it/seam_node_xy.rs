//! Seam do **X/Y do NÓ** (plano 25 §9, W6) — as duas fileiras existem quando há mediana, **não**
//! existem quando não há, e o número digitado chega ao barramento.
//!
//! ⚠️ **O número é DIGITADO, não escrito no store.** `set_number_value` espeta o valor e pula o
//! caminho de commit inteiro — foi assim que três gates de faixa do `motion-params` ficaram verdes
//! sobre uma feature que não funcionava em lugar nenhum. Aqui o gesto é `type_into_number`, que
//! foca, escreve caractere a caractere e aperta Enter pelos dispatchers reais.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_tool_vector::{VertexSel, VertexType};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// A seção Vertex só existe com uma seleção de nós; as duas fileiras, só com uma MEDIANA.
fn publish(pos: Option<[f64; 2]>) {
    ph2d_panel_vector::set_selected_vertex_type(Some(VertexSel::Uniform(VertexType::Corner)));
    ph2d_panel_vector::set_current_vertex_pos(pos);
}

/// **Com mediana, as duas fileiras são pintadas e o que se digita chega ao barramento.**
///
/// Mutações que têm de sangrar: (a) tirar os dois ids do `populate` — o campo fica pintado, vivo no
/// olho e **morto sob o teclado**; (b) tirá-los do `is_shell_owned_number` — ele aceita as teclas,
/// mostra o número a mudar e **nunca fala com ninguém**, que é a forma mais cara de um controlo
/// nascer morto (a cicatriz do Z-index, 2026-08-04).
#[test]
fn the_node_coordinate_rows_are_painted_and_what_is_typed_reaches_the_bus() {
    publish(Some([12.0, 25.0]));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    for (id, name) in [(ids::VECTOR_VERT_X, "X"), (ids::VECTOR_VERT_Y, "Y")] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_some(),
            "a fileira {name} do nó não foi pintada com uma mediana publicada"
        );
    }

    // A faixa e o link do chip nascem no `paint` — digitar antes de pintar mede uma fixture que o
    // produto não tem.
    host.paint::<VectorPanel>(&mut panel_state, VIEWPORT);
    let evs = host.type_into_number(ids::VECTOR_VERT_X, "30");
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }

    let forwarded = host.drained_actions().iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v))
                if *id == ids::VECTOR_VERT_X && (*v - 30.0).abs() < 1e-9
        )
    });
    assert!(
        forwarded,
        "o X digitado nunca chegou ao bus como SetValue — a shell não tem o que drenar"
    );
}

/// **Sem mediana, as duas fileiras não são pintadas.**
///
/// Um par de caixas mostrando `0, 0` afirmaria que a seleção está na origem — pior que a ausência,
/// porque é um número errado apresentado como certo. E é o estado normal: a seção Vertex aparece
/// assim que UM nó é selecionado, e um índice que já não descreve vértice nenhum não tem posição.
///
/// Mutação que tem de sangrar: pintar as fileiras incondicionalmente.
#[test]
fn without_a_median_the_rows_are_not_offered() {
    publish(None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    for (id, name) in [(ids::VECTOR_VERT_X, "X"), (ids::VECTOR_VERT_Y, "Y")] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "a fileira {name} foi pintada SEM mediana — ela descreveria a origem, não a seleção"
        );
    }
    // CONTROLE: a seção Vertex continua lá — o que some são as duas fileiras, não a seção.
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_VERT_DELETE)
            .is_some(),
        "o Delete Node tem de continuar pintado — se ele sumiu, a fixture não montou a seção e as \
         duas asserções acima estavam a varrer o vazio"
    );
}
