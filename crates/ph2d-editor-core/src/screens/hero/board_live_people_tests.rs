//! W23 A3 — o «+n» dos avatares e a lista das pessoas do painel «Live», pelo caminho real do ecrã.

use super::*;

/// ⭐ W23 A3 — com gente demais para a fila, os avatares acabam num «+n» que se CLICA: abre o painel
/// na lista a começar por quem não coube; o «+n» da lista mostra as seguintes e dá a volta.
#[test]
fn the_plus_n_beside_the_avatars_opens_everyone_and_the_list_pages() {
    let mut t = T::new();
    t.room(LiveRole::Guest { host: "Ana".into() });
    let n = 40_u64;
    t.hero.documents.live_view_mut().peers = (0..n)
        .map(|i| LivePeer {
            peer: i + 1,
            name: format!("P{i}"),
            color: (i % 16) as u8,
            me: i == 0,
        })
        .collect();
    t.paint();
    let shown = t.hero.documents.live_view.panel.avatars_shown;
    assert!(shown > 0 && (shown as u64) < n, "{shown}");
    t.live_click(Hit::Button(LiveButton::Everyone));
    assert!(live_board::is_open(&t.hero));
    let p = &t.hero.documents.live_view.panel;
    assert_eq!(
        p.people_from, shown,
        "a lista começa por quem não coube na fila"
    );
    let page = p.people_shown;
    assert!(page > 0 && (page as u64) < n, "{page}");
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..n {
        let from = t.hero.documents.live_view.panel.people_from;
        let page = t.hero.documents.live_view.panel.people_shown;
        seen.extend((0..page).map(|k| (from + k) % n as usize));
        if seen.len() == n as usize {
            break;
        }
        t.live_click(Hit::Button(LiveButton::MorePeople));
        assert_eq!(
            t.hero.documents.live_view.panel.people_from,
            (from + page) % n as usize
        );
    }
    assert_eq!(
        seen.len(),
        n as usize,
        "o «+n» da lista chega a toda a gente"
    );
}
