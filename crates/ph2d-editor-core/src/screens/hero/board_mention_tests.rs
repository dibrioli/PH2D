//! ⭐ W23 B14 — o `@` do compositor e as notificações pelo caminho REAL do ecrã (doc 17 §2.3): a lista
//! de quem se chama, ↑/↓ e `Enter`, a menção publicada como DADO; a mensagem para mim que chega dá o
//! aviso com «Show»; o que já lá estava não avisa mas conta na bolinha; «For you» filtra a lista.

use super::*;
use crate::documents::team::OpenThread;
use crate::interaction::{KEY_ARROW_DOWN, KEY_ARROW_UP, KEY_ENTER};
use crate::screens::hero::board_nav::team::notify;
use ph2d_board_model::Who;

impl T {
    fn type_in(&mut self, s: &str) {
        for ch in s.chars() {
            let arena = Bump::new();
            let ev = self.hero.handle_text_input(ch, &arena).to_vec();
            for e in ev {
                self.hero.apply_event(e);
            }
        }
        self.paint();
    }

    fn field(&self) -> String {
        board_nav::field_text_for_tests(&self.hero, crate::ids::BOARD_COMMENT_INPUT)
    }

    /// Dois fios de antes (do Rui e da Bia), a Ana a escrever, e o compositor aberto num fio novo.
    fn composing() -> Self {
        let mut t = T::new();
        t.hero.documents.set_author("Ana".into());
        let (board, _) = t.hero.documents.active_parts().unwrap();
        board.team.open(Pin::Point([900.0, 900.0]), "Rui", "hello", 1);
        board.team.open(Pin::Point([950.0, 950.0]), "Bia", "hi", 2);
        t.paint();
        t.note_at([-100.0, -100.0], "idea");
        t.shell_click_bar(Item::Fac(FacItem::Comment));
        let p = t.screen([0.0, 0.0]);
        t.click_board(p);
        t
    }

    fn row(&self, n: usize) -> bool {
        self.find(team_node(Hit::Mention(n))).is_some()
    }
}

#[test]
fn typing_at_lists_people_and_enter_calls_the_one_picked() {
    let mut t = T::composing();
    t.type_in("look @");
    assert!(t.row(0) && t.row(1) && t.row(2), "Bia, Rui e everyone");
    assert!(!t.row(3));
    t.type_in("r");
    assert!(t.row(0) && !t.row(1), "o R estreita a lista ao Rui");
    t.hero_key(KEY_ENTER);
    assert_eq!(t.field(), "look @Rui ", "o Enter com a lista aberta escolhe, não publica");
    assert!(t.team().threads.len() == 2);
    t.type_in("now");
    t.hero_key(KEY_ENTER);
    let th = t.team().threads.last().unwrap();
    assert_eq!(th.messages[0].text, "look @Rui now");
    assert_eq!(th.messages[0].mentions, vec![Who::Person("Rui".into())]);
}

#[test]
fn the_arrows_walk_the_list_and_a_click_picks() {
    let mut t = T::composing();
    t.type_in("@");
    t.hero_key(KEY_ARROW_DOWN);
    t.hero_key(KEY_ENTER);
    assert_eq!(t.field(), "@Rui ", "↓ desceu da Bia ao Rui");
    t.type_in("and @");
    t.hero_key(KEY_ARROW_UP);
    t.hero_key(KEY_ENTER);
    assert_eq!(t.field(), "@Rui and @everyone ", "↑ do 1.º dá a volta ao último");
    t.type_in("hey @b");
    t.shell_click_id(team_node(Hit::Mention(0)), "a Bia na lista");
    assert_eq!(t.field(), "@Rui and @everyone hey @Bia ");
    t.hero_key(KEY_ENTER);
    let m = &t.team().threads.last().unwrap().messages[0];
    assert_eq!(m.mentions.len(), 3);
    // Escrever o nome à mão não chama ninguém (o Miro): só quem se escolheu.
    assert!(!m.mentions.contains(&Who::Person("Ana".into())));
}

/// ⭐ Uma menção que CHEGA (de um colega ao vivo) dá o aviso com «Show»; «Show» abre o fio (lido) e a
/// bolinha some.
#[test]
fn a_mention_that_arrives_alerts_and_show_opens_the_thread() {
    let mut t = T::new();
    t.hero.documents.set_author("Ana".into());
    t.paint();
    let board = t.hero.documents.active().unwrap();
    let (b, _) = t.hero.documents.active_parts().unwrap();
    let th = b
        .team
        .open_calling(Pin::Point([10.0, 10.0]), "Rui", "@Ana look", &[Who::Person("Ana".into())], 5)
        .unwrap();
    t.paint();
    let alert = t.hero.documents.live.alert.as_ref().expect("o aviso");
    assert!(alert.text.starts_with("Rui mentioned you"), "{}", alert.text);
    assert!(notify::board_has_news(&t.hero.documents, board), "a bolinha");
    t.shell_click_id(crate::ids::BOARD_ALERT_ACTION, "Show");
    assert_eq!(
        t.hero.documents.live.team.open.get(&board),
        Some(&OpenThread::Thread(th))
    );
    assert!(!notify::board_has_news(&t.hero.documents, board), "lido: a bolinha some");
}

/// Uma resposta num fio onde escrevi também é para mim (o Feed do Miro; dono 20/10).
#[test]
fn a_reply_in_my_thread_alerts_too() {
    let mut t = T::new();
    t.hero.documents.set_author("Ana".into());
    let (b, _) = t.hero.documents.active_parts().unwrap();
    let th = b.team.open(Pin::Point([10.0, 10.0]), "Ana", "plan?", 1).unwrap();
    t.paint();
    let (b, _) = t.hero.documents.active_parts().unwrap();
    assert!(b.team.reply(th, "Rui", "yes", 2));
    t.paint();
    let alert = t.hero.documents.live.alert.as_ref().expect("o aviso");
    assert!(alert.text.starts_with("Rui replied"), "{}", alert.text);
}

/// O que já estava num quadro quando ele foi visto nesta sessão (o ficheiro que se abre) não avisa —
/// conta na bolinha da aba.
#[test]
fn what_was_there_before_does_not_alert_but_the_tab_dot_counts_it() {
    let mut t = T::new();
    t.hero.documents.set_author("Ana".into());
    let first = t.hero.documents.active().unwrap();
    let plan = t.hero.documents.create("Plan".into());
    let b = t.hero.documents.board_mut(plan).unwrap();
    b.team
        .open_calling(Pin::Point([0.0; 2]), "Rui", "@Ana", &[Who::Person("Ana".into())], 1)
        .unwrap();
    t.hero.documents.activate(Some(first));
    t.paint();
    assert!(t.hero.documents.live.alert.is_none(), "o de antes não avisa");
    assert!(notify::board_has_news(&t.hero.documents, plan));
    assert!(!notify::board_has_news(&t.hero.documents, first));
}

#[test]
fn for_you_keeps_only_my_threads_in_the_list() {
    let mut t = T::new();
    t.hero.documents.set_author("Ana".into());
    let (b, _) = t.hero.documents.active_parts().unwrap();
    let mine = b
        .team
        .open_calling(Pin::Point([0.0; 2]), "Rui", "@Ana", &[Who::Person("Ana".into())], 1)
        .unwrap();
    let other = b.team.open(Pin::Point([50.0; 2]), "Rui", "lunch", 2).unwrap();
    t.paint();
    t.shell_click_id(button_node(NavButton::Comments), "Comments");
    assert!(t.find(team_node(Hit::Row(other))).is_some());
    t.btn(TeamButton::ListForYou);
    assert!(t.find(team_node(Hit::Row(mine))).is_some());
    assert!(t.find(team_node(Hit::Row(other))).is_none(), "o almoço não é para a Ana");
}
