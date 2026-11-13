//! ⭐ Dono 13/11: os controlos de tocar FLUTUAM acima do painel, no canto superior esquerdo (⏮ ‹ ■ ▶ › ⏭ ●);
//! a caixa Auto Key sai da barra e vira o ● (o mesmo id, o mesmo clique); o ■ pára e volta ao início;
//! a duração do clipe entra entre as abas e o Time (e sai da engrenagem).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1040.0,
};

fn mesa() -> Mesa<TimelinePanel> {
    ferramenta::force_cards_on_this_thread(Some(true));
    set_current_timeline(Some(TimelineViewSnapshot {
        fps: 60.0,
        view_length_seconds: 4.0,
        view_length_explicit: true,
        clips: vec!["Main".to_owned()],
        clip_index: vec![0],
        ..TimelineViewSnapshot::default()
    }));
    let _ = drain_intents();
    let mut mesa = Mesa::nova(TimelinePanelState::default());
    mesa.assenta_em(JANELA);
    mesa
}

/// A fila fica ACIMA da faixa das abas, encostada à esquerda, por ordem; o Time e a duração não.
/// *Mutação: pintar a fila dentro do painel (o `rect` sem subir) ⇒ vermelho.*
#[test]
fn os_controlos_flutuam_acima_do_painel() {
    let m0 = mesa();
    let abas = m0.rect(ids::TIMELINE_TAB_KEYS).expect("a aba Keys");
    let fila = [
        ids::TIMELINE_GO_START,
        ids::TIMELINE_PREV_FRAME,
        ids::TIMELINE_STOP,
        ids::TIMELINE_PLAY,
        ids::TIMELINE_NEXT_FRAME,
        ids::TIMELINE_GO_END,
        ids::TIMELINE_AUTOKEY,
    ]
    .map(|id| m0.rect(id).unwrap_or_else(|| panic!("{id:?} não pintado")));
    for (i, r) in fila.iter().enumerate() {
        assert!(
            r.y + r.h <= abas.y,
            "o {i}.º controlo não está acima do painel"
        );
        if i > 0 {
            assert!(fila[i - 1].x < r.x, "fora de ordem no {i}.º");
        }
    }
    assert!(fila[0].x <= abas.x, "a fila não começa no canto esquerdo");
    let tempo = m0.rect(ids::TIMELINE_TIME_NUM).expect("o Time");
    let dur = m0
        .rect(ids::TIMELINE_LENGTH_NUM)
        .expect("a duração na barra");
    assert!((dur.y - tempo.y).abs() < 0.5 && dur.x < tempo.x && abas.x < dur.x);
    ferramenta::force_cards_on_this_thread(None);
}

/// O ● é o Auto Key de sempre: o clique chega à shell como o `Toggle` da caixa.
#[test]
fn o_gravar_liga_o_auto_key() {
    let mut m0 = mesa();
    let acoes = m0.clica(ids::TIMELINE_AUTOKEY);
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::TimelinePanelEvent(PanelEvent::Toggle(id, true)) if *id == ids::TIMELINE_AUTOKEY
        )),
        "{acoes:?}"
    );
    ferramenta::force_cards_on_this_thread(None);
}

/// O ■ pára e volta ao início. *Mutação: tirar o braço do Stop ⇒ nada drenado ⇒ vermelho.*
#[test]
fn o_stop_para_e_volta_ao_inicio() {
    let mut m0 = mesa();
    let _ = m0.clica(ids::TIMELINE_STOP);
    let i = drain_intents();
    assert!(
        i.contains(&TimelineIntent::Pause) && i.contains(&TimelineIntent::Scrub(0.0)),
        "{i:?}"
    );
    ferramenta::force_cards_on_this_thread(None);
}
