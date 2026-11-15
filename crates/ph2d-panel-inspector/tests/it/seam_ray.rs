//! **O cartão do Ray Sensor, versão B** (ficha `docs/GameEngineFuncional/componentes/07_ray_sensor.md`, dono
//! 15/11): Angle no lugar de Direction X·Y, o grupo Signals, o aviso da camada vazia — todo clique pelo
//! ponteiro REAL sobre o rect pintado, nos cartões e fora deles.

use crate::o_cartao_da_fisica::{achado, clica};
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::ray_edits::{InspectorRayInfo, RayFieldEdit, RayQueixa};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_cards_on, set_current_inspector_ray};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 3200.0,
};

fn raio() -> InspectorRayInfo {
    InspectorRayInfo {
        entity_bits: 0x0BAD_CAFE,
        angle: 0.0,
        reach: 6.0,
        layer: 0,
        layer_empty: false,
        on_enter: "wall_ahead".into(),
        on_exit: "wall_gone".into(),
        sees: "Wall".into(),
        sees_at: 4.37,
        clock_playing: true,
        selected_count: 1,
    }
}

fn nos_dois_modos(f: impl Fn(bool)) {
    for on in [true, false] {
        set_cards_on(on);
        f(on);
    }
    set_cards_on(false);
}

fn pinta(h: &mut MockPanelHost, st: &mut InspectorState, cartoes: bool) -> Vec<(NodeId, Rect)> {
    if cartoes {
        crate::o_cartao_da_fisica::pinta(h, st)
    } else {
        let _ = h.paint::<InspectorPanel>(st, VIEWPORT);
        h.settle_section_folds();
        h.paint::<InspectorPanel>(st, VIEWPORT)
    }
}

fn assente(info: &InspectorRayInfo, cartoes: bool) -> (MockPanelHost, InspectorState, Vec<(NodeId, Rect)>) {
    let mut h = crate::host_em_metros();
    h.store_mut()
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_RAY_SECTION, false);
    let mut st = InspectorState::default();
    set_current_inspector_ray(Some(info.clone()));
    let r = pinta(&mut h, &mut st, cartoes);
    (h, st, r)
}

fn edicoes(a: Vec<EditorAction>) -> Vec<RayFieldEdit> {
    a.into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                edit: ComponentEdit::Ray(e),
                ..
            } => Some(e),
            _ => None,
        })
        .collect()
}

/// ⭐ Angle, Reach e os dois sinais ganham o foco sob o dedo; Origin e Direction X·Y não existem mais.
///
/// **Mutação que deve sangrar:** tirar a linha Angle do `sections/ray.rs`.
#[test]
fn os_campos_ganham_o_foco_sob_o_dedo() {
    nos_dois_modos(|c| {
        for id in [
            i::INSP_RAY_ANGLE,
            i::INSP_RAY_REACH,
            i::INSP_RAY_ON_ENTER,
            i::INSP_RAY_ON_EXIT,
        ] {
            let (mut h, mut st, r) = assente(&raio(), c);
            let rect = achado(&r, id).unwrap_or_else(|| panic!("{id:?} não pintado (cartões {c})"));
            let _ = clica(&mut h, &mut st, rect);
            assert_eq!(h.store().focus_id(), Some(id), "{id:?} morto sob o dedo (cartões {c})");
            set_current_inspector_ray(None);
        }
    });
}

/// ⭐ Digitar 45 no Angle chega ao barramento como `Angle(45)` — em graus, sem conversão de mundo.
///
/// **Mutação que deve sangrar:** mapear `INSP_RAY_ANGLE` a outra edição no `event_ray`, ou pô-lo na
/// tabela de metros do `unidade_mundo` (sairia 0,45 ou 4500).
#[test]
fn digitar_45_no_angle_da_45_graus() {
    let (mut h, mut st, r) = assente(&raio(), true);
    assert!(achado(&r, i::INSP_RAY_ANGLE).is_some());
    let _ = h.drained_actions();
    for ev in h.type_into_number(i::INSP_RAY_ANGLE, "45") {
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let saiu = edicoes(h.drained_actions());
    assert!(
        saiu.iter()
            .any(|e| matches!(e, RayFieldEdit::Angle(v) if (*v - 45.0).abs() < 1e-4)),
        "o Angle não chegou como 45 graus: {saiu:?}"
    );
    set_current_inspector_ray(None);
    set_cards_on(false);
}

/// ⭐ O grupo Signals nasce ABERTO, fecha e abre pelo clique no cabeçalho, e os sinais somem e voltam.
///
/// **Mutação que deve sangrar:** pintar os sinais fora do grupo, ou o grupo nascer fechado.
#[test]
fn o_grupo_signals_abre_e_fecha_pelo_clique() {
    nos_dois_modos(|c| {
        let (mut h, mut st, mut r) = assente(&raio(), c);
        assert!(achado(&r, i::INSP_RAY_ON_ENTER).is_some(), "Signals fechado de fábrica (cartões {c})");
        for aberto in [false, true] {
            let cab = achado(&r, i::INSP_RAY_FOLD_SIGNALS).expect("o cabeçalho Signals");
            let _ = clica(&mut h, &mut st, cab);
            r = pinta(&mut h, &mut st, c);
            for id in [i::INSP_RAY_ON_ENTER, i::INSP_RAY_ON_EXIT] {
                assert_eq!(achado(&r, id).is_some(), aberto, "{id:?} com Signals aberto={aberto} (cartões {c})");
            }
        }
        set_current_inspector_ray(None);
    });
}

/// ⭐ O aviso «Nothing in the layers this ray sees» é a queixa da camada vazia e passa à frente do
/// «não vê nada»; o Angle tem dica de hover (0 = direita, 90 = cima).
#[test]
fn a_camada_vazia_e_a_dica_do_angle() {
    let mut info = raio();
    info.layer_empty = true;
    info.sees = String::new();
    assert_eq!(info.queixa(), Some(RayQueixa::CamadaVazia));
    let (h, _, _) = assente(&info, true);
    let dica = h.store().tooltip_for(i::INSP_RAY_ANGLE).unwrap_or_default();
    assert!(dica.contains("90"), "o Angle sem dica de direção: «{dica}»");
    set_current_inspector_ray(None);
    set_cards_on(false);
}
