//! ⭐ W23 B14 — **as notificações de um programa sem servidor** (doc 17 §2.3): uma mensagem PARA mim
//! ([`ph2d_board_model::Thread::for_me_at`]: chama-me, ou responde num fio onde escrevi) que CHEGA
//! nesta sessão (de um colega ao vivo) dá o aviso do quadro com «Show»; os fios para mim por ler dão
//! a bolinha no botão «Comments» e na aba do quadro — e é a bolinha que conta a quem abre o ficheiro
//! depois (o «novo» por pessoa, K6). O que já lá estava quando o quadro foi visto não avisa.

use super::super::super::HeroScreen;
use super::super::super::board_alert::{Action, Alert};
use crate::documents::Documents;
use crate::paint::resolve;
use crate::zones::Rect;
use ph2d_board_model::{Board, BoardId};
use ph2d_i18n::tr_with;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Quantos caracteres da mensagem o aviso mostra.
const SNIPPET: usize = 60;

/// Quem eu sou nos comentários (o nome de quem escreve; vazio = «Me»).
#[must_use]
pub fn me_of(docs: &Documents) -> String {
    let a = docs.team_session().author.trim();
    if a.is_empty() {
        ph2d_i18n::tr("board.comment.me").to_owned()
    } else {
        a.to_owned()
    }
}

/// ⭐ Os fios do quadro com mensagens PARA `me` que eu ainda não li (e não resolvidos).
#[must_use]
pub fn unread_for_me(board: &Board, me: &str) -> usize {
    board
        .team
        .threads
        .iter()
        .filter(|t| !t.resolved && t.unread_for(me) && t.for_me(me))
        .count()
}

/// A cada quadro: as mensagens novas desde a última vez; a mais nova PARA mim e por ler vira aviso.
pub(crate) fn tick(hero: &mut HeroScreen) {
    let me = me_of(&hero.documents);
    let mut alert = None;
    let (boards, team) = hero.documents.boards_and_team();
    for b in boards.boards() {
        let seeding = team.seeded.insert(b.id);
        for th in &b.team.threads {
            for (i, m) in th.messages.iter().enumerate() {
                if !team.notified.insert(m.id) || seeding {
                    continue;
                }
                let looking = matches!(
                    team.open.get(&b.id),
                    Some(crate::documents::team::OpenThread::Thread(t)) if *t == th.id
                );
                if th.for_me_at(i, &me) && th.unread_for(&me) && !looking {
                    alert = Some((b.id, th.id, m.author.clone(), m.calls(&me), m.text.clone()));
                }
            }
        }
    }
    let Some((board, thread, who, called, text)) = alert else {
        return;
    };
    let snippet: String = text.chars().take(SNIPPET).collect();
    let snippet = if text.chars().count() > SNIPPET {
        format!("{snippet}\u{2026}")
    } else {
        snippet
    };
    let key = if called {
        "board.comment.mentioned_you"
    } else {
        "board.comment.replied"
    };
    let line = tr_with(key, &[("name", &who), ("text", &snippet)]);
    hero.documents.live.alert = Some(Alert::new(line, Some(Action::ShowThread { board, thread })));
}

/// Tudo o que os quadros têm AGORA conta como visto (nada disto avisará): o que um projecto que se
/// abre traz. Uma mensagem que chegue depois avisa.
pub fn seen_now(hero: &mut HeroScreen) {
    let (boards, team) = hero.documents.boards_and_team();
    for b in boards.boards() {
        team.seeded.insert(b.id);
        team.notified
            .extend(b.team.threads.iter().flat_map(|t| t.messages.iter().map(|m| m.id)));
    }
}

/// A bolinha de «por ler para mim» no canto de cima à direita de `r`.
pub fn paint_dot(scene: &mut VectorScene, r: Rect, theme: Theme) {
    let rad = Spacing::Xs.px();
    let c = ph2d_vector::Circle::new(
        ph2d_vector::Point::new(
            f64::from(r.x + r.w - rad * 1.5),
            f64::from(r.y + rad * 1.5),
        ),
        f64::from(rad),
    );
    scene.inner_mut().fill(
        ph2d_vector::Fill::NonZero,
        ph2d_vector::Affine::IDENTITY,
        &ph2d_vector::Brush::Solid(resolve(ColorToken::Accent, theme)),
        None,
        &c,
    );
}

/// A bolinha do quadro `id` (as abas, o botão «Comments»): há fios para mim por ler.
#[must_use]
pub fn board_has_news(docs: &Documents, id: BoardId) -> bool {
    let me = me_of(docs);
    docs.boards().get(id).is_some_and(|b| unread_for_me(b, &me) > 0)
}
