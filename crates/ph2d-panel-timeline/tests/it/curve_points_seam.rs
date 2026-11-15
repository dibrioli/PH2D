//! **Os pontos da curva Custom no gráfico CLICAM, de ponta a ponta** (dono 15/11).
//!
//! A 1.ª volta tinha testes que chamavam as funções do ponto direto — verdes — e no app o botão
//! direito não apagava nada (o painel descartava todo gesto secundário) e os botões Corner ·
//! Smooth · Symmetric eram pedra (sem registo no `populate`). Aqui o painel é pintado de verdade,
//! o ponteiro vai pelo despacho REAL e o que conta é a curva que o shell receberia.

use ph2d_anim::{EaseSpline, HandleKind, Retime};
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

fn snap(interp: Interp, retime: Option<Retime>) -> TimelineViewSnapshot {
    let kv = |id, t, v, interp| KeyView {
        retime: if id == 1 { retime } else { None },
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
    pinta_r(host, st, interp, None)
}

fn pinta_r(
    host: &mut MockPanelHost,
    st: &mut TimelinePanelState,
    interp: Interp,
    retime: Option<Retime>,
) -> Vec<(ph2d_editor_core::ids::NodeId, Rect)> {
    set_current_timeline(Some(snap(interp, retime)));
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

/// O ritmo das provas: chega cedo e assenta devagar, longe dos cantos da caixa.
const RITMO: Retime = Retime {
    x1: 0.1,
    y1: 0.6,
    x2: 0.5,
    y2: 0.95,
};

/// Algum pedido tirou o ritmo do trecho?
fn tirou_o_ritmo(got: &[TimelineIntent]) -> bool {
    got.iter().any(|i| {
        matches!(
            i,
            TimelineIntent::Retime(ph2d_timeline::RetimeIntent::Set { retime: None, .. })
        )
    })
}

/// Num trecho RETIMADO (aberto 1 do dono, 15/11) o toque na linha DESENHADA cria o ponto ali —
/// o ponto pinta-se debaixo do rato, em cima de `E(W(u))` — e o ritmo fica.
#[test]
fn num_trecho_retimado_o_toque_na_linha_cria_o_ponto_debaixo_do_rato() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut st = TimelinePanelState::default();
    let regs = pinta_r(&mut host, &mut st, Interp::Linear, Some(RITMO));
    let (x, y) = centro(&regs, ids::timeline_curve_line_hit_id(TGT, 1, 10));
    let _ = drain_intents();
    let _ = host.click_at(x, y);
    let _ = pinta_r(&mut host, &mut st, Interp::Linear, Some(RITMO));
    let got = drain_intents();
    assert!(!tirou_o_ritmo(&got), "o toque tirou o ritmo: {got:?}");
    let Some(nova) = got.into_iter().rev().find_map(|i| match i {
        TimelineIntent::SetInterp { interp, .. } => Some(interp),
        _ => None,
    }) else {
        panic!("o toque na curva retimada não criou ponto");
    };
    assert!(matches!(nova, Interp::Spline(ref c) if c.points().len() == 1));
    let regs = pinta_r(&mut host, &mut st, nova, Some(RITMO));
    let (px, py) = centro(&regs, ids::timeline_curve_point_hit_id(TGT, 1, 1, 0));
    assert!(
        (px - x).abs() < 1.5 && (py - y).abs() < 4.0,
        "o ponto nasceu em ({px}, {py}), longe do toque ({x}, {y})"
    );
    set_current_timeline(None);
}

/// Num trecho RETIMADO com pontos, puxar o braço da PONTA na vista de valor edita a alça da ponta
/// da curva — o ritmo e os pontos ficam (o Override apagaria os pontos).
#[test]
fn num_trecho_retimado_com_pontos_o_braco_da_ponta_mantem_ritmo_e_pontos() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut st = TimelinePanelState::default();
    let regs = pinta_r(&mut host, &mut st, com_ponto(), Some(RITMO));
    let (x, y) = centro(&regs, ids::timeline_handle_hit_id(TGT, 1, 0));
    let _ = drain_intents();
    let _ = host.drag_at(x, y, x + 12.0, y - 20.0);
    let _ = pinta_r(&mut host, &mut st, com_ponto(), Some(RITMO));
    let got = drain_intents();
    assert!(!tirou_o_ritmo(&got), "o braço tirou o ritmo: {got:?}");
    let Some(Interp::Spline(c)) = got.into_iter().rev().find_map(|i| match i {
        TimelineIntent::SetInterp { interp, .. } => Some(interp),
        _ => None,
    }) else {
        panic!("o braço da ponta não gravou uma Custom com pontos");
    };
    assert_eq!(c.points().len(), 1, "os pontos sumiram");
    let Interp::Spline(base) = com_ponto() else {
        unreachable!()
    };
    let antes = base.end_handles();
    assert_ne!(c.end_handles().0, antes.0, "a alça da ponta não andou");
    assert_eq!(c.end_handles().1, antes.1, "a outra ponta andou");
    // Agarrar o braço onde está DESENHADO e mexê-lo só na vertical deixa o tempo dele onde estava:
    // o desenho e o arrasto usam o mesmo mapa.
    let mut st = TimelinePanelState::default();
    let regs = pinta_r(&mut host, &mut st, com_ponto(), Some(RITMO));
    let (x, y) = centro(&regs, ids::timeline_handle_hit_id(TGT, 1, 0));
    let _ = drain_intents();
    let _ = host.drag_at(x, y, x, y - 6.0);
    let _ = pinta_r(&mut host, &mut st, com_ponto(), Some(RITMO));
    let Some(Interp::Spline(c)) = ultima() else {
        panic!("o arrasto vertical não gravou");
    };
    let (a, b) = (c.end_handles().0, antes.0);
    assert!(
        (a[0] - b[0]).abs() < 1e-3 && a[1] != b[1],
        "o braço saltou de {b:?} para {a:?}"
    );
    set_current_timeline(None);
}

/// Smoke do dono 15/11 (passo 3): a ponta do braço da 1.ª chave, puxada acima da curva, ficou fora
/// da banda — escondida e sem alvo. Pintado de verdade: o alvo do braço existe, dentro da banda.
#[test]
fn a_ponta_do_braco_acima_da_curva_continua_a_vista_e_agarravel() {
    let Interp::Spline(c) = com_ponto() else {
        unreachable!()
    };
    let alta = Interp::Spline(c.with_end_handles([0.06, 1.3], c.end_handles().1));
    for retime in [None, Some(RITMO)] {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut st = TimelinePanelState::default();
        let regs = pinta_r(&mut host, &mut st, alta, retime);
        let braco = regs
            .iter()
            .find(|(w, _)| *w == ids::timeline_handle_hit_id(TGT, 1, 0))
            .map(|(_, r)| *r);
        let linha = regs
            .iter()
            .find(|(w, _)| *w == ids::timeline_curve_line_hit_id(TGT, 1, 0))
            .map(|(_, r)| *r);
        assert!(
            braco.is_some(),
            "{retime:?}: o braço acima da curva ficou sem alvo (fora da banda)"
        );
        assert!(linha.is_some(), "{retime:?}: a curva não se pintou");
        set_current_timeline(None);
    }
}
