//! **A costura da secção DRAWING RULES** (W4.5): o «×» e a medida de cada linha chegam ao barramento com o
//! `RegraId` DELA, pelo gesto REAL (`click_at`, `type_into_number`) — os ids da fixtura são `7` e `42`, nunca a
//! posição. O lado do documento está em `ph2d-app-vec` (`cad_inspector`).

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::cad_edits::{
    CadFieldEdit as E, EstadoDaRegra, InspectorCadInfo, InspectorRegraRow,
};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_cad};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x1_0000_00C7;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn regra(id: u32, nome: &'static str, medida: Option<f32>, referencia: bool) -> InspectorRegraRow {
    InspectorRegraRow {
        id,
        nome,
        estado: EstadoDaRegra::Ok,
        medida,
        graus: false,
        referencia,
        // A família CAD é a 9 (`ph2d_ecs::canal::familia::CAD`); o painel não depende da `ph2d-ecs`.
        canal: [9, 1, u16::try_from(id).unwrap_or(0), 0],
    }
}

/// Três linhas: uma horizontal (sem medida, id `7`), uma distância (`42`) e uma distância REF (`5`).
fn regras() -> InspectorCadInfo {
    InspectorCadInfo {
        entity_bits: ENTITY,
        rows: vec![
            regra(7, "app.vec.cad.lei.horizontal", None, false),
            regra(42, "app.vec.cad.lei.distancia", Some(120.0), false),
            regra(5, "app.vec.cad.lei.distancia", Some(80.0), true),
        ],
    }
}

fn pinta(host: &mut MockPanelHost, state: &mut InspectorState) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    host.paint::<InspectorPanel>(state, VIEWPORT)
}

fn aplica(host: &mut MockPanelHost, state: &mut InspectorState, evs: Vec<WidgetEvent>) {
    for ev in evs {
        let _ = host.apply_panel_event::<InspectorPanel>(state, ev);
    }
}

fn edicoes(acoes: Vec<EditorAction>) -> Vec<E> {
    acoes
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::Cad(e),
            } if entity_bits == ENTITY => Some(e),
            _ => None,
        })
        .collect()
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

/// ⭐⭐ O «×» da linha `k` manda apagar a regra DELA — o `42` da 2.ª linha, nunca `1`.
///
/// **Mutações que devem sangrar:** publicar a posição em vez do `row.id` · tirar o `populate_cad` · tirar o
/// braço do `event_cad`.
#[test]
fn o_x_de_cada_linha_apaga_a_regra_dela() {
    for (k, esperado) in [(1, 42), (0, 7)] {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_cad(Some(regras()));
        let rects = pinta(&mut host, &mut state);
        let id = ids::INSP_CAD_APAGAR[k];
        let r = rect_de(&rects, id).unwrap_or_else(|| panic!("a secção não pintou o «×» {k}"));
        let evs = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(!evs.is_empty(), "o «×» {k} está MORTO sob o dedo");
        aplica(&mut host, &mut state, evs);
        let e = edicoes(host.drained_actions());
        set_current_inspector_cad(None);
        assert_eq!(e, vec![E::Apagar(esperado)]);
    }
}

/// ⭐⭐ Escrever na medida da 2.ª linha manda `Medida(42, v)`; a lei sem medida não pinta campo, e a REF mostra o
/// valor sem campo nenhum.
#[test]
fn escrever_a_medida_manda_o_valor_com_o_id_da_regra() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_cad(Some(regras()));
    let rects = pinta(&mut host, &mut state);
    assert!(
        rect_de(&rects, ids::INSP_CAD_MEDIDA[0]).is_none(),
        "a horizontal não tem medida"
    );
    assert!(
        rect_de(&rects, ids::INSP_CAD_MEDIDA[2]).is_none(),
        "uma REF só mostra a medida: um campo ali mentiria"
    );
    assert!(rect_de(&rects, ids::INSP_CAD_MEDIDA[1]).is_some());
    assert_eq!(
        host.store().number_value(ids::INSP_CAD_MEDIDA[1]),
        Some(120.0),
        "o campo mostra a medida do documento"
    );
    let evs = host.type_into_number(ids::INSP_CAD_MEDIDA[1], "150");
    aplica(&mut host, &mut state, evs);
    let e = edicoes(host.drained_actions());
    set_current_inspector_cad(None);
    assert_eq!(e.last(), Some(&E::Medida(42, 150.0)), "{e:?}");
}

/// ⭐ Sem regras do path, a secção não aparece.
#[test]
fn sem_regras_a_seccao_nao_aparece() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_cad(None);
    let rects = pinta(&mut host, &mut state);
    assert!(rect_de(&rects, ph2d_editor_core::ids::INSP_LIVE_CAD_SECTION).is_none());
    assert!(rect_de(&rects, ids::INSP_CAD_APAGAR[0]).is_none());
}
