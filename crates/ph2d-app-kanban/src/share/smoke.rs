//! SMOKE J1 — o Kanban da cena 1 PARTILHADO, com o popover «Share» aberto (o link e o código), e um
//! colega de mentira («Rui», um convidado numa thread que entra pela rede local DESTA máquina): o
//! avatar dele no cabeçalho, o anel no cartão que ele tem aberto, e o cursor dele a andar no quadro.
//! Um segundo PH2D entra com o link copiado (Share → colar → Join).

use crate::docs::KanbanDocs;
use ph2d_board_net as net;
use ph2d_kanban_model::{CardId, KanbanId, KanbanSet};
use ph2d_kanban_sync::{KEvent, KanbanGuest, KanbanPresence};
use std::time::{Duration, Instant};

/// Quanto tempo o colega fica (o smoke do dono cabe aqui).
const STAYS: Duration = Duration::from_secs(30 * 60);
/// O passo do colega (~60 Hz) e o círculo que o cursor dele faz (px do conteúdo).
const STEP: Duration = Duration::from_millis(16);
const CENTER: (f64, f64) = (640.0, 260.0);
const RADIUS: f64 = 120.0;

/// Partilha o Kanban `id`, abre o popover e põe o colega a entrar.
pub(crate) fn stage(docs: &mut KanbanDocs, id: KanbanId) {
    docs.share_start(id.0);
    if let Some(t) = docs.tab_mut(id.0) {
        t.w6.over = Some(crate::w6::Over::Share);
    }
    let Some(s) = docs.share.session.as_ref() else {
        return;
    };
    let lan = s.lan_text();
    let card = docs.set.borrow().get(id).and_then(|k| {
        let doing = k.lists.values().find(|l| l.title == "Doing")?.id;
        k.cards_in(doing).first().map(|c| c.id)
    });
    std::thread::spawn(move || colleague(&lan, card));
}

/// O colega: entra, abre o cartão `card` e anda com o cursor em círculos.
fn colleague(lan: &str, card: Option<CardId>) {
    let mut parts = lan.split_whitespace();
    let (Some(addr), Some(code)) = (parts.next().and_then(|a| a.parse().ok()), parts.next()) else {
        return;
    };
    let Ok(n) = net::Guest::connect(
        addr,
        code,
        Duration::from_secs(4),
        std::sync::Arc::new(|| {}),
    ) else {
        return;
    };
    let mut set = KanbanSet::default();
    let mut g = KanbanGuest::new(
        net::random_u64(),
        "Rui".into(),
        net::random_u64(),
        net::random_u64(),
    );
    let _ = n.send(g.hello().encode());
    let start = Instant::now();
    let mut mine: Option<KanbanId> = None;
    while start.elapsed() < STAYS {
        #[allow(clippy::cast_possible_truncation)]
        let now = start.elapsed().as_millis() as u64;
        for e in n.poll() {
            match e {
                net::NetEvent::Frame(_, b) => {
                    let (out, ev) = g.frame(&mut set, &b, now);
                    for m in out {
                        let _ = n.send(m.encode());
                    }
                    for e in ev {
                        match e {
                            KEvent::Arrived(ids) => {
                                mine = mine
                                    .or_else(|| ids.into_iter().find(|k| *k != KanbanId::AGENDA));
                            }
                            KEvent::Ended => return,
                            _ => {}
                        }
                    }
                }
                net::NetEvent::Left(_) => return,
                _ => {}
            }
        }
        let wire = mine.and_then(|k| g.room.replica.as_ref()?.wire_id(k));
        let a = start.elapsed().as_secs_f64();
        let p = KanbanPresence {
            peer: 0,
            kanban: wire,
            card,
            cursor: Some([CENTER.0 + RADIUS * a.cos(), CENTER.1 + RADIUS * a.sin()]),
        };
        for m in g.tick(&mut set, now).into_iter().chain(g.presence(now, p)) {
            let _ = n.send(m.encode());
        }
        std::thread::sleep(STEP);
    }
}
