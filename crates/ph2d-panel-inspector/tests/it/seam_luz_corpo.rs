//! **A costura dos cartões do CORPO na luz** — «Light Occluder» (`BloqueiaLuz`) e «Unlit»
//! (`NaoRecebeLuz`). O defeito do smoke do dono: o «Add Component» punha o componente e o Inspector
//! não pintava cartão nenhum — nem para o ver, nem para o editar, nem para o retirar. Aqui: os
//! cartões existem onde o componente está, a caixa mostra a opacidade do documento, e cada controlo
//! chega ao barramento com a SUA edição, pelo gesto REAL. O lado do mundo está em
//! `ph2d-app-light2d` (`os_cartoes_do_corpo`).

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::luz_edits::{InspectorCorpoDaLuzInfo, LuzFieldEdit as E};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_corpo_luz};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x1_0000_0031;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn os_dois() -> InspectorCorpoDaLuzInfo {
    InspectorCorpoDaLuzInfo {
        entity_bits: ENTITY,
        oclusor: Some(0.5),
        sem_luz: true,
    }
}

fn pinta(
    info: Option<InspectorCorpoDaLuzInfo>,
) -> (
    MockPanelHost,
    InspectorState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_corpo_luz(info);
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    (host, state, rects)
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
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
                edit: ComponentEdit::Luz(e),
            } if entity_bits == ENTITY => Some(e),
            _ => None,
        })
        .collect()
}

/// ⭐⭐⭐ **Os dois cartões existem onde os componentes estão** — e somem sem eles.
///
/// **Mutações que devem sangrar:** tirar um dos `plano.push` dos cartões · a moldura que pinta sem
/// o componente.
#[test]
fn os_cartoes_pintam_onde_o_componente_esta() {
    let (_, _, rects) = pinta(Some(os_dois()));
    for id in [
        ph2d_editor_core::ids::INSP_LIVE_LUZ_OCLUSOR_SECTION,
        ph2d_editor_core::ids::INSP_LIVE_LUZ_SEM_LUZ_SECTION,
        ids::INSP_LUZ_OPACIDADE,
        ids::INSP_LUZ_RETIRAR_OCLUSOR,
        ids::INSP_LUZ_RETIRAR_SEM_LUZ,
    ] {
        assert!(rect_de(&rects, id).is_some(), "{id:?} não foi pintado");
    }
    let (_, _, so_sem_luz) = pinta(Some(InspectorCorpoDaLuzInfo {
        oclusor: None,
        ..os_dois()
    }));
    assert!(
        rect_de(
            &so_sem_luz,
            ph2d_editor_core::ids::INSP_LIVE_LUZ_SEM_LUZ_SECTION
        )
        .is_some()
    );
    for id in [
        ph2d_editor_core::ids::INSP_LIVE_LUZ_OCLUSOR_SECTION,
        ids::INSP_LUZ_OPACIDADE,
    ] {
        assert!(rect_de(&so_sem_luz, id).is_none(), "{id:?} sem BloqueiaLuz");
    }
    let (_, _, nada) = pinta(None);
    for id in [
        ph2d_editor_core::ids::INSP_LIVE_LUZ_OCLUSOR_SECTION,
        ph2d_editor_core::ids::INSP_LIVE_LUZ_SEM_LUZ_SECTION,
    ] {
        assert!(
            rect_de(&nada, id).is_none(),
            "{id:?} pintado sem componente"
        );
    }
}

/// ⭐⭐ A caixa da OPACIDADE mostra a do documento, não a partida do `populate`.
#[test]
fn a_caixa_mostra_a_opacidade_do_documento() {
    let (host, _, _) = pinta(Some(InspectorCorpoDaLuzInfo {
        oclusor: Some(0.35),
        ..os_dois()
    }));
    set_current_inspector_corpo_luz(None);
    let v = host.store().number_value(ids::INSP_LUZ_OPACIDADE);
    assert!(
        v.is_some_and(|v| (v - 0.35).abs() < 1e-6),
        "a caixa mostra {v:?}"
    );
}

/// ⭐⭐ Digitar a opacidade chega ao barramento como `Opacidade`.
///
/// **Mutações que devem sangrar:** tirar o número do `populate_luz` · o braço `ValueChanged`.
#[test]
fn digitar_a_opacidade_chega_ao_barramento() {
    let (mut host, mut state, _) = pinta(Some(os_dois()));
    let evs = host.type_into_number(ids::INSP_LUZ_OPACIDADE, "0.25");
    aplica(&mut host, &mut state, evs);
    let e = edicoes(host.drained_actions());
    set_current_inspector_corpo_luz(None);
    assert_eq!(e, vec![E::Opacidade(0.25)]);
}

/// ⭐⭐⭐ Cada botão de RETIRAR manda a SUA edição, pelo clique real.
///
/// **Mutações que devem sangrar:** tirar os botões do `populate_luz` · trocar os dois braços.
#[test]
fn cada_botao_de_retirar_manda_a_sua_edicao() {
    for (id, esperado) in [
        (ids::INSP_LUZ_RETIRAR_OCLUSOR, E::RetirarOclusor),
        (ids::INSP_LUZ_RETIRAR_SEM_LUZ, E::RetirarSemLuz),
    ] {
        let (mut host, mut state, rects) = pinta(Some(os_dois()));
        let r = rect_de(&rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado"));
        let evs = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(!evs.is_empty(), "{id:?} está MORTO sob o dedo");
        aplica(&mut host, &mut state, evs);
        let e = edicoes(host.drained_actions());
        set_current_inspector_corpo_luz(None);
        assert_eq!(e, vec![esperado], "{id:?}");
    }
}
