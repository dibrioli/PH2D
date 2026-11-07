//! ⭐ **Os marcadores em quadro, ao lado de On Loop** (SFX Lab W1): + cria, Frame e Signal escrevem o seu.

use crate::o_inspector_em_cartoes_por_tipo::{abre_tudo, pinta};
use crate::seam_anim::anim;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{AnimFieldEdit, InspectorAnimInfo, InspectorAnimMarker};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_anim,
};
use ph2d_ui_testkit::MockPanelHost;

fn com_marcas() -> InspectorAnimInfo {
    let mut a = anim(true, false);
    a.rows[0].markers = vec![
        InspectorAnimMarker {
            step: 1,
            signal: "left".into(),
        },
        InspectorAnimMarker {
            step: 3,
            signal: "right".into(),
        },
    ];
    a
}

fn palco(
    cartoes: bool,
    a: InspectorAnimInfo,
) -> (MockPanelHost, InspectorState, Vec<(NodeId, Rect)>) {
    set_cards_on(cartoes);
    set_current_inspector_anim(Some(a));
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    abre_tudo(&mut h);
    h.store_mut().set_collapsed(i::INSP_ANIM_ADVANCED, false);
    let _ = pinta(&mut h, &mut st);
    abre_tudo(&mut h);
    h.store_mut().set_collapsed(i::INSP_ANIM_ADVANCED, false);
    let rects = pinta(&mut h, &mut st);
    (h, st, rects)
}

fn fim() {
    set_current_inspector_anim(None);
    set_cards_on(false);
}

fn tem(rects: &[(NodeId, Rect)], id: NodeId) -> bool {
    rects
        .iter()
        .any(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
}

fn edicoes(h: &mut MockPanelHost) -> Vec<AnimFieldEdit> {
    h.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorAnimEdit { edit, .. } => Some(edit),
            _ => None,
        })
        .collect()
}

/// *Mutação: não pintar `marker_rows` no editor ⇒ vermelho.*
#[test]
fn the_rows_show_the_frame_counted_from_one_and_the_signal() {
    // Em cartões os marcadores moram no editor (`o_marcador_no_cartao_e_o_resumo_que_abre_o_editor`).
    {
        let cartoes = false;
        let (h, _st, rects) = palco(cartoes, com_marcas());
        for k in 0..2 {
            assert!(
                tem(&rects, i::INSP_ANIM_MARKER_FRAME[k]),
                "Frame {k} (cartões {cartoes})"
            );
            assert!(tem(&rects, i::INSP_ANIM_MARKER_SIGNAL[k]), "Signal {k}");
            assert!(tem(&rects, i::INSP_ANIM_MARKER_DEL[k]), "× {k}");
        }
        assert!(tem(&rects, i::INSP_ANIM_MARKER_ADD));
        assert_eq!(
            h.store().number_value(i::INSP_ANIM_MARKER_FRAME[1]),
            Some(4.0),
            "1-based na tela"
        );
        let texto = match h.store().get(i::INSP_ANIM_MARKER_SIGNAL[0]) {
            Some(InteractiveState::TextInput { text, .. }) => text.clone(),
            o => panic!("{o:?}"),
        };
        assert_eq!(texto, "left");
        // A escala do Frame vai até o número de quadros da animação.
        let (lo, hi, _) = h
            .store()
            .number_range(i::INSP_ANIM_MARKER_FRAME[0])
            .expect("faixa");
        assert_eq!((lo, hi), (1.0, 4.0));
        fim();
    }
}

#[test]
fn plus_frame_signal_and_remove_reach_the_bus() {
    let (mut h, mut st, _) = palco(false, com_marcas());
    let _ = h.drained_actions();
    h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::Click(i::INSP_ANIM_MARKER_ADD));
    h.store_mut()
        .set_number_value(i::INSP_ANIM_MARKER_FRAME[1], 2.0);
    h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::ValueChanged(i::INSP_ANIM_MARKER_FRAME[1]),
    );
    if let Some(InteractiveState::TextInput { text, .. }) =
        h.store_mut().get_mut(i::INSP_ANIM_MARKER_SIGNAL[0])
    {
        text.clear();
        text.push_str("thud");
    }
    h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::TextChanged(i::INSP_ANIM_MARKER_SIGNAL[0]),
    );
    h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::Click(i::INSP_ANIM_MARKER_DEL[1]));
    assert_eq!(
        edicoes(&mut h),
        [
            AnimFieldEdit::MarkerAdd(0),
            AnimFieldEdit::MarkerStep(0, 1, 1),
            AnimFieldEdit::MarkerSignal(0, 0, "thud".into()),
            AnimFieldEdit::MarkerRemove(0, 1),
        ]
    );
    fim();
}

/// Fora dos cartões, um marcador além do fim continua com a linha e o quadro que ele guarda à vista. Em cartões ele
/// aparece no editor (`um_marcador_passado_do_fim_mostra_se_no_ultimo_e_tira_se`).
#[test]
fn a_marker_past_the_end_keeps_its_row_and_its_number() {
    let mut a = com_marcas();
    a.rows[0].steps = 2;
    let (h, _st, rects) = palco(false, a);
    assert!(tem(&rects, i::INSP_ANIM_MARKER_FRAME[1]));
    assert_eq!(
        h.store().number_value(i::INSP_ANIM_MARKER_FRAME[1]),
        Some(4.0)
    );
    fim();
}

/// Sonda do limite (`MAX_ANIM_MARKER_ROWS`): o custo de pintar a lista cheia.
#[test]
#[ignore = "sonda: --ignored --nocapture"]
fn diag_o_custo_dos_marcadores() {
    use std::time::Instant;
    for n in [0, 8, i::MAX_ANIM_MARKER_ROWS] {
        let mut a = anim(true, false);
        a.rows[0].markers = (0..n)
            .map(|k| InspectorAnimMarker {
                step: 0,
                signal: format!("s{k}"),
            })
            .collect();
        let (mut h, mut st, rects) = palco(true, a);
        let mut melhor = u128::MAX;
        for _ in 0..30 {
            let t = Instant::now();
            let _ = h.paint::<InspectorPanel>(
                &mut st,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 1600.0,
                    h: 900.0,
                },
            );
            melhor = melhor.min(t.elapsed().as_micros());
        }
        let y = |k: usize| {
            rects
                .iter()
                .find(|(id, _)| *id == i::INSP_ANIM_MARKER_FRAME[k])
                .map(|(_, r)| r.y)
        };
        println!(
            "{n} marcadores: {melhor} us por pintura; passo {:?} px",
            y(0).zip(y(1)).map(|(a, b)| b - a)
        );
        fim();
    }
}
