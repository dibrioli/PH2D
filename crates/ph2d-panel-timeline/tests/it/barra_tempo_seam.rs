//! ⭐ **A barra de rolagem do tempo com alças de zoom CLICA, de ponta a ponta** (dono 16/11).
//!
//! O painel é pintado de verdade nos cartões (a UI do smoke do dono), o ponteiro vai pelo despacho
//! REAL e o que conta é a vista que a fita mostra no quadro seguinte (`view_start_s`, `px_per_s`) —
//! a mesma que as chaves e o gráfico lêem.

use ph2d_editor_core::ids::NodeId;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::ids::TIMELINE_TIME_BAR;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_timeline::{
    AnimTarget, Interp, KeyId, KeyView, PropKind, TimelineViewSnapshot, TrackView,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);
const PONTA_E: usize = 0;
const PONTA_D: usize = 1;
const MEIO: usize = 2;
const CALHA: usize = 3;

/// Um clipe de 4 s com chaves em 0 e 4 s.
fn snap() -> TimelineViewSnapshot {
    let kv = |id, t, v| KeyView {
        retime: None,
        id: KeyId::new(id),
        t_seconds: t,
        value: v,
        interp: Interp::Linear,
        selected: false,
        roving: false,
    };
    TimelineViewSnapshot {
        fps: 60.0,
        view_length_seconds: 4.0,
        tracks: vec![TrackView {
            target: AnimTarget::new(7),
            prop: PropKind::TranslationX,
            entity: 1,
            missing: false,
            keys: vec![kv(1, 0.0, 0.0), kv(2, 4.0, 10.0)],
            buffer_ghost: None,
            pre: ph2d_timeline::Extrap::Hold,
            post: ph2d_timeline::Extrap::Hold,
            expr: None,
        }],
        ..TimelineViewSnapshot::default()
    }
}

fn pinta(host: &mut MockPanelHost, st: &mut TimelinePanelState) -> Vec<(NodeId, Rect)> {
    set_current_timeline(Some(snap()));
    host.paint::<TimelinePanel>(st, VIEWPORT)
}

fn rect(regs: &[(NodeId, Rect)], parte: usize) -> Rect {
    regs.iter()
        .rev()
        .find(|(w, _)| *w == TIMELINE_TIME_BAR[parte])
        .map(|(_, r)| *r)
        .expect("a parte da barra não foi registada: morta debaixo do rato")
}

/// O painel aberto nos cartões, já com o zoom perto (1 s visível de 4) a começar em 1 s.
fn aberto_com_zoom() -> (MockPanelHost, TimelinePanelState, Vec<(NodeId, Rect)>) {
    ferramenta::force_cards_on_this_thread(Some(true));
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut st = TimelinePanelState::default();
    let _ = pinta(&mut host, &mut st);
    st.px_per_s = st.view_span_s * st.px_per_s; // 1 s cabe na coluna inteira
    st.view_start_s = 1.0;
    let _ = pinta(&mut host, &mut st);
    let regs = pinta(&mut host, &mut st);
    (host, st, regs)
}

fn ponteiro(host: &mut MockPanelHost, kind: PointerKind, x: f32, y: f32, ns: u128) {
    let _ = host.dispatch_pointer_event(PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: ns,
    });
}

/// A barra mora no pé da fita, sob a coluna do tempo, e NADA de faixa ou régua a cobre.
#[test]
fn a_barra_mora_no_pe_da_fita_e_nada_a_cobre() {
    let (host, _st, regs) = aberto_com_zoom();
    let calha = rect(&regs, CALHA);
    for parte in [PONTA_E, PONTA_D, MEIO, CALHA] {
        let r = rect(&regs, parte);
        // A calha é a barra inteira por baixo do polegar: o clique nela é o que cai fora dele.
        let x = if parte == CALHA {
            r.x + r.w - 3.0
        } else {
            r.x + r.w * 0.5
        };
        let y = r.y + r.h * 0.5;
        assert_eq!(
            host.hit_at(x, y),
            Some(TIMELINE_TIME_BAR[parte]),
            "a parte {parte} perde o clique para outra coisa"
        );
    }
    // Nenhum outro alvo da fita desce até à barra.
    let fura = regs.iter().any(|(id, r)| {
        !TIMELINE_TIME_BAR.contains(id)
            && r.y + r.h > calha.y + 0.5
            && r.y < calha.y + calha.h
            && r.x < calha.x + calha.w
            && r.x + r.w > calha.x
            && r.w < calha.w * 0.9
    });
    assert!(!fura, "um alvo da fita invade a barra do tempo");
    set_current_timeline(None);
}

/// Arrastar a ponta DIREITA para a esquerda dá zoom e a esquerda fica; a ESQUERDA para a direita
/// também, e aí a direita fica. *Mutação: a ponta direita a mover o início ⇒ reprova.*
#[test]
fn as_pontas_dao_zoom_e_a_outra_fica() {
    let (mut host, mut st, regs) = aberto_com_zoom();
    let (inicio, px) = (st.view_start_s, st.px_per_s);
    let d = rect(&regs, PONTA_D);
    let _ = host.drag_at(
        d.x + d.w * 0.5,
        d.y + d.h * 0.5,
        d.x - 60.0,
        d.y + d.h * 0.5,
    );
    let _ = pinta(&mut host, &mut st);
    assert!(
        st.px_per_s > px * 1.05,
        "a ponta direita não deu zoom: {px} → {}",
        st.px_per_s
    );
    assert!(
        (st.view_start_s - inicio).abs() < 1e-9,
        "o início mexeu: {inicio} → {}",
        st.view_start_s
    );

    let regs = pinta(&mut host, &mut st);
    let fim = st.view_start_s + st.view_span_s;
    let e = rect(&regs, PONTA_E);
    let _ = host.drag_at(
        e.x + e.w * 0.5,
        e.y + e.h * 0.5,
        e.x - 80.0,
        e.y + e.h * 0.5,
    );
    let _ = pinta(&mut host, &mut st);
    assert!(
        st.view_start_s < inicio,
        "a ponta esquerda não abriu para trás"
    );
    let fim2 = st.view_start_s + st.view_span_s;
    assert!((fim2 - fim).abs() < 1e-6, "o fim mexeu: {fim} → {fim2}");
    set_current_timeline(None);
}

/// O meio rola sem mudar o zoom. *Mutação: o meio a dar zoom ⇒ reprova.*
#[test]
fn o_meio_rola_sem_mudar_o_zoom() {
    let (mut host, mut st, regs) = aberto_com_zoom();
    let (inicio, px) = (st.view_start_s, st.px_per_s);
    let m = rect(&regs, MEIO);
    let (x, y) = (m.x + m.w * 0.5, m.y + m.h * 0.5);
    let _ = host.drag_at(x, y, x + 50.0, y);
    let _ = pinta(&mut host, &mut st);
    assert!(
        st.view_start_s > inicio + 0.05,
        "não rolou: {inicio} → {}",
        st.view_start_s
    );
    assert!((st.px_per_s - px).abs() < 1e-9, "o zoom mudou ao rolar");
    set_current_timeline(None);
}

/// Um clique na calha, longe do polegar, centra a janela ali. *Mutação: a calha inerte ⇒ reprova.*
#[test]
fn um_clique_na_calha_salta_para_ali() {
    let (mut host, mut st, regs) = aberto_com_zoom();
    let c = rect(&regs, CALHA);
    let m = rect(&regs, MEIO);
    let x = c.x + c.w - 4.0;
    assert!(x > m.x + m.w + 10.0, "o polegar já estava no fim");
    let antes = st.view_start_s;
    let _ = host.click_at(x, c.y + c.h * 0.5);
    let regs = pinta(&mut host, &mut st);
    assert!(
        st.view_start_s > antes + 1.0,
        "não saltou: {antes} → {}",
        st.view_start_s
    );
    let m2 = rect(&regs, MEIO);
    assert!(m2.x + m2.w > x - 1.0, "o polegar não chegou ao clique");
    set_current_timeline(None);
}

/// Duplo clique = o clipe inteiro à vista; outro duplo clique volta à vista de antes.
/// *Mutação: o segundo duplo clique a enquadrar outra vez ⇒ reprova.*
#[test]
fn o_duplo_clique_cabe_tudo_e_o_segundo_volta() {
    let (mut host, mut st, regs) = aberto_com_zoom();
    let antes = (st.view_start_s, st.px_per_s);
    let m = rect(&regs, MEIO);
    let (x, y) = (m.x + m.w * 0.5, m.y + m.h * 0.5);
    let duplo = |host: &mut MockPanelHost, base: u128| {
        for (i, k) in [
            PointerKind::Down,
            PointerKind::Up,
            PointerKind::Down,
            PointerKind::Up,
        ]
        .into_iter()
        .enumerate()
        {
            ponteiro(host, k, x, y, base + i as u128 * 50_000_000);
        }
    };
    duplo(&mut host, 100_000_000_000);
    let _ = pinta(&mut host, &mut st);
    assert!(
        st.view_start_s < 0.5,
        "o começo não está à vista: {}",
        st.view_start_s
    );
    assert!(
        st.view_start_s + st.view_span_s >= 4.0,
        "o fim do clipe não cabe"
    );
    let _ = pinta(&mut host, &mut st);
    let regs = pinta(&mut host, &mut st);
    let m = rect(&regs, MEIO);
    let (x2, y2) = (m.x + m.w * 0.5, m.y + m.h * 0.5);
    for (i, k) in [
        PointerKind::Down,
        PointerKind::Up,
        PointerKind::Down,
        PointerKind::Up,
    ]
    .into_iter()
    .enumerate()
    {
        ponteiro(
            &mut host,
            k,
            x2,
            y2,
            200_000_000_000 + i as u128 * 50_000_000,
        );
    }
    let _ = pinta(&mut host, &mut st);
    assert!(
        (st.view_start_s - antes.0).abs() < 1e-9 && (st.px_per_s - antes.1).abs() < 1e-9,
        "não voltou: {antes:?} → {:?}",
        (st.view_start_s, st.px_per_s)
    );
    set_current_timeline(None);
}

/// A roda simples sobre a barra rola o TEMPO, não as linhas. *Mutação: `roda` sem efeito ⇒ reprova.*
#[test]
fn a_roda_sobre_a_barra_rola_o_tempo() {
    let (mut host, mut st, regs) = aberto_com_zoom();
    let c = rect(&regs, CALHA);
    let (x, y) = (c.x + 3.0, c.y + c.h * 0.5);
    let _ = host.dispatch_pointer_event(PointerEvent {
        x,
        y,
        pressure: 0.0,
        kind: PointerKind::Move,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 300_000_000_000,
    });
    let antes = st.view_start_s;
    let _ = host.wheel_at(x, y, -48.0);
    let _ = pinta(&mut host, &mut st);
    assert!(
        (st.view_start_s - antes).abs() > 0.01,
        "a roda sobre a barra não rolou o tempo"
    );
    set_current_timeline(None);
}
