//! W23 — o cartão «Live» separado por caminho de ligação (pedido do dono 18/10,
//! `docs/MiroClone/12_cartao_live_por_ligacao.md`): cada coisa no cartão do caminho a que pertence,
//! provado pelo que se pinta e pelo sítio onde o rato acerta.

use super::*;
use crate::documents::live::{LiveFirewall, LiveInternet};
use live_board::LiveSection::{Internet, Network, People};

fn inside(c: Rect, p: (f32, f32)) -> bool {
    p.0 >= c.x && p.0 <= c.x + c.w && p.1 >= c.y && p.1 <= c.y + c.h
}

impl T {
    fn cards(&self) -> Vec<Rect> {
        self.hero.documents.live_view.panel.sections.clone()
    }

    /// O centro do botão `b` no ecrã (onde o rato o acerta).
    fn live_at(&self, b: LiveButton) -> (f32, f32) {
        self.find(live_node(Hit::Button(b)))
            .unwrap_or_else(|| panic!("{b:?} não está pintado"))
    }
}

/// Os textos pintados com pelo menos `glyphs` letras cuja linha de base cai em `band`.
fn texts_in(t: &mut T, band: Rect, glyphs: usize) -> usize {
    runs(&t.scene())
        .iter()
        .filter(|(_, n, at)| *n >= glyphs && inside(band, (at[0] as f32, at[1] as f32)))
        .count()
}

/// O título de cada cartão está pintado na faixa por cima dele.
fn titled(t: &mut T, cards: &[Rect]) {
    let s = crate::screens::hero::board_nav::facilitate::btn();
    for (c, sec) in cards.iter().zip(live_board::sections(&t.hero)) {
        let band = Rect::new(c.x, c.y - s, c.w, s);
        let key = match sec {
            Network => "board.live.sec_network",
            Internet => "board.live.sec_internet",
            People => "board.live.people",
        };
        let n = ph2d_i18n::tr(key)
            .chars()
            .filter(|c| !c.is_whitespace())
            .count();
        assert!(
            texts_in(t, band, n) >= 1,
            "o título de {sec:?} não está por cima de {c:?}"
        );
    }
}

/// ⭐ O anfitrião vê três cartões pela ordem rede local · internet · pessoas: a firewall e o endereço
/// no da rede local, «Copy the link again» no da internet, as pessoas no delas, e «Bring everyone
/// here»/«End session» abaixo de todos.
#[test]
fn the_host_card_groups_each_way_in_its_own_card() {
    let mut t = T::new();
    t.open_live();
    t.room(LiveRole::Hosting {
        code: "ABCDEF".into(),
        addrs: vec!["10.0.0.5:47320".into()],
    });
    let lv = t.hero.documents.live_view_mut();
    lv.internet = LiveInternet::Ready {
        link: "ph2d-board://v1/#x".into(),
    };
    lv.firewall = LiveFirewall {
        can_remove: true,
        ..LiveFirewall::default()
    };
    t.paint();
    assert_eq!(
        live_board::sections(&t.hero),
        vec![Network, Internet, People]
    );
    let c = t.cards();
    assert_eq!(c.len(), 3, "{c:?}");
    for w in c.windows(2) {
        assert!(w[0].y + w[0].h < w[1].y, "os cartões encostam: {c:?}");
    }
    assert!(inside(c[0], t.live_at(LiveButton::FirewallRemove)));
    assert!(inside(c[1], t.live_at(LiveButton::InviteInternet)));
    assert!(!inside(c[0], t.live_at(LiveButton::InviteInternet)));
    assert!(
        texts_in(&mut t, c[0], "10.0.0.5:47320".len()) >= 1,
        "o endereço fora da rede local"
    );
    assert!(
        texts_in(&mut t, c[2], "Bia".len()) >= 2,
        "as pessoas fora do cartão delas"
    );
    let last = c[2].y + c[2].h;
    for b in [LiveButton::Summon, LiveButton::End] {
        assert!(t.live_at(b).1 > last, "{b:?} não está abaixo dos cartões");
    }
    titled(&mut t, &c);
    // O botão da firewall continua a pedir o mesmo.
    t.asked();
    t.live_click(Hit::Button(LiveButton::FirewallRemove));
    assert_eq!(t.asked(), vec![LiveRequest::FirewallRemove]);
}

/// ⭐ Sem sessão: entrar com o código no cartão da rede local, e o link num campo SEU no da internet
/// (Enter e «Join by link» pedem a entrada com ele); «Invite over the internet» abre a sala e faz o
/// link de uma vez.
#[test]
fn off_the_link_has_its_own_field_in_the_internet_card() {
    const LINK: NodeId = crate::ids::BOARD_LIVE_LINK_INPUT;
    const CODE: NodeId = crate::ids::BOARD_LIVE_CODE_INPUT;
    let link = "ph2d-board://v1/127.0.0.1:5000#AbCdEfGhIjKlMnOpQrStUv";
    let mut t = T::new();
    t.open_live();
    t.asked();
    assert_eq!(live_board::sections(&t.hero), vec![Network, Internet]);
    let c = t.cards();
    assert_eq!(c.len(), 2, "{c:?}");
    for b in [LiveButton::Open, LiveButton::Join] {
        assert!(inside(c[0], t.live_at(b)), "{b:?} fora da rede local");
    }
    for b in [LiveButton::InviteInternet, LiveButton::JoinLink] {
        assert!(inside(c[1], t.live_at(b)), "{b:?} fora da internet");
    }
    assert!(inside(c[0], t.find(CODE).expect("o campo do código")));
    assert!(inside(c[1], t.find(LINK).expect("o campo do link")));
    titled(&mut t, &c);
    t.shell_click_id(LINK, "o campo do link");
    assert!(crate::interaction::apply_clipboard_paste(
        &mut t.hero.store,
        LINK,
        link
    ));
    t.enter();
    let want = vec![LiveRequest::JoinLink { link: link.into() }];
    assert_eq!(t.asked(), want, "Enter no campo do link");
    t.live_click(Hit::Button(LiveButton::JoinLink));
    assert_eq!(t.asked(), want, "«Join by link»");
    t.live_click(Hit::Button(LiveButton::InviteInternet));
    assert_eq!(
        t.asked(),
        vec![LiveRequest::Host, LiveRequest::InviteInternet]
    );
}

/// ⭐ Quem entra vê o cartão do caminho por onde entrou: pelo link, o da internet; pelo código, o da
/// rede local — à espera e já dentro; «Cancel»/«Leave» abaixo dos cartões.
#[test]
fn a_guest_sees_the_way_it_came_in() {
    let mut t = T::new();
    t.open_live();
    t.room(LiveRole::Guest { host: "Ana".into() });
    assert_eq!(live_board::sections(&t.hero), vec![Network, People]);
    t.hero.documents.live_view_mut().link.by_link = true;
    t.paint();
    assert_eq!(live_board::sections(&t.hero), vec![Internet, People]);
    let c = t.cards();
    titled(&mut t, &c);
    assert!(t.live_at(LiveButton::Leave).1 > c[1].y + c[1].h);
    let lv = t.hero.documents.live_view_mut();
    lv.role = LiveRole::Joining;
    lv.link.searching = Some(30);
    t.paint();
    assert_eq!(live_board::sections(&t.hero), vec![Internet]);
    let c = t.cards();
    assert!(t.live_at(LiveButton::Cancel).1 > c[0].y + c[0].h);
    t.hero.documents.live_view_mut().link = Default::default();
    t.paint();
    assert_eq!(live_board::sections(&t.hero), vec![Network]);
}
