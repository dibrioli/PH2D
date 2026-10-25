//! ⭐⭐ **«This frame» sob o dedo** (report do dono, 25/10): clicar no número seleciona-o INTEIRO (o
//! que se digita substitui), e a linha nunca some — sem tempo próprio ela fica APAGADA, mostra o
//! ritmo da animação e continua editável (digitar dá ao quadro o seu tempo).

use crate::seam_anim::{anim, clear, host_with};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::hero::{AnimFieldEdit, InspectorAnimInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_anim};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn edits(actions: &[EditorAction]) -> Vec<AnimFieldEdit> {
    actions
        .iter()
        .filter_map(|a| match a {
            EditorAction::InspectorAnimEdit { edit, .. } => Some(edit.clone()),
            _ => None,
        })
        .collect()
}

/// `walk` (90 ms por quadro) a mostrar a célula 1, com `own` ms nela (`0` = sem tempo próprio).
fn on_cell_1(own: u32) -> InspectorAnimInfo {
    let mut info = anim(false, false);
    info.rows[0].per_frame_ms = vec![0, own, 0, 0];
    info
}

fn rect_of(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

/// Clica no «This frame», digita `typed` + Enter e devolve (texto selecionado logo depois
/// do clique, buffer inteiro, edições levantadas).
fn click_then_type(info: InspectorAnimInfo, typed: &str) -> (String, String, Vec<AnimFieldEdit>) {
    let (mut host, mut st) = host_with(info);
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let r = rect_of(&rects, ids::INSP_ANIM_FRAME_MS_THIS).expect("«This frame» não foi pintado");
    // Um clique de mão REAL: treme 1 px entre o Down e o Up (o defeito vivia aí).
    let (x, y) = (r.x + 2.0, r.y + r.h * 0.5); // LITERAL-PX-OK: à esquerda do texto, byte 0
    for ev in host.drag_at(x, y, x + 1.0, y) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    // Um quadro passa entre o clique e a tecla, como no produto.
    let _ = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let (selected, buffer) = match host.store().get(ids::INSP_ANIM_FRAME_MS_THIS) {
        Some(InteractiveState::NumberInput {
            buffer,
            caret,
            selection_anchor,
            ..
        }) => {
            let a = selection_anchor.unwrap_or(*caret);
            let (lo, hi) = (a.min(*caret), a.max(*caret));
            (buffer[lo..hi].to_owned(), buffer.clone())
        }
        other => panic!("«This frame» não é um NumberInput: {other:?}"),
    };
    let _ = host.drained_actions();
    for ev in host.type_keys_then_enter(ids::INSP_ANIM_FRAME_MS_THIS, typed) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let out = edits(&host.drained_actions());
    clear();
    (selected, buffer, out)
}

/// ⭐ **Clicar seleciona o número INTEIRO, e o que se digita SUBSTITUI** — com tempo próprio (120)
/// e sem ele (o campo mostra o ritmo, 90).
#[test]
fn clicar_no_this_frame_seleciona_o_numero_e_digitar_substitui() {
    for (own, shown) in [(120, "120"), (0, "90")] {
        let (selected, buffer, out) = click_then_type(on_cell_1(own), "250");
        assert_eq!(buffer, shown, "own = {own}: o campo mostrava outro número");
        assert_eq!(
            selected, shown,
            "own = {own}: o clique não selecionou o número inteiro"
        );
        assert_eq!(
            out,
            [AnimFieldEdit::FrameMsAt(0, 1, 250)],
            "own = {own}: «250» + Enter tinha de dar 250 ms à célula 1 (substituir, não juntar)"
        );
    }
}

/// Pinta a fixtura e devolve os rectângulos e se o «This frame» está apagado.
fn painted(info: InspectorAnimInfo) -> (Vec<(ph2d_a11y::NodeId, Rect)>, bool) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_anim(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let dim = host
        .store()
        .number_input_inherited(ids::INSP_ANIM_FRAME_MS_THIS);
    clear();
    (rects, dim)
}

/// ⭐ **A linha nunca some**: sem tempo próprio (e com um tempo IGUAL ao ritmo, que não é próprio)
/// ela está pintada e APAGADA; com tempo próprio, normal — e o controlo seguinte (o «+ Add
/// Animation», logo abaixo) não salta.
#[test]
fn o_this_frame_fica_apagado_sem_tempo_proprio_e_nao_salta() {
    let (own, own_dim) = painted(on_cell_1(120));
    assert!(
        !own_dim,
        "com 120 ms próprios o campo não pode estar apagado"
    );
    for plain in [0, 90] {
        let (rects, dim) = painted(on_cell_1(plain));
        assert!(
            rect_of(&rects, ids::INSP_ANIM_FRAME_MS_THIS).is_some(),
            "{plain}: a linha sumiu"
        );
        assert!(
            dim,
            "{plain}: sem tempo próprio o campo tem de estar apagado"
        );
        assert_eq!(
            rect_of(&rects, ids::INSP_ANIM_FRAME_MS_THIS),
            rect_of(&own, ids::INSP_ANIM_FRAME_MS_THIS),
            "{plain}: o campo mudou de lugar"
        );
        let next = |r: &[(ph2d_a11y::NodeId, Rect)]| rect_of(r, ids::INSP_ANIM_ADD).map(|r| r.y);
        assert!(next(&rects).is_some(), "«+ Add Animation» não foi pintado");
        assert_eq!(
            next(&rects),
            next(&own),
            "{plain}: o controlo seguinte saltou"
        );
    }
}
