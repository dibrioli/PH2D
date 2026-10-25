//! As leis do modo (spec/06 F2). ⚠️ O esperado de cada gate está escrito À MÃO — nenhum lê a
//! resposta da função que mede.

use super::*;

const IMG: u64 = 7;
const OTHER: u64 = 9;

fn image_selected() -> ModeState {
    let mut s = ModeState::default();
    s.publish(Some((IMG, ObjectKind::Image)), &[ObjectMode::Draw]);
    s
}

fn object_only_selected() -> ModeState {
    let mut s = ModeState::default();
    s.publish(Some((OTHER, ObjectKind::Empty)), &[]);
    s
}

/// ⭐ GATE — as faces do seletor são os modos do tipo do activo, Object à frente.
#[test]
fn the_selector_faces_are_the_modes_of_the_active_type() {
    crate::test_support::ensure_panel_registry();
    let mut store = WidgetStore::default();
    let menu = image_selected().menu(&mut store).expect("há activo");
    assert_eq!(menu.faces, vec!["Object Mode", "Draw Mode"]);
    assert_eq!(menu.face, "Object Mode");
    let ids: Vec<_> = menu
        .rows
        .iter()
        .filter_map(ToolRailEntry::node_id)
        .collect();
    assert_eq!(
        ids,
        vec![crate::ids::OBJECT_MODE_OBJECT, crate::ids::OBJECT_MODE_DRAW]
    );
}

/// ⭐ GATE — um modo que o tipo não declara NÃO aparece (nem cinzento, D6); sem activo, não há
/// seletor.
#[test]
fn a_mode_the_type_does_not_have_does_not_appear() {
    let mut store = WidgetStore::default();
    let menu = object_only_selected().menu(&mut store).expect("há activo");
    assert_eq!(menu.faces, vec!["Object Mode"]);
    assert_eq!(menu.rows.len(), 1);
    assert!(ModeState::default().menu(&mut store).is_none());
}

/// A face fechada diz o modo em curso.
#[test]
fn the_face_reads_the_current_mode() {
    let mut s = image_selected();
    s.enter(IMG, ObjectMode::Draw);
    let mut store = WidgetStore::default();
    assert_eq!(s.menu(&mut store).expect("activo").face, "Draw Mode");
}

/// ⭐ GATE — `Tab` ida e volta: Object → o modo do tipo → Object → o MESMO modo outra vez.
#[test]
fn tab_goes_there_and_back() {
    let mut s = image_selected();
    assert_eq!(
        s.resolve(ModeRequest::Toggle),
        Step::Enter(ObjectMode::Draw)
    );
    s.enter(IMG, ObjectMode::Draw);
    assert_eq!(s.resolve(ModeRequest::Toggle), Step::Leave);
    s.leave();
    assert_eq!(s.current(), ObjectMode::Object);
    assert_eq!(
        s.resolve(ModeRequest::Toggle),
        Step::Enter(ObjectMode::Draw)
    );
}

/// `Tab` num objecto que só tem Object responde (com a razão), e sem activo também.
#[test]
fn tab_on_an_object_only_type_is_refused() {
    assert_eq!(
        object_only_selected().resolve(ModeRequest::Toggle),
        Step::Refuse
    );
    assert_eq!(
        ModeState::default().resolve(ModeRequest::Toggle),
        Step::Refuse
    );
}

/// O seletor: escolher o modo em curso não faz nada; Object sai; um modo alheio recusa.
#[test]
fn the_selector_rows_resolve() {
    let mut s = image_selected();
    assert_eq!(
        s.resolve(ModeRequest::Enter(ObjectMode::Object)),
        Step::Stay
    );
    assert_eq!(
        s.resolve(ModeRequest::Enter(ObjectMode::Draw)),
        Step::Enter(ObjectMode::Draw)
    );
    s.enter(IMG, ObjectMode::Draw);
    assert_eq!(s.resolve(ModeRequest::Enter(ObjectMode::Draw)), Step::Stay);
    assert_eq!(
        s.resolve(ModeRequest::Enter(ObjectMode::Object)),
        Step::Leave
    );
    assert_eq!(
        object_only_selected().resolve(ModeRequest::Enter(ObjectMode::Draw)),
        Step::Refuse
    );
}

/// ⭐ GATE — a aba que pede um modo (§3.3): entra se o activo o tem; senão Object com a ferramenta
/// de omissão — nunca cria nada.
#[test]
fn a_layout_asks_for_a_mode_only_the_active_can_give() {
    assert_eq!(
        image_selected().resolve(ModeRequest::Open(ObjectKind::Image, ObjectMode::Draw)),
        Step::Enter(ObjectMode::Draw)
    );
    assert_eq!(
        object_only_selected().resolve(ModeRequest::Open(ObjectKind::Image, ObjectMode::Draw)),
        Step::LeaveToDefault
    );
    assert_eq!(
        ModeState::default().resolve(ModeRequest::Open(ObjectKind::Image, ObjectMode::Draw)),
        Step::LeaveToDefault
    );
}

/// ⭐ GATE — o modo só se segura sobre a SUA entidade, sozinha, com o módulo em mãos.
#[test]
fn the_mode_holds_only_its_own_entity() {
    let mut s = image_selected();
    assert!(
        s.still_holds(None, &[], false),
        "em Object não há nada a segurar"
    );
    s.enter(IMG, ObjectMode::Draw);
    assert!(s.still_holds(Some(IMG), &[], true));
    assert!(
        !s.still_holds(Some(OTHER), &[], true),
        "a selecção mudou por outra porta"
    );
    assert!(
        s.still_holds(None, &[], true),
        "desseleccionar saiu do modo (escolha do dono, 05/10: não sai)"
    );
    assert!(
        !s.still_holds(Some(IMG), &[OTHER], true),
        "uma segunda seleccionada"
    );
    assert!(!s.still_holds(Some(IMG), &[], false), "o módulo largou-a");
}

/// ⭐⭐ GATE (dono 24/10) — escolher OUTRO objecto num modo sai dele; o próprio e o limpar ficam; em
/// Object nada sai.
#[test]
fn choosing_another_object_in_a_creation_mode_leaves_it() {
    assert!(
        !switches(None, None, false, Some(OTHER), false),
        "em Object"
    );
    assert!(
        !switches(None, None, false, None, true),
        "em Object, intervalo"
    );
    assert!(switches(Some(IMG), None, false, Some(OTHER), false));
    assert!(
        !switches(Some(IMG), None, false, Some(IMG), false),
        "o próprio"
    );
    assert!(
        !switches(Some(IMG), None, false, None, false),
        "limpar não sai (05/10)"
    );
    assert!(
        switches(Some(IMG), None, false, None, true),
        "um intervalo da Hierarquia"
    );
}

/// ⭐ O pedido de troca é consumido UMA vez, e só existe num modo.
#[test]
fn the_release_is_taken_once_and_only_in_a_mode() {
    let mut s = image_selected();
    s.ask_release();
    assert!(!s.take_release(), "em Object não há o que largar");
    s.enter(IMG, ObjectMode::Draw);
    s.ask_release();
    assert!(s.take_release());
    assert!(!s.take_release(), "consumido");
    s.ask_release();
    s.leave();
    assert!(!s.take_release(), "sair esquece o pedido");
}

/// ⭐ GATE — a aba pede o modo de UM tipo: o `Draw` é de três, e a aba do Flip não abre o Painter
/// numa imagem.
#[test]
fn a_layout_asks_for_the_mode_of_one_kind() {
    assert_eq!(
        image_selected().resolve(ModeRequest::Open(ObjectKind::Flip, ObjectMode::Draw)),
        Step::LeaveToDefault
    );
    let mut s = image_selected();
    s.enter(IMG, ObjectMode::Draw);
    assert_eq!(
        s.resolve(ModeRequest::Open(ObjectKind::Image, ObjectMode::Draw)),
        Step::Stay
    );
    assert_eq!(
        s.resolve(ModeRequest::Open(ObjectKind::Flip, ObjectMode::Draw)),
        Step::LeaveToDefault,
        "o Draw em curso é da imagem, não do Flip"
    );
}

/// ⭐ GATE — num modo que edita PARTES (os ossos do esqueleto) a selecção pode ser qualquer parte,
/// várias, ou nenhuma; outro objecto derruba o modo. Num modo do TIPO (o Draw do vetor) as partes são
/// os outros objectos do tipo, e escolher um deles sai (dono 24/10).
#[test]
fn a_mode_of_parts_holds_and_admits_its_parts_only() {
    const PART: u64 = 77;
    const PART2: u64 = 78;
    let mut s = image_selected();
    s.enter(IMG, ObjectMode::Edit);
    s.publish_parts(Some(vec![PART, PART2]));
    assert!(s.still_holds(Some(PART), &[PART2], true), "duas partes");
    assert!(
        s.still_holds(None, &[], true),
        "desseleccionar não sai do Edit"
    );
    assert!(
        s.still_holds(Some(IMG), &[PART], true),
        "o todo e uma parte"
    );
    assert!(!s.still_holds(Some(OTHER), &[], true), "outro objecto");
    assert!(
        !s.still_holds(Some(PART), &[OTHER], true),
        "uma parte e outro objecto"
    );
    assert!(!s.still_holds(Some(PART), &[], false), "o módulo largou-a");
    let parts = Some([PART, PART2].as_slice());
    assert!(
        !switches(Some(IMG), parts, false, Some(PART), false),
        "uma parte de dentro"
    );
    assert!(switches(Some(IMG), parts, false, Some(OTHER), false));
    assert!(!switches(Some(IMG), parts, false, None, false));
    assert!(
        switches(Some(IMG), parts, true, Some(PART), false),
        "num modo do TIPO a «parte» é outro objecto"
    );
    s.leave();
    assert_eq!(s.parts(), None, "sair esquece as partes");
}

/// A entidade trancada é a do modo em curso — e só num modo de criação.
#[test]
fn the_locked_entity_is_the_one_in_the_mode() {
    let mut s = image_selected();
    assert_eq!(s.locked_entity(), None);
    s.enter(IMG, ObjectMode::Draw);
    assert_eq!(s.locked_entity(), Some(IMG));
    s.leave();
    assert_eq!(s.locked_entity(), None);
}

/// O activo é o ÚLTIMO acrescentado — o mesmo que o Painter guarda ao colapsar.
#[test]
fn the_active_object_is_the_last_one_added() {
    assert_eq!(active_of(None, &[]), None);
    assert_eq!(active_of(Some(1), &[]), Some(1));
    assert_eq!(active_of(Some(1), &[2, 3]), Some(3));
}

/// Cada linha do seletor mapeia de volta a um só modo.
#[test]
fn every_row_maps_back_to_its_mode() {
    assert_eq!(
        ObjectMode::of_row(crate::ids::OBJECT_MODE_OBJECT),
        Some(ObjectMode::Object)
    );
    assert_eq!(
        ObjectMode::of_row(crate::ids::OBJECT_MODE_DRAW),
        Some(ObjectMode::Draw)
    );
    assert_eq!(ObjectMode::of_row(crate::ids::TOOL_BAR_OVERFLOW), None);
}

/// A fila dos pulldowns: o seletor fica à frente seja qual for a ordem em que os dois escrevem.
#[test]
fn the_mode_menu_leads_whatever_the_write_order() {
    let m = |l: &str| AreaMenu {
        label: l.into(),
        ..AreaMenu::default()
    };
    let labels = |s: &WidgetStore| {
        s.area_menus()
            .iter()
            .map(|a| a.label.clone())
            .collect::<Vec<_>>()
    };
    let mut store = WidgetStore::default();
    store.set_area_commands(vec![m("View"), m("Shading")], Vec::new());
    store.publish_mode_menu(Some(m("Mode")));
    assert_eq!(labels(&store), ["Mode", "View", "Shading"]);
    store.set_area_commands(vec![m("View")], Vec::new());
    assert_eq!(labels(&store), ["Mode", "View"]);
    store.publish_mode_menu(None);
    assert_eq!(labels(&store), ["View"]);
    store.publish_mode_menu(Some(m("Mode")));
    store.publish_mode_menu(Some(m("Mode")));
    assert_eq!(labels(&store), ["Mode", "View"]);
}
