//! W12 (ADR-0182) — a facilitação AO VIVO pelo caminho REAL do ecrã, com a sala SIMULADA (a família e a
//! rede são outra crate; o gate de duas janelas delas é `smoke_fac_live_tests`): o que a tela PEDE (os
//! `LiveRequest`) e o que faz com o que a sala diz (`live_view.fac`).

use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::documents::live::{LiveFac, LivePrivate, LiveRequest, LiveRole, LiveVote};
use crate::screens::hero::board_bar::fac::PRIVATE;
use crate::screens::hero::board_keys::BoardKey;
use ph2d_board_edit::VoteSetup;
use ph2d_host::Modifiers;

impl T {
    /// A sala aberta neste quadro (como a família a escreveria).
    fn go_live(&mut self) {
        let id = self.board_id();
        let v = self.hero.documents.live_view_mut();
        v.role = LiveRole::Hosting {
            code: "ABC-DEF".into(),
            addrs: Vec::new(),
        };
        v.board = Some(id);
        self.paint();
    }

    /// O que a sala diz agora.
    fn says(&mut self, f: impl FnOnce(&mut LiveFac)) {
        f(&mut self.hero.documents.live_view_mut().fac);
        self.paint();
    }

    /// O que a tela pediu à sala desde a última vez.
    fn fac_asked(&mut self) -> Vec<LiveRequest> {
        self.hero.documents.take_live()
    }

    fn painted(&self, b: FacButton) -> bool {
        self.find(fac_node(b)).is_some()
    }
}

fn open_vote(targets: BTreeSet<ElementId>) -> LiveVote {
    LiveVote {
        round: 1,
        by: "Ana".into(),
        may_end: true,
        setup: VoteSetup::default(),
        targets,
        left_ms: Some(60_000),
        done: 0,
        people: 2,
        used: Vec::new(),
        ended: false,
    }
}

fn ballot(votes: &[(ElementId, u32)], done: bool) -> LiveRequest {
    LiveRequest::Ballot {
        round: 1,
        votes: votes.iter().copied().collect::<BTreeMap<_, _>>(),
        done,
    }
}

/// ⭐ Ao vivo, «Start voting» PEDE à sala (não abre nada sozinho); a sessão chega e o painel segue-a;
/// cada `+` manda o boletim inteiro; «I'm done» marca-o (e diz que só avisa); «End for all» pede o
/// fim; os totais que a sala escreve mostram o resultado.
#[test]
fn a_live_vote_asks_the_room_and_each_plus_sends_my_ballot() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "Faster builds", 1);
    let b = t.put_note([-100.0, -100.0], "Pair more", 2);
    t.go_live();
    t.shell_click_bar(VOTE);
    t.fac(FacButton::VoteStart);
    let asked = t.fac_asked();
    assert!(
        matches!(&asked[..], [LiveRequest::VoteStart { targets, .. }] if targets.contains(&a) && targets.contains(&b)),
        "{asked:?}"
    );
    assert!(!board_nav::facilitate::vote_in_session(&t.hero), "nada abre sem a sala");
    t.says(|f| f.vote = Some(open_vote(BTreeSet::from([a, b]))));
    assert!(board_nav::facilitate::vote_in_session(&t.hero));
    assert_eq!(t.fac_asked(), vec![ballot(&[], false)]);
    assert!(!t.painted(FacButton::VoteAdd1), "o relógio da sala não se estica daqui");
    let corner = |e: &ph2d_board_model::Element| [e.x + e.w, e.y + e.h];
    let pa = corner(&t.elem(a));
    t.click_world(pa);
    assert_eq!(t.fac_asked(), vec![ballot(&[(a, 1)], false)]);
    t.fac(FacButton::VoteDone);
    t.paint();
    assert_eq!(t.fac_asked(), vec![ballot(&[(a, 1)], true)]);
    assert_eq!(
        t.hero.store.tooltip_for(fac_node(FacButton::VoteDone)),
        Some(ph2d_i18n::tr("board.livevote.done_tip"))
    );
    assert!(!t.painted(FacButton::VoteDone), "depois de «I'm done» espera-se");
    t.fac(FacButton::VoteEnd);
    assert_eq!(t.fac_asked(), vec![LiveRequest::VoteEnd]);
    assert!(board_nav::facilitate::vote_in_session(&t.hero), "o fim é da sala");
    // A sala acaba: os totais chegam ao quadro e o estado diz «acabou».
    let mut el = t.elem(a);
    el.set_votes(3);
    let (board, _) = t.hero.documents.active_parts().unwrap();
    let _ = BoardOp::Put(el).apply(&mut board.doc);
    t.says(|f| f.vote.as_mut().unwrap().ended = true);
    assert!(!board_nav::facilitate::vote_in_session(&t.hero));
    assert!(t.painted(FacButton::VoteNew), "o painel mostra o resultado");
}

/// ⭐ Uma nota escondida de mim: o desenho não lhe pinta o texto, `Enter` não a abre para escrever
/// (o aviso diz porquê) e o `+` não aparece nela; a MINHA escondida leva o olho.
#[test]
fn a_note_hidden_from_me_hides_its_text_and_refuses_writing_and_voting() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "", 1);
    let b = t.put_note([-100.0, -100.0], "minha", 2);
    t.go_live();
    t.says(|f| {
        f.hidden_to_me = BTreeSet::from([a]);
        f.mine_hidden = BTreeSet::from([b]);
    });
    assert_eq!(t.hero.documents.live.render_cache.unread, BTreeSet::from([a]));
    assert_eq!(t.hero.documents.live.render_cache.mine_private, BTreeSet::from([b]));
    t.pick(&[a]);
    t.key(BoardKey::Enter, Modifiers::default(), None);
    assert_eq!(
        t.hero.documents.live.notice.map(|n| n.0),
        Some("board.notice.private")
    );
    let ed = t.hero.documents.live.editor.as_ref().unwrap();
    assert!(!ed.is_editing_text());
    t.says(|f| f.vote = Some(open_vote(BTreeSet::from([a, b]))));
    let _ = t.fac_asked();
    let pa = [t.elem(a).x + t.elem(a).w, t.elem(a).y + t.elem(a).h];
    t.click_world(pa);
    assert_eq!(t.vote_mine(a), 0, "votou numa nota que não lê");
    assert!(t.fac_asked().is_empty());
}

/// ⭐ O painel do modo privado: fora da sala ensina a lá chegar (nenhum botão que não faria nada); na
/// sala as opções e «Turn on…» pedem o modo; ligado, quem o ligou tem a opção e o desligar; a contagem
/// tira o botão; a quem não ligou e pode revelar as suas, só esse botão. Fechar o painel não desliga.
#[test]
fn the_private_panel_teaches_live_first_then_asks_the_room() {
    let mut t = T::new();
    t.shell_click_bar(PRIVATE);
    assert!(t.painted(FacButton::PrivateClose), "o painel abriu");
    assert!(!t.painted(FacButton::PrivateOn), "fora da sala não se liga nada");
    t.go_live();
    t.fac(FacButton::PrivateAnonymous);
    assert!(t.fac_asked().is_empty(), "uma opção não é um pedido");
    t.fac(FacButton::PrivateOn);
    assert_eq!(
        t.fac_asked(),
        vec![LiveRequest::PrivateOn {
            anonymous: true,
            anyone_can_end: false
        }]
    );
    let mine = LivePrivate {
        by: "Ana".into(),
        by_me: true,
        anonymous: true,
        anyone_can_end: false,
        anyone_can_reveal: false,
        may_end: true,
        reveal_in_ms: None,
    };
    t.says(|f| f.private = Some(mine.clone()));
    assert!(t.painted(FacButton::PrivateAnyoneReveal) && t.painted(FacButton::PrivateOff));
    assert!(!t.painted(FacButton::PrivateRevealMine));
    t.fac(FacButton::PrivateAnyoneReveal);
    t.fac(FacButton::PrivateOff);
    assert_eq!(
        t.fac_asked(),
        vec![LiveRequest::AnyoneCanReveal(true), LiveRequest::PrivateOff]
    );
    t.says(|f| f.private.as_mut().unwrap().reveal_in_ms = Some(3_000));
    assert!(!t.painted(FacButton::PrivateOff), "a contagem tira o botão");
    let x = t.put_note([200.0, -100.0], "x", 3);
    t.says(|f| {
        f.private = Some(LivePrivate {
            by_me: false,
            may_end: false,
            anyone_can_reveal: true,
            ..mine
        });
        f.mine_hidden = BTreeSet::from([x]);
    });
    assert!(t.painted(FacButton::PrivateRevealMine));
    assert!(!t.painted(FacButton::PrivateOff) && !t.painted(FacButton::PrivateAnyoneReveal));
    t.fac(FacButton::PrivateRevealMine);
    assert_eq!(t.fac_asked(), vec![LiveRequest::RevealMine]);
    t.shell_click_bar(PRIVATE);
    assert!(!t.painted(FacButton::PrivateClose));
    assert!(t.fac_asked().is_empty(), "fechar o painel não desliga o modo");
}
