//! ⭐ **A peça escolhida** (doc 17 item 1, maquete B4): o clique no corpo PINTADO de uma strip
//! escolhe-a, arrastar não escolhe, o vazio da faixa larga, e a strip que some larga também.

use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_panel_timeline::tab::Tab;
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{StackHost, StripId, StripSource, TimelineDoc, TimelineViewSnapshot};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// Uma faixa com uma strip de 0 a 1 s.
fn documento(com_strip: bool) -> TimelineViewSnapshot {
    let mut doc = TimelineDoc::new();
    let base = doc.add_lane("Base".into()).expect("a faixa");
    if com_strip {
        doc.add_strip_to(StackHost::Document, base, StripSource::Clip(0), 0.0, 1.0)
            .expect("a strip");
    }
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    snap
}

fn mesa() -> (Mesa<TimelinePanel>, StripId) {
    ferramenta::force_cards_on_this_thread(Some(true));
    let snap = documento(true);
    let id = snap.lanes[0].strips[0].id;
    set_current_timeline(Some(snap));
    let mut mesa = Mesa::nova(TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    });
    mesa.assenta_em(JANELA);
    let _ = drain_intents();
    (mesa, id)
}

fn corpo(id: StripId) -> ph2d_a11y::NodeId {
    ids::timeline_strip_hit_id(0, id.0, 2)
}

#[test]
fn o_clique_no_corpo_escolhe_arrastar_nao_e_o_vazio_larga() {
    let (mut m, id) = mesa();
    let r = m.rect(corpo(id)).expect("o corpo pintado");

    // Arrastar além da folga não escolhe.
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = m.ponteiro(PointerKind::Down, cx, cy);
    ev.extend(m.ponteiro(PointerKind::Move, cx + 20.0, cy));
    ev.extend(m.ponteiro(PointerKind::Move, cx + 40.0, cy));
    ev.extend(m.ponteiro(PointerKind::Up, cx + 40.0, cy));
    let _ = m.aplica(ev);
    m.assenta_em(JANELA);
    assert_eq!(m.st.strip_escolhida, None, "o arrasto não escolhe");

    let _ = m.clica(corpo(id));
    m.assenta_em(JANELA);
    assert_eq!(m.st.strip_escolhida, Some(id), "o clique no corpo escolhe");

    // O vazio da faixa, à direita da strip.
    let (vx, vy) = (r.x + r.w + 30.0, cy);
    assert_eq!(
        m.host.hit_at(vx, vy),
        Some(ids::TIMELINE_LANES),
        "o vazio é a faixa"
    );
    let ev = m.host.click_at(vx, vy);
    let _ = m.aplica(ev);
    m.assenta_em(JANELA);
    assert_eq!(m.st.strip_escolhida, None, "o vazio larga");

    // A strip que deixa de existir larga a escolha.
    let _ = m.clica(corpo(id));
    m.assenta_em(JANELA);
    assert_eq!(m.st.strip_escolhida, Some(id));
    set_current_timeline(Some(documento(false)));
    m.assenta_em(JANELA);
    assert_eq!(m.st.strip_escolhida, None, "a sumida larga");
    let _ = drain_intents();
    ferramenta::force_cards_on_this_thread(None);
}

/// O Esc sem dono larga a peça escolhida; com um menu aberto, o Esc é do menu e a peça fica.
#[test]
fn o_esc_larga_a_peca_e_o_menu_aberto_fica_com_o_esc() {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::panel::PanelHostInternal;
    let (mut m, id) = mesa();
    let _ = m.clica(corpo(id));
    m.assenta_em(JANELA);
    assert_eq!(m.st.strip_escolhida, Some(id), "o clique no corpo escolhe");

    m.host.store_mut().open_context_menu(ContextMenuRequest {
        x: 10.0,
        y: 10.0,
        kind: ContextMenuKind::CreateNote {
            panel: ids::TIMELINE_LANES,
            section: None,
        },
    });
    let _ = m.esc();
    m.assenta_em(JANELA);
    assert!(
        m.host.store_mut().context_menu().is_none(),
        "o Esc fecha o menu"
    );
    assert_eq!(
        m.st.strip_escolhida,
        Some(id),
        "o Esc do menu não larga a peça"
    );

    let _ = m.esc();
    m.assenta_em(JANELA);
    assert_eq!(m.st.strip_escolhida, None, "o Esc sem dono larga a peça");
    let _ = drain_intents();
    ferramenta::force_cards_on_this_thread(None);
}
