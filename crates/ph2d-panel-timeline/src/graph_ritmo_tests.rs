use super::*;
use crate::graph::{apply_handle_gesture, resolve_drag};
use ph2d_editor_core::interaction::{GestureMods, GesturePhase, TimelineGesture, TimelineHitKind};
use ph2d_host::PointerButton;
use ph2d_timeline::{
    AnimTarget, Easing, EasingFamily, EasingMode, Interp, KeyId, PropKind, Retime,
};

const W: Retime = Retime {
    x1: 0.6,
    y1: 0.1,
    x2: 0.85,
    y2: 0.5,
};

fn bounce() -> Interp {
    Interp::Eased(Easing {
        family: EasingFamily::Bounce,
        mode: EasingMode::Out,
    })
}

/// `t = 0, v = 0` → `t = 1, v = 10`, ambas seleccionadas.
fn track(interp: Interp, retime: Option<Retime>) -> TrackView {
    let k = |id, t, v| KeyView {
        retime: None,
        id: KeyId::new(id),
        t_seconds: t,
        value: v,
        interp: Interp::Linear,
        selected: true,
        roving: false,
    };
    let (mut k0, k1) = (k(1, 0.0, 0.0), k(2, 1.0, 10.0));
    (k0.interp, k0.retime) = (interp, retime);
    TrackView {
        target: AnimTarget::new(9),
        prop: PropKind::TranslationX,
        entity: 1,
        missing: false,
        buffer_ghost: None,
        pre: ph2d_timeline::Extrap::Hold,
        post: ph2d_timeline::Extrap::Hold,
        expr: None,
        keys: vec![k0, k1],
    }
}

const VIEW: TimeView = TimeView {
    time_x: 0.0,
    right: 400.0,
    view_start: 0.0,
    px_per_s: 100.0,
};

fn gesto(phase: GesturePhase, x: f32, y: f32) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::CurveHandle {
            target: 9,
            key: 1,
            which: 0,
        },
        phase,
        x,
        y,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    }
}

/// Um arrasto inteiro da alça de saída na vista de velocidade, até `(x, y)`.
fn arrasta(
    retime_mode: bool,
    tr: &TrackView,
    band: &Band,
    (x, y): (f32, f32),
) -> Vec<TimelineIntent> {
    let mut st = TimelinePanelState {
        speed_view: true,
        speed_retime: retime_mode,
        ..TimelinePanelState::default()
    };
    apply_handle_gesture(&mut st, 9, 1, 0, gesto(GesturePhase::Begin, 0.0, 0.0));
    apply_handle_gesture(&mut st, 9, 1, 0, gesto(GesturePhase::Update, x, y));
    resolve_drag(&mut st, band, VIEW, tr);
    apply_handle_gesture(&mut st, 9, 1, 0, gesto(GesturePhase::End, x, y));
    resolve_drag(&mut st, band, VIEW, tr);
    state::drain_intents()
}

/// ⭐ Em Retime a alça escreve SÓ o ritmo da chave — nenhum `SetInterp`, o Bounce fica —, num passo
/// de desfazer, e o ritmo dá a velocidade pedida na ponta (`ref · W'(0)`, `ref` = 10 /s).
#[test]
fn em_retime_a_alca_muda_so_o_ritmo_num_passo() {
    let tr = track(bounce(), None);
    let band = Band::fit(Rect::new(0.0, 0.0, 400.0, 100.0), Some((0.0, 40.0)));
    let y = band.y(20.0);
    let got = arrasta(true, &tr, &band, (50.0, y));
    assert_eq!(got.first(), Some(&TimelineIntent::BeginEdit));
    assert_eq!(got.last(), Some(&TimelineIntent::EndEdit));
    assert!(
        !got.iter()
            .any(|i| matches!(i, TimelineIntent::SetInterp { .. })),
        "o easing não se toca: {got:?}"
    );
    let Some(TimelineIntent::Retime(RetimeIntent::Set {
        retime: Some(w), ..
    })) = got
        .iter()
        .rev()
        .find(|i| matches!(i, TimelineIntent::Retime(_)))
    else {
        panic!("nenhum ritmo em {got:?}")
    };
    assert!(
        (w.x1 - 0.5).abs() < 1.0e-6,
        "a influência é o x do ponteiro: {w:?}"
    );
    assert!(
        (10.0 * w.slope(0.0) - 20.0).abs() < 1.0e-2,
        "a velocidade pedida: {w:?}"
    );
}

/// Override RECONSTRÓI (dono 15/11: «exatamente como agora»): sobre um trecho já retimado o arrasto
/// tira o ritmo, a ponta arrastada toca a velocidade pedida e a OUTRA ponta toca o que tocava antes.
#[test]
fn em_override_o_arrasto_tira_o_ritmo_e_a_outra_ponta_nao_salta() {
    let tr = track(Interp::Linear, Some(W));
    let antes_fim = ph2d_timeline::segment_endpoint_speed(&tr.keys[0], &tr.keys[1], 1);
    let band = Band::fit(Rect::new(0.0, 0.0, 400.0, 100.0), Some((0.0, 40.0)));
    let got = arrasta(false, &tr, &band, (50.0, band.y(20.0)));
    assert!(
        got.iter().any(|i| matches!(
            i,
            TimelineIntent::Retime(RetimeIntent::Set { retime: None, .. })
        )),
        "o ritmo sai: {got:?}"
    );
    let Some(TimelineIntent::SetInterp { interp, .. }) = got
        .iter()
        .rev()
        .find(|i| matches!(i, TimelineIntent::SetInterp { .. }))
    else {
        panic!("nenhum easing em {got:?}")
    };
    let mut depois = tr.keys.clone();
    (depois[0].interp, depois[0].retime) = (*interp, None);
    let toca = ph2d_timeline::segment_endpoint_speed(&depois[0], &depois[1], 0);
    let pedida = band.value(band.y(20.0));
    assert!((toca - pedida).abs() < 1.0e-3, "{toca} × {pedida}");
    let fim = ph2d_timeline::segment_endpoint_speed(&depois[0], &depois[1], 1);
    assert!(
        (fim - antes_fim).abs() < 1.0e-3,
        "a outra ponta: {fim} × {antes_fim}"
    );
}

/// As duas células mudam o modo; o «Reset timing» repõe o ritmo das seleccionadas.
#[test]
fn as_celulas_mudam_o_modo_e_o_reset_repoe() {
    let mut st = TimelinePanelState::default();
    let clica = |st: &mut TimelinePanelState, id| apply_event(st, WidgetEvent::Click(id));
    assert!(clica(&mut st, ids::TIMELINE_SPEED_RETIME).is_some());
    assert!(st.speed_retime);
    assert!(clica(&mut st, ids::TIMELINE_SPEED_OVERRIDE).is_some());
    assert!(!st.speed_retime);
    let _ = state::drain_intents();
    assert!(clica(&mut st, ids::TIMELINE_RESET_TIMING).is_some());
    assert_eq!(
        state::drain_intents(),
        vec![TimelineIntent::Retime(RetimeIntent::ResetSelected)]
    );
    assert!(
        clica(&mut st, ids::TIMELINE_SPEED).is_none(),
        "o resto não é comigo"
    );
}
