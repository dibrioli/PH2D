//! ⭐ Dono 12/11: os clipes ficam em chips lado a lado, mas «Show clips as a list» (na engrenagem)
//! troca-os por UMA lista suspensa — a de sempre (`TIMELINE_CLIP_DD`), o «⋯» do escolhido e o «+».
//! E «Record while playing» saiu da barra para a engrenagem.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::TimelineViewSnapshot;
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
        clips: vec!["Main".to_owned(), "Walk".to_owned()],
        clip_index: vec![0, 1],
        ..TimelineViewSnapshot::default()
    }));
    let _ = drain_intents();
    let mut mesa = Mesa::nova(TimelinePanelState::default());
    mesa.assenta_em(JANELA);
    mesa
}

fn lista_aberta(mesa: &Mesa<TimelinePanel>) -> bool {
    matches!(
        mesa.host.store().get(ids::TIMELINE_CLIP_DD),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// Ligar a opção na engrenagem troca os chips pela lista; a lista abre pelo clique e o «⋯» e o «+»
/// ficam. *Mutação: ignorar `clips_as_list` em `barra_clips::paint` ⇒ os chips continuam ⇒ vermelho.*
#[test]
fn a_opcao_da_engrenagem_troca_os_chips_por_uma_lista() {
    let mut m0 = mesa();
    assert!(
        m0.rect(ids::TIMELINE_CLIP_OPT[0]).is_some(),
        "por omissão, chips"
    );
    assert!(m0.rect(ids::TIMELINE_CLIP_DD).is_none());

    let _ = m0.clica(ids::TIMELINE_OPTIONS);
    m0.assenta_em(JANELA);
    let _ = m0.clica(ids::TIMELINE_CLIPS_AS_LIST);
    m0.assenta_em(JANELA);
    assert!(
        m0.st.clips_as_list,
        "a caixa da engrenagem não ligou a lista"
    );

    for i in 0..2 {
        assert!(
            m0.rect(ids::TIMELINE_CLIP_OPT[i]).is_none(),
            "o chip {i} continua na barra"
        );
    }
    let lista = m0.rect(ids::TIMELINE_CLIP_DD).expect("a lista na barra");
    let menu = m0.rect(ids::TIMELINE_CLIP_MENU).expect("o ⋯ do escolhido");
    let mais = m0.rect(ids::TIMELINE_ADD_CLIP).expect("o +");
    assert!(lista.x + lista.w <= menu.x + 0.5 && menu.x + menu.w <= mais.x + 0.5);

    let _ = m0.clica(ids::TIMELINE_CLIP_DD);
    m0.assenta_em(JANELA);
    assert!(lista_aberta(&m0), "a lista não abriu");
    ferramenta::force_cards_on_this_thread(None);
}

/// «Record while playing» não está na barra; mora na engrenagem.
#[test]
fn record_mora_na_engrenagem() {
    let mut m0 = mesa();
    assert!(m0.rect(ids::TIMELINE_RECORD).is_none(), "Record na barra");
    let _ = m0.clica(ids::TIMELINE_OPTIONS);
    m0.assenta_em(JANELA);
    assert!(
        m0.rect(ids::TIMELINE_RECORD).is_some(),
        "Record fora da engrenagem"
    );
    ferramenta::force_cards_on_this_thread(None);
}
