//! ⭐ **«Share» — um Kanban ao vivo SOZINHO (J1, W5; plano §3.4)**: a sala da `ph2d-kanban-sync`
//! ([`KanbanHost`]/[`KanbanGuest`]) pelo transporte do Quadro — a rede local (`ph2d-board-net`) e a
//! internet com o link com chave (`ph2d-board-internet`, QUIC + DHT). O anfitrião abre a sala sobre o
//! Kanban da aba (e a AGENDA do projecto); quem entra por link recebe-o numa aba nova. A cada imagem
//! [`KanbanDocs::share_tick`] serve a rede, a minha presença e o que o ecrã mostra ([`Look`]).

mod net_io;
pub(crate) mod sheet;
pub(crate) mod smoke;

use crate::docs::KanbanDocs;
use ph2d_kanban_model::{CardId, KanbanId, Member};
use ph2d_kanban_sync::{Viewers, port};
use std::collections::BTreeMap;

pub(crate) use net_io::{JoinRefusal, Session};

/// Um alvo do popover «Share».
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShareAct {
    /// Partilhar este Kanban (abre a sala e prepara o link).
    Start,
    CopyLink,
    Stop,
    /// A caixa onde se cola um link para entrar.
    LinkBox,
    Join,
}

/// O que o ecrã sabe da partilha.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum Role {
    #[default]
    Off,
    /// A sala do Kanban `kanban` (id local) está aberta; o link chega quando a internet estiver pronta.
    Hosting {
        kanban: KanbanId,
        link: Option<String>,
        /// O endereço e o código da rede local.
        lan: String,
    },
    /// A ligar por um link.
    Joining,
    /// Dentro da sala de outro; `kanban` = o Kanban dela no meu projecto.
    Guest { kanban: Option<KanbanId> },
}

/// Quem está a ver um Kanban (id local): os avatares, os cartões abertos, os cursores.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Seen {
    pub people: Vec<Member>,
    pub cards: BTreeMap<CardId, Vec<Member>>,
    pub cursors: Vec<(Member, [f64; 2])>,
    /// J4 — o fio caiu e a sala está a voltar (o selo «Reconnecting…»).
    pub reconnecting: bool,
}

/// O que se pinta da partilha neste quadro.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Look {
    pub role: Role,
    pub seen: BTreeMap<KanbanId, Seen>,
}

/// A partilha da família: a sala (uma de cada vez) e o que ela pede ao mundo.
#[derive(Default)]
pub(crate) struct Share {
    pub session: Option<Session>,
    pub look: Look,
    /// Os testes: só esta máquina (porta do sistema, internet sem STUN nem DHT).
    pub local_only: bool,
    /// O link a pôr na área de transferência (servido pelo `frame` da família).
    pub copy: Option<String>,
    /// J7 — quem vê os Kanbans da sala do QUADRO (lida da [`port`]), somada ao [`Look`].
    pub room: BTreeMap<KanbanId, Seen>,
}

impl std::fmt::Debug for Share {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Share")
            .field("look", &self.look)
            .finish_non_exhaustive()
    }
}

/// O relógio da sala (ms desde o 1.º uso).
pub(crate) fn now_ms() -> u64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    u64::try_from(
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_millis(),
    )
    .unwrap_or(u64::MAX)
}

/// Quem vê cada Kanban de uma sala (`kanbans` = pares (sessão, local)): eu primeiro, depois os
/// colegas, os cartões que têm abertos e os cursores.
pub(crate) fn seen_of(
    viewers: &Viewers,
    names: &BTreeMap<u64, String>,
    kanbans: &[(KanbanId, KanbanId)],
    me: &Member,
    now: u64,
) -> BTreeMap<KanbanId, Seen> {
    let mut out = BTreeMap::new();
    for (wire, local) in kanbans {
        let mut s = Seen {
            people: vec![me.clone()],
            ..Seen::default()
        };
        for peer in viewers.on_kanban(*wire) {
            let who = Member(names.get(&peer).cloned().unwrap_or_default());
            if let Some(c) = viewers.get(peer).and_then(|p| p.card) {
                s.cards.entry(c).or_default().push(who.clone());
            }
            if let Some(at) = viewers.cursor(peer, now) {
                s.cursors.push((who.clone(), at));
            }
            s.people.push(who);
        }
        out.insert(*local, s);
    }
    out
}

/// A sonda da presença: os cartões abertos por colegas (com quem) e os cursores deles.
pub type SeenProbe = (Vec<(CardId, Vec<String>)>, Vec<(String, [f64; 2])>);

impl KanbanDocs {
    /// ⭐ Abre a sala do Kanban da aba `id` (o anfitrião). Já numa sala: nada.
    pub(crate) fn share_start(&mut self, id: u64) {
        if self.share.session.is_some() {
            return;
        }
        let Some(kid) = self.kanban_of(id) else {
            return;
        };
        let local = self.share.local_only;
        let me = self.me.0.clone();
        let set = self.set();
        match Session::host(&mut set.borrow_mut(), kid, &me, local) {
            Ok(s) => {
                self.share.look.role = Role::Hosting {
                    kanban: kid,
                    link: None,
                    lan: s.lan_text(),
                };
                self.share.session = Some(s);
            }
            Err(e) => self
                .notices
                .push(ph2d_i18n::tr_with("kanban.share.failed", &[("error", &e)])),
        }
    }

    /// Entra na sala do link (ou `endereço código` da rede local) que a pessoa colou.
    pub(crate) fn share_join(&mut self, text: &str) {
        if self.share.session.is_some() {
            return;
        }
        match Session::join(text, &self.me.0, self.share.local_only) {
            Ok(s) => {
                self.share.session = Some(s);
                self.share.look.role = Role::Joining;
            }
            Err(JoinRefusal::Board) => {
                self.notices
                    .push(ph2d_i18n::tr("kanban.share.board_link").to_owned());
                port::post_board_link(text.to_owned());
            }
            Err(JoinRefusal::Bad) => self
                .notices
                .push(ph2d_i18n::tr("kanban.share.link_bad").to_owned()),
        }
    }

    /// ⭐ J4 — entra na sala de um Kanban por um link colado NOUTRO sítio (o «Join» do Quadro):
    /// o Kanban chega numa aba, como pelo popover.
    pub fn join_shared(&mut self, link: &str) {
        self.share_join(link.trim());
    }

    /// Sai da sala (o anfitrião fecha-a para todos).
    pub(crate) fn share_stop(&mut self) {
        if let Some(s) = self.share.session.take() {
            s.close(&mut self.set.borrow_mut());
        }
        self.share.look = Look::default();
    }

    /// ⭐ A cada imagem: a rede, a minha presença e o [`Look`].
    pub(crate) fn share_tick(&mut self) {
        port::set_here(self.here());
        if let Some(link) = port::take_kanban_link() {
            self.share_join(link.trim());
        }
        self.pull_room();
        let Some(mut s) = self.share.session.take() else {
            return;
        };
        let now = now_ms();
        let presence = self.my_presence(&s);
        let set = self.set();
        let out = s.tick(&mut set.borrow_mut(), now, presence);
        for k in out.arrived {
            let guest = matches!(
                self.share.look.role,
                Role::Joining | Role::Guest { kanban: None }
            );
            if k != KanbanId::AGENDA && guest {
                self.open(k);
                self.share.look.role = Role::Guest { kanban: Some(k) };
            }
        }
        if out.welcomed && self.share.look.role == Role::Joining {
            self.share.look.role = Role::Guest { kanban: None };
        }
        if let Some(link) = out.link {
            if let Role::Hosting { link: l, .. } = &mut self.share.look.role {
                *l = Some(link.clone());
            }
            self.share.copy = Some(link);
            self.notices
                .push(ph2d_i18n::tr("kanban.share.link_ready").to_owned());
        }
        self.notices.extend(out.notices);
        if out.ended {
            s.close(&mut set.borrow_mut());
            self.share.look = Look::default();
            self.share.look.seen.extend(self.share.room.clone());
            return;
        }
        self.share.look.seen = s.seen(&self.me, now);
        for seen in self.share.look.seen.values_mut() {
            seen.reconnecting = out.reconnecting;
        }
        self.share.look.seen.extend(self.share.room.clone());
        self.share.session = Some(s);
    }

    /// Onde estou na sala do Kanban sozinho (o id da sessão).
    fn my_presence(&self, s: &Session) -> Option<(KanbanId, Option<CardId>, Option<[f64; 2]>)> {
        let (local, card, cursor) = self.here()?;
        Some((s.wire_of(local)?, card, cursor))
    }

    /// ⭐ J7 — onde estou: o Kanban à frente (id LOCAL; a última aba pintada), o cartão aberto e o
    /// cursor no conteúdo. A sala do Kanban sozinho e a do Quadro traduzem-no para o id da sessão.
    #[must_use]
    pub fn here(&self) -> Option<(KanbanId, Option<CardId>, Option<[f64; 2]>)> {
        let id = self.active?;
        let t = self.tabs.iter().find(|t| t.id() == id)?;
        let card = t.back.as_ref().map(|b| b.card);
        let cursor = t
            .cursor
            .filter(|_| t.other.is_none() && t.back.is_none())
            .filter(|p| t.layout.board.contains(p.0, p.1))
            .map(|p| crate::presence::to_content(t, p));
        Some((t.view.kanban, card, cursor))
    }

    /// ⭐ J7 — a presença nos Kanbans da sala do QUADRO (a [`port::Room`]): os anéis, os avatares
    /// no cartão e no cabeçalho pintam-se como na sala do Kanban sozinho; a voltar, esmaecidos.
    fn pull_room(&mut self) {
        let me = &self.me;
        let seen = port::with_room(|r| {
            r.map(|r| {
                let mut seen = seen_of(&r.viewers, &r.names, &r.pairs, me, r.now);
                for s in seen.values_mut() {
                    s.reconnecting = r.reconnecting;
                }
                seen
            })
            .unwrap_or_default()
        });
        self.set_room(seen);
    }

    fn set_room(&mut self, seen: BTreeMap<KanbanId, Seen>) {
        for k in std::mem::take(&mut self.share.room).into_keys() {
            self.share.look.seen.remove(&k);
        }
        self.share.look.seen.extend(seen.clone());
        self.share.room = seen;
    }

    /// ⚠️ SONDA (J1): `(papel, link, quem vê o Kanban da aba id)`.
    #[must_use]
    pub fn probe_share(&self, id: u64) -> (String, Option<String>, Vec<String>) {
        let link = match &self.share.look.role {
            Role::Hosting { link, .. } => link.clone(),
            _ => None,
        };
        let role = match &self.share.look.role {
            Role::Off => "Off",
            Role::Hosting { .. } => "Hosting",
            Role::Joining => "Joining",
            Role::Guest { .. } => "Guest",
        };
        let people = self
            .kanban_of(id)
            .and_then(|k| self.share.look.seen.get(&k))
            .map(|s| s.people.iter().map(|p| p.0.clone()).collect())
            .unwrap_or_default();
        (role.to_owned(), link, people)
    }

    /// ⚠️ SONDA (J1): os cartões abertos por colegas e os cursores deles, no Kanban da aba `id`.
    #[must_use]
    pub fn probe_seen(&self, id: u64) -> SeenProbe {
        let Some(s) = self
            .kanban_of(id)
            .and_then(|k| self.share.look.seen.get(&k))
        else {
            return (Vec::new(), Vec::new());
        };
        (
            s.cards
                .iter()
                .map(|(c, w)| (*c, w.iter().map(|m| m.0.clone()).collect()))
                .collect(),
            s.cursors.iter().map(|(m, p)| (m.0.clone(), *p)).collect(),
        )
    }

    /// Os testes: o fio da sala cai (o convidado volta sozinho).
    pub fn share_drop_for_tests(&mut self) {
        if let Some(s) = &mut self.share.session {
            s.drop_link();
        }
    }

    /// ⚠️ SONDA (J4): a sala do Kanban da aba `id` está a voltar depois de cair.
    #[must_use]
    pub fn probe_reconnecting(&self, id: u64) -> bool {
        self.kanban_of(id)
            .and_then(|k| self.share.look.seen.get(&k))
            .is_some_and(|s| s.reconnecting)
    }

    /// Os testes: a partilha só nesta máquina.
    pub fn share_local_for_tests(&mut self) {
        self.share.local_only = true;
    }

    /// Sai da sala (a app vai fechar, ou o projecto mudou).
    pub fn share_shutdown(&mut self) {
        self.share_stop();
    }
}
