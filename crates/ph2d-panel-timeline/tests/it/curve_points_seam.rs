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

/// Na vista de VELOCIDADE (Override), puxar o braço de uma Custom com pontos já não a converte
/// (apagava os pontos e o ritmo): o painel pintado, o arrasto pelo despacho real.
#[test]
fn na_vista_de_velocidade_o_braco_da_custom_com_pontos_nao_apaga_nada() {
    for retime in [None, Some(RITMO)] {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut st = TimelinePanelState {
            speed_view: true,
            ..TimelinePanelState::default()
        };
        let regs = pinta_r(&mut host, &mut st, com_ponto(), retime);
        let (x, y) = centro(&regs, ids::timeline_handle_hit_id(TGT, 1, 0));
        let _ = drain_intents();
        let _ = host.drag_at(x, y, x + 10.0, y - 15.0);
        let _ = pinta_r(&mut host, &mut st, com_ponto(), retime);
        let got = drain_intents();
        assert!(!tirou_o_ritmo(&got), "{retime:?}: tirou o ritmo: {got:?}");
        let Some(Interp::Spline(c)) = got.into_iter().rev().find_map(|i| match i {
            TimelineIntent::SetInterp { interp, .. } => Some(interp),
            _ => None,
        }) else {
            panic!("{retime:?}: o braço de velocidade apagou os pontos");
        };
        assert_eq!(c.points().len(), 1);
        set_current_timeline(None);
    }
}

/// Os pontos ganham braços de VELOCIDADE no Override (dono 15/11): pintados de verdade, o braço de
/// saída do ponto agarra-se e puxa-se pelo despacho real — a curva continua com o ponto e o ritmo;
/// no Retime (edita-se o ritmo) os braços dos pontos não existem.
#[test]
fn no_override_o_ponto_tem_braco_de_velocidade_que_se_puxa() {
    for retime in [None, Some(RITMO)] {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut st = TimelinePanelState {
            speed_view: true,
            ..TimelinePanelState::default()
        };
        let regs = pinta_r(&mut host, &mut st, com_ponto(), retime);
        let (x, y) = centro(&regs, ids::timeline_curve_point_hit_id(TGT, 1, 1, 2));
        let _ = drain_intents();
        let _ = host.drag_at(x, y, x, y - 25.0);
        let _ = pinta_r(&mut host, &mut st, com_ponto(), retime);
        let got = drain_intents();
        assert!(!tirou_o_ritmo(&got), "{retime:?}: {got:?}");
        let Some(Interp::Spline(c)) = got.into_iter().rev().find_map(|i| match i {
            TimelineIntent::SetInterp { interp, .. } => Some(interp),
            _ => None,
        }) else {
            panic!("{retime:?}: o braço de velocidade do ponto não gravou nada");
        };
        assert_eq!(c.points().len(), 1);
        let Interp::Spline(base) = com_ponto() else {
            unreachable!()
        };
        assert_ne!(
            c.node(1).h_out,
            base.node(1).h_out,
            "{retime:?}: a alça não andou"
        );
        set_current_timeline(None);
    }
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut st = TimelinePanelState {
        speed_view: true,
        speed_retime: true,
        ..TimelinePanelState::default()
    };
    let regs = pinta_r(&mut host, &mut st, com_ponto(), Some(RITMO));
    let id = ids::timeline_curve_point_hit_id(TGT, 1, 1, 2);
    assert!(
        !regs.iter().any(|(w, _)| *w == id),
        "no Retime não há braços de pontos"
    );
    set_current_timeline(None);
}

/// Dono 15/11: «permita inserir pontos mesmo se a curva for um easing». Um toque na linha de uma
/// curva PRONTA (Elastic Out, Quad In-out) cria o ponto debaixo do rato: a curva vira uma Custom
/// com a mesma cara, sem ter de mexer numa alça antes.
#[test]
fn um_toque_numa_curva_pronta_cria_o_ponto_debaixo_do_rato() {
    use ph2d_anim::{Easing, EasingFamily as F, EasingMode as M};
    for interp in [
        Interp::Eased(Easing::new(F::Elastic, M::Out)),
        Interp::Eased(Easing::new(F::Quad, M::InOut)),
    ] {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut st = TimelinePanelState::default();
        let regs = pinta(&mut host, &mut st, interp);
        // Um pedaço da linha que nada cobre (o último registado ganha: braços e âncoras por cima).
        let livre = (5..80)
            .map(|i| ids::timeline_curve_line_hit_id(TGT, 1, i))
            .filter(|id| regs.iter().any(|(w, _)| w == id))
            .map(|id| (id, centro(&regs, id)))
            .find(|(id, (x, y))| {
                let dentro =
                    |r: &Rect| *x >= r.x && *x <= r.x + r.w && *y >= r.y && *y <= r.y + r.h;
                regs.iter().rev().find(|(_, r)| dentro(r)).map(|(w, _)| w) == Some(id)
            });
        let (_, (x, y)) = livre.expect("um pedaço livre da linha");
        let _ = drain_intents();
        let _ = host.click_at(x, y);
        let _ = pinta(&mut host, &mut st, interp);
        let todos = drain_intents();
        let ultima = todos.iter().rev().find_map(|i| match i {
            TimelineIntent::SetInterp { interp, .. } => Some(*interp),
            _ => None,
        });
        let Some(nova @ Interp::Spline(_)) = ultima else {
            panic!("{interp:?}: o toque na curva pronta não criou ponto: {todos:?}");
        };
        let escolhido = st.ponto_escolhido.expect("o ponto criado fica escolhido");
        let regs = pinta(&mut host, &mut st, nova);
        let (px, py) = centro(
            &regs,
            ids::timeline_curve_point_hit_id(TGT, 1, escolhido.2 as u8, 0),
        );
        assert!(
            (px - x).abs() < 1.5 && (py - y).abs() < 9.0,
            "{interp:?}: o ponto nasceu em ({px}, {py}), longe do toque ({x}, {y})"
        );
        set_current_timeline(None);
    }
}

/// Auditoria do fecho (15/11): rolada a vista para lá do ponto, a pintura sai cortada e nenhum
/// alvo invisível fica a flutuar sobre a coluna de rótulos (o padrão de `paint_handles`).
#[test]
fn o_ponto_fora_da_vista_nao_deixa_alvo_de_clique() {
    for speed_view in [false, true] {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut st = TimelinePanelState {
            speed_view,
            ..TimelinePanelState::default()
        };
        let curva = {
            let mut c = EaseSpline::from_bezier(1.0 / 3.0, 0.0, 2.0 / 3.0, 1.0);
            assert_eq!(c.insert(0.5), Some(1));
            Interp::Spline(c)
        };
        // Controlo: na vista inteira o ponto (ou o braço de chegada dele) regista-se.
        let parte = if speed_view { 1 } else { 0 };
        let regs = pinta(&mut host, &mut st, curva);
        let _ = centro(&regs, ids::timeline_curve_point_hit_id(TGT, 1, 1, parte));
        st.view_start_s = 0.6;
        let regs = pinta(&mut host, &mut st, curva);
        for part in [0u8, 1] {
            let id = ids::timeline_curve_point_hit_id(TGT, 1, 1, part);
            assert!(
                regs.iter().all(|(w, _)| *w != id),
                "vista {speed_view}: o alvo {part} do ponto fora da vista ficou registado"
            );
        }
        set_current_timeline(None);
        let _ = drain_intents();
    }
}

/// Auditoria do fecho (15/11): na vista de velocidade, numa Custom com pontos RETIMADA, puxar o
/// braço da ponta só na vertical não muda a influência (a ponta desenhada é a alça guardada, lida
/// pelo ritmo, como na vista de valor — antes era o braço encurtado e o agarrão saltava).
#[test]
fn na_vista_de_velocidade_puxar_na_vertical_nao_muda_a_influencia_da_custom_retimada() {
    for retime in [None, Some(RITMO)] {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut st = TimelinePanelState {
            speed_view: true,
            ..TimelinePanelState::default()
        };
        let Interp::Spline(antes) = com_ponto() else {
            unreachable!()
        };
        let regs = pinta_r(&mut host, &mut st, com_ponto(), retime);
        let (x, y) = centro(&regs, ids::timeline_handle_hit_id(TGT, 1, 0));
        let _ = drain_intents();
        let _ = host.drag_at(x, y, x, y - 15.0);
        let _ = pinta_r(&mut host, &mut st, com_ponto(), retime);
        let Some(Interp::Spline(c)) = ultima() else {
            panic!("{retime:?}: o arrasto não deu uma Custom");
        };
        let (a, d) = (antes.end_handles().0[0], c.end_handles().0[0]);
        assert!(
            (a - d).abs() < 0.01,
            "{retime:?}: a influência saltou {a} → {d} sem o rato andar na horizontal"
        );
        set_current_timeline(None);
    }
}
