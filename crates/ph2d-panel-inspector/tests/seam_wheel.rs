//! **Behavioral SEAM sweep for §13 Pulley Wheel** (W-Pulley W1).
//!
//! Irmão de `seam_joint.rs`, e com a MESMA disciplina: uma varredura, não uma
//! amostra, e todo clique passa pelo `click_at` REAL. Um `WidgetEvent`
//! sintético pula a checagem de focabilidade do store, então um widget deixado
//! de fora do `populate` fica pintado, hit-registrado, com arm e **morto sob o
//! mouse**, com um teste verde ao lado (o buraco das 36 células do W2c).
//!
//! ⚠️ **Duas destas rows são o ÚNICO gesto que existe.** Até esta seção,
//! `Over`/`Under` e a `order` eram estado autorado, serializado, que muda a
//! rota da corda — e que nenhum gesto do editor alcançava. Por isso a
//! varredura aqui não é higiene: é o que prova que o estado deixou de ser
//! inalcançável.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{InspectorNameInfo, InspectorWheelInfo, WheelFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_name, set_current_inspector_wheel,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_0043;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// Uma roldana presa a uma corda — o estado em que o artista está quando afina
/// uma.
fn wheel() -> InspectorWheelInfo {
    InspectorWheelInfo {
        entity_bits: ENTITY,
        rope_name: "Simple Rope".into(),
        bound: true,
        radius: 0.45,
        order_ui: 1,
        wrap_tag: 0,
    }
}

fn click_real(info: InspectorWheelInfo, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_wheel(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("§13 never painted the widget {id:?}"));
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "clicar no meio de {id:?} produziu {events:?} — o widget é pintado e \
         hit-registrado, mas o store não o considera focável: ele está morto \
         sob o mouse"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    set_current_inspector_wheel(None);
    out
}

fn commit(info: InspectorWheelInfo, id: ph2d_a11y::NodeId, v: f64) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_wheel(Some(info));
    host.set_number_value(id, v);
    let _ = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::ValueChanged(id));
    let out = host.drained_actions();
    set_current_inspector_wheel(None);
    out
}

#[track_caller]
fn expect(actions: &[EditorAction], edit: WheelFieldEdit, what: &str) {
    assert_eq!(
        actions,
        [EditorAction::InspectorWheelEdit {
            entity_bits: ENTITY,
            edit,
        }],
        "{what} não levantou a edição que devia"
    );
}

/// **Cada chip de Wrap escolhe o próprio lado** — o escape manual do pedido (7).
#[test]
fn the_wrap_chips_each_pick_their_own_side() {
    for (i, &id) in ids::INSP_WHEEL_WRAP.iter().enumerate() {
        expect(
            &click_real(wheel(), id),
            WheelFieldEdit::Wrap(i as u8),
            &format!("chip de wrap {i}"),
        );
    }
}

/// **O raio digitado chega ao barramento em metros**, sem conversão nenhuma.
#[test]
fn typing_a_radius_reaches_the_bus() {
    expect(
        &commit(wheel(), ids::INSP_WHEEL_RADIUS, 0.8),
        WheelFieldEdit::Radius(0.8),
        "Radius",
    );
}

/// **A ordem é ORDINAL e 1-based**, e o arredondamento mora na fronteira.
///
/// ⚠️ O piso é 1: a row conta de um, e um `0` chegaria à shell como *"o nó
/// anterior ao primeiro"*, que não existe. Um arrasto que passe do piso é
/// clampado aqui em vez de virar `u16::MAX` num wrap silencioso lá dentro.
#[test]
fn the_order_row_is_a_one_based_ordinal() {
    expect(
        &commit(wheel(), ids::INSP_WHEEL_ORDER, 3.4),
        WheelFieldEdit::Order(3),
        "Order 3,4 arredondado",
    );
    expect(
        &commit(wheel(), ids::INSP_WHEEL_ORDER, 0.0),
        WheelFieldEdit::Order(1),
        "Order 0 pousando no piso",
    );
}

/// **Sem roldana selecionada a §13 não existe** — e nenhum id dela é pintado.
///
/// A metade de AUSÊNCIA da mesma pergunta: uma seção que aparecesse sobre um
/// sprite ofereceria raio e lado a um objeto que não tem nenhum dos dois.
#[test]
fn no_wheel_selected_paints_no_wheel_section() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_wheel(None);
    let painted: Vec<_> = host
        .paint::<InspectorPanel>(&mut state, VIEWPORT)
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    for id in [
        ids::INSP_LIVE_WHEEL_SECTION,
        ids::INSP_WHEEL_RADIUS,
        ids::INSP_WHEEL_ORDER,
        ids::INSP_WHEEL_WRAP[0],
    ] {
        assert!(
            !painted.contains(&id),
            "{id:?} foi pintado sem roldana selecionada"
        );
    }
}

/// **Toda row NUMÉRICA que a §13 pinta é SEMEADA, ESPELHADA e ROTEADA** — o
/// irmão exato do gate da §12, e pela razão que aquele doc conta por extenso: a
/// row `Ratio` shipou com as três metades faltando de uma vez, e o seam de
/// PRESENÇA ao lado ficou verde o tempo todo, porque ele pergunta *a row está na
/// tela?* e ela estava.
///
/// A lista de rows **não é escrita aqui**: é a DIFERENÇA entre o que o Inspector
/// pinta com uma roldana selecionada e sem nenhuma, menos os chips enumerados.
#[test]
fn every_number_row_the_wheel_section_paints_is_seeded_synced_and_routed() {
    let mut not_a_number: Vec<ph2d_a11y::NodeId> = ids::INSP_WHEEL_WRAP.to_vec();
    not_a_number.extend_from_slice(&[
        ids::INSP_WHEEL_WRAP_GROUP,
        ids::INSP_LIVE_WHEEL_SECTION,
        ids::INSP_LIVE_WHEEL_COLOR,
        ph2d_editor_core::widget::INSPECTOR_SCROLLBAR_ID,
    ]);

    // Sentinelas distintas entre si e diferentes das sementes do
    // `populate_wheel` (0,25 e 1), dentro da faixa das duas rows.
    const RADIUS: f32 = 3.5;
    const ORDER: u32 = 7;
    let numbered = InspectorWheelInfo {
        radius: RADIUS,
        order_ui: ORDER,
        ..wheel()
    };

    // ⚠️ O `Name` entra nos DOIS passes, pela razão que o gate da §12 documenta:
    // o `entity_changed` do sync — a porta que decide re-semear as caixas — só
    // dispara com uma entidade selecionada.
    let name = || {
        set_current_inspector_name(Some(InspectorNameInfo {
            entity_bits: ENTITY,
            name: "Simple Rope Wheel 1".into(),
        }));
    };

    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    name();
    let base: Vec<_> = host
        .paint::<InspectorPanel>(&mut state, VIEWPORT)
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    set_current_inspector_name(None);

    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    name();
    set_current_inspector_wheel(Some(numbered.clone()));
    let painted: Vec<_> = host
        .paint::<InspectorPanel>(&mut state, VIEWPORT)
        .into_iter()
        .map(|(n, _)| n)
        .collect();

    let rows: Vec<_> = painted
        .iter()
        .copied()
        .filter(|id| !base.contains(id) && !not_a_number.contains(id))
        .collect();
    assert_eq!(
        rows.len(),
        2,
        "a §13 pintou {} caixas de número; ela tem duas (Radius e Order) — se \
         uma row nova chegou, ela entra nesta varredura sozinha, que é o ponto",
        rows.len()
    );

    let mut seen: Vec<f64> = Vec::new();
    for &id in &rows {
        let v = host.store().number_value(id).unwrap_or_else(|| {
            panic!(
                "{id:?} é pintada e hit-registrada mas o store não a conhece \
                 como número — ela não está no `populate_wheel`, logo está \
                 morta sob o mouse"
            )
        });
        assert!(
            [f64::from(RADIUS), f64::from(ORDER)].contains(&v),
            "{id:?} mostra {v}, que não é nenhum valor do snapshot — ela não \
             está em `sync_wheel_fields`, então a caixa é WRITE-ONLY: digitar \
             funciona e re-selecionar mostra a semente"
        );
        assert!(
            !seen.contains(&v),
            "{id:?} mostra {v}, o MESMO campo que a outra row"
        );
        seen.push(v);
    }
    set_current_inspector_wheel(None);
    set_current_inspector_name(None);

    for &id in &rows {
        let actions = commit(numbered.clone(), id, 5.0);
        assert!(
            matches!(
                actions.as_slice(),
                [EditorAction::InspectorWheelEdit { .. }]
            ),
            "mudar {id:?} produziu {actions:?} — a row não tem braço no \
             `event_wheel`, então digitar nela não faz nada"
        );
    }
}
