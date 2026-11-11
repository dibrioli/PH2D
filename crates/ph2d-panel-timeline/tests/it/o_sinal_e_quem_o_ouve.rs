//! ⭐ **Quem ouve um sinal** (doc 17 item 8, maquete B5): o clique parado na alça PINTADA de um marcador
//! com sinal vai até ele e abre o cartão; o painel pede o nome à shell e pinta a lista que ela devolve;
//! sem ouvinte, «Nobody listens»; marcador sem sinal só vai até ele; o Esc fecha.

use ph2d_anim::RationalTime;
use ph2d_core::Playhead;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_panel_timeline::tab::Tab;
use ph2d_panel_timeline::{TimelinePanel, ids, set_signal_listeners, signal_asked};
use ph2d_signal_ends::{Listener, Quem, SignalListeners};
use ph2d_timeline::{TimelineIntent, TimelineState, TimelineViewSnapshot, apply_intent};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// Três marcadores; só o último («land», em 1 s) tem sinal.
fn publica() {
    let mut st = TimelineState::new();
    for (t, n) in [(0.0, "takeoff"), (0.5, "top"), (1.0, "land")] {
        st.doc.add_marker(RationalTime::from_seconds(t), n);
    }
    st.doc.set_marker_signal(2, Some("land".into()));
    st.doc.add_clip("walk".into());
    st.doc.set_active(0);
    st.keys_mode = true;
    let mut ed = Playhead::new(1.0 / 24.0);
    for i in drain_intents() {
        apply_intent(&mut st, &mut ed, i);
    }
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ed, true);
    set_current_timeline(Some(snap));
}

fn mesa() -> Mesa<TimelinePanel> {
    ferramenta::force_cards_on_this_thread(Some(true));
    publica();
    let mut m = Mesa::nova(TimelinePanelState {
        tab: Tab::Keys,
        ..TimelinePanelState::default()
    });
    m.assenta_em(JANELA);
    m
}

/// Clica a alça PINTADA do marcador `i`, num ponto onde ela é o alvo.
fn clica_na_alca(m: &mut Mesa<TimelinePanel>, i: usize) {
    let id = ids::timeline_marker_hit_id(i);
    let r = m.rect(id).expect("a alça pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(m.host.hit_at(cx, cy), Some(id), "o meio da alça é dela");
    let ev = m.host.click_at(cx, cy);
    let _ = m.aplica(ev);
    m.assenta_em(JANELA); // o gesto drena-se na pintura
}

fn pintados(m: &mut Mesa<TimelinePanel>) -> Vec<String> {
    let (h, st) = (&mut m.host, &mut m.st);
    h.medindo_a_pintura::<TimelinePanel>(st, JANELA)
        .into_iter()
        .map(|t| t.texto)
        .collect()
}

/// *Mutação: tirar o `abre` do clique ⇒ `signal_asked` fica `None`; tirar o Esc do cartão ⇒ fica aberto.*
#[test]
fn o_clique_no_marcador_com_sinal_abre_quem_o_ouve_e_o_esc_fecha() {
    let mut m = mesa();
    let _ = drain_intents();
    clica_na_alca(&mut m, 2);
    assert!(
        drain_intents().contains(&TimelineIntent::Scrub(1.0)),
        "o clique continua a ir até o marcador"
    );
    assert_eq!(
        signal_asked().as_deref(),
        Some("land"),
        "o painel pede «land»"
    );
    set_signal_listeners(
        "land",
        SignalListeners {
            list: vec![
                Listener {
                    object: Some("Hero".into()),
                    what: Quem::Component("SignalActions"),
                },
                Listener {
                    object: None,
                    what: Quem::Doc("MotionGraph"),
                },
            ],
            script_may: true,
        },
    );
    let t = pintados(&mut m);
    for frase in [
        "Signal at frame 24 \u{b7} land",
        "Heard by",
        "Hero \u{b7} Signal Actions",
        "Motion input",
        "A script may listen",
    ] {
        assert!(t.iter().any(|x| x == frase), "pinta «{frase}»: {t:?}");
    }
    assert!(!t.iter().any(|x| x == "Nobody listens"));

    let _ = m.esc();
    m.assenta_em(JANELA);
    assert_eq!(m.st.sinal_aberto, None, "o Esc fecha o cartão");
    assert_eq!(signal_asked(), None, "e o pedido à shell");
    let _ = drain_intents();
    ferramenta::force_cards_on_this_thread(None);
}

/// *Mutação: abrir sem olhar o sinal ⇒ o marcador sem sinal abre um cartão.*
#[test]
fn o_marcador_sem_sinal_so_vai_ate_ele() {
    let mut m = mesa();
    let _ = drain_intents();
    clica_na_alca(&mut m, 0);
    assert!(drain_intents().contains(&TimelineIntent::Scrub(0.0)));
    assert_eq!(m.st.sinal_aberto, None);
    assert_eq!(signal_asked(), None);
    ferramenta::force_cards_on_this_thread(None);
}

/// *Mutação: tirar a linha do aviso ⇒ a lista vazia não diz nada.*
#[test]
fn ninguem_a_ouvir_pinta_nobody_listens() {
    let mut m = mesa();
    clica_na_alca(&mut m, 2);
    set_signal_listeners("land", SignalListeners::default());
    let t = pintados(&mut m);
    assert!(t.iter().any(|x| x == "Nobody listens"), "{t:?}");
    assert!(!t.iter().any(|x| x == "A script may listen"));
    let _ = drain_intents();
    ferramenta::force_cards_on_this_thread(None);
}
