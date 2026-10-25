//! **Seam do bloco *Animations* da §14** (GameEngineFuncional N1): cada controlo, pelo `click_at` REAL,
//! levanta a edição certa; desligado, o bloco é só o interruptor.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{
    InspectorCharacterAnimInfo, InspectorPlayerInfo, PlayerFieldEdit,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_player};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn com_anim(has_tags: bool) -> InspectorPlayerInfo {
    InspectorPlayerInfo {
        anim: Some(InspectorCharacterAnimInfo {
            names: ["idle".into(), "run".into(), "jump".into(), "tumble".into()],
            known: [true, true, true, false],
            has_tags,
            run_from: 0.5,
            flip: true,
        }),
        ..crate::seam_player::player()
    }
}

fn painted(info: InspectorPlayerInfo) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_player(None);
    rects
}

fn tem(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> bool {
    rects.iter().any(|(n, _)| *n == id)
}

fn click_real(info: InspectorPlayerInfo, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let r = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("o bloco nunca pintou {id:?}"));
    let events = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "{id:?} está pintado mas morto sob o rato: {events:?}"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    set_current_inspector_player(None);
    out
}

fn edits(actions: &[EditorAction]) -> Vec<PlayerFieldEdit> {
    actions
        .iter()
        .filter_map(|a| match a {
            EditorAction::InspectorPlayerEdit { edit, .. } => Some(*edit),
            _ => None,
        })
        .collect()
}

/// Sem o componente o bloco é UMA linha — o interruptor — e ligá-lo levanta o `AnimateByState(true)`.
#[test]
fn desligado_e_so_o_interruptor_e_liga_lo_anexa() {
    let off = crate::seam_player::player();
    let r = painted(off.clone());
    assert!(tem(&r, ids::INSP_PLAYER_ANIM_IDS[1]));
    for id in ids::INSP_PLAYER_ANIM_STATE_IDS {
        assert!(
            !tem(&r, id),
            "um estado pintado sem o componente é um controlo sem dono"
        );
    }
    assert!(!tem(&r, ids::INSP_PLAYER_ANIM_RUN_FROM));
    assert_eq!(
        edits(&click_real(off, ids::INSP_PLAYER_ANIM_IDS[1])),
        [PlayerFieldEdit::AnimateByState(true)]
    );
}

/// Ligado: cada estado é um botão vivo que pede a animação seguinte; o espelho e o limiar escrevem.
#[test]
fn ligado_cada_estado_e_um_botao_vivo() {
    for (i, id) in ids::INSP_PLAYER_ANIM_STATE_IDS.into_iter().enumerate() {
        assert_eq!(
            edits(&click_real(com_anim(true), id)),
            [PlayerFieldEdit::AnimNext(i as u8)],
            "o estado {i}"
        );
    }
    assert_eq!(
        edits(&click_real(
            com_anim(true),
            ids::INSP_PLAYER_ANIM_FLIP_IDS[0]
        )),
        [PlayerFieldEdit::AnimFlip(false)]
    );
    assert_eq!(
        edits(&click_real(com_anim(true), ids::INSP_PLAYER_ANIM_IDS[0])),
        [PlayerFieldEdit::AnimateByState(false)]
    );
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(com_anim(true)));
    host.set_number_value(ids::INSP_PLAYER_ANIM_RUN_FROM, 2.0);
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::INSP_PLAYER_ANIM_RUN_FROM),
    );
    assert_eq!(
        edits(&host.drained_actions()),
        [PlayerFieldEdit::AnimRunFrom(2.0)]
    );
    set_current_inspector_player(None);
}

/// Sem animações na lista os estados não se oferecem (o aviso diz porquê) e o clique sintético recusa.
#[test]
fn sem_animacoes_os_estados_nao_se_oferecem() {
    let r = painted(com_anim(false));
    for id in ids::INSP_PLAYER_ANIM_STATE_IDS {
        assert!(!tem(&r, id));
    }
    assert!(tem(&r, ids::INSP_PLAYER_ANIM_RUN_FROM));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(com_anim(false)));
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Click(ids::INSP_PLAYER_ANIM_IDLE),
    );
    assert!(edits(&host.drained_actions()).is_empty());
    set_current_inspector_player(None);
}
