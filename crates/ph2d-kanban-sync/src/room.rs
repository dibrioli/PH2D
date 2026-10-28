//! **A sala do Kanban** — o que o anfitrião e o convidado fazem com [`KMsg`], seja dentro da sessão de
//! um Quadro (`ph2d_board_sync::Msg::Kanban`) seja numa sessão só do Kanban ([`crate::KanbanHost`]).
//! Máquinas de estado PURAS: mensagens (e o relógio, injectado) → mensagens e eventos.

use ph2d_board_sync::{ConnId, PeerId, PeerInfo, SEND_EVERY_MS, To};
use ph2d_kanban_model::{KanbanId, KanbanSet};

use crate::msg::{KMsg, KWrite};
use crate::presence::{KanbanPresence, Viewers};
use crate::replica::KanbanReplica;

/// O que o ecrã tem de saber.
#[derive(Clone, Debug, PartialEq)]
pub enum KEvent {
    /// Entrei (ou voltei) na sala.
    Welcomed,
    /// Kanbans da sessão entraram no MEU projecto (ids locais): a UI abre-os ou mostra-os.
    Arrived(Vec<KanbanId>),
    /// O anfitrião fala outra versão.
    Refused {
        theirs: u32,
    },
    Joined(PeerInfo),
    Left(PeerId),
    /// O anfitrião fechou a sala.
    Ended,
}

/// As escritas em mensagens: cada ficheiro na sua (são os grandes), o resto junto.
fn by_file(ws: Vec<KWrite>) -> Vec<KMsg> {
    let (files, rest): (Vec<KWrite>, Vec<KWrite>) =
        ws.into_iter().partition(|w| matches!(w, KWrite::File(_)));
    let mut out: Vec<KMsg> = files.into_iter().map(|f| KMsg::Writes(vec![f])).collect();
    if !rest.is_empty() {
        out.push(KMsg::Writes(rest));
    }
    out
}

/// A minha presença: sai já se mudou de Kanban ou de cartão; o cursor no máximo a cada
/// [`SEND_EVERY_MS`] (os ~30 Hz do Quadro), e o último nunca se perde.
#[derive(Clone, Debug, Default)]
struct Pace {
    sent: Option<KanbanPresence>,
    at: Option<u64>,
    queued: Option<KanbanPresence>,
}

impl Pace {
    fn offer(&mut self, now_ms: u64, p: KanbanPresence) -> Option<KanbanPresence> {
        if self.sent.as_ref() == Some(&p) {
            self.queued = None;
            return None;
        }
        let place = self
            .sent
            .as_ref()
            .is_none_or(|s| (s.kanban, s.card) != (p.kanban, p.card));
        let due = self
            .at
            .is_none_or(|t| now_ms.saturating_sub(t) >= SEND_EVERY_MS);
        if place || due {
            (self.sent, self.at, self.queued) = (Some(p.clone()), Some(now_ms), None);
            Some(p)
        } else {
            self.queued = Some(p);
            None
        }
    }

    fn flush(&mut self, now_ms: u64) -> Option<KanbanPresence> {
        let p = self.queued.take()?;
        self.offer(now_ms, p)
    }
}

/// O anfitrião (e o retransmissor: a sala é uma estrela, como a do Quadro).
#[derive(Debug)]
pub struct HostRoom {
    pub replica: KanbanReplica,
    pub viewers: Viewers,
    pace: Pace,
    /// J4 — o retrato de quem entra sai da janela ([`Self::portraits_off_window`]).
    off_window: bool,
    portraits: Vec<crate::Portrait>,
}

impl HostRoom {
    #[must_use]
    pub fn new(replica: KanbanReplica) -> Self {
        Self {
            replica,
            viewers: Viewers::default(),
            pace: Pace::default(),
            off_window: false,
            portraits: Vec::new(),
        }
    }

    /// ⭐ J4 — daqui em diante o `welcome` só COPIA o retrato ([`crate::Portrait`]); quem compõe tira-o
    /// por [`Self::take_portraits`], corre [`crate::Portrait::messages`] num fio e manda-as à ligação.
    pub fn portraits_off_window(&mut self) {
        self.off_window = true;
    }

    /// Os retratos por fazer (fora da janela).
    pub fn take_portraits(&mut self) -> Vec<crate::Portrait> {
        std::mem::take(&mut self.portraits)
    }

    /// Quem entra (`conn`): o que eu tinha por enviar vai aos outros; a ele o retrato, os ficheiros
    /// e onde estão os outros.
    pub fn welcome(&mut self, set: &mut KanbanSet, conn: ConnId) -> Vec<(To, KMsg)> {
        let mut out: Vec<(To, KMsg)> = by_file(self.replica.local(set))
            .into_iter()
            .map(|m| (To::AllBut(conn), m))
            .collect();
        if self.off_window {
            self.portraits.push(self.replica.portrait(set, conn));
        } else {
            let snap = self.replica.snapshot(set);
            out.extend(
                crate::snapshot::messages(snap, crate::SNAPSHOT_BLOCK)
                    .into_iter()
                    .map(|m| (To::One(conn), m)),
            );
        }
        for m in by_file(self.replica.all_files(set)) {
            out.push((To::One(conn), m));
        }
        let seen: Vec<KanbanPresence> = self
            .pace
            .sent
            .iter()
            .cloned()
            .chain(self.viewers.all().cloned())
            .collect();
        out.extend(seen.into_iter().map(|p| (To::One(conn), KMsg::Presence(p))));
        out
    }

    /// Uma mensagem de `from` (na ligação `conn`).
    pub fn frame(
        &mut self,
        set: &mut KanbanSet,
        conn: ConnId,
        from: PeerId,
        msg: KMsg,
        now_ms: u64,
    ) -> (Vec<(To, KMsg)>, Vec<KEvent>) {
        let mut events = Vec::new();
        let out = match msg {
            KMsg::Want => self.welcome(set, conn),
            KMsg::Writes(ws) => {
                let before = self.replica.kanbans();
                self.replica.receive(set, &ws);
                arrived(&self.replica, &before, &mut events);
                vec![(To::AllBut(conn), KMsg::Writes(ws))]
            }
            KMsg::Presence(p) if p.peer == from => {
                self.viewers.heard(now_ms, p.clone());
                vec![(To::AllBut(conn), KMsg::Presence(p))]
            }
            _ => Vec::new(),
        };
        (out, events)
    }

    /// Bytes de `ph2d_board_sync::Host::take_kanban` (a sala do Quadro): a resposta embrulha-se em
    /// `Msg::Kanban(m.encode())`.
    pub fn frame_bytes(
        &mut self,
        set: &mut KanbanSet,
        conn: ConnId,
        from: PeerId,
        bytes: &[u8],
        now_ms: u64,
    ) -> (Vec<(To, KMsg)>, Vec<KEvent>) {
        match KMsg::decode(bytes) {
            Some(m) => self.frame(set, conn, from, m, now_ms),
            None => (Vec::new(), Vec::new()),
        }
    }

    /// A cada imagem: o que mudei, e a minha presença por enviar.
    pub fn tick(&mut self, set: &mut KanbanSet, now_ms: u64) -> Vec<(To, KMsg)> {
        let mut out: Vec<(To, KMsg)> = by_file(self.replica.local(set))
            .into_iter()
            .map(|m| (To::All, m))
            .collect();
        if let Some(p) = self.pace.flush(now_ms) {
            out.push((To::All, KMsg::Presence(p)));
        }
        out
    }

    /// A minha presença (`peer` é posto aqui).
    pub fn presence(&mut self, now_ms: u64, mut p: KanbanPresence) -> Vec<(To, KMsg)> {
        p.peer = self.replica.peer;
        self.pace
            .offer(now_ms, p)
            .map(|p| vec![(To::All, KMsg::Presence(p))])
            .unwrap_or_default()
    }

    /// `peer` saiu da sala.
    pub fn left(&mut self, peer: PeerId) -> Vec<(To, KMsg)> {
        self.viewers.gone(peer);
        vec![(To::All, KMsg::Gone(peer))]
    }
}

/// Os Kanbans que entraram desde `before` (ids locais).
fn arrived(r: &KanbanReplica, before: &[(KanbanId, KanbanId)], events: &mut Vec<KEvent>) {
    let new: Vec<KanbanId> = r
        .kanbans()
        .into_iter()
        .filter(|p| !before.contains(p))
        .map(|(_, l)| l)
        .collect();
    if !new.is_empty() {
        events.push(KEvent::Arrived(new));
    }
}

/// Um convidado.
#[derive(Debug)]
pub struct GuestRoom {
    pub peer: PeerId,
    seed: u64,
    pub replica: Option<KanbanReplica>,
    pub viewers: Viewers,
    pace: Pace,
    /// O retrato em blocos a chegar, e as escritas que chegaram no meio (aplicam-se depois dele).
    parts: crate::snapshot::Parts,
    held: Vec<KWrite>,
    /// J4 — pedi um retrato e ele ainda não chegou: as escritas esperam por ele (o retrato pode vir
    /// de um fio, DEPOIS de escritas mais novas do que ele).
    pub(crate) awaiting: bool,
}

impl GuestRoom {
    /// `seed` vem do sistema (os ids que eu sortear).
    #[must_use]
    pub fn new(peer: PeerId, seed: u64) -> Self {
        Self {
            peer,
            seed,
            replica: None,
            viewers: Viewers::default(),
            pace: Pace::default(),
            parts: crate::snapshot::Parts::default(),
            held: Vec::new(),
            awaiting: true,
        }
    }

    /// Pede os Kanbans (ou volta a pedi-los ao voltar): até o retrato chegar, as escritas esperam.
    pub fn ask(&mut self) -> KMsg {
        self.entered();
        KMsg::Want
    }

    /// Entrei (ou voltei) na sala: o retrato vem a seguir, e a minha presença sai outra vez inteira
    /// (o anfitrião esqueceu-a quando caí).
    pub(crate) fn entered(&mut self) {
        self.awaiting = true;
        self.pace = Pace::default();
    }

    /// Na sala de um Quadro, depois do `Welcome` dela: pede os Kanbans.
    #[must_use]
    pub fn want(&self) -> KMsg {
        KMsg::Want
    }

    /// Uma mensagem do anfitrião.
    pub fn frame(
        &mut self,
        set: &mut KanbanSet,
        msg: KMsg,
        now_ms: u64,
    ) -> (Vec<KMsg>, Vec<KEvent>) {
        let mut events = Vec::new();
        let mut out = Vec::new();
        match msg {
            KMsg::Snapshot(snap) => self.snapshot(set, &snap, now_ms, &mut out, &mut events),
            KMsg::SnapshotPart { part, of, bytes } => {
                if let Some(snap) = self.parts.offer(part, of, bytes) {
                    self.snapshot(set, &snap, now_ms, &mut out, &mut events);
                }
            }
            KMsg::Writes(ws) if self.parts.busy() || self.awaiting => {
                self.held.extend(ws);
                crate::replica::cap_writes(&mut self.held);
            }
            KMsg::Writes(ws) => {
                if let Some(r) = &mut self.replica {
                    let before = r.kanbans();
                    r.receive(set, &ws);
                    arrived(r, &before, &mut events);
                }
            }
            KMsg::Presence(p) if p.peer != self.peer => self.viewers.heard(now_ms, p),
            KMsg::Gone(p) => self.viewers.gone(p),
            _ => {}
        }
        (out, events)
    }

    /// Um retrato inteiro: entrar (ou voltar), e as escritas que chegaram enquanto ele vinha.
    fn snapshot(
        &mut self,
        set: &mut KanbanSet,
        snap: &crate::KSnapshot,
        now_ms: u64,
        out: &mut Vec<KMsg>,
        events: &mut Vec<KEvent>,
    ) {
        match &mut self.replica {
            Some(r) => {
                let before = r.kanbans();
                if r.rejoin(set, snap).is_err() {
                    return;
                }
                arrived(r, &before, events);
            }
            None => {
                let Ok((r, ids)) = KanbanReplica::join(self.peer, self.seed, snap, set) else {
                    return;
                };
                self.replica = Some(r);
                events.push(KEvent::Arrived(ids));
            }
        }
        self.awaiting = false;
        let held = std::mem::take(&mut self.held);
        if let Some(r) = &mut self.replica
            && !held.is_empty()
        {
            let before = r.kanbans();
            r.receive(set, &held);
            arrived(r, &before, events);
        }
        out.extend(self.tick(set, now_ms));
    }

    /// Bytes de `ph2d_board_sync::Guest::take_kanban`.
    pub fn frame_bytes(
        &mut self,
        set: &mut KanbanSet,
        bytes: &[u8],
        now_ms: u64,
    ) -> (Vec<KMsg>, Vec<KEvent>) {
        match KMsg::decode(bytes) {
            Some(m) => self.frame(set, m, now_ms),
            None => (Vec::new(), Vec::new()),
        }
    }

    /// A cada imagem: o que mudei, e a minha presença por enviar.
    pub fn tick(&mut self, set: &mut KanbanSet, now_ms: u64) -> Vec<KMsg> {
        let mut out = match &mut self.replica {
            Some(r) => by_file(r.local(set)),
            None => Vec::new(),
        };
        if let Some(p) = self.pace.flush(now_ms) {
            out.push(KMsg::Presence(p));
        }
        out
    }

    pub fn presence(&mut self, now_ms: u64, mut p: KanbanPresence) -> Vec<KMsg> {
        p.peer = self.peer;
        self.pace
            .offer(now_ms, p)
            .map(|p| vec![KMsg::Presence(p)])
            .unwrap_or_default()
    }
}
