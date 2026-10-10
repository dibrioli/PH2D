//! ⭐⭐ O interruptor **«Autoplay»** (onda 4, item 1): com o clique REAL, o clipe em edição passa a ser
//! a ENTRADA da animação do objecto — um passo do undo da timeline. No clipe que já é a entrada ele
//! pinta-se ligado e não regista clique (há sempre uma); na Cena não existe.

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

/// ⭐⭐ Clicar o «Autoplay» do «pular» fá-lo a entrada (um passo de undo); e lá ele já não regista.
/// (Mutação: o `apply_event` sem o `push_intent` ⇒ RED.)
#[test]
fn ligar_o_autoplay_faz_do_clipe_a_entrada() {
    let mut st = heroi("pular");
    let andar = st.doc.clip_named(HEROI, "andar").unwrap();
    let pular = st.doc.clip_named(HEROI, "pular").unwrap();
    assert_eq!(
        st.doc.entry_clip_of(HEROI),
        Some(andar),
        "por omissão, o 1.º"
    );
    let (mut host, mut panel, r) = pinta(&mut st);
    let r = r.expect("o interruptor do clipe que NÃO é a entrada está vivo");
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<TimelinePanel>(&mut panel, ev);
    }
    let mut ph = playhead();
    for p in ph2d_panel_timeline::drain_intents() {
        apply_intent(&mut st, &mut ph, p);
    }
    assert_eq!(st.doc.entry_clip_of(HEROI), Some(pular));
    assert!(st.history.can_undo(), "um passo do undo");
    apply_intent(&mut st, &mut ph, I::Undo);
    assert_eq!(
        st.doc.entry_clip_of(HEROI),
        Some(andar),
        "o Ctrl+Z devolve a entrada"
    );

    st.doc.set_entry_clip(pular);
    assert!(
        pinta(&mut st).2.is_none(),
        "na entrada, morto: desligar não teria resposta"
    );
}

/// Na Cena (que toca o Arrange) não há interruptor nenhum — nem pintado nem registado.
#[test]
fn a_cena_nao_tem_autoplay() {
    let mut st = TimelineState::new();
    assert!(pinta(&mut st).2.is_none());
}

/// ⭐⭐ O **Play All** (pedido do dono, 09/10): existe em qualquer aba, e o clique real pede-o pelo
/// canal de intents — a shell drena-o (`take_play_all`) e põe a CENA inteira a tocar. Na Cena
/// também (é a cena inteira, não a animação em vista).
#[test]
fn o_play_all_pede_a_cena_inteira() {
    for mut st in [heroi("andar"), TimelineState::new()] {
        let (mut host, mut panel, _) = pinta(&mut st);
        let regs = host.paint::<TimelinePanel>(&mut panel, VIEWPORT);
        let r = regs
            .iter()
            .find(|(w, _)| *w == ids::TIMELINE_PLAY_ALL)
            .map(|(_, r)| *r)
            .expect("o Play All está pintado e registado");
        let _ = ph2d_panel_timeline::drain_intents();
        for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
            host.apply_panel_event::<TimelinePanel>(&mut panel, ev);
        }
        assert_eq!(ph2d_panel_timeline::drain_intents(), [I::PlayAll]);
    }
}
