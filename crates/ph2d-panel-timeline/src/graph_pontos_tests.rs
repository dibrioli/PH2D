//! Os pontos da curva Custom no gráfico (dono 15/11). Banda `from_range(0, 10)` com 100 px de
//! altura e 100 px/s ⇒ à mão: `x = 100·t`, `y = 10·(10 − v)` — o oráculo nunca é a conta sob teste.
use super::*;
use ph2d_anim::Interp;
use ph2d_editor_core::interaction::GestureMods;
use ph2d_timeline::{AnimTarget, KeyId, PropKind, Retime, TimelineIntent};
use ph2d_tokens::Theme;

const VIEW: TimeView = TimeView {
    time_x: 0.0,
    right: 400.0,
    view_start: 0.0,
    px_per_s: 100.0,
};

fn band() -> Band {
    Band::from_range(Rect::new(0.0, 0.0, 400.0, 100.0), 0.0, 10.0)
}

/// `t = 0, v = 0` → `t = 1, v = 10`, ambas seleccionadas.
fn track(interp: Interp) -> TrackView {
    let k = |id, t, v| KeyView {
        retime: None,
        id: KeyId::new(id),
        t_seconds: t,
        value: v,
        interp: Interp::Linear,
        selected: true,
        roving: false,
    };
    let mut k0 = k(1, 0.0, 0.0);
    k0.interp = interp;
    TrackView {
        target: AnimTarget::new(9),
        prop: PropKind::TranslationX,
        entity: 1,
        missing: false,
        buffer_ghost: None,
        pre: ph2d_timeline::Extrap::Hold,
        post: ph2d_timeline::Extrap::Hold,
        expr: None,
        keys: vec![k0, k(2, 1.0, 10.0)],
    }
}

/// A reta com um ponto no meio (`u = 0,5` ⇒ ecrã `(50, 50)`).
fn com_ponto() -> Interp {
    let mut c = EaseSpline::from_bezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0);
    assert_eq!(c.insert(0.5), Some(1));
    Interp::Spline(c)
}

fn g(phase: GesturePhase, (x, y): (f32, f32), button: PointerButton, alt: bool) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::Lane,
        phase,
        x,
        y,
        button,
        mods: GestureMods {
            alt,
            ..GestureMods::default()
        },
    }
}

fn prim(phase: GesturePhase, p: (f32, f32)) -> TimelineGesture {
    g(phase, p, PointerButton::Primary, false)
}

/// A última curva gravada.
fn ultima(intents: &[TimelineIntent]) -> Interp {
    intents
        .iter()
        .rev()
        .find_map(|i| match i {
            TimelineIntent::SetInterp { interp, .. } => Some(*interp),
            _ => None,
        })
        .expect("gravou uma curva")
}

fn espiral(intents: &[TimelineIntent]) {
    assert_eq!(
        intents.first(),
        Some(&TimelineIntent::BeginEdit),
        "{intents:?}"
    );
    assert_eq!(
        intents.last(),
        Some(&TimelineIntent::EndEdit),
        "{intents:?}"
    );
    let n = |w| intents.iter().filter(|i| **i == w).count();
    assert_eq!(n(TimelineIntent::BeginEdit), 1, "um passo de undo");
    assert_eq!(n(TimelineIntent::EndEdit), 1, "um passo de undo");
}

fn no(interp: Interp, i: usize) -> ph2d_anim::SplinePoint {
    match interp {
        Interp::Spline(c) => c.node(i),
        o => panic!("não é Custom com pontos: {o:?}"),
    }
}

/// ⭐ Um toque na reta de uma chave seleccionada cria um ponto ali, EM CIMA da reta, e nenhuma
/// chave: só o easing do trecho muda (num passo de undo), e o ponto fica escolhido.
#[test]
fn um_toque_na_linha_cria_um_ponto_em_cima_dela_sem_chave_nova() {
    let _ = crate::state::drain_intents();
    let tr = track(Interp::Linear);
    let mut st = TimelinePanelState::default();
    apply_line_gesture(&mut st, (9, 1), prim(GesturePhase::Begin, (40.0, 60.0)));
    apply_line_gesture(&mut st, (9, 1), prim(GesturePhase::Click, (40.0, 60.0)));
    resolve_drag(&mut st, &band(), VIEW, &tr);
    let got = crate::state::drain_intents();
    espiral(&got);
    assert!(
        got.iter().all(|i| matches!(
            i,
            TimelineIntent::BeginEdit | TimelineIntent::EndEdit | TimelineIntent::SetInterp { .. }
        )),
        "só o easing muda: {got:?}"
    );
    let p = no(ultima(&got), 1).p;
    assert!(
        (p[0] - 0.4).abs() < 1e-5 && (p[1] - 0.4).abs() < 1e-5,
        "{p:?}"
    );
    assert_eq!(st.ponto_escolhido, Some((9, 1, 1)));
    assert!(st.ponto_drag.is_none());
}

/// Arrastar um ponto leva-o para debaixo do rato: `(70, 20)` = `t 0,7`, `v 8` ⇒ `(0,7; 0,8)`.
#[test]
fn arrastar_um_ponto_leva_o_para_debaixo_do_rato() {
    let _ = crate::state::drain_intents();
    let tr = track(com_ponto());
    let mut st = TimelinePanelState::default();
    let alvo = (9, 1, 1, 0);
    apply_point_gesture(&mut st, alvo, prim(GesturePhase::Begin, (50.0, 50.0)));
    apply_point_gesture(&mut st, alvo, prim(GesturePhase::Update, (70.0, 20.0)));
    resolve_drag(&mut st, &band(), VIEW, &tr);
    apply_point_gesture(&mut st, alvo, prim(GesturePhase::End, (70.0, 20.0)));
    resolve_drag(&mut st, &band(), VIEW, &tr);
    let got = crate::state::drain_intents();
    espiral(&got);
    let p = no(ultima(&got), 1).p;
    assert!(
        (p[0] - 0.7).abs() < 1e-5 && (p[1] - 0.8).abs() < 1e-5,
        "{p:?}"
    );
}

/// O botão direito num ponto apaga-o (o último ponto ⇒ volta à Curve de duas alças).
#[test]
fn o_botao_direito_num_ponto_apaga_o() {
    let _ = crate::state::drain_intents();
    let tr = track(com_ponto());
    let mut st = TimelinePanelState {
        ponto_escolhido: Some((9, 1, 1)),
        ..TimelinePanelState::default()
    };
    let dir = g(
        GesturePhase::Click,
        (50.0, 50.0),
        PointerButton::Secondary,
        false,
    );
    apply_point_gesture(&mut st, (9, 1, 1, 0), dir);
    assert_eq!(st.ponto_escolhido, Some((9, 1, 1)), "o direito não escolhe");
    resolve_drag(&mut st, &band(), VIEW, &tr);
    let got = crate::state::drain_intents();
    espiral(&got);
    assert!(matches!(ultima(&got), Interp::Bezier { .. }), "{got:?}");
    assert_eq!(st.ponto_escolhido, None);
}

/// Corner · Smooth · Symmetric trocam o tipo do ponto escolhido, num passo de undo; Alt ao
/// agarrar uma alça quebra-o em Corner.
#[test]
fn os_tipos_e_o_alt_trocam_o_tipo_do_ponto() {
    let _ = crate::state::drain_intents();
    let tr = track(com_ponto());
    let mut st = TimelinePanelState {
        ponto_escolhido: Some((9, 1, 1)),
        ..TimelinePanelState::default()
    };
    let clique = WidgetEvent::Click(ids::TIMELINE_POINT_CORNER);
    assert!(apply_event(&mut st, clique).is_some());
    resolve_drag(&mut st, &band(), VIEW, &tr);
    let got = crate::state::drain_intents();
    espiral(&got);
    let Interp::Spline(c) = ultima(&got) else {
        panic!("{got:?}");
    };
    assert_eq!(c.kind(1), Some(HandleKind::Corner));

    let tr = track(com_ponto());
    let mut st = TimelinePanelState::default();
    let alvo = (9, 1, 1, 2);
    let alt = g(
        GesturePhase::Begin,
        (60.0, 40.0),
        PointerButton::Primary,
        true,
    );
    apply_point_gesture(&mut st, alvo, alt);
    apply_point_gesture(&mut st, alvo, prim(GesturePhase::End, (60.0, 30.0)));
    resolve_drag(&mut st, &band(), VIEW, &tr);
    let Interp::Spline(c) = ultima(&crate::state::drain_intents()) else {
        panic!("não gravou a Custom");
    };
    assert_eq!(c.kind(1), Some(HandleKind::Corner), "Alt + alça = Corner");
}

/// Sem lugar para mais um ponto (8), carregar na linha não muda nada e fecha o passo de undo.
#[test]
fn sem_lugar_carregar_na_linha_nao_muda_nada() {
    let _ = crate::state::drain_intents();
    let mut c = EaseSpline::from_bezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0);
    for i in 1..=8 {
        assert!(c.insert(f64::from(i) / 9.0).is_some());
    }
    let tr = track(Interp::Spline(c));
    let mut st = TimelinePanelState::default();
    apply_line_gesture(&mut st, (9, 1), prim(GesturePhase::Begin, (95.0, 5.0)));
    resolve_drag(&mut st, &band(), VIEW, &tr);
    let got = crate::state::drain_intents();
    assert_eq!(
        got,
        vec![TimelineIntent::BeginEdit, TimelineIntent::EndEdit]
    );
    assert!(st.ponto_drag.is_none());
}

fn pinta(tr: &TrackView) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    use ph2d_editor_core::screens::hero::HeroLayout;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 800.0, 400.0));
    let st = TimelinePanelState::default();
    host.paint_fn(&layout, |ctx| {
        paint_line_hits(ctx, &band(), VIEW, tr);
        paint_points(ctx, Theme::Forge, &st, &band(), VIEW, tr);
    })
}

/// Quem apanha o clique em `(x, y)`: o ÚLTIMO registo que o contém (a regra do índice de clique).
fn apanha(regs: &[(ph2d_a11y::NodeId, Rect)], x: f32, y: f32) -> Option<ph2d_a11y::NodeId> {
    regs.iter()
        .rev()
        .find(|(_, r)| r.contains(x, y))
        .map(|(id, _)| *id)
}

/// ⭐ No ecrã: o clique no centro do ponto DESENHADO apanha o ponto (não a linha por baixo); perto
/// da linha (4 px) apanha a linha; longe dela, nada (a faixa fica com o clique — caixa de seleção).
#[test]
fn o_clique_apanha_o_ponto_desenhado_e_a_linha_por_perto() {
    let regs = pinta(&track(com_ponto()));
    let ponto = ids::timeline_curve_point_hit_id(9, 1, 1, 0);
    assert_eq!(apanha(&regs, 50.0, 50.0), Some(ponto));
    let linha = apanha(&regs, 20.0, 84.0).expect("perto da linha");
    let linhas: Vec<_> = (0..40)
        .map(|i| ids::timeline_curve_line_hit_id(9, 1, i))
        .collect();
    assert!(linhas.contains(&linha), "{linha:?}");
    assert_eq!(apanha(&regs, 20.0, 40.0), None, "longe da linha: 40 px");
}

/// Um trecho retimado mostra os pontos (em cima do que toca); sem chave seleccionada, nada se
/// regista.
#[test]
fn trecho_retimado_tem_pontos_e_sem_selecao_nao_tem_nada() {
    let mut tr = track(com_ponto());
    tr.keys[0].retime = Some(Retime::new(0.1, 0.6, 0.5, 0.95));
    assert!(!pinta(&tr).is_empty(), "retimado");
    let mut tr = track(com_ponto());
    for k in &mut tr.keys {
        k.selected = false;
    }
    assert!(pinta(&tr).is_empty(), "sem seleção");
}
