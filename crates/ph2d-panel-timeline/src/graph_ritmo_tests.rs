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

/// O ritmo que um arrasto deixou (o último `Retime::Set` da lista).
fn ritmo_de(got: &[TimelineIntent]) -> Retime {
    let Some(TimelineIntent::Retime(RetimeIntent::Set {
        retime: Some(w), ..
    })) = got
        .iter()
        .rev()
        .find(|i| matches!(i, TimelineIntent::Retime(_)))
    else {
        panic!("nenhum ritmo em {got:?}")
    };
    *w
}

/// ⭐ Em Retime (pesquisa 19, opção A) a alça é um ponto da CAIXA DO TEMPO: o ponteiro dentro dela é
/// a alça; o arrasto escreve SÓ o ritmo da chave — nenhum `SetInterp`, o Bounce fica —, num passo de
/// desfazer.
#[test]
fn em_retime_a_alca_e_o_ponto_da_caixa_num_passo() {
    let tr = track(bounce(), None);
    let band = Band::fit(Rect::new(0.0, 0.0, 400.0, 100.0), Some((0.0, 40.0)));
    let c = caixa(band.rect, VIEW, &tr.keys[0], &tr.keys[1]);
    let (x, y) = (c.x + 0.5 * c.w, c.y + c.h - 0.8 * c.h);
    let got = arrasta(true, &tr, &band, (x, y));
    assert_eq!(got.first(), Some(&TimelineIntent::BeginEdit));
    assert_eq!(got.last(), Some(&TimelineIntent::EndEdit));
    assert!(
        !got.iter()
            .any(|i| matches!(i, TimelineIntent::SetInterp { .. })),
        "o easing não se toca: {got:?}"
    );
    let w = ritmo_de(&got);
    assert!(
        (w.x1 - 0.5).abs() < 1.0e-5 && (w.y1 - 0.8).abs() < 1.0e-5,
        "{w:?}"
    );
}

/// O rato acima e à direita da caixa: a alça fica na BORDA, debaixo do rato na coluna dele; e a
/// caixa é do rectângulo da banda, não da escala da velocidade — outra escala, a mesma alça.
#[test]
fn alem_da_caixa_a_alca_fica_na_borda_debaixo_do_rato() {
    let tr = track(bounce(), None);
    let rect = Rect::new(0.0, 0.0, 400.0, 100.0);
    let mut alcas = Vec::new();
    for escala in [(0.0, 40.0), (-300.0, 900.0)] {
        let band = Band::fit(rect, Some(escala));
        let c = caixa(band.rect, VIEW, &tr.keys[0], &tr.keys[1]);
        let got = arrasta(true, &tr, &band, (c.x + 0.3 * c.w, c.y - 40.0));
        alcas.push(ritmo_de(&got));
    }
    let w = alcas[0];
    assert!(
        (w.x1 - 0.3).abs() < 1.0e-5 && w.y1 == 1.0,
        "na borda de cima: {w:?}"
    );
    assert_eq!(
        alcas[0], alcas[1],
        "a escala da velocidade não move a caixa"
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

// ── Auditoria 15/11 (lente da costura): a caixa PINTADA e o clique que ela regista ──────────────

/// Pinta as caixas de `tr` numa banda e devolve o que ficou registado no índice de clique.
fn pinta_caixas(tr: &TrackView, band: &Band, view: TimeView) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    use ph2d_editor_core::screens::hero::HeroLayout;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 800.0, 400.0));
    let st = TimelinePanelState {
        speed_view: true,
        speed_retime: true,
        ..TimelinePanelState::default()
    };
    host.paint_fn(&layout, |ctx| {
        paint_caixas(ctx, Theme::Forge, &st, band, view, tr);
    })
}

/// Quem apanha o clique em `(x, y)`: o ÚLTIMO registo que o contém (a regra do índice de clique).
fn apanha(regs: &[(ph2d_a11y::NodeId, Rect)], x: f32, y: f32) -> Option<ph2d_a11y::NodeId> {
    regs.iter()
        .rev()
        .find(|(_, r)| r.contains(x, y))
        .map(|(id, _)| *id)
}

fn alca(which: u8) -> ph2d_a11y::NodeId {
    ids::timeline_handle_hit_id(9, 1, which)
}

/// ⭐ O clique no centro da alça DESENHADA apanha essa alça. A posição esperada sai da conta à mão
/// (100 px/s, caixa com a folga `Spacing::Sm`), não da `caixa`/`no_ecra` sob teste — a 1.ª volta
/// tinha só testes que usavam a própria `caixa` como oráculo.
#[test]
fn o_clique_no_centro_da_alca_desenhada_apanha_essa_alca() {
    let tr = track(bounce(), Some(W));
    let band = Band::fit(Rect::new(0.0, 0.0, 400.0, 100.0), Some((0.0, 40.0)));
    let regs = pinta_caixas(&tr, &band, VIEW);
    let sm = Spacing::Sm.px();
    let ecra = |u: f32, w: f32| (u * 100.0, sm + (1.0 - w) * (100.0 - 2.0 * sm));
    for (which, (u, w)) in [(0u8, (W.x1, W.y1)), (1, (W.x2, W.y2))] {
        let (x, y) = ecra(u, w);
        assert_eq!(
            apanha(&regs, x, y),
            Some(alca(which)),
            "{which} em ({x}, {y})"
        );
    }
    assert_eq!(regs.len(), 2, "duas alças, nada mais");
}

/// A alça cuja ponta sai da banda (zoom/rolagem com a chave fora) não deixa clique fantasma, e a
/// área de clique da outra nunca passa da borda da banda.
#[test]
fn a_alca_fora_da_banda_nao_deixa_clique() {
    let tr = track(bounce(), None);
    let band = Band::fit(Rect::new(50.0, 0.0, 400.0, 100.0), Some((0.0, 40.0)));
    let view = TimeView {
        time_x: 50.0,
        view_start: 1.0 / 3.0 + 0.03, // a ponta de saída 3 px à esquerda da banda: o círculo
        ..VIEW                        // ainda pinga para dentro, mas a alça não está lá
    };
    let regs = pinta_caixas(&tr, &band, view);
    let ids: Vec<_> = regs.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, vec![alca(1)], "a de u = 1/3 está à esquerda da banda");
    assert_eq!(
        apanha(&regs, 51.0, 50.0),
        None,
        "sem clique fantasma rente à borda"
    );
    for (_, r) in &regs {
        assert!(
            r.x >= band.rect.x && r.x + r.w <= band.rect.x + band.rect.w,
            "{r:?}"
        );
        assert!(
            r.y >= band.rect.y && r.y + r.h <= band.rect.y + band.rect.h,
            "{r:?}"
        );
    }
}

/// Duas alças quase no mesmo ponto: o clique vai à mais perto do rato — de cada lado da mediana.
#[test]
fn com_duas_alcas_juntas_o_clique_vai_a_mais_perto() {
    let junto = Retime {
        x1: 0.5,
        y1: 0.5,
        x2: 0.56,
        y2: 0.5,
    };
    let tr = track(bounce(), Some(junto));
    let band = Band::fit(Rect::new(0.0, 0.0, 400.0, 100.0), Some((0.0, 40.0)));
    let regs = pinta_caixas(&tr, &band, VIEW);
    let y = 50.0;
    assert_eq!(apanha(&regs, 51.0, y), Some(alca(0)), "perto da de saída");
    assert_eq!(apanha(&regs, 55.0, y), Some(alca(1)), "perto da de entrada");
    assert_eq!(
        apanha(&regs, 45.0, y),
        Some(alca(0)),
        "do lado de fora da de saída"
    );
}

/// Um trecho de 8 px ainda mostra as duas alças (a 1.ª volta não pintava nada abaixo de 14 px).
#[test]
fn um_trecho_estreito_ainda_tem_as_duas_alcas() {
    let tr = track(bounce(), None);
    let band = Band::fit(Rect::new(0.0, 0.0, 400.0, 100.0), Some((0.0, 40.0)));
    let view = TimeView {
        px_per_s: 8.0,
        ..VIEW
    };
    let regs = pinta_caixas(&tr, &band, view);
    let mut ids: Vec<_> = regs.iter().map(|(id, _)| *id).collect();
    ids.sort_by_key(|id| id.0);
    let mut esperado = vec![alca(0), alca(1)];
    esperado.sort_by_key(|id| id.0);
    assert_eq!(ids, esperado);
}

/// A banda mais baixa que o artista consegue arrastar ainda pinta e regista a pílula Override ·
/// Retime e o «Reset timing» (a 1.ª volta escondia-os abaixo de ~71 px, com mínimo de 64).
#[test]
fn na_banda_mais_baixa_a_pilula_e_o_reset_continuam_clicaveis() {
    use ph2d_editor_core::screens::hero::HeroLayout;
    let inset = crate::graph_paint::BAND_INSET;
    let h = crate::graph::clamp_graph_h(0.0);
    let row = Rect::new(0.0, 0.0, 600.0, h);
    let band_rect = Rect::new(150.0, inset, 450.0, h - 2.0 * inset);
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 800.0, 400.0));
    let st = TimelinePanelState {
        speed_view: true,
        speed_retime: true,
        ..TimelinePanelState::default()
    };
    let regs = host.paint_fn(&layout, |ctx| {
        paint_switch(ctx, Theme::Forge, &st, (row, band_rect), 150.0);
    });
    for id in [
        ids::TIMELINE_SPEED_OVERRIDE,
        ids::TIMELINE_SPEED_RETIME,
        ids::TIMELINE_RESET_TIMING,
    ] {
        let r = regs.iter().find(|(i, _)| *i == id).map(|(_, r)| *r);
        let r = r.unwrap_or_else(|| panic!("{id:?} não registado a {h} px"));
        assert!(
            r.y >= row.y && r.y + r.h <= row.y + row.h,
            "{id:?} fora da linha: {r:?}"
        );
    }
}
