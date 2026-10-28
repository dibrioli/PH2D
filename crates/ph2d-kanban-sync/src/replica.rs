//! **A réplica** dos Kanbans de uma sessão: os carimbos de cada campo, a captura do que esta pessoa
//! mudou (por diferença com a SOMBRA — o último estado sincronizado — para nenhuma porta de mudança
//! escapar) e a fusão do que chega. A lei é a do Quadro ([`wins`]): comutativa, associativa e
//! idempotente.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_sync::{Clock, Lamport, PeerId, wins};
use ph2d_kanban_model::{FileId, Kanban, KanbanId, KanbanOp, KanbanSet, Record, RecordId};

use crate::msg::{Elem, ElemKey, FieldKey, KWrite, Target};
use crate::shape::shape_of;
use crate::table::{all_ids, files_of, meta_mut};

type Fields = Vec<(&'static str, Vec<u8>)>;

/// A chave de uma escrita minha por reenviar (depois de uma queda): um campo guarda só a última.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OutKey {
    Born(KanbanId, RecordId),
    Set(FieldKey),
    Elem(ElemKey),
    Shared(KanbanId),
    File(FileId),
}

#[derive(Clone, Debug)]
pub struct KanbanReplica {
    pub peer: PeerId,
    pub(crate) host: bool,
    pub(crate) clock: Clock,
    pub(crate) seed: u64,
    pub(crate) minted: u64,
    /// O id da sessão → o meu.
    pub(crate) map: BTreeMap<KanbanId, KanbanId>,
    /// O último estado sincronizado de cada Kanban (pelo id da sessão).
    pub(crate) shadow: BTreeMap<KanbanId, Kanban>,
    pub(crate) stamps: BTreeMap<FieldKey, Lamport>,
    pub(crate) elems: BTreeMap<ElemKey, Elem>,
    /// O maior carimbo que tocou cada alvo — a `version` (e o desempate) dele, igual em todos.
    pub(crate) tops: BTreeMap<(KanbanId, Target), Lamport>,
    pub(crate) pending: Vec<KWrite>,
    pub(crate) outgoing: Vec<KWrite>,
    outbox: BTreeMap<OutKey, KWrite>,
    pub(crate) files: BTreeSet<FileId>,
}

pub(crate) fn fields_of(k: &Kanban, t: Target) -> Option<Fields> {
    match t {
        Target::Header => Some(k.header.fields()),
        Target::Record(id) => k.record(id).map(|r| r.fields()),
    }
}

/// Põe campos no alvo pela porta do modelo (a `Put` sobe o contador de ids como em toda a parte).
fn write_fields(k: &mut Kanban, t: Target, changes: &[(&str, &[u8])]) {
    if changes.is_empty() {
        return;
    }
    let op = match t {
        Target::Header => k.header.with_fields(changes).map(KanbanOp::Header),
        Target::Record(id) => k
            .record(id)
            .and_then(|r| r.with_fields(changes))
            .map(KanbanOp::put),
    };
    if let Some(op) = op {
        let _ = op.apply(k);
    }
}

impl KanbanReplica {
    pub(crate) fn blank(peer: PeerId, seed: u64, host: bool) -> Self {
        Self {
            peer,
            host,
            clock: Clock::new(peer),
            seed,
            minted: 0,
            map: BTreeMap::new(),
            shadow: BTreeMap::new(),
            stamps: BTreeMap::new(),
            elems: BTreeMap::new(),
            tops: BTreeMap::new(),
            pending: Vec::new(),
            outgoing: Vec::new(),
            outbox: BTreeMap::new(),
            files: BTreeSet::new(),
        }
    }

    /// O id local do Kanban da sessão `wire`.
    #[must_use]
    pub fn local_id(&self, wire: KanbanId) -> Option<KanbanId> {
        self.map.get(&wire).copied()
    }

    /// O id da sessão do meu Kanban `local` (`None` = não está partilhado).
    #[must_use]
    pub fn wire_id(&self, local: KanbanId) -> Option<KanbanId> {
        self.map.iter().find(|(_, l)| **l == local).map(|(w, _)| *w)
    }

    /// Os Kanbans da sessão: `(id da sessão, id local)`.
    #[must_use]
    pub fn kanbans(&self) -> Vec<(KanbanId, KanbanId)> {
        self.map.iter().map(|(w, l)| (*w, *l)).collect()
    }

    fn touch(&mut self, wire: KanbanId, t: Target, at: Lamport) {
        let top = self.tops.entry((wire, t)).or_default();
        *top = (*top).max(at);
    }

    fn remember(&mut self, w: &KWrite) {
        let key = match w {
            KWrite::Born { kanban, rec, .. } => OutKey::Born(*kanban, rec.id()),
            KWrite::Set {
                kanban,
                target,
                field,
                ..
            } => OutKey::Set((*kanban, *target, *field)),
            KWrite::Elem {
                kanban,
                target,
                field,
                key,
                ..
            } => OutKey::Elem((*kanban, *target, *field, key.clone())),
            KWrite::Shared { kanban, .. } => OutKey::Shared(*kanban),
            KWrite::File(f) => OutKey::File(f.id()),
        };
        self.outbox.insert(key, w.clone());
    }

    /// As minhas escritas por confirmar, para voltar depois de uma queda.
    pub(crate) fn mine(&self) -> Vec<KWrite> {
        self.outbox.values().cloned().collect()
    }

    /// Esqueço as minhas escritas por reenviar (o anfitrião confirmou-as).
    pub fn forget_outbox(&mut self) {
        self.outbox.clear();
    }

    /// Escritas à espera de um registo (ou Kanban) que ainda não chegou aqui.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    /// Os registos de elemento de um campo-conjunto, criados (com o carimbo zero) a partir do valor
    /// de `fields` se ainda não há nenhum: o valor de antes de a sessão o mexer, igual em todos.
    fn seed_elems(&mut self, wire: KanbanId, t: Target, ix: u8, bytes: &[u8], name: &str) {
        let lo = (wire, t, ix, Vec::new());
        let has = self
            .elems
            .range(lo..)
            .next()
            .is_some_and(|((w, tt, i, _), _)| (*w, *tt, *i) == (wire, t, ix));
        if has {
            return;
        }
        let Some(s) = shape_of(name, bytes) else {
            return;
        };
        for (pos, (key, value)) in (s.split)(bytes).unwrap_or_default().into_iter().enumerate() {
            let e = Elem {
                value: Some(value),
                at: Lamport::default(),
                idx: u32::try_from(pos).unwrap_or(u32::MAX),
            };
            self.elems.insert((wire, t, ix, key), e);
        }
    }

    /// Todos os campos de um alvo que nasceu, com o carimbo `at`.
    fn born(&mut self, wire: KanbanId, t: Target, fields: &Fields, at: Lamport) {
        for (ix, (name, b)) in fields.iter().enumerate() {
            let ix = u8::try_from(ix).expect("um registo tem menos de 256 campos");
            match shape_of(name, b) {
                Some(s) => {
                    for (pos, (key, value)) in
                        (s.split)(b).unwrap_or_default().into_iter().enumerate()
                    {
                        let e = Elem {
                            value: Some(value),
                            at,
                            idx: u32::try_from(pos).unwrap_or(u32::MAX),
                        };
                        self.offer(wire, t, ix, key, e);
                    }
                }
                None => {
                    let s = self.stamps.entry((wire, t, ix)).or_default();
                    *s = (*s).max(at);
                }
            }
        }
        self.touch(wire, t, at);
    }

    fn offer(&mut self, wire: KanbanId, t: Target, ix: u8, key: Vec<u8>, e: Elem) -> bool {
        let slot = self.elems.get(&(wire, t, ix, key.clone()));
        if slot.is_none_or(|cur| wins(e.at, cur.at)) {
            self.elems.insert((wire, t, ix, key), e);
            return true;
        }
        false
    }

    /// As escritas de um alvo que mudou (de `before` para `now`), todas com o carimbo `at`.
    fn diff(
        &mut self,
        wire: KanbanId,
        t: Target,
        before: &Fields,
        now: &Fields,
        at: Lamport,
    ) -> Vec<KWrite> {
        let mut out = Vec::new();
        for (ix, ((name, b), (_, n))) in before.iter().zip(now).enumerate() {
            if b == n {
                continue;
            }
            let ix = u8::try_from(ix).expect("um registo tem menos de 256 campos");
            let shaped = shape_of(name, n).zip(shape_of(name, b));
            let Some((s, _)) = shaped else {
                self.stamps.insert((wire, t, ix), at);
                out.push(KWrite::Set {
                    kanban: wire,
                    target: t,
                    field: ix,
                    value: n.clone(),
                    at,
                });
                continue;
            };
            self.seed_elems(wire, t, ix, b, name);
            let old: BTreeMap<Vec<u8>, Vec<u8>> =
                (s.split)(b).unwrap_or_default().into_iter().collect();
            let new = (s.split)(n).unwrap_or_default();
            let mut put = Vec::new();
            for (pos, (key, value)) in new.iter().enumerate() {
                if old.get(key) != Some(value) {
                    put.push((key.clone(), Some(value.clone()), pos));
                }
            }
            for key in old.keys().filter(|k| !new.iter().any(|(x, _)| x == *k)) {
                put.push((key.clone(), None, 0));
            }
            for (key, value, pos) in put {
                let idx = u32::try_from(pos).unwrap_or(u32::MAX);
                let e = Elem {
                    value: value.clone(),
                    at,
                    idx,
                };
                self.elems.insert((wire, t, ix, key.clone()), e);
                out.push(KWrite::Elem {
                    kanban: wire,
                    target: t,
                    field: ix,
                    key,
                    value,
                    idx,
                    at,
                });
            }
        }
        if !out.is_empty() {
            self.touch(wire, t, at);
        }
        out
    }

    /// ⭐ O que esta pessoa mudou nos Kanbans da sessão desde a última vez, como escritas a enviar
    /// (e as capturadas ao receber).
    pub fn local(&mut self, set: &mut KanbanSet) -> Vec<KWrite> {
        self.arm(set);
        let mut out = std::mem::take(&mut self.outgoing);
        let pairs: Vec<(KanbanId, KanbanId)> = self.kanbans();
        for (wire, local) in pairs {
            let Some(k) = set.get(local) else {
                continue;
            };
            let Some(shadow) = self.shadow.get(&wire) else {
                continue;
            };
            let mut ws = Vec::new();
            let mut touched = Vec::new();
            let (hb, hn) = (shadow.header.fields(), k.header.fields());
            if hb != hn {
                let at = self.clock.tick();
                ws.extend(self.diff(wire, Target::Header, &hb, &hn, at));
                touched.push(Target::Header);
            }
            let shadow = self.shadow.get(&wire).expect("acima");
            let mut changed: Vec<(RecordId, Option<Record>, Record)> = Vec::new();
            // Ida e volta ao mesmo conteúdo (marcar e desmarcar) subiu só a `version` daqui: volta a
            // de antes, a que todos têm.
            let mut back: Vec<RecordId> = Vec::new();
            let header_back =
                (hb == hn && shadow.header != k.header).then(|| shadow.header.version);
            for id in all_ids(k) {
                let now = k.record(id).expect("o id veio da tabela");
                match shadow.record(id) {
                    Some(b) if b == now => {}
                    Some(b) if b.same_content(&now) => back.push(id),
                    b => changed.push((id, b, now)),
                }
            }
            if header_back.is_some() || !back.is_empty() {
                let k = set.doc_mut(local).expect("acima");
                if let Some(v) = header_back {
                    k.header.version = v;
                }
                let shadow = self.shadow.get_mut(&wire).expect("acima");
                for id in back {
                    if let (Some(m), Some(was)) = (meta_mut(k, id), meta_mut(shadow, id)) {
                        *m = *was;
                    }
                }
            }
            for (id, before, now) in changed {
                let t = Target::Record(id);
                let at = self.clock.tick();
                let nf = now.fields();
                match before {
                    None => {
                        self.born(wire, t, &nf, at);
                        ws.push(KWrite::Born {
                            kanban: wire,
                            rec: now,
                            at,
                        });
                    }
                    Some(b) => ws.extend(self.diff(wire, t, &b.fields(), &nf, at)),
                }
                touched.push(t);
            }
            if ws.is_empty() && touched.is_empty() {
                continue;
            }
            out.extend(self.new_files(set, local));
            out.extend(ws);
            for t in touched {
                self.settle(set, wire, t);
            }
        }
        for w in &out {
            self.remember(w);
        }
        out
    }

    /// Os ficheiros que o Kanban `local` usa e ainda não viajaram.
    fn new_files(&mut self, set: &KanbanSet, local: KanbanId) -> Vec<KWrite> {
        let Some(k) = set.get(local) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for id in files_of(k) {
            if let Some(f) = set.file(id)
                && self.files.insert(id)
            {
                out.push(KWrite::File(f.clone()));
            }
        }
        out
    }

    /// Os ficheiros de todos os Kanbans da sessão (para quem entra).
    pub(crate) fn all_files(&mut self, set: &KanbanSet) -> Vec<KWrite> {
        let mut out = Vec::new();
        for (_, local) in self.kanbans() {
            if let Some(k) = set.get(local) {
                for id in files_of(k) {
                    self.files.insert(id);
                    if let Some(f) = set.file(id) {
                        out.push(KWrite::File(f.clone()));
                    }
                }
            }
        }
        out
    }

    /// Depois de mexer num alvo: os campos-conjunto como os registos os ditam (a ordem de entrada),
    /// a `version` pelo maior carimbo, e a sombra igual ao documento.
    fn settle(&mut self, set: &mut KanbanSet, wire: KanbanId, t: Target) {
        let Some(local) = self.local_id(wire) else {
            return;
        };
        let Some(k) = set.doc_mut(local) else {
            return;
        };
        if let Some(fields) = fields_of(k, t) {
            let mut changes: Vec<(&str, Vec<u8>)> = Vec::new();
            for (ix, (name, b)) in fields.iter().enumerate() {
                let Some(s) = shape_of(name, b) else {
                    continue;
                };
                let ix = u8::try_from(ix).expect("menos de 256 campos");
                let lo = (wire, t, ix, Vec::new());
                let mut live: Vec<(&Vec<u8>, &Elem)> = self
                    .elems
                    .range(lo..)
                    .take_while(|((w, tt, i, _), _)| (*w, *tt, *i) == (wire, t, ix))
                    .filter(|(_, e)| e.value.is_some())
                    .map(|((_, _, _, key), e)| (key, e))
                    .collect();
                if live.is_empty() && !self.has_elems(wire, t, ix) {
                    continue;
                }
                if s.ordered {
                    live.sort_by(|a, b| (a.1.at, a.1.idx, a.0).cmp(&(b.1.at, b.1.idx, b.0)));
                }
                let es: Vec<(&[u8], &[u8])> = live
                    .iter()
                    .map(|(key, e)| (key.as_slice(), e.value.as_deref().unwrap_or_default()))
                    .collect();
                if let Some(joined) = (s.join)(&es)
                    && joined != *b
                {
                    changes.push((name, joined));
                }
            }
            let changes: Vec<(&str, &[u8])> =
                changes.iter().map(|(n, b)| (*n, b.as_slice())).collect();
            write_fields(k, t, &changes);
        }
        if let Some(top) = self.tops.get(&(wire, t)).copied() {
            match t {
                Target::Header => k.header.version = top.n,
                Target::Record(id) => {
                    if let Some(m) = meta_mut(k, id) {
                        m.version = top.n;
                        m.nonce = top.peer as u32;
                    }
                }
            }
        }
        let shadow = self
            .shadow
            .get_mut(&wire)
            .expect("todo Kanban da sessão tem sombra");
        match t {
            Target::Header => shadow.header = k.header.clone(),
            Target::Record(id) => {
                if let Some(r) = k.record(id) {
                    let _ = KanbanOp::put(r).apply(shadow);
                }
                // A sombra leva a `version` exacta (a `Put` sobe a dela à sua maneira).
                if let (Some(m), Some(was)) = (meta_mut(shadow, id), meta_mut(k, id)) {
                    *m = *was;
                }
            }
        }
    }

    fn has_elems(&self, wire: KanbanId, t: Target, ix: u8) -> bool {
        self.elems
            .range((wire, t, ix, Vec::new())..)
            .next()
            .is_some_and(|((w, tt, i, _), _)| (*w, *tt, *i) == (wire, t, ix))
    }

    /// ⭐ Funde escritas que chegaram. O que esta pessoa tinha mudado e ainda não saiu fica para o
    /// próximo [`Self::local`].
    pub fn receive(&mut self, set: &mut KanbanSet, writes: &[KWrite]) {
        let mine = self.local(set);
        self.outgoing.extend(mine);
        let mut todo = std::mem::take(&mut self.pending);
        todo.extend(writes.iter().cloned());
        let mut dirty: BTreeSet<(KanbanId, Target)> = BTreeSet::new();
        loop {
            let before = todo.len();
            let mut again = Vec::new();
            for w in todo {
                if !self.merge(set, &w, &mut dirty) {
                    again.push(w);
                }
            }
            todo = again;
            if todo.is_empty() || todo.len() == before {
                break;
            }
        }
        self.pending = todo;
        for (wire, t) in dirty {
            self.settle(set, wire, t);
        }
    }

    /// Funde UMA escrita. `false` = o registo (ou o Kanban) ainda não chegou aqui.
    fn merge(
        &mut self,
        set: &mut KanbanSet,
        w: &KWrite,
        dirty: &mut BTreeSet<(KanbanId, Target)>,
    ) -> bool {
        if let Some(at) = w.at() {
            self.clock.see(at);
        }
        match w {
            KWrite::File(f) => {
                let id = set.put_file(f.bytes.0.clone(), f.size);
                self.files.insert(id);
                true
            }
            KWrite::Shared { kanban, bytes } => {
                if !self.map.contains_key(kanban)
                    && let Ok(k) = postcard::from_bytes::<Kanban>(bytes)
                {
                    self.adopt_kanban(set, *kanban, k);
                }
                true
            }
            KWrite::Born { kanban, rec, at } => {
                let Some(k) = self.local_id(*kanban).and_then(|l| set.doc_mut(l)) else {
                    return false;
                };
                let t = Target::Record(rec.id());
                let fields = rec.fields();
                if k.record(rec.id()).is_none() {
                    let _ = KanbanOp::put(rec.clone()).apply(k);
                    self.born(*kanban, t, &fields, *at);
                } else {
                    let now = k.record(rec.id()).map(|r| r.fields()).unwrap_or_default();
                    let mut changes: Vec<(&str, Vec<u8>)> = Vec::new();
                    for (ix, (name, b)) in fields.iter().enumerate() {
                        let ix = u8::try_from(ix).expect("menos de 256 campos");
                        if let Some(s) = shape_of(name, b) {
                            if let Some((_, cur)) = now.get(usize::from(ix)) {
                                self.seed_elems(*kanban, t, ix, cur, name);
                            }
                            for (pos, (key, value)) in
                                (s.split)(b).unwrap_or_default().into_iter().enumerate()
                            {
                                let idx = u32::try_from(pos).unwrap_or(u32::MAX);
                                let e = Elem {
                                    value: Some(value),
                                    at: *at,
                                    idx,
                                };
                                self.offer(*kanban, t, ix, key, e);
                            }
                        } else if wins(*at, self.stamp(*kanban, t, ix)) {
                            self.stamps.insert((*kanban, t, ix), *at);
                            changes.push((name, b.clone()));
                        }
                    }
                    let k = self
                        .local_id(*kanban)
                        .and_then(|l| set.doc_mut(l))
                        .expect("acima");
                    let changes: Vec<(&str, &[u8])> =
                        changes.iter().map(|(n, b)| (*n, b.as_slice())).collect();
                    write_fields(k, t, &changes);
                    self.touch(*kanban, t, *at);
                }
                dirty.insert((*kanban, t));
                true
            }
            KWrite::Set {
                kanban,
                target,
                field,
                value,
                at,
            } => {
                let Some(k) = self.local_id(*kanban).and_then(|l| set.doc_mut(l)) else {
                    return false;
                };
                let Some(fields) = fields_of(k, *target) else {
                    return false;
                };
                if wins(*at, self.stamp(*kanban, *target, *field))
                    && let Some((name, _)) = fields.get(usize::from(*field))
                {
                    self.stamps.insert((*kanban, *target, *field), *at);
                    write_fields(k, *target, &[(name, value)]);
                }
                self.touch(*kanban, *target, *at);
                dirty.insert((*kanban, *target));
                true
            }
            KWrite::Elem {
                kanban,
                target,
                field,
                key,
                value,
                idx,
                at,
            } => {
                let Some(k) = self.local_id(*kanban).and_then(|l| set.doc_mut(l)) else {
                    return false;
                };
                let Some(fields) = fields_of(k, *target) else {
                    return false;
                };
                if let Some((name, cur)) = fields.get(usize::from(*field)) {
                    self.seed_elems(*kanban, *target, *field, cur, name);
                }
                let e = Elem {
                    value: value.clone(),
                    at: *at,
                    idx: *idx,
                };
                self.offer(*kanban, *target, *field, key.clone(), e);
                self.touch(*kanban, *target, *at);
                dirty.insert((*kanban, *target));
                true
            }
        }
    }

    fn stamp(&self, wire: KanbanId, t: Target, ix: u8) -> Lamport {
        self.stamps.get(&(wire, t, ix)).copied().unwrap_or_default()
    }
}

#[path = "replica_join.rs"]
mod join;

#[cfg(test)]
#[path = "replica_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "replica_live_tests.rs"]
mod live_tests;
