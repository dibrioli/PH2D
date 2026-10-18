//! O quadro ao vivo (W11) pelo caminho REAL do ecrã: o botão «Live» e o painel pela ordem da shell
//! (`shell_click_id`), os campos pelo teclado do hero, o quadro pela porta do ponteiro e da roda
//! (`board_view`), e o que se pinta lido no `encoding` da cena (os glifos de cada texto).

use super::*;
use crate::documents::live::{LiveFound, LiveOther, LivePeer, LiveRequest, LiveRole};
use crate::screens::hero::board_nav::team::live_board::{self, Hit, LiveButton, node as live_node};
use crate::screens::hero::board_nav::{NavButton, button_node};

impl T {
    fn live_click(&mut self, h: Hit) {
        self.shell_click_id(live_node(h), &format!("{h:?}"));
    }

    fn asked(&mut self) -> Vec<LiveRequest> {
        self.hero.documents.take_live()
    }

    fn open_live(&mut self) {
        self.shell_click_id(button_node(NavButton::Live), "Live");
    }

    /// Escreve `s` no campo com foco pelo despacho do hero (as letras uma a uma).
    fn type_text(&mut self, s: &str) {
        for ch in s.chars() {
            let arena = Bump::new();
            let ev = self.hero.handle_text_input(ch, &arena).to_vec();
            for e in ev {
                self.hero.apply_event(e);
            }
        }
        self.paint();
    }

    fn enter(&mut self) {
        let arena = Bump::new();
        let e = ph2d_host::KeyEvent {
            keycode: crate::interaction::KEY_ENTER,
            modifiers: Modifiers::default(),
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = self.hero.handle_key(e, &arena).to_vec();
        for e in ev {
            self.hero.apply_event(e);
        }
        self.paint();
    }

    /// A sala com a pessoa `me` (eu) e a Bia (par 2, cor 3), como a família a escreve.
    fn room(&mut self, role: LiveRole) {
        let board = self.hero.documents.active();
        let lv = self.hero.documents.live_view_mut();
        lv.role = role;
        lv.board = board;
        lv.peers = vec![
            LivePeer {
                peer: 1,
                name: "Eu".into(),
                color: 0,
                me: true,
            },
            LivePeer {
                peer: 2,
                name: "Bia".into(),
                color: 3,
                me: false,
            },
        ];
        self.paint();
    }

    fn following(&self) -> Option<u64> {
        self.hero.documents.live_view().following
    }

    /// A cena inteira desta imagem (o mesmo pintor do `paint`).
    fn scene(&mut self) -> VectorScene {
        let mut scene = VectorScene::new();
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        paint_hero_screen(&mut self.hero, vp, &mut scene, &mut self.ts);
        scene
    }
}

/// Os textos pintados: `(tamanho da letra, nº de glifos, origem do 1.º glifo)`.
fn runs(scene: &VectorScene) -> Vec<(f32, usize, [f64; 2])> {
    let res = &scene.inner().encoding().resources;
    res.glyph_runs
        .iter()
        .filter_map(|r| {
            let g = res.glyphs[r.glyphs.clone()].first()?;
            let t = r.transform.translation;
            let at = [f64::from(t[0] + g.x), f64::from(t[1] + g.y)];
            Some((r.font_size, r.glyphs.len(), at))
        })
        .collect()
}

/// ⭐ O botão «Live» abre o painel (pintado na coluna da equipa) e procura quadros na rede logo, e de
/// novo a cada 3 s pelo tique; fechar pára a procura.
#[test]
fn the_live_button_opens_the_panel_and_discovers_every_three_seconds() {
    let mut t = T::new();
    assert!(
        t.find(live_node(Hit::Button(LiveButton::Surface)))
            .is_none()
    );
    t.open_live();
    assert!(live_board::is_open(&t.hero));
    assert!(
        t.find(live_node(Hit::Button(LiveButton::Surface)))
            .is_some()
    );
    assert_eq!(t.asked(), vec![LiveRequest::Discover]);
    t.hero.tick_motion(1.0);
    assert!(t.asked().is_empty(), "procurou antes dos 3 s");
    t.hero.tick_motion(2.1); // LITERAL-PX-OK: segundos do relógio do teste
    assert_eq!(t.asked(), vec![LiveRequest::Discover]);
    // Com «Comments» aberto depois, o «Live» cede-lhe a coluna; e o × fecha.
    t.shell_click_id(button_node(NavButton::Comments), "Comments");
    assert!(
        !live_board::is_open(&t.hero),
        "dois painéis na mesma coluna"
    );
    t.open_live();
    assert!(
        !t.hero.documents.live.team.list_open,
        "o Live não fechou a lista"
    );
    t.live_click(Hit::Button(LiveButton::Close));
    assert!(!live_board::is_open(&t.hero));
    t.asked();
    t.hero.tick_motion(5.0); // LITERAL-PX-OK: segundos do relógio do teste
    assert!(t.asked().is_empty(), "fechado e ainda a procurar");
}

/// `Off` — «Open to the network» pede para abrir a sala.
#[test]
fn open_to_the_network_asks_to_host() {
    let mut t = T::new();
    t.open_live();
    t.asked();
    t.live_click(Hit::Button(LiveButton::Open));
    assert_eq!(t.asked(), vec![LiveRequest::Host]);
}

/// ⭐ `Off` — o código escreve-se de qualquer maneira, mostra-se em maiúsculas, e «Join» pede com ele
/// limpo; curto, não pede (e diz porquê). Uma linha achada preenche o endereço e o Enter entra.
#[test]
fn a_typed_code_joins_normalized_and_a_found_row_fills_the_address() {
    let mut t = T::new();
    t.open_live();
    t.asked();
    t.shell_click_id(crate::ids::BOARD_LIVE_CODE_INPUT, "o campo do código");
    t.type_text("ab3");
    t.live_click(Hit::Button(LiveButton::Join));
    assert!(t.asked().is_empty(), "um código curto pediu para entrar");
    assert_eq!(
        t.hero.documents.live.notice.map(|n| n.0),
        Some("board.live.code_short")
    );
    t.shell_click_id(crate::ids::BOARD_LIVE_CODE_INPUT, "o campo do código");
    t.type_text(" x-9z");
    let shown = crate::screens::hero::board_nav::field_text_for_tests(
        &t.hero,
        crate::ids::BOARD_LIVE_CODE_INPUT,
    );
    assert_eq!(shown, "AB3 X-9Z", "o campo não mostra em maiúsculas");
    t.live_click(Hit::Button(LiveButton::Join));
    assert_eq!(
        t.asked(),
        vec![LiveRequest::Join {
            addr: None,
            code: "AB3X9Z".into()
        }]
    );
    // Uma linha achada: o endereço vai para o campo, o teclado para o código.
    t.hero.documents.live_view_mut().found = vec![LiveFound {
        board: "Plan".into(),
        host: "Ana".into(),
        addr: "10.0.0.5:7777".into(),
    }];
    t.paint();
    t.live_click(Hit::Found(0));
    assert_eq!(
        crate::screens::hero::board_nav::field_text_for_tests(
            &t.hero,
            crate::ids::BOARD_LIVE_ADDR_INPUT
        ),
        "10.0.0.5:7777"
    );
    t.type_text("qwe rty");
    t.enter();
    assert_eq!(
        t.asked(),
        vec![LiveRequest::Join {
            addr: Some("10.0.0.5:7777".into()),
            code: "QWERTY".into()
        }]
    );
}

/// `Hosting` pinta o código grande em duas metades; `Joining` cancela; `Guest` traz todos e sai.
#[test]
fn each_role_paints_its_own_panel_and_buttons() {
    let mut t = T::new();
    t.open_live();
    t.room(LiveRole::Hosting {
        code: "ABCDEF".into(),
        addrs: vec!["192.168.0.7:7777".into()],
    });
    let big = ph2d_tokens::TypeToken::Xl2.px();
    let code = runs(&t.scene())
        .into_iter()
        .filter(|r| r.0 == big)
        .map(|r| r.1)
        .collect::<Vec<_>>();
    assert_eq!(code, vec![7], "o código ABC-DEF não está pintado");
    t.asked();
    t.live_click(Hit::Button(LiveButton::Summon));
    t.live_click(Hit::Button(LiveButton::End));
    assert_eq!(t.asked(), vec![LiveRequest::Summon, LiveRequest::Leave]);
    t.hero.documents.live_view_mut().role = LiveRole::Joining;
    t.paint();
    assert!(t.find(live_node(Hit::Button(LiveButton::Open))).is_none());
    t.live_click(Hit::Button(LiveButton::Cancel));
    assert_eq!(t.asked(), vec![LiveRequest::Leave]);
    t.hero.documents.live_view_mut().role = LiveRole::Guest { host: "Ana".into() };
    t.paint();
    t.live_click(Hit::Button(LiveButton::Leave));
    assert_eq!(t.asked(), vec![LiveRequest::Leave]);
}

/// ⭐ c21 — o avatar de OUTRA pessoa liga o seguir (o selo aparece), o × desliga; clicar de novo no
/// avatar desliga; e cada gesto meu na vista (pan com o meio, a roda, carregar no quadro) pára-o.
#[test]
fn an_avatar_follows_and_my_own_view_gestures_stop_following() {
    let mut t = T::new();
    t.room(LiveRole::Guest { host: "Ana".into() });
    assert!(
        t.find(live_node(Hit::Avatar(0))).is_none(),
        "o meu avatar segue-me"
    );
    t.live_click(Hit::Avatar(1));
    assert_eq!(t.following(), Some(2));
    assert!(t.find(live_node(Hit::Button(LiveButton::Badge))).is_some());
    t.live_click(Hit::Button(LiveButton::Unfollow));
    assert_eq!(t.following(), None);
    assert!(t.find(live_node(Hit::Button(LiveButton::Badge))).is_none());
    t.live_click(Hit::Avatar(1));
    t.live_click(Hit::Avatar(1));
    assert_eq!(t.following(), None, "o 2.º clique não desligou");
    let (a, b) = (t.at(0.5, 0.6), t.at(0.6, 0.7));
    // Pan com o botão do meio.
    t.live_click(Hit::Avatar(1));
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    let mid = PointerButton::Middle;
    board_view::pointer(h, inp(ts, c), PointerKind::Down, mid, a.0, a.1, true);
    board_view::pointer_move(h, inp(ts, c), b.0, b.1);
    board_view::pointer(h, inp(ts, c), PointerKind::Up, mid, b.0, b.1, true);
    assert_eq!(t.following(), None, "o pan não parou de seguir");
    // A roda (zoom).
    t.paint();
    t.live_click(Hit::Avatar(1));
    assert!(board_view::wheel(&mut t.hero, a.0, a.1, 1.0));
    assert_eq!(t.following(), None, "o zoom não parou de seguir");
    // Carregar no quadro.
    t.paint();
    t.live_click(Hit::Avatar(1));
    t.drag(a, b);
    assert_eq!(
        t.following(),
        None,
        "o clique no quadro não parou de seguir"
    );
    // Quem sai da sala deixa de ser seguido.
    t.live_click(Hit::Avatar(1));
    t.hero.documents.live_view_mut().peers.truncate(1);
    t.hero.tick_motion(0.0);
    assert_eq!(t.following(), None);
}

/// O meu cursor: no mundo do quadro da sala ao mexer o rato sobre ele; `None` fora dele ou sem sala.
#[test]
fn moving_the_mouse_over_the_room_board_writes_my_cursor() {
    let mut t = T::new();
    let p = t.at(0.5, 0.5);
    board_view::pointer_move(&mut t.hero, inp(&mut t.ts, &mut t.clock), p.0, p.1);
    assert_eq!(t.hero.documents.live_view().my_cursor, None, "sem sala");
    t.room(LiveRole::Guest { host: "Ana".into() });
    board_view::pointer_move(&mut t.hero, inp(&mut t.ts, &mut t.clock), p.0, p.1);
    let a = board_view::area_of(t.hero.last_canvas);
    let cam = t.hero.documents.active_board().unwrap().camera;
    let want = cam.to_world(a, [f64::from(p.0), f64::from(p.1)]);
    assert_eq!(t.hero.documents.live_view().my_cursor, Some(want));
    let c = t.hero.last_canvas;
    board_view::pointer_move(
        &mut t.hero,
        inp(&mut t.ts, &mut t.clock),
        c.x - 1.0,
        c.y + 1.0,
    );
    assert_eq!(
        t.hero.documents.live_view().my_cursor,
        None,
        "fora do quadro"
    );
}

/// ⭐ Um `LiveOther` com cursor pinta a etiqueta com o NOME ao lado da ponta (os glifos do nome, no
/// tamanho da etiqueta, logo abaixo e à direita do ponto); fora do quadro da sala, nada.
#[test]
fn another_persons_cursor_paints_their_name_beside_it() {
    let mut t = T::new();
    t.room(LiveRole::Guest { host: "Ana".into() });
    let at = t.at(0.5, 0.5);
    let a = board_view::area_of(t.hero.last_canvas);
    let cam = t.hero.documents.active_board().unwrap().camera;
    let world = cam.to_world(a, [f64::from(at.0), f64::from(at.1)]);
    t.hero.documents.live_view_mut().others = vec![LiveOther {
        peer: 2,
        name: "Bianca".into(),
        color: 3,
        cursor: Some(world),
        selection: Vec::new(),
        anonymous: false,
    }];
    let xs = ph2d_tokens::TypeToken::Xs.px();
    let near = |r: &(f32, usize, [f64; 2])| {
        let reach = f64::from(ph2d_tokens::Spacing::Xl4.px()) * 2.0;
        r.0 == xs
            && r.1 == "Bianca".len()
            && r.2[0] > f64::from(at.0)
            && r.2[0] < f64::from(at.0) + reach
            && r.2[1] > f64::from(at.1)
            && r.2[1] < f64::from(at.1) + reach
    };
    assert!(
        runs(&t.scene()).iter().any(near),
        "o nome não está ao lado do cursor: {:?}",
        runs(&t.scene())
    );
    // Controlo: o mesmo par fora do quadro da sala não se pinta.
    t.hero.documents.live_view_mut().board = None;
    assert!(!runs(&t.scene()).iter().any(near), "pintou fora da sala");
}

/// ADR-0181 §8 — o botão da firewall: aparece antes de abrir e na sala do anfitrião quando a família
/// diz que ela fecha esta rede, pede pela costura REAL, some enquanto a senha é pedida, e nunca ao
/// convidado.
#[test]
fn the_firewall_buttons_show_when_the_family_says_and_ask_it() {
    use crate::documents::live::LiveFirewall;
    let allow = live_node(Hit::Button(LiveButton::FirewallAllow));
    let remove = live_node(Hit::Button(LiveButton::FirewallRemove));
    let mut t = T::new();
    t.open_live();
    t.paint();
    assert!(
        t.find(allow).is_none() && t.find(remove).is_none(),
        "sem firewall, sem botão"
    );
    let set = |t: &mut T, can_allow, can_remove, busy| {
        t.hero.documents.live_view_mut().firewall = LiveFirewall {
            can_allow,
            can_remove,
            busy,
        };
        t.paint();
    };
    set(&mut t, true, false, false);
    assert!(t.find(remove).is_none());
    t.asked();
    t.live_click(Hit::Button(LiveButton::FirewallAllow));
    assert_eq!(t.asked(), vec![LiveRequest::FirewallAllow]);
    set(&mut t, false, true, false);
    assert!(t.find(allow).is_none());
    t.live_click(Hit::Button(LiveButton::FirewallRemove));
    assert_eq!(t.asked(), vec![LiveRequest::FirewallRemove]);
    set(&mut t, true, true, true);
    assert!(
        t.find(allow).is_none() && t.find(remove).is_none(),
        "à espera da senha não se pede outra vez"
    );
    set(&mut t, true, false, false);
    t.room(LiveRole::Hosting {
        code: "ABCDEF".into(),
        addrs: vec!["192.168.0.7:47320".into()],
    });
    t.asked();
    t.live_click(Hit::Button(LiveButton::FirewallAllow));
    assert_eq!(
        t.asked(),
        vec![LiveRequest::FirewallAllow],
        "na sala do anfitrião"
    );
    t.room(LiveRole::Guest { host: "Ana".into() });
    assert!(t.find(allow).is_none(), "o convidado não abre a firewall");
}

/// Uma lista que não cabe (muitos quadros achados) acaba em «+n» com o que ficou de fora, e o
/// cartão não passa do canto — antes cortava-se calada.
#[test]
fn a_list_that_does_not_fit_ends_in_plus_n() {
    let mut t = T::new();
    t.open_live();
    t.hero.documents.live_view_mut().found = (0..200)
        .map(|i| LiveFound {
            board: format!("B{i}"),
            host: "Ana".into(),
            addr: format!("10.0.0.{}:47320", i % 250),
        })
        .collect();
    t.paint();
    let shown = t.hero.documents.live_view.panel.found.len();
    assert!(shown > 0 && shown < 200, "{shown}");
    let more = format!("+{}", 200 - shown).chars().count();
    let xs = ph2d_tokens::TypeToken::Xs.px();
    let plus = |t: &mut T| {
        runs(&t.scene())
            .iter()
            .filter(|r| r.0 == xs && r.1 == more)
            .count()
    };
    let cut = plus(&mut t);
    // O cartão pára ACIMA da fila do canto (o «Live»): a fila do «+n» saiu do que cabe.
    let live_top = crate::screens::hero::board_nav::button_rects(t.hero.last_canvas)
        .iter()
        .find(|(b, _)| *b == NavButton::Live)
        .map(|(_, r)| r.y)
        .unwrap();
    let surface = live_node(Hit::Button(LiveButton::Surface));
    let x = t.find(surface).unwrap().0;
    let vp = t.hero.last_viewport;
    let mut y = vp.y;
    let mut low = vp.y;
    while y < vp.y + vp.h {
        if t.hero
            .hit_index
            .hit(x, y)
            .is_some_and(|id| live_board::hit_of(id).is_some())
        {
            low = y;
        }
        y += 1.0;
    }
    assert!(
        low < live_top,
        "o cartão desce a {low} e o «Live» começa a {live_top}"
    );
    t.hero.documents.live_view_mut().found.truncate(2);
    t.paint();
    assert_eq!(
        t.hero.documents.live_view.panel.found.len(),
        2,
        "o que cabe mostra-se todo"
    );
    assert_eq!(cut, plus(&mut t) + 1, "não pintou a fila do «+n»");
}

/// O cartão que desce até à fila do canto (muitos endereços, janela baixa) não engole o clique nos
/// avatares: eles registam-se DEPOIS dele (auditoria W11 — antes «seguir» morria calado).
#[test]
fn a_tall_panel_never_swallows_the_avatar_click() {
    let mut t = T::new();
    t.open_live();
    t.room(LiveRole::Hosting {
        code: "ABCDEF".into(),
        addrs: (0..40).map(|i| format!("10.0.{i}.7:47320")).collect(),
    });
    t.paint();
    t.live_click(Hit::Avatar(1));
    assert_eq!(t.following(), Some(2), "o clique no avatar caiu no cartão");
}

/// ⭐ ADR-0187 — um link COLADO no campo do código (o caminho do Ctrl+V da shell: inserir e
/// `TextChanged`) fica como está — a chave distingue maiúsculas —, e «Join by link», o «Join» e o
/// Enter pedem para entrar com ele inteiro. Um texto que não é link não pede e diz porquê.
#[test]
fn a_pasted_link_keeps_its_case_and_joins_by_link() {
    const CODE: NodeId = crate::ids::BOARD_LIVE_CODE_INPUT;
    let link = "ph2d-board://v1/127.0.0.1:5000#AbCdEfGhIjKlMnOpQrStUv";
    let mut t = T::new();
    t.open_live();
    t.asked();
    t.live_click(Hit::Button(LiveButton::JoinLink));
    assert!(t.asked().is_empty(), "sem link pediu para entrar");
    assert_eq!(
        t.hero.documents.live.notice.map(|n| n.0),
        Some("board.live.link_bad")
    );
    t.shell_click_id(CODE, "o campo do código");
    assert!(crate::interaction::apply_clipboard_paste(
        &mut t.hero.store,
        CODE,
        link
    ));
    t.hero.apply_event(WidgetEvent::TextChanged(CODE));
    t.paint();
    let shown = crate::screens::hero::board_nav::field_text_for_tests(&t.hero, CODE);
    assert_eq!(shown, link, "o link colado passou a maiúsculas");
    let want = vec![LiveRequest::JoinLink { link: link.into() }];
    t.live_click(Hit::Button(LiveButton::JoinLink));
    assert_eq!(t.asked(), want);
    t.live_click(Hit::Button(LiveButton::Join));
    assert_eq!(t.asked(), want, "o «Join» com um link entra pelo link");
    t.shell_click_id(CODE, "o campo do código");
    t.enter();
    assert_eq!(t.asked(), want, "o Enter também");
}

/// ⭐ O HUD de estatísticas é centrado na JANELA; numa janela estreita passava por cima do canto do
/// quadro (foto da cena 22 a 1366×768 com os painéis laterais: o «Live» meio tapado, a fila da
/// navegação inteira por baixo). A fila sobe por cima dele e o «Live» abre o painel pelo clique da
/// shell; na janela larga, a fila fica onde estava.
#[test]
fn the_corner_rises_above_the_stats_hud_in_a_narrow_window() {
    let live = button_node(NavButton::Live);
    let hud = |t: &mut T| {
        let vp = Rect::new(0.0, 0.0, t.vp_w, t.vp_h);
        let layout = crate::screens::hero::frame_layout::frame_layout(&t.hero, vp, &mut t.ts);
        crate::screens::hero::bottom_hud::bottom_hud_rect(&layout, &mut t.ts, t.hero.stats)
    };
    let rows = |t: &T| {
        crate::screens::hero::board_nav::button_rects(crate::screens::hero::board_nav::corner(
            &t.hero,
            t.hero.last_canvas,
        ))
    };
    let mut t = T::new();
    let wide = rows(&t);
    assert_eq!(
        wide,
        crate::screens::hero::board_nav::button_rects(t.hero.last_canvas),
        "na janela larga a fila mudou de sítio"
    );
    t.vp_w = 640.0; // LITERAL-PX-OK: uma janela mais estreita que o HUD (≈1 000 px de texto)
    t.paint();
    t.paint();
    let h = hud(&mut t);
    let under = |r: &Rect| r.x < h.x + h.w && h.x < r.x + r.w && r.y + r.h > h.y;
    assert!(
        crate::screens::hero::board_nav::button_rects(t.hero.last_canvas)
            .iter()
            .any(|(_, r)| under(r)),
        "o controlo: sem a cura, a fila cai debaixo do HUD"
    );
    for (b, r) in rows(&t) {
        assert!(!under(&r), "{b:?} {r:?} está debaixo do HUD {h:?}");
    }
    t.open_live();
    assert!(live_board::is_open(&t.hero), "o «Live» não abriu");
    let (_, y) = t.find(live).expect("pintado");
    assert!(y < h.y, "o «Live» pinta-se em {y}, o HUD começa em {}", h.y);
}

/// Os glifos pintados nesta imagem (todos os textos).
fn glyphs(t: &mut T) -> usize {
    runs(&t.scene()).iter().map(|r| r.1).sum()
}

/// `text` está nas filas do painel E foi pintado: a imagem com ele tem pelo menos os glifos dele a
/// mais que `before` (a mesma imagem sem ele).
fn painted(t: &mut T, before: usize, key: &str) {
    let text = ph2d_i18n::tr(key);
    assert!(
        live_board::row_texts(&t.hero).iter().any(|r| r == text),
        "«{key}» não está no painel: {:?}",
        live_board::row_texts(&t.hero)
    );
    let ink = text.chars().filter(|c| !c.is_whitespace()).count();
    let now = glyphs(t);
    assert!(
        now >= before + ink,
        "«{key}» não se pintou: {before} -> {now}"
    );
}

/// ⭐ ADR-0187 — a rede que pode bloquear o convite diz-se em palavras simples: o anfitrião clica
/// «Invite over the internet» (o clique real), o link fica pronto numa rede de NAT simétrico e o painel
/// avisa; sem a medição, nada.
#[test]
fn a_strict_network_is_said_to_the_host_in_plain_words() {
    use crate::documents::live::LiveInternet;
    let mut t = T::new();
    t.open_live();
    t.room(LiveRole::Hosting {
        code: "ABCDEF".into(),
        addrs: vec!["10.0.0.5:47320".into()],
    });
    t.asked();
    t.live_click(Hit::Button(LiveButton::InviteInternet));
    assert_eq!(t.asked(), vec![LiveRequest::InviteInternet]);
    t.hero.documents.live_view_mut().internet = LiveInternet::Ready {
        link: "ph2d-board://v1/#x".into(),
    };
    t.paint();
    let before = glyphs(&mut t);
    let warn = ph2d_i18n::tr("board.live.nat_strict_host");
    assert!(!live_board::row_texts(&t.hero).iter().any(|r| r == warn));
    t.hero.documents.live_view_mut().link.strict_nat = true;
    t.paint();
    painted(&mut t, before, "board.live.nat_strict_host");
    // O botão continua lá e clicável por baixo do aviso.
    t.live_click(Hit::Button(LiveButton::InviteInternet));
    assert_eq!(t.asked(), vec![LiveRequest::InviteInternet]);
}

/// ⭐ ADR-0187 — quem entra por link (o clique real em «Join by link») vê que se PROCURA o quadro na
/// internet e até quando — não o «Connecting…» da rede local —, com o aviso da rede dele se for o
/// caso; ao falhar, a frase do que fazer fica no painel por cima de «Join by link».
#[test]
fn joining_by_link_says_it_searches_the_internet_and_what_to_do_when_it_fails() {
    const CODE: NodeId = crate::ids::BOARD_LIVE_CODE_INPUT;
    let mut t = T::new();
    t.open_live();
    t.shell_click_id(CODE, "o campo do código");
    assert!(crate::interaction::apply_clipboard_paste(
        &mut t.hero.store,
        CODE,
        "ph2d-board://v1/127.0.0.1:5000#AbCdEfGhIjKlMnOpQrStUv"
    ));
    t.asked();
    t.live_click(Hit::Button(LiveButton::JoinLink));
    assert!(matches!(&t.asked()[..], [LiveRequest::JoinLink { .. }]));
    // A família responde como em `join_by` (link) e na medição do STUN.
    let lv = t.hero.documents.live_view_mut();
    lv.role = LiveRole::Joining;
    lv.link.searching = Some(30);
    t.paint();
    let texts = live_board::row_texts(&t.hero);
    assert!(
        !texts
            .iter()
            .any(|r| r == ph2d_i18n::tr("board.live.connecting")),
        "por link disse o «Connecting…» da rede local"
    );
    let hint = ph2d_i18n::tr_with("board.live.link_searching_hint", &[("s", &30)]);
    assert!(texts.contains(&hint), "{texts:?}");
    let before = glyphs(&mut t);
    t.hero.documents.live_view_mut().link.strict_nat = true;
    t.paint();
    painted(&mut t, before, "board.live.nat_strict_guest");
    t.live_click(Hit::Button(LiveButton::Cancel));
    assert_eq!(t.asked(), vec![LiveRequest::Leave]);
    // Pela rede local, a espera é a de sempre.
    t.hero.documents.live_view_mut().link = Default::default();
    t.paint();
    assert!(
        live_board::row_texts(&t.hero)
            .iter()
            .any(|r| r == ph2d_i18n::tr("board.live.connecting"))
    );
    // Falhou: de volta ao `Off`, a frase fica no painel e «Join by link» clica-se outra vez.
    let lv = t.hero.documents.live_view_mut();
    lv.role = LiveRole::Off;
    t.paint();
    let before = glyphs(&mut t);
    t.hero.documents.live_view_mut().link.failed =
        Some(ph2d_i18n::tr("board.live.link_closed").to_owned());
    t.paint();
    painted(&mut t, before, "board.live.link_closed");
    t.live_click(Hit::Button(LiveButton::JoinLink));
    assert!(matches!(&t.asked()[..], [LiveRequest::JoinLink { .. }]));
}

/// ADR-0187 — o anfitrião vê «Invite over the internet» e ele pede a entrada pela internet; a
/// preparar, o botão sai e fica a frase; pronta, «Copy the link again» pede o mesmo.
#[test]
fn the_host_invites_over_the_internet_and_copies_again() {
    use crate::documents::live::LiveInternet;
    let mut t = T::new();
    t.open_live();
    t.room(LiveRole::Hosting {
        code: "ABCDEF".into(),
        addrs: vec!["10.0.0.5:47320".into()],
    });
    t.paint();
    t.asked();
    t.live_click(Hit::Button(LiveButton::InviteInternet));
    assert_eq!(t.asked(), vec![LiveRequest::InviteInternet]);
    t.hero.documents.live_view_mut().internet = LiveInternet::Preparing;
    t.paint();
    let id = live_node(Hit::Button(LiveButton::InviteInternet));
    assert!(t.find(id).is_none(), "a preparar, o botão não se clica");
    t.hero.documents.live_view_mut().internet = LiveInternet::Ready {
        link: "ph2d-board://v1/#x".into(),
    };
    t.paint();
    t.live_click(Hit::Button(LiveButton::InviteInternet));
    assert_eq!(t.asked(), vec![LiveRequest::InviteInternet]);
}

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
