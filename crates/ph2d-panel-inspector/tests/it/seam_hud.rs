//! **A costura dos COMPRIMENTOS da secção HUD** — report do dono, 27/10, com foto: o cartão
//! «HUD Canvas» mostrava «Reference Width 32 m» com o projecto em px. O componente guarda METROS;
//! o ecrã mostra o que o seletor do projecto escolheu, pela mesma porta do cartão Constraints
//! (`sync_constraint::no_ecra` / `do_ecra`), e a escrita volta por ela.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::hud_edits::{HudFieldEdit as E, HudNumber as N, InspectorHudInfo};
use ph2d_editor_core::project::DisplayUnit;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_hud};
use ph2d_ui_testkit::MockPanelHost;

const BITS: u64 = 0x_C0_17_3B;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};

fn canvas() -> InspectorHudInfo {
    InspectorHudInfo {
        entity_bits: BITS,
        has_canvas: true,
        ref_w: 32.0,
        ref_h: 18.0,
        ..InspectorHudInfo::default()
    }
}

fn numeros(h: &mut MockPanelHost) -> Vec<(N, f32)> {
    h.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits: BITS,
                edit: ComponentEdit::Hud(E::Number(n, v)),
            } => Some((n, v)),
            _ => None,
        })
        .collect()
}

fn escreve(h: &mut MockPanelHost, st: &mut InspectorState, id: ph2d_a11y::NodeId, txt: &str) {
    for ev in h.type_into_number(id, txt) {
        let _ = h.apply_panel_event::<InspectorPanel>(st, ev);
    }
}

/// ⭐⭐ **Com o projecto em px a caixa de referência mostra-se em px** (32 m a 100 px/m = 3200;
/// 18 m = 1800), e o `4000` escrito volta ao componente como 40 m. ⚠️ Trocar o seletor com o
/// mesmo objecto selecionado re-semeia os números (a assinatura da semente leva a unidade).
#[test]
fn a_caixa_de_referencia_do_hud_obedece_a_px_metros_do_projecto() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    h.project_mut().display_unit = DisplayUnit::Meters;
    h.project_mut().pixels_per_meter = 100.0;
    set_current_inspector_hud(Some(canvas()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let em_metros = h.store().number_value(ids::INSP_HUD_NUM[0]);

    h.project_mut().display_unit = DisplayUnit::Pixels;
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let w = h.store().number_value(ids::INSP_HUD_NUM[0]);
    let hh = h.store().number_value(ids::INSP_HUD_NUM[1]);
    let _ = h.drained_actions();
    escreve(&mut h, &mut st, ids::INSP_HUD_NUM[0], "4000");
    let e = numeros(&mut h);
    set_current_inspector_hud(None);

    assert!(
        em_metros.is_some_and(|v| (v - 32.0).abs() < 1e-3),
        "CONTROLO: em metros mostra 32, mostrou {em_metros:?}"
    );
    assert!(
        w.is_some_and(|v| (v - 3200.0).abs() < 1e-2),
        "Reference Width em px mostrou {w:?}"
    );
    assert!(
        hh.is_some_and(|v| (v - 1800.0).abs() < 1e-2),
        "Reference Height em px mostrou {hh:?}"
    );
    assert!(
        e.iter()
            .any(|&(n, v)| n == N::RefWidth && (v - 40.0).abs() < 1e-4),
        "o 4000 px escrito tem de voltar como 40 m: {e:?}"
    );
}

/// ⚠️ **CONTROLO: o contador NÃO é comprimento** — com o projecto em px o `Start` mostra-se e
/// volta tal qual (uma conversão em todos os números da secção ficaria verde acima).
#[test]
fn o_inicio_do_contador_nao_se_converte_em_px() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    h.project_mut().display_unit = DisplayUnit::Pixels;
    h.project_mut().pixels_per_meter = 100.0;
    set_current_inspector_hud(Some(InspectorHudInfo {
        entity_bits: BITS,
        has_counter: true,
        counter_name: "pontos".into(),
        counter_start: 3.0,
        ..InspectorHudInfo::default()
    }));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let mostrado = h.store().number_value(ids::INSP_HUD_NUM[2]);
    let _ = h.drained_actions();
    escreve(&mut h, &mut st, ids::INSP_HUD_NUM[2], "5");
    let e = numeros(&mut h);
    set_current_inspector_hud(None);
    assert!(
        mostrado.is_some_and(|v| (v - 3.0).abs() < 1e-6),
        "{mostrado:?}"
    );
    assert!(
        e.iter()
            .any(|&(n, v)| n == N::CounterStart && (v - 5.0).abs() < 1e-6),
        "{e:?}"
    );
}
