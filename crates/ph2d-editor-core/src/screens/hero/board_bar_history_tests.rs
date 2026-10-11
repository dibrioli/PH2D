//! O histórico das votações (W13, ADR-0183) pelo caminho REAL do ecrã: o separador «Results», a lista
//! (a mais nova em cima, com página), abrir uma sessão, o toggle, renomear (duplo clique), o lixo (dois
//! cliques; qualquer outro desarma) — os cliques pelo despacho sobre o que o pintor registou.

use std::collections::BTreeMap;

use super::*;
use crate::interaction::WidgetEvent;
use crate::screens::hero::board_bar::fac::VOTE;
use crate::screens::hero::board_nav;
use crate::screens::hero::board_nav::facilitate::{FacButton, column, fac_node, poll_row_node};
use ph2d_board_model::{ElementId, PollId, Team};

impl T {
    /// Uma sessão acabada no quadro activo (`(elemento, votos)`).
    fn poll(&mut self, totals: &[(ElementId, u32)]) -> PollId {
        let b = self.hero.documents.active_board_mut().unwrap();
        let t: BTreeMap<ElementId, u32> = totals.iter().copied().collect();
        b.team.add_poll(t, "Ana", 1, 0)
    }

    fn polls(&self) -> Team {
        self.hero.documents.active_board().unwrap().team.clone()
    }

    fn poll_btn(&mut self, b: FacButton) {
        self.shell_click_id(fac_node(b), &format!("{b:?}"));
    }
}

/// ⭐ «Voting» abre o separador «Results» quando há sessões: a mais nova em cima; abrir a de baixo
/// passa o quadro a mostrá-la (o toggle está ligado), e a sessão aberta oferece voltar e «New vote».
#[test]
fn the_results_tab_lists_every_vote_newest_first_and_opens_one() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "a", 1);
    let b = t.put_note([-100.0, -100.0], "b", 2);
    let old = t.poll(&[(a, 3)]);
    let new = t.poll(&[(b, 1)]);
    t.shell_click_bar(VOTE);
    let (_, y0) = t.find(poll_row_node(0)).expect("a 1.ª fila");
    let (_, y1) = t.find(poll_row_node(1)).expect("a 2.ª fila");
    assert!(y0 < y1);
    assert!(
        t.find(poll_row_node(2)).is_none(),
        "duas sessões, duas filas"
    );
    assert!(t.find(fac_node(FacButton::VoteTabResults)).is_some());
    assert_eq!(t.polls().shown, Some(new));
    t.shell_click_id(poll_row_node(1), "a de baixo");
    assert_eq!(t.polls().shown, Some(old), "a sessão aberta é a mostrada");
    assert_eq!(t.shown(a), 3);
    for b in [
        FacButton::PollBack,
        FacButton::PollDelete,
        FacButton::VoteNew,
    ] {
        assert!(t.find(fac_node(b)).is_some(), "{b:?}");
    }
    t.poll_btn(FacButton::PollBack);
    assert!(t.find(poll_row_node(0)).is_some(), "de volta à lista");
    t.poll_btn(FacButton::VoteTabNew);
    assert!(
        t.find(fac_node(FacButton::VoteStart)).is_some(),
        "a abertura"
    );
}

/// O toggle «Show results on the board» esconde os selos e volta a mostrá-los — os da sessão aberta,
/// senão os da mais nova.
#[test]
fn the_switch_hides_and_brings_back_the_badges() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "a", 1);
    let one = t.poll(&[(a, 2)]);
    t.poll(&[(a, 5)]);
    t.shell_click_bar(VOTE);
    t.poll_btn(FacButton::ShowOnBoard);
    assert_eq!((t.polls().shown, t.shown(a)), (None, 0));
    t.poll_btn(FacButton::ShowOnBoard);
    assert_eq!(t.shown(a), 5, "a mais nova");
    t.shell_click_id(poll_row_node(1), "a antiga");
    t.poll_btn(FacButton::ShowOnBoard);
    t.poll_btn(FacButton::ShowOnBoard);
    assert_eq!(t.polls().shown, Some(one), "a aberta");
}

/// ⭐ Renomear é o DUPLO clique no título (o idioma das versões); o clique simples não abre o campo;
/// Enter grava, Esc larga.
#[test]
fn a_double_click_renames_and_a_single_click_does_not() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "a", 1);
    let p = t.poll(&[(a, 1)]);
    t.shell_click_bar(VOTE);
    t.shell_click_id(poll_row_node(0), "a sessão");
    t.poll_btn(FacButton::PollRename);
    assert!(
        t.find(crate::ids::BOARD_POLL_NAME_INPUT).is_none(),
        "um clique não renomeia"
    );
    t.hero
        .apply_event(WidgetEvent::DoubleClick(fac_node(FacButton::PollRename)));
    t.paint();
    assert!(
        t.find(crate::ids::BOARD_POLL_NAME_INPUT).is_some(),
        "o campo no título"
    );
    let field = board_nav::field_text(&t.hero.store, crate::ids::BOARD_POLL_NAME_INPUT);
    assert_eq!(
        field,
        ph2d_i18n::tr_with("board.vote.session_default", &[("n", &1)])
    );
    t.write_and_enter("Retro");
    assert_eq!(t.polls().poll(p).unwrap().name, "Retro");
    assert!(
        t.find(crate::ids::BOARD_POLL_NAME_INPUT).is_none(),
        "gravado, o campo sai"
    );
    t.hero
        .apply_event(WidgetEvent::DoubleClick(fac_node(FacButton::PollRename)));
    t.hero
        .apply_event(WidgetEvent::Cancel(crate::ids::BOARD_POLL_NAME_INPUT));
    t.paint();
    assert_eq!(t.polls().poll(p).unwrap().name, "Retro", "Esc não grava");
}

/// ⭐ O lixo pede o 2.º clique NO MESMO; qualquer outro clique desarma. Apagar a mostrada desliga o
/// selo e volta à lista.
#[test]
fn the_bin_needs_two_clicks_and_any_other_click_disarms_it() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "a", 1);
    t.poll(&[(a, 1)]);
    let p = t.poll(&[(a, 4)]);
    t.shell_click_bar(VOTE);
    t.shell_click_id(poll_row_node(0), "a nova");
    t.poll_btn(FacButton::PollDelete);
    t.poll_btn(FacButton::ShowOnBoard);
    t.poll_btn(FacButton::PollDelete);
    assert!(t.polls().poll(p).is_some(), "o clique de permeio desarmou");
    t.poll_btn(FacButton::PollDelete);
    assert!(t.polls().poll(p).is_none());
    assert_eq!((t.polls().polls.len(), t.polls().shown), (1, None));
    assert!(t.find(poll_row_node(0)).is_some(), "de volta à lista");
}

/// A lista mostra as filas que CABEM e as setas passam a página (nenhuma fila sai da área); a seta que
/// não mexeria não se pinta.
#[test]
fn the_list_pages_when_it_does_not_fit() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "a", 1);
    let ids: Vec<PollId> = (1..=40).map(|n| t.poll(&[(a, n)])).collect();
    t.shell_click_bar(VOTE);
    let c = t.hero.last_canvas;
    let fit = (0..40)
        .take_while(|k| t.find(poll_row_node(*k)).is_some())
        .count();
    assert!((3..40).contains(&fit), "{fit} filas");
    let (_, y) = t.find(poll_row_node(fit - 1)).unwrap();
    assert!(y < c.y + c.h, "a última fila fora da área");
    // E acima dos botões do canto (a foto a9 da cena 14: a lista passava por baixo do Live).
    let floor = board_nav::button_rects(c)
        .iter()
        .map(|(_, r)| r.y)
        .fold(f32::MAX, f32::min);
    assert!(
        y < floor,
        "a última fila ({y}) por baixo dos botões do canto ({floor})"
    );
    assert!(
        t.find(crate::widget::BOARD_FACILITATE_SCROLLBAR_ID)
            .is_none(),
        "a página cheia cabe: nada rola por uns píxeis da moldura"
    );
    assert!(t.find(fac_node(FacButton::PollUp)).is_none());
    t.poll_btn(FacButton::PollDown);
    assert!(t.find(fac_node(FacButton::PollUp)).is_some());
    t.shell_click_id(poll_row_node(0), "a 1.ª da 2.ª página");
    assert_eq!(
        t.polls().shown,
        Some(ids[39 - fit]),
        "a fila 0 da página seguinte"
    );
}

/// ⛔ Régua de largura: as frases novas do painel cabem na coluna (o controlo é a frase que a foto da
/// W12 mostrou cortada — sem fontes a régua mediria zero e passava sempre).
#[test]
fn every_w13_panel_line_fits_the_column() {
    use crate::screens::hero::board_nav::facilitate::btn;
    use ph2d_tokens::{Spacing, TypeToken};
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let w = column(
        crate::zones::Rect::new(0.0, 0.0, 2000.0, 1000.0),
        false,
        false,
    )
    .2;
    let (pad, g, s) = (Spacing::Xs.px(), ph2d_tokens::control_gap_px(), btn());
    let px = TypeToken::Sm.px();
    let text = |k: &str| ph2d_i18n::tr(k).to_owned();
    let n = [("n", &99 as &dyn std::fmt::Display), ("name", &"Bianca")];
    let half = (w - 2.0 * pad - g) / 2.0;
    let lines: Vec<(String, f32)> = vec![
        (text("board.vote.tab_new"), half),
        (text("board.vote.results"), half),
        (text("board.vote.show_on_board"), w - 2.0 * s),
        (text("board.vote.no_sessions"), w - 2.0 * pad),
        (text("board.vote.no_votes"), w - 2.0 * pad),
        (
            format!(
                "{} · {}",
                ph2d_i18n::tr_with("board.ago.day", &n),
                ph2d_i18n::tr_with("board.vote.by", &n)
            ),
            w - 2.0 * pad,
        ),
        (
            ph2d_i18n::tr_with("board.vote.session_default", &n),
            w - 2.0 * pad - 3.0 * s - 3.0 * g,
        ),
        (text("board.vote.delete_confirm"), w - 2.0 * pad),
        // A abertura: o rótulo vai até ao `−` do contador (o «Votes per per…» da foto a5).
        (
            text("board.vote.votes"),
            w - pad - 3.0 * s - 3.0 * g - Spacing::Sm.px(),
        ),
        (
            text("board.vote.minutes"),
            w - pad - 3.0 * s - 3.0 * g - Spacing::Sm.px(),
        ),
        (text("board.vote.new_short"), half),
        (text("board.vote.see_all"), half),
    ];
    let old = "Text on sticky notes is hidden from others";
    assert!(
        ts.prefix_width(old, px) > w - 2.0 * pad,
        "a régua mede zero"
    );
    let wide: Vec<_> = lines
        .iter()
        .filter(|(l, room)| ts.prefix_width(l, px) > *room)
        .collect();
    assert!(wide.is_empty(), "não cabem: {wide:?}");
}

/// ⭐ Auditoria W13 — sem sessões, o toggle não se pinta (não faria nada); apagar a última deixa a lista
/// vazia com «No votes yet».
#[test]
fn with_no_votes_there_is_no_switch() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "a", 1);
    t.poll(&[(a, 1)]);
    t.shell_click_bar(VOTE);
    t.shell_click_id(poll_row_node(0), "a única");
    t.poll_btn(FacButton::PollDelete);
    t.poll_btn(FacButton::PollDelete);
    assert!(t.polls().polls.is_empty());
    assert!(
        t.find(fac_node(FacButton::ShowOnBoard)).is_none(),
        "um toggle morto"
    );
    assert!(t.find(fac_node(FacButton::VoteTabNew)).is_some());
}

/// ⭐ Auditoria W13 — a fila abre a sessão que a PINTURA mostrou nela: uma sessão que chega (da rede)
/// entre a pintura e o clique não a desloca.
#[test]
fn a_row_opens_what_was_painted_even_if_the_list_changes_before_the_click() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "a", 1);
    let old = t.poll(&[(a, 1)]);
    t.poll(&[(a, 2)]);
    t.shell_click_bar(VOTE);
    // Chega uma nova sem pintura no meio: a fila 1 continua a ser a «Voting 1».
    t.poll(&[(a, 3)]);
    t.hero.apply_event(WidgetEvent::Click(poll_row_node(1)));
    assert_eq!(t.polls().shown, Some(old));
}

/// ⭐ Auditoria W13 — o lixo armado e o renomear são do quadro: trocar de quadro desarma; e o separador
/// «New session» já aceso não repõe a abertura.
#[test]
fn switching_boards_disarms_the_bin_and_the_lit_tab_keeps_the_setup() {
    use crate::documents::fac::VotePanel;
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "a", 1);
    let p = t.poll(&[(a, 1)]);
    t.shell_click_bar(VOTE);
    t.shell_click_id(poll_row_node(0), "a sessão");
    t.poll_btn(FacButton::PollDelete);
    let here = t.hero.documents.active().unwrap();
    let other = t.hero.documents.create("B".into());
    t.hero.documents.activate(Some(other));
    t.paint();
    t.hero.documents.activate(Some(here));
    t.paint();
    t.poll_btn(FacButton::PollDelete);
    assert!(
        t.polls().poll(p).is_some(),
        "um clique só, depois de voltar: armou de novo"
    );
    t.poll_btn(FacButton::PollBack);
    t.poll_btn(FacButton::VoteTabNew);
    t.poll_btn(FacButton::VotesPlus);
    t.poll_btn(FacButton::VoteTabNew);
    let panel = t.hero.documents.live.fac.panels.get(&here).copied();
    assert!(
        matches!(panel, Some(VotePanel::Setup(s)) if s.votes == ph2d_board_model::VOTES_DEFAULT + 1),
        "{panel:?}"
    );
}

/// ⭐ Auditoria W13 — quem acaba uma votação de um ecrã com «Make names anonymous» ligado não fica com
/// o nome no histórico. ⛔ Controlo: sem a opção, fica.
#[test]
fn a_single_screen_vote_under_anonymous_names_keeps_no_name() {
    use crate::documents::live::LivePrivate;
    for anonymous in [true, false] {
        let mut t = T::new();
        t.hero.documents.set_author("Ana".into());
        let a = t.put_note([-400.0, -100.0], "a", 1);
        t.hero.documents.live_view_mut().fac.private = Some(LivePrivate {
            by: String::new(),
            by_me: false,
            anonymous,
            anyone_can_end: false,
            anyone_can_reveal: false,
            may_end: false,
            reveal_in_ms: None,
        });
        t.shell_click_bar(VOTE);
        t.poll_btn(FacButton::VoteStart);
        let e = t.elem(a);
        t.click_world([e.x + e.w, e.y + e.h]);
        t.poll_btn(FacButton::VoteEnd);
        let by = t.polls().polls[0].by.clone();
        assert_eq!(by.is_empty(), anonymous, "anonymous={anonymous}: «{by}»");
    }
}

impl T {
    /// Os ids da coluna da facilitação que respondem ao clique numa faixa larga à volta dela (o
    /// salto dela: os botões, as filas e o fundo), e a banda que a coluna ocupou.
    fn column_ids(&self) -> (std::collections::BTreeSet<u64>, crate::zones::Rect) {
        let salt = fac_node(FacButton::Surface).0 >> 40;
        let band = self
            .hero
            .documents
            .live
            .fac
            .column
            .expect("a coluna pintou")
            .1;
        let mut ids = std::collections::BTreeSet::new();
        let c = self.hero.last_canvas;
        let mut y = c.y;
        while y < c.y + c.h {
            let mut x = band.x - band.w;
            while x < band.x + 2.0 * band.w {
                if let Some(id) = self.hero.hit_index.hit(x, y).filter(|i| i.0 >> 40 == salt) {
                    ids.insert(id.0);
                }
                x += 2.0;
            }
            y += 2.0;
        }
        (ids, band)
    }

    /// O cronómetro e o resultado de uma votação: a coluna mais alta que a cena pede (medido: numa
    /// área de `337` px ela já passava do chão).
    fn tall_column(&mut self) {
        let a = self.put_note([-400.0, -100.0], "a", 1);
        let b = self.put_note([-100.0, -100.0], "b", 2);
        let c = self.put_note([200.0, -100.0], "c", 3);
        for n in 1..=40 {
            self.poll(&[(a, n), (b, 1), (c, 2)]);
        }
        self.shell_click_bar(crate::screens::hero::board_bar::fac::TIMER);
        self.shell_click_bar(VOTE);
        self.shell_click_id(poll_row_node(0), "a 1.ª");
    }

    fn zoom(&self) -> f64 {
        self.hero.documents.active_board().unwrap().camera.zoom
    }
}

/// ⭐ Numa janela BAIXA a coluna da facilitação nunca passa do chão (os botões do canto): o que não
/// cabe ROLA — a barra aparece à direita dos cartões, a roda sobre a coluna rola-a (nunca dá zoom no
/// quadro), e rolando até ao fim alcança-se todo controlo que a janela alta mostra.
#[test]
fn a_low_window_scrolls_the_column_and_reaches_every_control() {
    let mut t = T::new();
    t.tall_column();
    let (tall, _) = t.column_ids();
    t.vp_h = 300.0; // LITERAL-PX-OK: uma janela baixa (a área fica abaixo dos 337 px medidos)
    t.paint();
    let c = t.hero.last_canvas;
    let floor = board_nav::button_rects(c)
        .iter()
        .map(|(_, r)| r.y)
        .fold(c.y + c.h, f32::min);
    let (mut seen, band) = t.column_ids();
    assert!(
        band.y + band.h <= floor,
        "a banda ({band:?}) passa do chão {floor}"
    );
    let bar = crate::widget::BOARD_FACILITATE_SCROLLBAR_ID;
    let (bx, _) = t.find(bar).expect("a barra");
    let (cx, _, cw) = column(c, false, false);
    assert!(
        bx > band.x + cw,
        "a barra à direita dos cartões, nunca por cima"
    );
    assert!(
        band.x + band.w <= cx + cw,
        "a barra entra na margem da área (ou no painel de Frames)"
    );
    assert!(
        !tall.is_subset(&seen),
        "nada ficou escondido: a janela não está baixa o bastante"
    );
    let zoom = t.zoom();
    let mid = (band.x + band.w / 2.0, band.y + band.h / 2.0);
    for _ in 0..40 {
        assert!(board_view::wheel(&mut t.hero, mid.0, mid.1, -30.0));
        t.paint();
        let (ids, _) = t.column_ids();
        for id in &ids {
            let (x, y) = t.find(NodeId(*id)).unwrap();
            assert!(y < floor && x < band.x + band.w, "{id:#x} fora da banda");
        }
        seen.extend(ids);
    }
    assert_eq!(t.zoom(), zoom, "a roda sobre a coluna deu zoom no quadro");
    let lost: Vec<_> = tall.difference(&seen).collect();
    assert!(lost.is_empty(), "inalcançáveis: {lost:x?}");
}

/// O controlo: numa janela alta a coluna fica onde estava (sem barra); a roda sobre ela não dá zoom,
/// e fora dela continua a dar.
#[test]
fn a_tall_window_keeps_the_column_and_the_wheel_off_it_still_zooms() {
    let mut t = T::new();
    t.tall_column();
    let c = t.hero.last_canvas;
    let (_, band) = t.column_ids();
    assert_eq!(band.x, column(c, false, false).0);
    assert!(
        t.find(crate::widget::BOARD_FACILITATE_SCROLLBAR_ID)
            .is_none()
    );
    let zoom = t.zoom();
    assert!(board_view::wheel(
        &mut t.hero,
        band.x + 4.0,
        band.y + 4.0,
        -30.0
    ));
    assert_eq!(t.zoom(), zoom);
    let (x, y) = t.at(0.3, 0.6);
    assert!(board_view::wheel(&mut t.hero, x, y, -30.0));
    assert_ne!(t.zoom(), zoom, "o controlo: fora da coluna a roda dá zoom");
}

/// Os painéis da equipa põem-se à esquerda do que a coluna OCUPA — com a barra, quando ela rola —
/// e nada se pinta por cima dela.
#[test]
fn a_team_panel_never_covers_a_scrolling_column() {
    let mut t = T::new();
    t.tall_column();
    t.vp_h = 300.0; // LITERAL-PX-OK: a janela baixa do teste acima
    t.hero.documents.live.team.list_open = true;
    t.paint();
    let salt = fac_node(FacButton::Surface).0 >> 40;
    let (_, band) = t.column_ids();
    let mut y = band.y;
    while y < band.y + band.h {
        let mut x = band.x;
        while x < band.x + band.w {
            if let Some(id) = t
                .hero
                .hit_index
                .hit(x, y)
                .filter(|i| *i != crate::widget::BOARD_FACILITATE_SCROLLBAR_ID)
            {
                assert_eq!(id.0 >> 40, salt, "({x}, {y}) é {id:?}, não da coluna");
            }
            x += 2.0;
        }
        y += 2.0;
    }
}

/// A lista do histórico pagina até UMA fila antes de a coluna rolar: na janela baixa, com o
/// cronómetro, ela cabe sem barra (paginar e rolar ao mesmo tempo era o que o mínimo de três filas
/// fazia).
#[test]
fn the_list_pages_down_to_one_row_before_the_column_scrolls() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "a", 1);
    for n in 1..=40 {
        t.poll(&[(a, n)]);
    }
    t.shell_click_bar(crate::screens::hero::board_bar::fac::TIMER);
    t.shell_click_bar(VOTE);
    // Medido: a 360 a lista de UMA fila e o cronómetro cabem (233 de 251 px); a de três filas, não.
    t.vp_h = 360.0; // LITERAL-PX-OK: a janela que separa «pagina até uma» de «mínimo de três»
    t.paint();
    assert!(t.find(poll_row_node(0)).is_some());
    assert!(
        t.find(crate::widget::BOARD_FACILITATE_SCROLLBAR_ID)
            .is_none(),
        "a coluna rola com a lista a caber numa página"
    );
    assert!(t.find(fac_node(FacButton::PollDown)).is_some(), "e pagina");
    // ⛔ A página cheia passa do chão pela moldura do cartão (uns 4 px) só numa faixa de alturas:
    // varrem-se TODAS as de dois passos de fila (uma grelha folgada nunca cai nela).
    let cw = column(t.hero.last_canvas, false, false).2;
    for h in 360..=420 {
        t.vp_h = h as f32;
        t.paint();
        let band = t.hero.documents.live.fac.column.unwrap().1;
        assert_eq!(band.w, cw, "a {h} px a lista rola em vez de paginar");
    }
}
