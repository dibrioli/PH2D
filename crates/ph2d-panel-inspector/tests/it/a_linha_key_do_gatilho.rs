//! ⭐ **A linha Key do cartão do Trigger** (onda 4 do F3, dono 09/11): o clique REAL no botão pede
//! ao mapa a tecla da acção da fileira ABERTA; à escuta, o mesmo botão cancela; sem nome, o botão
//! não é alvo. A isca é a fileira `0` (outro nome): um despacho que lesse a linha errada sangra.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::action_trigger_edits::{
    InspectorActionTriggerInfo, InspectorTriggerRow, NoMapa, TeclaDaAccao,
};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_action_trigger};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};
const SEC: u128 = 1_000_000_000;
const FIRE: ph2d_input::ActionId = ph2d_input::ActionId(7);

fn linha(action: &str, no_mapa: NoMapa, tecla: TeclaDaAccao) -> InspectorTriggerRow {
    InspectorTriggerRow {
        item: Default::default(),
        action: action.into(),
        edge: 0,
        signal: "shoot".into(),
        no_mapa,
        tecla,
    }
}

fn fire() -> TeclaDaAccao {
    TeclaDaAccao {
        id: Some(FIRE),
        teclas: vec!["Space".into()],
        tambem: Some(("Space".into(), "jump".into())),
    }
}

/// A fileira aberta é a SEGUNDA (`trigger_selected = 1`); a primeira é a isca.
fn host(alvo: InspectorTriggerRow) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_action_trigger(Some(InspectorActionTriggerInfo {
        entity_bits: 0x7E_C1A,
        rows: vec![linha("isca", NoMapa::Ligada, TeclaDaAccao::default()), alvo],
        clock_playing: true,
        selected_count: 1,
    }));
    let st = InspectorState {
        trigger_selected: 1,
        ..InspectorState::default()
    };
    (h, st)
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

fn clica(h: &mut MockPanelHost, st: &mut InspectorState) -> Vec<EditorAction> {
    clica_em(h, st, SEC)
}

fn clica_em(h: &mut MockPanelHost, st: &mut InspectorState, t0: u128) -> Vec<EditorAction> {
    let rects = h.paint::<InspectorPanel>(st, VIEWPORT);
    let r = rect_de(&rects, ids::INSP_TRIGGER_SET_KEY).expect("a linha Key foi pintada como alvo");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let ponto = |kind, t| PointerEvent {
        kind,
        x: cx,
        y: cy,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    };
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(ponto(PointerKind::Down, t0));
    evs.extend(h.dispatch_pointer_event(ponto(PointerKind::Up, t0 + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre o botão Key não virou evento"
    );
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(st, ev);
    }
    h.drained_actions()
}

fn pedidos(acoes: &[EditorAction]) -> Vec<&str> {
    acoes
        .iter()
        .filter_map(|a| match a {
            EditorAction::ListenForTriggerKey { name } => Some(name.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn o_botao_key_pede_a_tecla_da_fileira_aberta() {
    let (mut h, mut st) = host(linha("fire", NoMapa::Ligada, fire()));
    let acoes = clica(&mut h, &mut st);
    assert_eq!(
        pedidos(&acoes),
        vec!["fire"],
        "o nome da fileira ABERTA, não o da isca"
    );
    set_current_inspector_action_trigger(None);
}

/// «Cria e espera» (dono 09/11): a acção que o mapa não conhece também pede — quem cria é o mapa.
#[test]
fn o_botao_key_numa_accao_por_criar_tambem_pede() {
    let (mut h, mut st) = host(linha("fier", NoMapa::Desconhecida, TeclaDaAccao::default()));
    let acoes = clica(&mut h, &mut st);
    assert_eq!(pedidos(&acoes), vec!["fier"]);
    set_current_inspector_action_trigger(None);
}

/// À escuta, o mesmo botão CANCELA — e não pede outra vez.
#[test]
fn a_escuta_o_botao_key_cancela() {
    let (mut h, mut st) = host(linha("fire", NoMapa::Ligada, fire()));
    h.store_mut().listen_for_binding_replacing(FIRE);
    let acoes = clica(&mut h, &mut st);
    assert!(
        pedidos(&acoes).is_empty(),
        "à escuta o clique cancela, não pede"
    );
    assert_eq!(h.store().input_map_listening(), None, "a escuta desarmou");
    set_current_inspector_action_trigger(None);
}

/// Sem nome na fileira o botão é pintado DESLIGADO e não é alvo.
#[test]
fn sem_nome_o_botao_key_nao_e_alvo() {
    let (mut h, mut st) = host(linha("", NoMapa::Ligada, TeclaDaAccao::default()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rect_de(&rects, ids::INSP_TRIGGER_SET_KEY).is_none());
    set_current_inspector_action_trigger(None);
}

/// O rótulo diz a tecla, «Press a key…» à escuta, «Create and set key» por criar.
#[test]
fn o_rotulo_do_botao_key_diz_o_estado() {
    use ph2d_panel_inspector::rotulo_da_tecla_do_gatilho as rotulo;
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    let com = linha("fire", NoMapa::Ligada, fire());
    assert_eq!(rotulo(h.store(), &com), "Space");
    let sem = linha(
        "fire",
        NoMapa::SemTecla,
        TeclaDaAccao {
            id: Some(FIRE),
            ..TeclaDaAccao::default()
        },
    );
    assert_eq!(
        rotulo(h.store(), &sem),
        ph2d_i18n::tr("panel.inspector.trigger.set_key")
    );
    let por_criar = linha("fier", NoMapa::Desconhecida, TeclaDaAccao::default());
    assert_eq!(
        rotulo(h.store(), &por_criar),
        ph2d_i18n::tr("panel.inspector.trigger.create_and_set_key")
    );
    let mut h = h;
    h.store_mut().listen_for_binding_replacing(FIRE);
    assert_eq!(
        rotulo(h.store(), &com),
        ph2d_i18n::tr("panel.inspector.trigger.press_a_key")
    );
    assert_eq!(
        rotulo(h.store(), &sem),
        ph2d_i18n::tr("panel.inspector.trigger.press_a_key")
    );
}

/// O `Down` sobre o próprio botão Key já desarmou (porta do clique da shell): o clique que ele começa
/// CANCELA, não re-arma; o clique seguinte arma de novo.
#[test]
fn o_clique_que_desarmou_nao_rearma() {
    let (mut h, mut st) = host(linha("fire", NoMapa::Ligada, fire()));
    h.store_mut().listen_for_binding_replacing(FIRE);
    h.store_mut().desarma_escuta_no_clique();
    let acoes = clica(&mut h, &mut st);
    assert!(
        pedidos(&acoes).is_empty(),
        "o clique que cancelou não pede outra vez"
    );
    // ⚠️ Um segundo depois: no mesmo instante seria um DUPLO clique.
    let acoes = clica_em(&mut h, &mut st, 3 * SEC);
    assert_eq!(pedidos(&acoes), vec!["fire"], "o seguinte arma");
    set_current_inspector_action_trigger(None);
}
