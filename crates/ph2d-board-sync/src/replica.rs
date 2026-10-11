//! **A réplica** de um quadro numa sessão ao vivo: os carimbos de cada registo, a captura do que a
//! pessoa mudou e a fusão do que chega (ADR-0181 §1–§2). Comutativa, associativa e idempotente: a
//! ordem, o atraso e a duplicação das escritas não mudam o resultado.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{Board, BoardOp, BoardSet, Element, ElementId, Field, Stamp};
use serde::{Deserialize, Serialize};

use crate::team::{Shadow, TeamState, TeamWrite};
use crate::tokens::Tokens;

/// Quem é quem numa sessão (sorteado por quem entra).
pub type PeerId = u64;

/// Um carimbo de Lamport: ordem total, o contador primeiro e o par a desempatar.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Lamport {
    pub n: u64,
    pub peer: PeerId,
}

/// Um registo do QUADRO (não de um elemento).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BoardField {
    Name,
    Sketch,
}

/// Uma mudança que viaja.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Write {
    /// Um elemento que nasceu: todos os registos com este carimbo.
    Born { el: Element, at: Lamport },
    /// Um registo de um elemento (bytes de [`Element::read`]).
    Set {
        id: ElementId,
        field: Field,
        value: Vec<u8>,
        at: Lamport,
    },
    /// Um carimbo dado (`take: None` — a ficha é `op`) ou tirado (a ficha `take`).
    Stamp {
        id: ElementId,
        stamp: Stamp,
        op: Lamport,
        take: Option<Lamport>,
    },
    /// Um registo do quadro.
    Board {
        field: BoardField,
        value: Vec<u8>,
        at: Lamport,
    },
    /// Os comentários, as versões e a apresentação.
    Team { w: TeamWrite, at: Lamport },
    /// W14 — os bytes de uma imagem (uma vez; `replica_files.rs`).
    File {
        file: ph2d_board_model::ImageFile,
        at: Lamport,
    },
}

impl Write {
    pub(crate) fn at(&self) -> Lamport {
        match self {
            Write::Born { at, .. }
            | Write::Set { at, .. }
            | Write::Board { at, .. }
            | Write::Team { at, .. }
            | Write::File { at, .. } => *at,
            Write::Stamp { op, .. } => *op,
        }
    }

    fn id(&self) -> Option<ElementId> {
        match self {
            Write::Born { el, .. } => Some(el.id),
            Write::Set { id, .. } | Write::Stamp { id, .. } => Some(*id),
            Write::Board { .. } | Write::Team { .. } | Write::File { .. } => None,
        }
    }
}

/// Os carimbos de um elemento no retrato: o mais comum dos registos dele e os que diferem.
type PackedStamps = (ElementId, Lamport, Vec<(Field, Lamport)>);

/// Tudo o que quem entra precisa: o quadro e os carimbos.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub board: Vec<u8>,
    /// Por elemento: o carimbo mais comum dos registos dele e os que diferem (medido: um carimbo por
    /// registo eram ~500 dos 603 B/elemento de um quadro ao vivo).
    stamps: Vec<PackedStamps>,
    board_stamps: Vec<(BoardField, Lamport)>,
    tokens: Tokens,
    tops: Vec<(ElementId, Lamport)>,
    team: TeamState,
    clock: u64,
}

/// A chave de uma escrita minha por reenviar: um registo guarda só a última.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OutKey {
    Born(ElementId),
    Set(ElementId, Field),
    Stamp(Lamport),
    Board(BoardField),
    Team(Lamport),
    File(ph2d_board_model::FileId),
}

#[derive(Clone, Debug)]
pub struct Replica {
    pub peer: PeerId,
    clock: u64,
    stamps: BTreeMap<(ElementId, Field), Lamport>,
    board_stamps: BTreeMap<BoardField, Lamport>,
    tokens: Tokens,
    /// O maior carimbo que tocou cada elemento — a `version` dele.
    tops: BTreeMap<ElementId, Lamport>,
    /// Escritas de elementos que ainda não nasceram aqui.
    pending: Vec<Write>,
    /// Capturadas ao receber, por enviar.
    outgoing: Vec<Write>,
    /// As minhas, para reenviar depois de uma queda.
    outbox: BTreeMap<OutKey, Write>,
    shadow: (String, bool),
    team: TeamState,
    team_shadow: Shadow,
    sealed: Sealed,
    /// W14 — os ficheiros de imagem que este par já mandou ou recebeu.
    files: BTreeSet<ph2d_board_model::FileId>,
}

#[path = "replica_files.rs"]
mod files;
#[path = "replica_sealed.rs"]
mod sealed;
pub use files::by_file;
pub(crate) use sealed::sealed_field;
use sealed::{Seal, Sealed};

impl Replica {
    fn blank(peer: PeerId, board: &Board) -> Self {
        Self {
            peer,
            clock: 0,
            stamps: BTreeMap::new(),
            board_stamps: BTreeMap::new(),
            tokens: Tokens::default(),
            tops: BTreeMap::new(),
            pending: Vec::new(),
            outgoing: Vec::new(),
            outbox: BTreeMap::new(),
            shadow: (board.name.clone(), board.sketch),
            team: TeamState::default(),
            team_shadow: Shadow::of(&board.team),
            sealed: Sealed::default(),
            files: board.files.keys().copied().collect(),
        }
    }

    /// Abre uma sessão sobre `board` (o anfitrião). `seed` (do sistema) sorteia os ids novos.
    pub fn host(peer: PeerId, board: &mut Board, seed: u64) -> Self {
        board.set_session(Some(seed));
        let mut r = Self::blank(peer, board);
        r.team.seed(&board.team);
        r.team.settle(&mut board.team);
        r.team_shadow = Shadow::of(&board.team);
        r
    }

    /// Entra numa sessão a partir do retrato do anfitrião.
    ///
    /// # Errors
    /// Um retrato corrompido.
    pub fn join(peer: PeerId, snap: &Snapshot, seed: u64) -> Result<(Self, Board), String> {
        let mut board = BoardSet::board_from_bytes(&snap.board)?;
        board.set_session(Some(seed));
        let mut r = Self::blank(peer, &board);
        r.adopt(snap);
        Ok((r, board))
    }

    /// Volta depois de uma queda: o quadro passa a ser o do retrato (a câmara fica), e as minhas
    /// escritas reaplicam-se por cima e reenviam-se (as que lá chegaram são idempotentes).
    ///
    /// # Errors
    /// Um retrato corrompido.
    pub fn rejoin(&mut self, board: &mut Board, snap: &Snapshot, seed: u64) -> Result<(), String> {
        let camera = board.camera;
        let _ = self.local(board);
        // O retrato não leva os bytes das imagens: os que já tinha ficam (o anfitrião manda-os outra vez).
        let files = std::mem::take(&mut board.files);
        *board = BoardSet::board_from_bytes(&snap.board)?;
        (board.camera, board.files) = (camera, files);
        board.set_session(Some(seed));
        let mine: Vec<Write> = self.outbox.values().cloned().collect();
        // O meu relógio nunca volta atrás: dois carimbos meus iguais seriam a mesma ficha.
        *self = Self {
            outbox: std::mem::take(&mut self.outbox),
            clock: self.clock,
            sealed: std::mem::take(&mut self.sealed),
            ..Self::blank(self.peer, board)
        };
        self.adopt(snap);
        self.receive(board, &mine);
        // O que era selado volta ao cofre, nunca a todos.
        let (sealed, public): (Vec<Write>, Vec<Write>) = mine.into_iter().partition(|w| {
            sealed_field(w).is_some_and(|(id, f, at)| self.sealed.mine.get(&(id, f)) == Some(&at))
        });
        self.outgoing.extend(public);
        self.sealed.out.extend(sealed);
        Ok(())
    }

    fn adopt(&mut self, snap: &Snapshot) {
        self.clock = self.clock.max(snap.clock);
        self.stamps = snap
            .stamps
            .iter()
            .flat_map(|(id, common, odd)| {
                let odd: BTreeMap<Field, Lamport> = odd.iter().copied().collect();
                Field::ALL
                    .into_iter()
                    .filter_map(|f| {
                        odd.get(&f)
                            .copied()
                            .or(Some(*common))
                            .map(|l| ((*id, f), l))
                    })
                    .filter(|(_, l)| *l != Lamport::default())
                    .collect::<Vec<_>>()
            })
            .collect();
        self.board_stamps = snap.board_stamps.iter().copied().collect();
        self.tokens = snap.tokens.clone();
        self.tops = snap.tops.iter().copied().collect();
        self.team = snap.team.clone();
    }

    /// O retrato para quem entra. Os MEUS registos selados saem com o valor e o carimbo que os outros
    /// têm (ADR-0182 §3) — sem o carimbo, a escrita solta depois empataria e não entraria.
    #[must_use]
    pub fn snapshot(&self, board: &Board) -> Snapshot {
        let mut stamps = self.stamps.clone();
        let bytes = match self.masked(board) {
            None => BoardSet::board_bytes_lean(board),
            Some((b, at)) => {
                stamps.extend(at);
                BoardSet::board_bytes_lean(&b)
            }
        };
        Snapshot {
            board: bytes,
            stamps: packed_stamps(&stamps),
            board_stamps: self.board_stamps.iter().map(|(k, v)| (*k, *v)).collect(),
            tokens: self.tokens.clone(),
            tops: self.tops.iter().map(|(k, v)| (*k, *v)).collect(),
            team: self.team.clone(),
            clock: self.clock,
        }
    }

    fn tick(&mut self) -> Lamport {
        self.clock += 1;
        Lamport {
            n: self.clock,
            peer: self.peer,
        }
    }

    fn stamp(&self, id: ElementId, f: Field) -> Lamport {
        self.stamps.get(&(id, f)).copied().unwrap_or_default()
    }

    fn touch(&mut self, id: ElementId, at: Lamport) {
        let top = self.tops.entry(id).or_default();
        *top = (*top).max(at);
    }

    fn remember(&mut self, w: &Write) {
        let key = match w {
            Write::Born { el, .. } => OutKey::Born(el.id),
            Write::Set { id, field, .. } => OutKey::Set(*id, *field),
            Write::Stamp { op, .. } => OutKey::Stamp(*op),
            Write::Board { field, .. } => OutKey::Board(*field),
            Write::Team { at, .. } => OutKey::Team(*at),
            Write::File { file, .. } => OutKey::File(file.id()),
        };
        self.outbox.insert(key, w.clone());
    }

    /// ⭐ O que esta pessoa mudou desde a última vez, como escritas a enviar (e as capturadas ao
    /// receber).
    pub fn local(&mut self, board: &mut Board) -> Vec<Write> {
        let mut out = std::mem::take(&mut self.outgoing);
        out.extend(self.new_files(board));
        let mut sealed = Vec::new();
        let mut reverts: Vec<(ElementId, Vec<u8>)> = Vec::new();
        let mut lists = Vec::new();
        for (id, before) in board.doc.take_changes() {
            let Some(now) = board.doc.get_any(id).cloned() else {
                continue;
            };
            let Some(before) = before else {
                let at = self.tick();
                let mut born = now;
                let text = self.seal_born(&mut born, at);
                for f in Field::ALL.into_iter().filter(|f| born.read(*f).is_some()) {
                    self.stamps.insert((id, f), at);
                }
                self.tokens.seed(&born);
                self.touch(id, at);
                if let Some(w) = text {
                    let at = w.at();
                    self.stamps.insert((id, Field::Text), at);
                    self.touch(id, at);
                    self.sealed.mine.insert((id, Field::Text), at);
                    sealed.push(w);
                }
                out.push(Write::Born { el: born, at });
                continue;
            };
            let fields = before.changed_fields(&now);
            if fields.is_empty() {
                // Uma operação que não mudou nada subiu a `version` só aqui: volta a de antes.
                board.doc.set_mark(id, before.version, before.nonce);
                continue;
            }
            let at = self.tick();
            for f in fields {
                if f.is_counter() {
                    self.tokens.seed(&before);
                    for stamp in Stamp::ALL {
                        let delta = count(&now, stamp) - count(&before, stamp);
                        for _ in 0..delta.unsigned_abs() {
                            let take = if delta < 0 {
                                let Some(t) = self.tokens.pick(id, stamp) else {
                                    break;
                                };
                                Some(t)
                            } else {
                                None
                            };
                            let op = self.tick();
                            match take {
                                Some(t) => self.tokens.take(id, stamp, t),
                                None => self.tokens.give(id, stamp, op),
                            }
                            self.touch(id, op);
                            out.push(Write::Stamp {
                                id,
                                stamp,
                                op,
                                take,
                            });
                        }
                    }
                    lists.push(id);
                } else if let Some(value) = now.read(f) {
                    let (public, hidden) = match self.seal_set(&before, &now, f, value, at) {
                        Seal::Public(w) => (Some(w), None),
                        Seal::Sealed(w) => (None, Some(w)),
                        Seal::Split(p, w) => (Some(p), Some(*w)),
                        Seal::Revert => {
                            reverts.push((id, before.read(Field::Text).unwrap_or_default()));
                            (None, None)
                        }
                    };
                    if let Some(w) = public {
                        // Um registo meu escrito às claras substitui o selado no reenvio.
                        if hidden.is_none() {
                            self.sealed.mine.remove(&(id, f));
                        }
                        self.stamps.insert((id, f), at);
                        self.touch(id, at);
                        out.push(w);
                    }
                    if let Some(w) = hidden {
                        let at = w.at();
                        self.stamps.insert((id, f), at);
                        self.touch(id, at);
                        self.sealed.mine.insert((id, f), at);
                        sealed.push(w);
                    }
                }
            }
        }
        // O texto de uma nota que não leio volta ao que era (a escrita não sai).
        for (id, old) in reverts {
            if let Some(mut el) = board.doc.get_any(id).cloned() {
                el.write(Field::Text, &old);
                let _ = BoardOp::Put(el).apply(&mut board.doc);
            }
        }
        let _ = board.doc.take_changes();
        for (field, value, old) in [
            (BoardField::Name, enc(&board.name), enc(&self.shadow.0)),
            (BoardField::Sketch, enc(&board.sketch), enc(&self.shadow.1)),
        ] {
            if old != value {
                let at = self.tick();
                self.board_stamps.insert(field, at);
                out.push(Write::Board { field, value, at });
            }
        }
        self.shadow = (board.name.clone(), board.sketch);
        let (mut clock, peer) = (self.clock, self.peer);
        let mut team = self.team.capture(&mut board.team, &self.team_shadow, || {
            clock += 1;
            Lamport { n: clock, peer }
        });
        // Uma versão guardada com algo meu selado leva o quadro como todos o vêem (aqui também).
        for (_, w) in &mut team {
            if let TeamWrite::Version(v) = w {
                self.mask_version(v);
                if let Some(mine) = board.team.versions.iter_mut().find(|m| m.id == v.id) {
                    mine.bytes.clone_from(&v.bytes);
                }
            }
        }
        self.clock = clock;
        self.team_shadow = Shadow::of(&board.team);
        out.extend(team.into_iter().map(|(at, w)| Write::Team { w, at }));
        for id in lists {
            self.settle_list(board, id);
        }
        let ids: BTreeSet<ElementId> = out.iter().chain(&sealed).filter_map(Write::id).collect();
        for id in ids {
            self.mark(board, id);
        }
        for w in out.iter().chain(&sealed) {
            self.remember(w);
        }
        self.sealed.out.extend(sealed);
        out
    }

    /// ⭐ Funde escritas que chegaram. O que esta pessoa tinha mudado e ainda não saiu fica para
    /// o próximo [`Self::local`].
    pub fn receive(&mut self, board: &mut Board, writes: &[Write]) {
        let mine = self.local(board);
        self.outgoing.extend(mine);
        let mut cache: BTreeMap<ElementId, Element> = BTreeMap::new();
        let (team, writes): (Vec<&Write>, Vec<&Write>) =
            writes.iter().partition(|w| matches!(w, Write::Team { .. }));
        let team: Vec<_> = team
            .into_iter()
            .filter_map(|w| match w {
                Write::Team { w, at } => {
                    self.clock = self.clock.max(at.n);
                    Some((*at, w.clone()))
                }
                _ => None,
            })
            .collect();
        self.team.receive(&mut board.team, &team);
        self.team_shadow = Shadow::of(&board.team);
        let mut todo: Vec<Write> = std::mem::take(&mut self.pending);
        todo.extend(
            writes
                .into_iter()
                .filter(|w| !self.file_in(board, w))
                .cloned(),
        );
        loop {
            let before = todo.len();
            let mut again = Vec::new();
            for w in todo {
                if !self.merge(board, &mut cache, &w) {
                    again.push(w);
                }
            }
            todo = again;
            if todo.is_empty() || todo.len() == before {
                break;
            }
        }
        self.pending = todo;
        let ids: Vec<ElementId> = cache.keys().copied().collect();
        for el in cache.into_values() {
            // Uma escrita velha não muda nada: não se reescreve (nem sobe a revisão).
            if board.doc.get_any(el.id) != Some(&el) {
                let _ = BoardOp::Put(el).apply(&mut board.doc);
            }
        }
        let _ = board.doc.take_changes();
        for id in ids {
            self.mark(board, id);
        }
        self.shadow = (board.name.clone(), board.sketch);
    }

    /// Funde UMA escrita. `false` = o elemento ainda não nasceu aqui (fica pendente).
    fn merge(
        &mut self,
        board: &mut Board,
        cache: &mut BTreeMap<ElementId, Element>,
        w: &Write,
    ) -> bool {
        let at = w.at();
        self.clock = self.clock.max(at.n);
        if let Write::Board { field, value, at } = w {
            if *at > self.board_stamps.get(field).copied().unwrap_or_default() {
                self.board_stamps.insert(*field, *at);
                match field {
                    BoardField::Name => board.name = dec(value).unwrap_or_default(),
                    BoardField::Sketch => board.sketch = dec(value).unwrap_or_default(),
                }
            }
            return true;
        }
        let id = w.id().expect("uma escrita de elemento");
        if let std::collections::btree_map::Entry::Vacant(slot) = cache.entry(id) {
            match board.doc.get_any(id) {
                Some(el) => {
                    slot.insert(el.clone());
                }
                None => {
                    let Write::Born { el, at } = w else {
                        return false;
                    };
                    for f in Field::ALL.into_iter().filter(|f| el.read(*f).is_some()) {
                        self.stamps.insert((id, f), *at);
                    }
                    self.tokens.seed(el);
                    self.touch(id, *at);
                    slot.insert(el.clone());
                    return true;
                }
            }
        }
        let el = cache.get_mut(&id).expect("posto acima");
        match w {
            Write::Born { el: born, at } => {
                for f in Field::ALL.into_iter().filter(|f| !f.is_counter()) {
                    if *at > self.stamp(id, f)
                        && let Some(v) = born.read(f)
                    {
                        el.write(f, &v);
                        self.stamps.insert((id, f), *at);
                    }
                }
            }
            Write::Set {
                field, value, at, ..
            } => {
                if *at > self.stamp(id, *field) {
                    el.write(*field, value);
                    self.stamps.insert((id, *field), *at);
                }
                self.overtaken(id, *field, *at);
            }
            Write::Stamp {
                stamp, op, take, ..
            } => {
                self.tokens.seed(el);
                match take {
                    Some(t) => self.tokens.take(id, *stamp, *t),
                    None => self.tokens.give(id, *stamp, *op),
                }
                el.org_mut().stamps = self.tokens.list(id);
                if el.org.as_deref().is_some_and(|o| *o == Default::default()) {
                    el.org = None;
                }
            }
            Write::Board { .. } | Write::Team { .. } | Write::File { .. } => {
                unreachable!("tratado acima")
            }
        }
        self.touch(id, at);
        true
    }

    /// A lista de carimbos de `id` como as fichas a ditam (a ordem de nascença).
    fn settle_list(&mut self, board: &mut Board, id: ElementId) {
        let Some(el) = board.doc.get_any(id) else {
            return;
        };
        let list = self.tokens.list(id);
        if list.as_slice() == el.stamps() {
            return;
        }
        let mut el = el.clone();
        el.org_mut().stamps = list;
        if el.org.as_deref().is_some_and(|o| *o == Default::default()) {
            el.org = None;
        }
        let _ = BoardOp::Put(el).apply(&mut board.doc);
        let _ = board.doc.take_changes();
    }

    /// A `version`/`nonce` do elemento: o maior carimbo que o tocou (igual em todos os pares).
    fn mark(&self, board: &mut Board, id: ElementId) {
        if let Some(top) = self.tops.get(&id) {
            board.doc.set_mark(id, top.n, top.peer as u32);
        }
    }

    /// Esqueço as minhas escritas por reenviar (o anfitrião confirmou-as todas).
    pub fn forget_outbox(&mut self) {
        self.outbox.clear();
    }

    /// Escritas à espera de um elemento que ainda não nasceu aqui.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.pending.len() + self.team.pending()
    }
}

/// Os carimbos por elemento: o mais comum e os que diferem dele (os registos sem carimbo — que o
/// tipo não tem, ou nunca escritos — diferem com o carimbo zero).
fn packed_stamps(stamps: &BTreeMap<(ElementId, Field), Lamport>) -> Vec<PackedStamps> {
    let mut by: BTreeMap<ElementId, BTreeMap<Field, Lamport>> = BTreeMap::new();
    for ((id, f), l) in stamps {
        by.entry(*id).or_default().insert(*f, *l);
    }
    by.into_iter()
        .map(|(id, fields)| {
            let mut count: BTreeMap<Lamport, usize> = BTreeMap::new();
            for l in fields.values() {
                *count.entry(*l).or_default() += 1;
            }
            let common = count
                .iter()
                .max_by_key(|(l, n)| (**n, std::cmp::Reverse(**l)))
                .map(|(l, _)| *l)
                .unwrap_or_default();
            let odd = Field::ALL
                .into_iter()
                .filter_map(|f| {
                    let l = fields.get(&f).copied().unwrap_or_default();
                    (l != common).then_some((f, l))
                })
                .collect();
            (id, common, odd)
        })
        .collect()
}

fn count(el: &Element, s: Stamp) -> i64 {
    el.stamps()
        .iter()
        .find(|c| c.stamp == s)
        .map_or(0, |c| i64::from(c.count))
}

fn enc<T: Serialize>(v: &T) -> Vec<u8> {
    postcard::to_allocvec(v).expect("postcard escreve qualquer valor do modelo")
}

fn dec<T: serde::de::DeserializeOwned>(b: &[u8]) -> Option<T> {
    postcard::from_bytes(b).ok()
}

#[cfg(test)]
#[path = "replica_tests.rs"]
mod tests;
