//! **Os pontos da curva Custom no gráfico CLICAM, de ponta a ponta** (dono 15/11).
//!
//! A 1.ª volta tinha testes que chamavam as funções do ponto direto — verdes — e no app o botão
//! direito não apagava nada (o painel descartava todo gesto secundário) e os botões Corner ·
//! Smooth · Symmetric eram pedra (sem registo no `populate`). Aqui o painel é pintado de verdade,
//! o ponteiro vai pelo despacho REAL e o que conta é a curva que o shell receberia.

use ph2d_anim::{EaseSpline, HandleKind};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::ids;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_timeline::{
    AnimTarget, Interp, KeyId, KeyView, PropKind, TimelineIntent, TimelineViewSnapshot, TrackView,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);
const TGT: u64 = 7;

/// A reta com um ponto no meio — a Custom que o cartão da chave faria.
fn com_ponto() -> Interp {
    let mut c = EaseSpline::from_bezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0);
    assert_eq!(c.insert(0.5), Some(1));
    Interp::Spline(c)
}

fn snap(interp: Interp) -> TimelineViewSnapshot {
    let kv = |id, t, v, interp| KeyView {
        retime: None,
        id: KeyId::new(id),
        t_seconds: t,
        value: v,
        interp,
        selected: true,
        roving: false,
    };
    TimelineViewSnapshot {
        fps: 60.0,
        tracks: vec![TrackView {
            target: AnimTarget::new(TGT),
            prop: PropKind::TranslationX,
            entity: 1,
            missing: false,
            keys: vec![kv(1, 0.0, 0.0, interp), kv(2, 1.0, 10.0, Interp::Linear)],
            buffer_ghost: None,
            pre: ph2d_timeline::Extrap::Hold,
            post: ph2d_timeline::Extrap::Hold,
            expr: None,
        }],
        ..TimelineViewSnapshot::default()
    }
}

fn pinta(
    host: &mut MockPanelHost,
    st: &mut TimelinePanelState,
    interp: Interp,
) -> Vec<(ph2d_editor_core::ids::NodeId, Rect)> {
    set_current_timeline(Some(snap(interp)));
    st.expanded = vec![TGT];
    host.paint::<TimelinePanel>(st, VIEWPORT)
}

fn centro(
    regs: &[(ph2d_editor_core::ids::NodeId, Rect)],
    id: ph2d_editor_core::ids::NodeId,
) -> (f32, f32) {
    let r = regs
        .iter()
        .find(|(w, _)| *w == id)
        .map(|(_, r)| *r)
        .expect("pintado mas nunca registado: morto debaixo do rato");
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// A última curva que o shell receberia.
fn ultima() -> Option<Interp> {
    drain_intents().into_iter().rev().find_map(|i| match i {
        TimelineIntent::SetInterp { interp, .. } => Some(interp),
        _ => None,
    })
}

#[test]
fn o_botao_direito_no_ponto_desenhado_apaga_o() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut st = TimelinePanelState::default();
    let regs = pinta(&mut host, &mut st, com_ponto());
    let (x, y) = centro(&regs, ids::timeline_curve_point_hit_id(TGT, 1, 1, 0));
    let _ = drain_intents();
    let _ = host.right_click_at(x, y);
    let _ = pinta(&mut host, &mut st, com_ponto());
    let got = ultima();
    assert!(
        matches!(got, Some(Interp::Bezier { .. })),
        "o ponto ficou: {got:?}"
    );
    set_current_timeline(None);
}

#[test]
fn o_botao_corner_troca_o_tipo_do_ponto_escolhido() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut st = TimelinePanelState {
        ponto_escolhido: Some((TGT, 1, 1)),
        ..TimelinePanelState::default()
    };
    let regs = pinta(&mut host, &mut st, com_ponto());
    let (x, y) = centro(&regs, ids::TIMELINE_POINT_CORNER);
    let _ = drain_intents();
    for ev in host.click_at(x, y) {
        host.apply_panel_event::<TimelinePanel>(&mut st, ev);
    }
    let _ = pinta(&mut host, &mut st, com_ponto());
    let Some(Interp::Spline(c)) = ultima() else {
        panic!("o Corner não gravou nada");
    };
    assert_eq!(c.kind(1), Some(HandleKind::Corner));
    set_current_timeline(None);
}

#[test]
fn um_toque_na_linha_da_curva_cria_um_ponto() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut st = TimelinePanelState::default();
    let regs = pinta(&mut host, &mut st, Interp::Linear);
    let (x, y) = centro(&regs, ids::timeline_curve_line_hit_id(TGT, 1, 10));
    let _ = drain_intents();
    let _ = host.click_at(x, y);
    let _ = pinta(&mut host, &mut st, Interp::Linear);
    let Some(Interp::Spline(c)) = ultima() else {
        panic!("o toque na reta não criou ponto");
    };
    assert_eq!(c.points().len(), 1);
    set_current_timeline(None);
}
