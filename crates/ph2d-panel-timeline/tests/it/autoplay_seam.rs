//! ⭐⭐ O interruptor **«Autoplay»** (onda 4, item 1 — o do Godot, dono 10/10): com o clique REAL, liga
//! e desliga o Autoplay do clipe em edição — um passo do undo da timeline cada; por omissão, nenhum;
//! a Cena também o tem.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{TimelineIntent as I, TimelineState, TimelineViewSnapshot, apply_intent};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);
const HEROI: u64 = 7;

fn playhead() -> ph2d_core::Playhead {
    ph2d_core::Playhead::new(1.0 / 60.0)
}

/// O Herói com «andar» e «pular», em vista, com `activo` em edição.
fn heroi(activo: &str) -> TimelineState {
    let mut st = TimelineState::new();
    let andar = st.doc.add_clip_for("andar".into(), HEROI).unwrap();
    let pular = st.doc.add_clip_for("pular".into(), HEROI).unwrap();
    st.view.owner = HEROI;
    st.doc
        .set_active(if activo == "andar" { andar } else { pular });
    st
}

/// Pinta; devolve o rect do interruptor, se ele se registou.
fn pinta(st: &mut TimelineState) -> (MockPanelHost, TimelinePanelState, Option<Rect>) {
    let _ = ph2d_panel_timeline::drain_intents();
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(st, &playhead(), false);
    set_current_timeline(Some(snap));
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut panel = TimelinePanelState::default();
    let regs = host.paint::<TimelinePanel>(&mut panel, VIEWPORT);
    let r = regs
        .iter()
        .find(|(w, _)| *w == ids::TIMELINE_CLIP_AUTOPLAY)
        .map(|(_, r)| *r);
    (host, panel, r)
}

/// Clica o interruptor e aplica o que o painel pediu.
fn clica(st: &mut TimelineState) {
    let (mut host, mut panel, r) = pinta(st);
    let r = r.expect("o interruptor está pintado e registado");
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<TimelinePanel>(&mut panel, ev);
    }
    let mut ph = playhead();
    for p in ph2d_panel_timeline::drain_intents() {
        apply_intent(st, &mut ph, p);
    }
}

/// ⭐⭐ Por omissão desligado; um clique liga (um passo do undo), outro desliga, o Ctrl+Z desfaz. Ligar
/// o «pular» desliga o «andar». (Mutações: o `apply_event` sem o `push_intent`; o `on` sem negar ⇒ RED.)
#[test]
fn o_autoplay_liga_e_desliga_com_o_clique() {
    let mut st = heroi("pular");
    let andar = st.doc.clip_named(HEROI, "andar").unwrap();
    let pular = st.doc.clip_named(HEROI, "pular").unwrap();
    assert_eq!(st.doc.entry_clip_of(HEROI), None, "por omissão, nenhum");
    clica(&mut st);
    assert_eq!(st.doc.entry_clip_of(HEROI), Some(pular), "ligado");
    assert!(st.history.can_undo(), "um passo do undo");
    clica(&mut st);
    assert_eq!(st.doc.entry_clip_of(HEROI), None, "outro clique desliga");
    let mut ph = playhead();
    apply_intent(&mut st, &mut ph, I::Undo);
    assert_eq!(
        st.doc.entry_clip_of(HEROI),
        Some(pular),
        "o Ctrl+Z volta a ligar"
    );
    st.doc.set_active(andar);
    clica(&mut st);
    assert_eq!(
        st.doc.entry_clip_of(HEROI),
        Some(andar),
        "ligar um desliga o outro"
    );
}

/// A Cena também tem o interruptor: o clipe dela toca quando a cena corre com o Arrange vazio.
#[test]
fn a_cena_tem_autoplay() {
    let mut st = TimelineState::new();
    clica(&mut st);
    assert_eq!(st.doc.entry_clip_of(ph2d_timeline::SCENE), Some(0));
}
