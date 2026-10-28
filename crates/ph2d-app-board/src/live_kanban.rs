//! ⭐ **J4 — a sala do Quadro leva os Kanbans que ele referencia** (ADR-0195 §3): a sala da
//! `ph2d-kanban-sync` ([`HostRoom`]/[`GuestRoom`]) dentro de `Msg::Kanban`. O anfitrião abre-a sobre os
//! Kanbans do quadro (e a AGENDA); um Kanban posto no quadro a meio partilha-se (`share`); cada par
//! mora no SEU id e a referência do elemento traduz-se na borda da réplica do Quadro
//! (`ph2d_board_sync::Replica::map_kanbans`).

use ph2d_board_model::BoardId;
use ph2d_board_net as net;
use ph2d_board_sync::{self as sync, KANBAN_WAITING, Msg, To};
use ph2d_editor_core::screens::HeroScreen;
use ph2d_kanban_model::{KanbanId, KanbanSet};
use ph2d_kanban_sync::{GuestRoom, HostRoom, KMsg, KanbanReplica, Portrait};
use std::sync::mpsc;

use super::send;

/// A sala do anfitrião e os retratos a fazer num fio (MEDICOES.md §J4: 39 → 9,4 ms na janela).
pub(super) struct KHost {
    room: HostRoom,
    tx: mpsc::Sender<(sync::ConnId, Vec<KMsg>)>,
    rx: mpsc::Receiver<(sync::ConnId, Vec<KMsg>)>,
}

/// Os retratos por fazer vão para fios; os feitos vão a quem entrou. `true` = algum grande demais.
fn portraits(n: &net::Host, k: &mut KHost) -> bool {
    for p in k.room.take_portraits() {
        let tx = k.tx.clone();
        let _ = std::thread::Builder::new()
            .name("kanban-portrait".into())
            .spawn(move || {
                let conn = p.conn;
                let _ = tx.send((conn, Portrait::messages(p)));
            });
    }
    let mut big = false;
    while let Ok((conn, ms)) = k.rx.try_recv() {
        big |= send::host(
            n,
            wrap(ms.into_iter().map(|m| (To::One(conn), m)).collect()),
        );
    }
    big
}

/// Os Kanbans (ids locais) que o quadro `board` mostra.
fn referenced(hero: &HeroScreen, board: BoardId) -> Vec<KanbanId> {
    let Some(b) = hero.documents.boards().get(board) else {
        return Vec::new();
    };
    let mut ids: Vec<KanbanId> = b
        .doc
        .live_with_hidden()
        .filter_map(|e| e.kanban().map(|k| k.kanban))
        .filter(|k| *k != KANBAN_WAITING)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// Os do quadro que ainda não estão na sala entram (a meio: um elemento novo).
fn share_new(r: &mut KanbanReplica, set: &KanbanSet, mine: &[KanbanId]) {
    for l in mine {
        if r.wire_id(*l).is_none() && set.get(*l).is_some() {
            let _ = r.share(set, *l);
        }
    }
}

fn wrap(out: Vec<(To, KMsg)>) -> Vec<(To, Msg)> {
    out.into_iter()
        .map(|(to, m)| (to, Msg::Kanban(m.encode())))
        .collect()
}

/// A sala do anfitrião, aberta com a do Quadro.
pub(super) fn host_open(hero: &mut HeroScreen, s: &mut sync::Host, board: BoardId) -> KHost {
    let mut ids = referenced(hero, board);
    ids.push(KanbanId::AGENDA);
    let shared = hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    let r = KanbanReplica::host(s.me.peer, super::random(), &set, &ids);
    r.arm(&mut set);
    let mut room = HostRoom::new(r);
    room.portraits_off_window();
    if let Some(b) = hero.documents.board_mut(board) {
        s.map_kanbans(b, &room.replica.kanbans());
    }
    let (tx, rx) = mpsc::channel();
    KHost { room, tx, rx }
}

/// ⭐ Uma imagem do anfitrião (antes do `tick` do Quadro: a tradução já conhece os novos).
pub(super) fn host_step(
    hero: &mut HeroScreen,
    n: &net::Host,
    s: &mut sync::Host,
    board: BoardId,
    k: &mut KHost,
    now: u64,
    armed: &mut Vec<KanbanId>,
) -> bool {
    let room = &mut k.room;
    let mine = referenced(hero, board);
    let shared = hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    let mut big = false;
    for (conn, from, bytes) in s.take_kanban() {
        let (out, _) = room.frame_bytes(&mut set, conn, from, &bytes, now);
        big |= send::host(n, wrap(out));
    }
    share_new(&mut room.replica, &set, &mine);
    big |= send::host(n, wrap(room.tick(&mut set, now)));
    drop(set);
    let pairs = room.replica.kanbans();
    big |= portraits(n, k);
    if let Some(b) = hero.documents.board_mut(board) {
        s.map_kanbans(b, &pairs);
    }
    *armed = pairs.into_iter().map(|(_, l)| l).collect();
    big
}

/// `peer` saiu da sala do Quadro: sai da do Kanban.
pub(super) fn host_left(n: &net::Host, k: &mut KHost, peer: u64) {
    send::host(n, wrap(k.room.left(peer)));
}

/// A sala do convidado (a tradução liga-se já: o retrato do Quadro chega com os ids da sessão).
pub(super) fn guest_new(s: &mut sync::Guest) -> GuestRoom {
    s.map_kanbans(None, &[]);
    GuestRoom::new(s.peer(), super::random())
}

/// ⭐ Uma imagem do convidado. `welcomed` = a sala do Quadro acabou de me (re)aceitar: peço os
/// Kanbans (ao voltar, o retrato traz o que perdi e o que fiz offline funde por cima).
#[allow(clippy::too_many_arguments)]
pub(super) fn guest_step(
    hero: &mut HeroScreen,
    n: &net::Guest,
    s: &mut sync::Guest,
    board: Option<BoardId>,
    room: &mut GuestRoom,
    welcomed: bool,
    now: u64,
    armed: &mut Vec<KanbanId>,
) {
    if welcomed {
        let _ = n.send(sync::Guest::kanban(room.ask().encode()).encode());
    }
    let mine = board.map(|b| referenced(hero, b)).unwrap_or_default();
    let shared = hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    for bytes in s.take_kanban() {
        let (out, _) = room.frame_bytes(&mut set, &bytes, now);
        for m in out {
            let _ = n.send(sync::Guest::kanban(m.encode()).encode());
        }
    }
    let Some(r) = &mut room.replica else {
        return;
    };
    share_new(r, &set, &mine);
    for m in room.tick(&mut set, now) {
        let _ = n.send(sync::Guest::kanban(m.encode()).encode());
    }
    drop(set);
    let pairs = room
        .replica
        .as_ref()
        .map(KanbanReplica::kanbans)
        .unwrap_or_default();
    s.map_kanbans(board.and_then(|b| hero.documents.board_mut(b)), &pairs);
    *armed = pairs.into_iter().map(|(_, l)| l).collect();
}

/// A sala acabou: os Kanbans dela voltam ao contador local.
pub(super) fn disarm(hero: &mut HeroScreen, armed: &mut Vec<KanbanId>) {
    if armed.is_empty() {
        return;
    }
    let shared = hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    for l in armed.drain(..) {
        if let Some(k) = set.doc_mut(l) {
            k.set_session(None);
        }
    }
}

/// J4 — um convite de KANBAN colado no «Join» do Quadro: di-lo, e a sala abre-se numa aba Kanban.
pub(super) fn join_kanban_link(hero: &mut HeroScreen, link: &str) {
    hero.documents
        .tell(ph2d_i18n::tr("kanban.share.kanban_link").to_owned());
    if let Some(docs) = hero
        .documents
        .surfaces_mut()
        .find_mut::<ph2d_app_kanban::KanbanDocs>()
    {
        docs.join_shared(link);
    }
}

/// J4 — os convites de QUADRO colados no «Join» do Kanban: entram como o «Join by link» do Quadro.
pub(super) fn board_links(
    hero: &mut HeroScreen,
) -> Vec<ph2d_editor_core::documents::live::LiveRequest> {
    hero.documents
        .surfaces_mut()
        .find_mut::<ph2d_app_kanban::KanbanDocs>()
        .and_then(ph2d_app_kanban::KanbanDocs::take_board_link)
        .map(|link| ph2d_editor_core::documents::live::LiveRequest::JoinLink { link })
        .into_iter()
        .collect()
}
