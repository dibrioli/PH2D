//! ⭐⭐⭐ (W7) **As secções NAV COST AREA e NAV LINK e o *Avoid Harm* são PINTADOS, estão VIVOS sob o
//! dedo, e cada clique REAL chega ao barramento com a variante dele** (plano 30, W7).
//!
//! Irmã do [`super::a_seccao_nav_esta_viva`], com o mesmo molde: o clique no meio da caixa pintada,
//! os eventos que o `click_at` devolve, e o barramento drenado.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::nav_edits::{
    InspectorNavAgent, InspectorNavCostArea, InspectorNavInfo, InspectorNavLink, NavAlvoModo,
    NavFieldEdit as E,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_nav};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 3200.0,
};

fn agente(avoid_harm: bool, has_health: bool) -> InspectorNavAgent {
    InspectorNavAgent {
        alvo_modo: NavAlvoModo::Ponto,
        alvo_nome: String::new(),
        alvo_perdido: false,
        alvo_tag: 0,
        alvo_ponto: [1.0, 2.0],
        radius: 0.4,
        arrive: 0.2,
        repath: 0.5,
        stuck_after: 1.0,
        active: true,
        avoidance: true,
        avoid_harm,
        has_health,
        on_arrived: String::new(),
        on_no_path: String::new(),
        on_stuck: String::new(),
        has_body: true,
        has_mover: true,
        mover_reads_keys: false,
        has_platformer: false,
        in_region: true,
        agora: None,
        ordem: None,
        alvo_da_ordem: String::new(),
    }
}

/// Fora do neutro (`NavCostArea::default()` é `3`, não proibida).
fn area(forbidden: bool) -> InspectorNavCostArea {
    InspectorNavCostArea {
        cost: 0.75,
        forbidden,
        has_shape: true,
        body_moves: false,
    }
}

/// Fora do neutro em todo campo (`NavLink::default()` é teleporte, um sentido, custo 0, calado).
fn atalho(teleport: bool) -> InspectorNavLink {
    InspectorNavLink {
        to_nome: "Exit".into(),
        to_perdido: false,
        two_way: true,
        teleport,
        cost: 4.5,
        on_crossed: "passou".into(),
    }
}

fn info(
    agent: Option<InspectorNavAgent>,
    cost_area: Option<InspectorNavCostArea>,
    link: Option<InspectorNavLink>,
) -> InspectorNavInfo {
    InspectorNavInfo {
        entity_bits: 0x00AB_1234,
        region: None,
        agent,
        cost_area,
        link,
        clock_playing: true,
        selected_count: 1,
    }
}

fn host(i: InspectorNavInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_nav(Some(i));
    (h, InspectorState::default())
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

fn edicoes(h: &mut MockPanelHost) -> Vec<E> {
    h.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                edit: ComponentEdit::Nav(e),
                ..
            } => Some(e),
            _ => None,
        })
        .collect()
}

/// O clique REAL no meio da caixa `id`, com os eventos que o despachante devolver aplicados.
fn clica(info: InspectorNavInfo, id: ph2d_a11y::NodeId) -> Vec<E> {
    let (mut h, mut st) = host(info);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let r = rect_de(&rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    for ev in h.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let e = edicoes(&mut h);
    set_current_inspector_nav(None);
    e
}

/// ⭐⭐ **Todo campo das duas secções novas é PINTADO com área clicável** — e o *Avoid Harm* no agente.
///
/// **Mutação que deve sangrar:** tirar qualquer linha dos pintores · tirar uma moldura do
/// `paint_optional_nav`.
#[test]
fn todo_campo_das_seccoes_da_w7_e_pintado() {
    let (mut h, mut st) = host(info(
        Some(agente(true, true)),
        Some(area(false)),
        Some(atalho(true)),
    ));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for id in [
        ids::INSP_NAV_AVOID_HARM,
        ids::INSP_NAV_AREA_COST,
        ids::INSP_NAV_AREA_FORBIDDEN,
        ids::INSP_NAV_LINK_TO,
        ids::INSP_NAV_LINK_TWO_WAY,
        ids::INSP_NAV_LINK_TELEPORT,
        ids::INSP_NAV_LINK_COST,
        ids::INSP_NAV_LINK_ON_CROSSED,
    ] {
        assert!(
            rect_de(&rects, id).is_some(),
            "o campo {id:?} não foi PINTADO com área clicável"
        );
    }
    set_current_inspector_nav(None);
}

/// ⭐⭐⭐ **Cada secção só existe para quem TEM o componente dela** (ADR-0166) — e a área PROIBIDA
/// não pinta o custo (um furo não lê custo: a linha seria um controlo morto).
#[test]
fn cada_seccao_so_existe_para_quem_a_tem_e_a_proibida_nao_le_o_custo() {
    let (mut h, mut st) = host(info(None, Some(area(false)), None));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        rect_de(&rects, ids::INSP_NAV_AREA_COST).is_some(),
        "o CONTROLO"
    );
    assert!(
        rect_de(&rects, ids::INSP_NAV_LINK_TO).is_none(),
        "atalho SEM atalho"
    );
    assert!(
        rect_de(&rects, ids::INSP_NAV_AVOID_HARM).is_none(),
        "agente SEM agente"
    );
    set_current_inspector_nav(None);

    let (mut h, mut st) = host(info(None, Some(area(true)), None));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rect_de(&rects, ids::INSP_NAV_AREA_FORBIDDEN).is_some());
    assert!(
        rect_de(&rects, ids::INSP_NAV_AREA_COST).is_none(),
        "proibida pintou o custo — um campo que ninguém lê"
    );
    set_current_inspector_nav(None);
}

/// ⭐⭐⭐ **«Avoid Harm» sob o DEDO pede o contrário do que o objecto tem.**
///
/// **Mutações que devem sangrar:** tirar o registo do `populate_nav` · tirar o braço do
/// `interruptor_w7` · o braço pedir o mesmo valor · trocar o id com o do `Avoid Others`.
#[test]
fn clicar_em_avoid_harm_pede_o_contrario() {
    for tem in [true, false] {
        assert_eq!(
            clica(
                info(Some(agente(tem, true)), None, None),
                ids::INSP_NAV_AVOID_HARM
            ),
            vec![E::AvoidHarm(!tem)],
            "com o Avoid Harm a {tem}"
        );
    }
}

/// ⭐⭐⭐ **«Forbidden» sob o DEDO pede o contrário.**
#[test]
fn clicar_em_forbidden_pede_o_contrario() {
    for tem in [true, false] {
        assert_eq!(
            clica(
                info(None, Some(area(tem)), None),
                ids::INSP_NAV_AREA_FORBIDDEN
            ),
            vec![E::CostAreaForbidden(!tem)],
            "com a área proibida a {tem}"
        );
    }
}

/// ⭐⭐⭐ **«Teleport» e «Both Ways» sob o DEDO pedem o contrário** — cada um a SUA variante.
///
/// **Mutação que deve sangrar:** trocar os dois braços do `interruptor_w7`.
#[test]
fn clicar_no_teleporte_e_nos_dois_sentidos_pede_o_contrario() {
    for tem in [true, false] {
        assert_eq!(
            clica(
                info(None, None, Some(atalho(tem))),
                ids::INSP_NAV_LINK_TELEPORT
            ),
            vec![E::LinkTeleport(!tem)],
            "com o teleporte a {tem}"
        );
    }
    assert_eq!(
        clica(
            info(None, None, Some(atalho(true))),
            ids::INSP_NAV_LINK_TWO_WAY
        ),
        vec![E::LinkTwoWay(false)],
        "a fixtura tem os dois sentidos ligados"
    );
}

/// ⭐⭐⭐ **Os campos de texto e de número das secções novas estão VIVOS sob o dedo** (o clique dá o
/// foco) **e chegam ao barramento com a variante deles.**
///
/// **Mutações que devem sangrar:** tirar um id do `populate_nav` · tirar ou trocar um braço do texto
/// ou do número no despacho.
#[test]
fn os_campos_novos_estao_vivos_e_chegam_ao_barramento() {
    let i = || info(None, Some(area(false)), Some(atalho(true)));
    for id in [
        ids::INSP_NAV_AREA_COST,
        ids::INSP_NAV_LINK_TO,
        ids::INSP_NAV_LINK_COST,
        ids::INSP_NAV_LINK_ON_CROSSED,
    ] {
        let (mut h, mut st) = host(i());
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let r = rect_de(&rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado"));
        let _ = h.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(
            h.store().focus_id(),
            Some(id),
            "clicar no meio de {id:?} não lhe deu o foco — MORTO SOB O DEDO"
        );
        set_current_inspector_nav(None);
    }
    const ESCRITO: &str = "Portal B";
    for (id, e) in [
        (ids::INSP_NAV_LINK_TO, E::LinkTo(ESCRITO.into())),
        (
            ids::INSP_NAV_LINK_ON_CROSSED,
            E::LinkOnCrossed(ESCRITO.into()),
        ),
    ] {
        let (mut h, mut st) = host(i());
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        h.set_text(id, ESCRITO);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::TextChanged(id));
        assert_eq!(edicoes(&mut h), vec![e], "escrever em {id:?}");
        set_current_inspector_nav(None);
    }
    for (id, e) in [
        (ids::INSP_NAV_AREA_COST, E::CostAreaCost(7.25)),
        (ids::INSP_NAV_LINK_COST, E::LinkCost(7.25)),
    ] {
        let (mut h, mut st) = host(i());
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        h.set_number_value(id, 7.25);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::ValueChanged(id));
        assert_eq!(edicoes(&mut h), vec![e], "o número de {id:?}");
        set_current_inspector_nav(None);
    }
}

/// ⭐⭐⭐ **Os campos mostram o que o OBJECTO tem, e não os de fábrica** — a semente do `sync_nav`.
#[test]
fn os_campos_novos_mostram_o_que_o_objecto_tem() {
    let (mut h, mut st) = host(info(None, Some(area(false)), Some(atalho(true))));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, esperado) in [
        (ids::INSP_NAV_AREA_COST, 0.75),
        (ids::INSP_NAV_LINK_COST, 4.5),
    ] {
        let lido = h.store().number_value(id).expect("registado");
        assert!((lido - esperado).abs() < 1.0e-5, "{id:?} mostra {lido}");
    }
    assert_eq!(h.store().text(ids::INSP_NAV_LINK_TO), Some("Exit"));
    assert_eq!(
        h.store().text(ids::INSP_NAV_LINK_ON_CROSSED),
        Some("passou")
    );
    set_current_inspector_nav(None);
}

/// As frases PINTADAS numa pintura deste instantâneo (o censo dos avisos).
fn frases(i: InspectorNavInfo) -> Vec<String> {
    let (mut h, mut st) = host(i);
    let (_, avisos) = ph2d_panel_inspector::censo_dos_avisos::medindo(|| {
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    });
    set_current_inspector_nav(None);
    avisos.into_iter().map(|a| a.texto).collect()
}

/// ⭐⭐⭐ **Cada caixa que não faz nada DIZ porquê, NO PAINEL** — e cala-se quando faz.
///
/// **Mutações que devem sangrar:** tirar qualquer `aviso` do `nav_custo` · pintar a frase da vida
/// com o Health presente · a queixa da área ignorar a forma.
#[test]
fn as_frases_da_w7_aparecem_quando_e_so_quando_valem() {
    use ph2d_i18n::tr;
    let sem_vida = tr("panel.inspector.nav.no_health_nothing_hurts_it");
    assert!(frases(info(Some(agente(true, false)), None, None)).contains(&sem_vida.into()));
    for (a, porque) in [
        (agente(true, true), "com Health"),
        (agente(false, false), "desligado"),
    ] {
        assert!(
            !frases(info(Some(a), None, None)).contains(&sem_vida.into()),
            "{porque}: a frase «nada o fere» não vale"
        );
    }

    let sem_forma = tr("panel.inspector.nav.area_needs_a_collider");
    let anda = tr("panel.inspector.nav.area_body_moves");
    let proibida = tr("panel.inspector.nav.no_agent_enters");
    let mut c = area(false);
    assert!(
        frases(info(None, Some(c), None))
            .iter()
            .all(|f| f != sem_forma && f != anda && f != proibida),
        "a área boa e não proibida cala-se"
    );
    c.has_shape = false;
    assert!(frases(info(None, Some(c), None)).contains(&sem_forma.into()));
    c.has_shape = true;
    c.body_moves = true;
    assert!(frases(info(None, Some(c), None)).contains(&anda.into()));
    assert!(frases(info(None, Some(area(true)), None)).contains(&proibida.into()));

    let sem_saida = tr("panel.inspector.nav.link_has_no_exit");
    let perdida = tr("panel.inspector.nav.link_exit_lost");
    let mut l = atalho(true);
    assert!(
        frases(info(None, None, Some(l.clone())))
            .iter()
            .all(|f| f != sem_saida && f != perdida),
        "o atalho com saída cala-se"
    );
    l.to_nome.clear();
    assert!(frases(info(None, None, Some(l.clone()))).contains(&sem_saida.into()));
    l.to_perdido = true;
    assert!(frases(info(None, None, Some(l))).contains(&perdida.into()));
}
