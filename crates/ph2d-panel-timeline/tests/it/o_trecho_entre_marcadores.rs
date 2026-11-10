//! ⭐ **Tocar só um trecho** (doc 17 item 2, maquete B6): o clique no vão PINTADO entre dois marcadores
//! arma esse trecho no loop vivo e o Play dá a volta nele; a pílula «takeoff → top» devolve o loop do
//! clipe (com e sem loop autorado); nada se grava nem se desfaz; trocar de clipe larga.

use ph2d_anim::RationalTime;
use ph2d_core::Playhead;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_panel_timeline::tab::Tab;
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{TimelineState, TimelineViewSnapshot, apply_intent, arm_view_loop};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

struct Motor {
    st: TimelineState,
    ed: Playhead,
    clip: Playhead,
}

impl Motor {
    fn novo(laco: Option<(f64, f64)>) -> Self {
        let mut st = TimelineState::new();
        for (t, n) in [(0.0, "takeoff"), (0.5, "top"), (1.0, "land")] {
            st.doc.add_marker(RationalTime::from_seconds(t), n);
        }
        st.doc.add_clip("walk".into());
        st.doc.set_active(0);
        st.doc.set_active_loop_for(true, laco);
        st.keys_mode = true;
        let mut m = Self {
            st,
            ed: Playhead::new(1.0 / 24.0),
            clip: Playhead::new(1.0 / 24.0),
        };
        m.quadro();
        m
    }
    /// Um quadro da shell: drena o painel, arma, publica o instantâneo.
    fn quadro(&mut self) {
        for i in drain_intents() {
            apply_intent(&mut self.st, &mut self.ed, i);
        }
        arm_view_loop(&mut self.st, &mut self.ed, &mut self.clip);
        let mut snap = TimelineViewSnapshot::default();
        snap.rebuild(&mut self.st, &self.ed, true);
        set_current_timeline(Some(snap));
    }
}

fn mesa() -> Mesa<TimelinePanel> {
    let mut m = Mesa::nova(TimelinePanelState {
        tab: Tab::Keys,
        ..TimelinePanelState::default()
    });
    m.assenta_em(JANELA);
    m
}

/// Clica o vão PINTADO num ponto onde ele é o alvo (a bandeirola e a alça da duração ganham-lhe).
fn clica_no_vao(m: &mut Mesa<TimelinePanel>, a: usize) {
    let id = ids::timeline_marker_gap_hit_id(a);
    let r = m.rect(id).expect("o vão pintado");
    let cy = r.y + r.h * 0.5;
    let x = (0..r.w as i32)
        .map(|d| r.x + d as f32)
        .find(|&x| m.host.hit_at(x, cy) == Some(id))
        .expect("algum ponto do vão é dele");
    let ev = m.host.click_at(x, cy);
    let _ = m.aplica(ev);
}

#[test]
fn o_clique_entre_dois_marcadores_toca_o_trecho_e_o_chip_devolve_o_clipe() {
    ferramenta::force_cards_on_this_thread(Some(true));
    for laco in [None, Some((0.25, 0.75))] {
        let mut e = Motor::novo(laco);
        let antes = e.st.doc.to_bytes().expect("doc");
        let mut m = mesa();
        assert!(
            m.rect(ids::TIMELINE_STRETCH_CLEAR).is_none(),
            "sem trecho, sem chip"
        );
        clica_no_vao(&mut m, 0);
        m.assenta_em(JANELA); // o gesto drena-se na pintura
        e.quadro();
        assert_eq!(
            e.ed.loop_range(),
            Some((0.0, 0.5)),
            "o trecho takeoff → top"
        );
        for _ in 0..40 {
            e.ed.advance();
            assert!(e.ed.time() < 0.5, "o Play dá a volta no trecho");
        }
        m.assenta_em(JANELA);
        let _ = m.clica(ids::TIMELINE_STRETCH_CLEAR);
        e.quadro();
        assert_eq!(e.ed.loop_range(), laco, "o chip devolve o loop do clipe");
        m.assenta_em(JANELA);
        assert!(m.rect(ids::TIMELINE_STRETCH_CLEAR).is_none(), "o chip some");
        assert_eq!(e.st.doc.to_bytes().expect("doc"), antes, "nada gravado");
        assert!(!e.st.history.can_undo(), "nenhum passo de undo");
    }
    // Trocar de clipe larga o trecho, no motor e no painel.
    let mut e = Motor::novo(None);
    let mut m = mesa();
    clica_no_vao(&mut m, 1);
    m.assenta_em(JANELA); // o gesto drena-se na pintura
    e.quadro();
    m.assenta_em(JANELA);
    assert_eq!(e.ed.loop_range(), Some((0.5, 1.0)));
    assert!(
        m.rect(ids::TIMELINE_STRETCH_CLEAR).is_some(),
        "o chip «top → land»"
    );
    e.st.doc.set_active(1);
    e.quadro();
    m.assenta_em(JANELA);
    assert_eq!(e.ed.loop_range(), None);
    assert!(
        m.rect(ids::TIMELINE_STRETCH_CLEAR).is_none(),
        "outro clipe larga o chip"
    );
    let _ = drain_intents();
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐⭐ **A faixa «Markers» é onde o vão e os marcadores moram** (maquete B6; dono 10/11: o marcador é a
/// alça do fade no topo de uma linha): o vão pintado e a alça de cada marcador estão na MESMA linha,
/// abaixo da régua (nenhum alvo de marcador na régua); o alvo da alça é a faixa inteira (20 × 22, não
/// encolhe) e o vão começa onde ele acaba; os gestos de sempre continuam vivos na alça — arrastar move
/// o marcador, o botão direito abre o menu dele (Rename / Set Signal / Delete).
/// *Mutação: registar a alça na régua ⇒ «abaixo da régua» reprova.*
#[test]
fn a_faixa_markers_tem_o_vao_as_alcas_e_os_gestos() {
    use ph2d_editor_core::interaction::ContextMenuKind;
    ferramenta::force_cards_on_this_thread(Some(true));
    let mut e = Motor::novo(None);
    let mut m = mesa();
    let regua = m.rect(ids::TIMELINE_RULER).expect("a régua");
    let vao = m.rect(ids::timeline_marker_gap_hit_id(0)).expect("o vão");
    assert!(
        vao.y >= regua.y + regua.h,
        "o vão na faixa, abaixo da régua: {vao:?} {regua:?}"
    );
    for i in 0..3 {
        let c = m.rect(ids::timeline_marker_hit_id(i)).expect("a alça");
        assert!(
            (c.y - vao.y).abs() < 0.5,
            "a alça {i} na linha do vão: {c:?}"
        );
        assert!(
            c.w >= 20.0 && c.h >= 22.0,
            "o alvo do marcador não encolheu: {c:?}"
        );
    }
    let c0 = m.rect(ids::timeline_marker_hit_id(0)).expect("takeoff");
    assert!(
        (vao.x - (c0.x + c0.w)).abs() < 0.5,
        "o vão começa onde a alça acaba: {vao:?} {c0:?}"
    );
    // Arrastar a alça de «top» move «top».
    let c = m.rect(ids::timeline_marker_hit_id(1)).expect("top");
    let (cx, cy) = (c.x + c.w * 0.5, c.y + c.h * 0.5);
    let ev = m.host.drag_at(cx, cy, cx + 40.0, cy);
    let _ = m.aplica(ev);
    m.assenta_em(JANELA);
    let movido = drain_intents().iter().any(|i| {
        matches!(
            i,
            ph2d_timeline::TimelineIntent::MoveMarker { index: 1, .. }
        )
    });
    assert!(movido, "arrastar a alça move o marcador");
    e.quadro();
    // O botão direito na alça de «land» abre o menu desse marcador.
    m.assenta_em(JANELA);
    let c = m.rect(ids::timeline_marker_hit_id(2)).expect("land");
    let ev = m.host.right_click_at(c.x + c.w * 0.5, c.y + c.h * 0.5);
    let _ = m.aplica(ev);
    let menu = m
        .host
        .store()
        .context_menu()
        .or_else(|| m.host.store().last_context_menu());
    assert!(
        matches!(
            menu.map(|r| r.kind),
            Some(ContextMenuKind::TimelineMarker { index: 2 })
        ),
        "o menu do marcador: {menu:?}"
    );
    let _ = drain_intents();
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ **O ✕ da pílula devolve o loop do clipe** (maquete B6: «▷ takeoff → top ✕»): clicar no PONTO do ✕
/// — a ponta direita da pílula pintada — larga o trecho. *Mutação: a pílula registar só o texto ⇒ o ✕
/// não é dela.*
#[test]
fn o_x_da_pilula_devolve_o_clipe() {
    ferramenta::force_cards_on_this_thread(Some(true));
    let mut e = Motor::novo(None);
    let mut m = mesa();
    clica_no_vao(&mut m, 0);
    m.assenta_em(JANELA);
    e.quadro();
    m.assenta_em(JANELA);
    assert_eq!(e.ed.loop_range(), Some((0.0, 0.5)));
    let p = m.rect(ids::TIMELINE_STRETCH_CLEAR).expect("a pílula");
    let (x, y) = (p.x + p.w - p.h * 0.5, p.y + p.h * 0.5);
    assert_eq!(
        m.host.hit_at(x, y),
        Some(ids::TIMELINE_STRETCH_CLEAR),
        "o ✕ é da pílula"
    );
    let ev = m.host.click_at(x, y);
    let _ = m.aplica(ev);
    e.quadro();
    assert_eq!(e.ed.loop_range(), None, "o ✕ devolve o clipe");
    let _ = drain_intents();
    ferramenta::force_cards_on_this_thread(None);
}
