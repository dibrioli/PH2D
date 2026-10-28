//! **A família PESSOAL do Kanban** (TrelloClone W9, plano §1.6 e §8.1) — o **Calendário do
//! projecto** (todas as datas de todos os Kanbans e os eventos próprios; arrastar muda a data), o
//! **Planner** (a semana com a coluna Inbox + os meus cartões sem data, que se arrastam para blocos de
//! tempo), **My cards** e o **sino** das notificações (menções, atribuições, o que sigo, lembretes).
//! Tudo DENTRO do projecto, sem Google/Microsoft; a porta para fora é o `.ics` (RFC 5545).
//!
//! Os dados moram na agenda do `KanbanSet` (`KanbanId::AGENDA`, formato v5); a grelha do calendário é
//! a vista da W7 (`ph2d-kanban-views`) sobre a [`source::ProjectSource`].

mod docs;
mod event_pop;
pub mod ics;
mod metrics;
pub mod notify;
pub mod personal;
pub mod recur;
mod side;
pub mod smoke;
pub mod source;
mod surface;
mod ui;

use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::toast::Toast;
use ph2d_kanban_model::{KanbanId, Member, Moment, splitmix64};

pub use docs::{Hit, Mode, PlannerDocs, TAB};
pub use event_pop::{EventPop, PopHit};
pub use surface::{CMD_CALENDAR, CMD_INBOX, CMD_MY_CARDS, CMD_PLANNER};
pub use ui::notice_text;

/// O id do nó `Document` da aba.
pub(crate) const A11Y_SALT: u64 = 0x504C_4E52_0000_0000;
/// O nó dos popovers (o do evento e o do sino).
pub(crate) const POP_NODE: ph2d_a11y::NodeId = ph2d_tool_registry::hash_node_id("planner_pop");

/// A declaração da família ao registo (`ph2d-app-registry-init`).
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "planner",
    routers: smoke::ROUTERS,
};

/// O relógio de parede (ms UTC).
#[must_use]
pub fn wall_clock() -> Moment {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
    Moment(ms)
}

/// O arranque da família (DEPOIS da do Kanban: a cena de smoke parte da dele).
pub fn install(hero: &mut HeroScreen) {
    hero.documents
        .install_surface(Box::new(PlannerDocs::default()));
    smoke::stage_armed_smokes(hero);
}

/// O quadro da família (a shell chama-o uma vez por quadro, ao lado do do Kanban): o projecto, o
/// relógio, quem sou, as notificações e os lembretes a avisar, e os pedidos para o mundo.
pub fn frame(hero: &mut HeroScreen, toasts: &mut ToastQueue) {
    let shared = hero.documents.kanbans_shared();
    let author = hero.documents.author().trim().to_owned();
    let team: Vec<Member> = hero
        .documents
        .boards()
        .boards()
        .iter()
        .flat_map(|b| b.team.people(&author, &[]))
        .map(Member)
        .collect();
    let clock = hero.store.clock_ns();
    let Some(docs) = hero.documents.surfaces_mut().find_mut::<PlannerDocs>() else {
        return;
    };
    docs.follow_project(shared.clone());
    {
        // A agenda sorteia SEMPRE os ids: duas agendas de projectos diferentes juntam-se numa sessão
        // ao vivo sem colidir.
        let mut set = shared.borrow_mut();
        if let Some(a) = set.doc_mut(KanbanId::AGENDA)
            && !a.in_session()
        {
            #[allow(clippy::cast_possible_truncation)]
            a.set_session(Some(splitmix64(clock as u64 ^ wall_clock().0.cast_unsigned())));
        }
    }
    let me = if author.is_empty() {
        Member(ph2d_i18n::tr("kanban.back.me").to_owned())
    } else {
        Member(author)
    };
    let now = docs.pinned.unwrap_or_else(wall_clock);
    docs.set_now(now, me, team);
    for n in docs.refresh_notices() {
        toasts.push(Toast::info(notice_text(&n, docs).0));
    }
    let (cards, msgs, ics) = docs.take_requests();
    for m in msgs {
        toasts.push(Toast::info(m));
    }
    if let Some(text) = ics {
        save_ics(&text, toasts);
    }
    if cards.is_empty() {
        return;
    }
    if let Some(k) = hero
        .documents
        .surfaces_mut()
        .find_mut::<ph2d_app_kanban::KanbanDocs>()
    {
        for c in cards {
            k.open(c.kanban);
            k.open_back(c.kanban.0, c.card);
        }
    }
}

fn save_ics(text: &str, toasts: &mut ToastQueue) {
    let dialog = rfd::FileDialog::new()
        .set_file_name(ph2d_i18n::tr("planner.export.file"))
        .add_filter(ph2d_i18n::tr("planner.export.filter"), &["ics"]);
    let Some(path) = ph2d_app_host::modal::save_file(dialog) else {
        return;
    };
    let name = path.display().to_string();
    match std::fs::write(&path, text) {
        Ok(()) => toasts.push(Toast::success(ph2d_i18n::tr_with(
            "planner.export.done",
            &[("path", &name)],
        ))),
        Err(e) => toasts.push(Toast::error(ph2d_i18n::tr_with(
            "planner.export.failed",
            &[("path", &name), ("error", &e.to_string())],
        ))),
    };
}
