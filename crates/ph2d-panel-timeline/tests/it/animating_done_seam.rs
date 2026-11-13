//! ⭐ **«Animating · Done»** (doc 17 item 9, maquete B7): na aba Keys dos cartões a barra mostra a
//! pílula «Animating jump» com o Done PINTADO; clicá-lo pára a sessão (a shell devolve a cena), e um
//! gesto na Timeline volta a animar. Gesto real sobre rects pintados.

use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_panel_timeline::{TimelinePanel, animating, ids, parked, sessao, take_ended};
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: ph2d_editor_core::zones::Rect =
    ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 700.0);

fn mesa(cartoes: bool) -> Mesa<TimelinePanel> {
    ferramenta::force_cards_on_this_thread(Some(cartoes));
    set_current_timeline(Some(TimelineViewSnapshot {
        fps: 24.0,
        view_length_seconds: 2.0,
        view_length_explicit: true,
        clips: vec!["jump".into()],
        clip_index: vec![0],
        active_clip: 0,
        ..TimelineViewSnapshot::default()
    }));
    let mut m = Mesa::nova(TimelinePanelState::default());
    m.assenta_em(JANELA);
    let _ = drain_intents();
    let _ = m.host.drained_actions();
    let _ = sessao(true, false);
    let _ = take_ended();
    m
}

/// A pílula e o Done estão na barra da aba Keys; clicar o Done pinta a barra sem ela, `parked`, e a
/// sessão DESCE uma vez (a shell devolve a cena). Um gesto (a agulha) volta a animar.
/// *Mutação: o braço `Click` do Done que não chama `done()` ⇒ vermelho.*
#[test]
fn done_parks_the_session_and_a_gesture_resumes_it() {
    let mut m = mesa(true);
    assert!(
        m.rect(ids::TIMELINE_ANIMATING_DONE).is_some(),
        "o Done está pintado na aba Keys"
    );
    assert_eq!(sessao(true, false), (true, false), "a animar");
    m.clica(ids::TIMELINE_ANIMATING_DONE);
    m.assenta_em(JANELA);
    assert!(parked() && !animating(), "o Done pára a sessão");
    assert!(
        m.rect(ids::TIMELINE_ANIMATING_DONE).is_none(),
        "parada, a pílula sai"
    );
    assert_eq!(
        sessao(true, false),
        (false, true),
        "a aba Keys não põe o clipe na cena"
    );
    assert!(take_ended(), "a descida chega à shell uma vez");
    assert!(!take_ended());
    assert_eq!(sessao(true, true), (true, false), "um gesto volta a animar");
    m.assenta_em(JANELA);
    assert!(m.rect(ids::TIMELINE_ANIMATING_DONE).is_some());
    ferramenta::force_cards_on_this_thread(None);
}

/// Sair da aba Keys acaba a sessão (a cena volta) sem Done.
/// *Mutação: `publica` sem o `keys` ⇒ vermelho.*
#[test]
fn leaving_the_keys_tab_ends_the_session() {
    let mut m = mesa(true);
    assert!(sessao(true, false).0);
    m.clica(ids::TIMELINE_TAB_ARRANGE);
    m.assenta_em(JANELA);
    assert!(m.rect(ids::TIMELINE_ANIMATING_DONE).is_none());
    assert!(!sessao(true, false).0);
    assert!(take_ended(), "trocar de aba devolve a cena");
    ferramenta::force_cards_on_this_thread(None);
}

/// No clássico (`PH2D_CARDS=0`) não há sessão: a Timeline responde como hoje.
#[test]
fn the_classic_panel_has_no_session() {
    let m = mesa(false);
    assert!(m.rect(ids::TIMELINE_ANIMATING_DONE).is_none());
    assert_eq!(sessao(true, false), (false, false));
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ Dono 13/11: o Done é um botão (✓ verde) colado ao clipe — no fim do chip escolhido, depois do
/// «⋯»; com os clipes em lista, à direita do «⋯» da lista e antes do «+». A pílula «Animating» fica só
/// com o aviso. *Mutação: o Done de volta à pílula ⇒ fica à esquerda do «⋯» ⇒ vermelho.*
#[test]
fn o_done_e_um_botao_colado_ao_clipe_depois_do_menu() {
    let mut m = mesa(true);
    let r = |m: &Mesa<TimelinePanel>, id| m.rect(id).unwrap_or_else(|| panic!("{id:?}"));
    for lista in [false, true] {
        m.st.clips_as_list = lista;
        m.assenta_em(JANELA);
        let (done, menu, mais) = (
            r(&m, ids::TIMELINE_ANIMATING_DONE),
            r(&m, ids::TIMELINE_CLIP_MENU),
            r(&m, ids::TIMELINE_ADD_CLIP),
        );
        assert!(
            done.x >= menu.x + menu.w - 0.5,
            "lista={lista}: o Done antes do ⋯"
        );
        assert!(
            done.x + done.w <= mais.x + 0.5,
            "lista={lista}: o Done depois do +"
        );
        assert!(
            ((done.y + done.h * 0.5) - (menu.y + menu.h * 0.5)).abs() < 0.5,
            "lista={lista}: fora da fileira do clipe"
        );
        assert!(
            (done.w - done.h).abs() < 0.5,
            "um botão quadrado/redondo, sem palavra"
        );
    }
    ferramenta::force_cards_on_this_thread(None);
}
