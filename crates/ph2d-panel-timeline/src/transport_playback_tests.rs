//! **A lista de Containers não tem modo de playback** (Enio, 2026-07-22: *"os comandos de
//! play/pause devem ficar desativados e a marca de loop não deve aparecer e só voltam a
//! funcionar ao entrar no container, ou ao voltar para os clips ou ao ir para arrange"*).
//!
//! Os gates rodam o **paint REAL** do painel (`MockPanelHost::paint`) e leem o que ele tornou
//! CLICÁVEL — porque a metade perigosa deste pedido é a de sempre: um botão esmaecido que
//! ainda despacha mente, e um gate que só olhasse a cor do pixel ficaria verde sobre ele.
//! A recusa do painel é a AUSÊNCIA do hit; as camadas do shell (intent + backstop do relógio)
//! têm gates próprios em `timeline_bridge_tests`
//! ([[feedback_layered_defenses_need_per_layer_gates]]).

use super::*;
use crate::state::TimelinePanelState;
use ph2d_timeline::{StackHost, StripSource, TimelineDoc, TimelineViewSnapshot};

/// Um documento com um container habitado, uma lane na cena e um LOOP armado no relógio da
/// timeline — a marca que o Enio viu sobrar na lista. `inside` acrescenta a migalha que
/// significa "o animador ENTROU no container".
fn snap_with_loop(inside: bool) -> TimelineViewSnapshot {
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("Walk".into());
    let lane = doc
        .add_lane_in(StackHost::Container(c), "in".into())
        .unwrap();
    doc.add_strip_to(
        StackHost::Container(c),
        lane,
        StripSource::Clip(0),
        0.0,
        2.0,
    )
    .unwrap();
    doc.add_lane("L".into()).unwrap();
    doc.set_active_loop_for(false, Some((0.0, 2.0)));
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut out = TimelineViewSnapshot::default();
    out.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    assert!(
        out.loop_range.is_some(),
        "a fixture tem de conter o fenômeno: um loop armado"
    );
    if inside {
        out.crumbs = vec![(0, "Walk".into())];
    }
    out
}

/// Pinta o painel de verdade numa vista e devolve os hits registrados.
fn painted_hits(tab: crate::tab::Tab, inside: bool) -> Vec<ph2d_a11y::NodeId> {
    crate::state::set_current_timeline(Some(snap_with_loop(inside)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let mut state = TimelinePanelState {
        tab,
        ..TimelinePanelState::default()
    };
    let hits = host.paint::<crate::TimelinePanel>(&mut state, Rect::new(0.0, 0.0, 1400.0, 500.0));
    crate::state::set_current_timeline(None);
    hits.into_iter().map(|(id, _)| id).collect()
}

/// **O play/pause está MORTO na lista — e vivo em todas as outras vistas.**
///
/// Morto = sem hit: a recusa mora na costura, não na cor. O go-to-start é o controle
/// positivo (o cluster do transporte continua lá; só o playback é recusado), e as três
/// vistas onde ele VOLTA são exatamente as três que o Enio nomeou.
#[test]
fn the_play_button_is_dead_on_the_containers_list_and_alive_everywhere_else() {
    let cases = [
        (crate::tab::Tab::Containers, false, false), // a LISTA: morto
        (crate::tab::Tab::Containers, true, true),   // dentro de um container: volta
        (crate::tab::Tab::Keys, false, true),        // de volta aos clips: volta
        (crate::tab::Tab::Arrange, false, true),     // no arrange: volta
    ];
    for (tab, inside, expect_play) in cases {
        let hits = painted_hits(tab, inside);
        assert_eq!(
            hits.contains(&ids::TIMELINE_PLAY),
            expect_play,
            "play em {tab:?} (inside={inside}) devia ser vivo={expect_play}"
        );
        assert!(
            hits.contains(&ids::TIMELINE_GO_START),
            "controle positivo: o resto do transporte segue clicável em {tab:?}"
        );
    }
}

/// **A marca de loop não aparece na lista** — nem a banda nem as GARRAS (uma banda oculta
/// com grips vivos seria um controle invisível). O mesmo snapshot no Arrange pinta as garras
/// — o controle positivo sem o qual "sem braces em lugar nenhum" ficaria verde.
#[test]
fn the_loop_mark_leaves_the_containers_list_and_stays_on_arrange() {
    let braces = |hits: &[ph2d_a11y::NodeId]| {
        (0..=2)
            .filter(|e| hits.contains(&ids::timeline_loop_brace_id(*e)))
            .count()
    };
    let list = painted_hits(crate::tab::Tab::Containers, false);
    assert_eq!(
        braces(&list),
        0,
        "a lista não oferece as garras do loop — a marca não existe ali"
    );
    let arrange = painted_hits(crate::tab::Tab::Arrange, false);
    assert!(
        braces(&arrange) > 0,
        "o MESMO loop pinta as garras no Arrange — a fixture contém o fenômeno"
    );
}

/// **O painel publica a vista para o shell** — é deste bit (`state::containers_list`) que as
/// camadas do shell (recusa do intent, pausa do bridge, barra de espaço) leem. Publicado no
/// paint, dos dois lados: `true` na lista, `false` em qualquer outra vista.
#[test]
fn the_panel_publishes_the_list_view_and_clears_it_on_every_other_view() {
    let _ = painted_hits(crate::tab::Tab::Containers, false);
    assert!(
        crate::state::containers_list(),
        "a lista publica true — sem isso o shell nunca fica sabendo"
    );
    let _ = painted_hits(crate::tab::Tab::Keys, false);
    assert!(
        !crate::state::containers_list(),
        "sair da lista publica false — um true velho congelaria o playback do app"
    );
    let _ = painted_hits(crate::tab::Tab::Containers, true);
    assert!(
        !crate::state::containers_list(),
        "DENTRO de um container não é a lista: o playback volta ali"
    );
}

/// **Selecionar um objeto novo aterrissa o painel na aba Keys** (Enio, 2026-07-22) — o
/// consumo do request, no paint REAL, pela porta única `set_tab`. O controle positivo é
/// a metade que importa: SEM request o paint não mexe na aba, senão o animador nunca
/// ficaria em Arrange.
#[test]
fn a_keys_tab_request_is_honoured_by_the_next_paint_and_only_then() {
    use ph2d_editor_core::zones::Rect;
    crate::state::set_current_timeline(Some(snap_with_loop(false)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let mut state = TimelinePanelState {
        tab: crate::tab::Tab::Arrange,
        ..TimelinePanelState::default()
    };
    let vp = Rect::new(0.0, 0.0, 1400.0, 500.0);

    // Sem request, a aba fica onde o animador a deixou.
    host.paint::<crate::TimelinePanel>(&mut state, vp);
    assert_eq!(
        state.tab,
        crate::tab::Tab::Arrange,
        "paint sem request não pode puxar a aba"
    );

    // Com request, o MESMO paint aterrissa em Keys — e o consumo é one-shot.
    crate::state::request_keys_tab();
    host.paint::<crate::TimelinePanel>(&mut state, vp);
    assert_eq!(
        state.tab,
        crate::tab::Tab::Keys,
        "o request aterrissa em Keys"
    );
    state.tab = crate::tab::Tab::Arrange;
    host.paint::<crate::TimelinePanel>(&mut state, vp);
    assert_eq!(
        state.tab,
        crate::tab::Tab::Arrange,
        "consumido: o frame seguinte não re-aplica o pedido"
    );
    crate::state::set_current_timeline(None);
}
