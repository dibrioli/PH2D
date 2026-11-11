//! ⭐ **Arrastar um chip da barra para uma faixa** (doc 17 item 6, maquete B4): gesto real, do rect
//! PINTADO do chip até o rect PINTADO da faixa; o efeito lê-se nos intents e no documento.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_panel_timeline::tab::Tab;
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{
    StackHost, StripSource, TimelineDoc, TimelineIntent, TimelineState, TimelineViewSnapshot,
    apply_intent,
};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1040.0,
};

/// «Walk» (2 s dentro, posto em 0 s na Base) e «Jump» vazio; faixas Base e Upper body.
fn estado() -> TimelineState {
    let mut doc = TimelineDoc::new();
    let walk = doc.add_container("Walk".into());
    let dentro = doc
        .add_lane_in(StackHost::Container(walk), "in".into())
        .expect("a faixa de dentro");
    doc.add_strip_to(
        StackHost::Container(walk),
        dentro,
        StripSource::Clip(0),
        0.0,
        2.0,
    )
    .expect("a peça de dentro");
    doc.add_container("Jump".into());
    let base = doc.add_lane("Base".into()).expect("Base");
    doc.add_lane("Upper body".into()).expect("Upper body");
    doc.add_strip_to(
        StackHost::Document,
        base,
        StripSource::Container(0),
        0.0,
        2.0,
    )
    .expect("Walk na Base");
    let mut st = TimelineState::new();
    st.doc = doc;
    st
}

fn snap_de(st: &mut TimelineState) -> TimelineViewSnapshot {
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(st, &ph2d_core::Playhead::default(), false);
    snap
}

fn mesa(st: &mut TimelineState) -> Mesa<TimelinePanel> {
    ferramenta::force_cards_on_this_thread(Some(true));
    set_current_timeline(Some(snap_de(st)));
    let _ = drain_intents();
    let mut m = Mesa::nova(TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    });
    m.assenta_em(JANELA);
    let _ = drain_intents();
    let _ = m.host.drained_actions();
    m
}

fn rect(m: &Mesa<TimelinePanel>, id: NodeId) -> Rect {
    m.rect(id).unwrap_or_else(|| panic!("{id:?} não pintado"))
}

/// Aperta no meio do chip, leva-o a `(x, y)` e (se `solta`) larga; pinta entre os passos.
fn arrasta(m: &mut Mesa<TimelinePanel>, chip: NodeId, (x, y): (f32, f32), solta: bool) {
    let c = rect(m, chip);
    let (cx, cy) = (c.x + c.w * 0.5, c.y + c.h * 0.5);
    let _ = m.ponteiro(PointerKind::Down, cx, cy);
    let _ = m.ponteiro(PointerKind::Move, cx + 12.0, cy + 12.0);
    let _ = m.ponteiro(PointerKind::Move, x, y);
    m.assenta_em(JANELA);
    if solta {
        let _ = m.ponteiro(PointerKind::Up, x, y);
        m.assenta_em(JANELA);
    }
}

/// O meio vertical da faixa `i` e um x da área de tempo (o «+» da faixa mora na coluna dela).
fn sobre_a_faixa(m: &Mesa<TimelinePanel>, i: usize, dx: f32) -> (f32, f32) {
    let mais = rect(m, ids::TIMELINE_LANE_ADD_STRIP[i]);
    let regua = rect(m, ids::TIMELINE_RULER);
    (regua.x + dx, mais.y + mais.h * 0.5)
}

fn add_strips(i: &[TimelineIntent]) -> Vec<(usize, StripSource, f64, f64)> {
    i.iter()
        .filter_map(|i| match *i {
            TimelineIntent::AddStrip {
                lane,
                source,
                t_start,
                t_end,
            } => Some((lane, source, t_start, t_end)),
            _ => None,
        })
        .collect()
}

/// ⭐⭐ Do chip «Walk» pintado à faixa «Upper body» pintada: UMA peça nova, com a fonte, no tempo do
/// ponteiro encaixado ao quadro e com a duração da fonte (2 s). *Mutação: largar na faixa de cima ⇒
/// vermelho.*
#[test]
fn arrastar_o_chip_para_a_faixa_cria_a_peca_ali() {
    let mut st = estado();
    let mut m = mesa(&mut st);
    let alvo = sobre_a_faixa(&m, 1, 300.0);
    arrasta(&mut m, ids::TIMELINE_CONT_OPT[0], alvo, true);
    let feitos = drain_intents();
    let pecas = add_strips(&feitos);
    assert_eq!(pecas.len(), 1, "uma peça: {feitos:?}");
    let (lane, source, t0, t1) = pecas[0];
    assert_eq!(lane, 1, "a faixa sob o ponteiro");
    assert_eq!(source, StripSource::Container(0), "a fonte do chip");
    assert!(
        (t1 - t0 - 2.0).abs() < 1e-9,
        "a duração da fonte: {t0}..{t1}"
    );
    let fps = snap_de(&mut st).fps;
    assert!(
        ((t0 * fps).round() - t0 * fps).abs() < 1e-6,
        "{t0} fora do quadro"
    );
    let regua = rect(&m, ids::TIMELINE_RULER);
    let esperado = m.st.view_start_s + f64::from(alvo.0 - regua.x) / m.st.px_per_s;
    assert!(
        (t0 - esperado).abs() <= 1.0 / fps,
        "{t0} longe do ponteiro {esperado}"
    );
    assert!(
        !feitos.iter().any(|i| matches!(i, TimelineIntent::AddLane)),
        "nenhuma faixa nova"
    );
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ O clique curto no chip faz o de antes: escolhe o container, sem peça.
#[test]
fn o_clique_curto_no_chip_so_escolhe() {
    let mut st = estado();
    let mut m = mesa(&mut st);
    let _ = m.clica(ids::TIMELINE_CONT_OPT[1]);
    m.assenta_em(JANELA);
    assert_eq!(m.st.source_container, Some(1), "o clique não escolheu");
    assert!(
        add_strips(&drain_intents()).is_empty(),
        "o clique criou uma peça"
    );
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐⭐ Abaixo da última faixa: a largada cria a faixa E a peça, e um Ctrl+Z desfaz as duas.
#[test]
fn abaixo_da_ultima_faixa_cria_faixa_e_peca_num_passo() {
    let mut st = estado();
    let mut m = mesa(&mut st);
    let (x, y) = sobre_a_faixa(&m, 1, 300.0);
    let h =
        rect(&m, ids::TIMELINE_LANE_ADD_STRIP[1]).y - rect(&m, ids::TIMELINE_LANE_ADD_STRIP[0]).y;
    arrasta(&mut m, ids::TIMELINE_CONT_OPT[0], (x, y + h), true);
    let feitos = drain_intents();
    let pecas = add_strips(&feitos);
    assert_eq!(pecas.len(), 1, "{feitos:?}");
    assert_eq!(pecas[0].0, 2, "a peça na faixa nova");
    let mut ph = ph2d_core::Playhead::default();
    let antes = st.doc.stack().len();
    for i in feitos {
        apply_intent(&mut st, &mut ph, i);
    }
    assert_eq!(st.doc.stack().len(), antes + 1, "a faixa nova");
    assert_eq!(st.doc.stack()[2].strips.len(), 1, "a peça nela");
    apply_intent(&mut st, &mut ph, TimelineIntent::Undo);
    assert_eq!(
        st.doc.stack().len(),
        antes,
        "um Ctrl+Z não desfez faixa e peça"
    );
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ O Esc a meio do arrasto cancela: largar depois não cria nada. Largar fora das faixas também não.
#[test]
fn esc_ou_fora_das_faixas_nao_cria_nada() {
    let mut st = estado();
    let mut m = mesa(&mut st);
    let alvo = sobre_a_faixa(&m, 0, 300.0);
    arrasta(&mut m, ids::TIMELINE_CONT_OPT[0], alvo, false);
    let _ = m.esc();
    m.assenta_em(JANELA);
    let _ = m.ponteiro(PointerKind::Up, alvo.0, alvo.1);
    m.assenta_em(JANELA);
    assert!(
        add_strips(&drain_intents()).is_empty(),
        "o Esc não cancelou"
    );
    let regua = rect(&m, ids::TIMELINE_RULER);
    arrasta(
        &mut m,
        ids::TIMELINE_CONT_OPT[0],
        (regua.x + 300.0, regua.y - 40.0),
        true,
    );
    assert!(
        add_strips(&drain_intents()).is_empty(),
        "largar sobre a barra criou uma peça"
    );
    ferramenta::force_cards_on_this_thread(None);
}
