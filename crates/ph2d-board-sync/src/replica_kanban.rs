//! J4 — a REFERÊNCIA do elemento Kanban (`Field::Kanban`) atravessa a sala no id DA SESSÃO e mora no
//! documento de cada par no id LOCAL (o meu Kanban 1 não é o 1 do anfitrião; ADR-0195 §3). A tradução
//! é aqui, na borda da réplica: o que sai leva o id da sessão, o que entra (escritas e retrato) passa
//! ao meu. Uma referência a um Kanban que ainda não chegou fica no marcador [`KANBAN_WAITING`] — nunca
//! no id de OUTRO Kanban meu — e volta ao certo quando a sala do Kanban o traz ([`Replica::map_kanbans`]).
//! Desligada (nenhum `map_kanbans`), nada se traduz: a sala sem Kanbans fica como era.
//! J7 — as VERSÕES do quadro (o histórico da equipa) levam o documento inteiro: as referências delas
//! traduzem-se igual, ao sair, ao entrar e no retrato; a que espera um Kanban guarda os bytes da
//! sessão e resolve-se quando ele chega. Um Kanban fora da sala sai como o marcador, nunca como o id
//! local de quem a guardou (que no outro par é OUTRO Kanban).

use std::collections::BTreeMap;

use ph2d_board_model::{Board, BoardOp, Element, ElementId, Field, KanbanId, Version, VersionId};

use super::{Replica, Write};
use crate::team::TeamWrite;

/// A referência de um elemento cujo Kanban ainda não chegou: um id que nenhum contador alcança e
/// que o sorteio da sessão (bit alto ligado) nunca dá.
pub const KANBAN_WAITING: KanbanId = KanbanId((1 << 63) - 1);

#[derive(Clone, Debug, Default)]
pub(super) struct KanbanIds {
    on: bool,
    /// Sessão → meu, e o contrário.
    local: BTreeMap<KanbanId, KanbanId>,
    wire: BTreeMap<KanbanId, KanbanId>,
    /// Os elementos no marcador, e o id da sessão por que esperam.
    waiting: BTreeMap<ElementId, KanbanId>,
    /// J7 — as versões com um Kanban por chegar: os bytes delas no id da sessão.
    versions: BTreeMap<VersionId, Vec<u8>>,
}

fn kanban_of(el: &Element) -> Option<KanbanId> {
    el.kanban().map(|k| k.kanban)
}

fn set_kanban(el: &mut Element, id: KanbanId) {
    if let Some(k) = el.kanban_mut() {
        k.kanban = id;
    }
}

/// Os bytes de `v` com cada referência passada por `f`; `None` = nada muda (ou não se lê).
fn retarget(v: &Version, mut f: impl FnMut(KanbanId) -> KanbanId) -> Option<Vec<u8>> {
    let mut b = v.read().ok()?;
    let fix: Vec<(ElementId, KanbanId)> = b
        .doc
        .live_with_hidden()
        .filter_map(|e| {
            let k = kanban_of(e)?;
            let n = f(k);
            (n != k).then_some((e.id, n))
        })
        .collect();
    if fix.is_empty() {
        return None;
    }
    for (id, k) in fix {
        if let Some(mut el) = b.doc.get_any(id).cloned() {
            set_kanban(&mut el, k);
            let _ = BoardOp::Put(el).apply(&mut b.doc);
        }
    }
    Some(b.version_bytes())
}

impl KanbanIds {
    /// J7 — uma versão minha a sair: ids locais → da sessão; um Kanban fora da sala → o marcador.
    fn version_out(&self, v: &mut Version) {
        if let Some(bytes) = retarget(v, |l| {
            if l == KANBAN_WAITING {
                l
            } else {
                self.wire.get(&l).copied().unwrap_or(KANBAN_WAITING)
            }
        }) {
            v.bytes = bytes;
        }
    }

    /// J7 — uma versão que entrou (escrita ou retrato): ids da sessão → meus; o que ainda não chegou
    /// fica no marcador e os bytes da sessão esperam por ele.
    fn version_in(&mut self, v: &mut Version) {
        let mut missing = false;
        let wire = v.bytes.clone();
        if let Some(bytes) = retarget(v, |w| match self.local.get(&w) {
            Some(l) => *l,
            None => {
                missing |= w != KANBAN_WAITING;
                KANBAN_WAITING
            }
        }) {
            v.bytes = bytes;
        }
        if missing {
            self.versions.insert(v.id, wire);
        } else {
            self.versions.remove(&v.id);
        }
    }

    /// J7 — as versões do quadro que esperavam e cujo Kanban chegou.
    fn versions_ready(&mut self, board: &mut Board) {
        let waiting = std::mem::take(&mut self.versions);
        for (id, wire) in waiting {
            if let Some(v) = board.team.versions.iter_mut().find(|v| v.id == id) {
                v.bytes = wire;
                self.version_in(v);
            }
        }
    }

    fn to_local(&mut self, el: ElementId, wire: KanbanId) -> KanbanId {
        match self.local.get(&wire) {
            Some(l) => {
                self.waiting.remove(&el);
                *l
            }
            None => {
                self.waiting.insert(el, wire);
                KANBAN_WAITING
            }
        }
    }

    fn to_wire(&self, el: ElementId, local: KanbanId) -> KanbanId {
        if local == KANBAN_WAITING {
            return self.waiting.get(&el).copied().unwrap_or(local);
        }
        self.wire.get(&local).copied().unwrap_or(local)
    }

    /// O que sai: ids locais → da sessão.
    pub(super) fn outgoing(&self, ws: &mut [Write]) {
        if !self.on {
            return;
        }
        for w in ws {
            match w {
                Write::Born { el, .. } => {
                    if let Some(l) = kanban_of(el) {
                        set_kanban(el, self.to_wire(el.id, l));
                    }
                }
                Write::Set {
                    id, field, value, ..
                } if *field == Field::Kanban => {
                    if let Ok(l) = postcard::from_bytes::<KanbanId>(value) {
                        *value = enc(self.to_wire(*id, l));
                    }
                }
                Write::Team {
                    w: TeamWrite::Version(v),
                    ..
                } => self.version_out(v),
                _ => {}
            }
        }
    }

    /// O que entra: ids da sessão → meus (ou o marcador). `None` = nada a traduzir.
    pub(super) fn incoming(&mut self, ws: &[Write]) -> Option<Vec<Write>> {
        let touches = |w: &Write| match w {
            Write::Born { el, .. } => el.kanban().is_some(),
            Write::Set { field, .. } => *field == Field::Kanban,
            Write::Team {
                w: TeamWrite::Version(_),
                ..
            } => true,
            _ => false,
        };
        if !self.on || !ws.iter().any(touches) {
            return None;
        }
        let mut out = ws.to_vec();
        for w in &mut out {
            match w {
                Write::Born { el, .. } => {
                    if let Some(k) = kanban_of(el) {
                        let l = self.to_local(el.id, k);
                        set_kanban(el, l);
                    }
                }
                Write::Set {
                    id, field, value, ..
                } if *field == Field::Kanban => {
                    if let Ok(k) = postcard::from_bytes::<KanbanId>(value) {
                        *value = enc(self.to_local(*id, k));
                    }
                }
                Write::Team {
                    w: TeamWrite::Version(v),
                    ..
                } => self.version_in(v),
                _ => {}
            }
        }
        Some(out)
    }

    /// O quadro como os outros o têm (para o retrato): `None` = igual.
    pub(super) fn wired(&self, board: &Board) -> Option<Board> {
        if !self.on {
            return None;
        }
        let change: Vec<(ElementId, KanbanId)> = board
            .doc
            .live_with_hidden()
            .filter_map(|e| {
                let l = kanban_of(e)?;
                let w = self.to_wire(e.id, l);
                (w != l).then_some((e.id, w))
            })
            .collect();
        let versions: Vec<(usize, Vec<u8>)> = board
            .team
            .versions
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                // A que ainda espera sai como chegou (no id da sessão).
                if let Some(wire) = self.versions.get(&v.id) {
                    return Some((i, wire.clone()));
                }
                let mut out = v.clone();
                self.version_out(&mut out);
                (out.bytes != v.bytes).then_some((i, out.bytes))
            })
            .collect();
        if change.is_empty() && versions.is_empty() {
            return None;
        }
        let mut b = board.clone();
        for (id, w) in change {
            if let Some(mut el) = b.doc.get_any(id).cloned() {
                set_kanban(&mut el, w);
                let _ = BoardOp::Put(el).apply(&mut b.doc);
            }
        }
        for (i, bytes) in versions {
            b.team.versions[i].bytes = bytes;
        }
        Some(b)
    }
}

fn enc(id: KanbanId) -> Vec<u8> {
    postcard::to_allocvec(&id).expect("postcard escreve o id")
}

impl Replica {
    /// ⭐ Os Kanbans da sala: `(id da sessão, id local)` — de `ph2d_kanban_sync::KanbanReplica::kanbans`.
    /// Liga a tradução (a 1.ª vez) e põe no id certo os elementos que esperavam por um deles. Sem
    /// escrita a sair: é o MEU id do mesmo Kanban, não uma mudança.
    pub fn map_kanbans(&mut self, board: &mut Board, pairs: &[(KanbanId, KanbanId)]) {
        self.kanbans.on = true;
        for (w, l) in pairs {
            self.kanbans.local.insert(*w, *l);
            self.kanbans.wire.insert(*l, *w);
        }
        let ready: Vec<(ElementId, KanbanId)> = self
            .kanbans
            .waiting
            .iter()
            .filter_map(|(e, w)| self.kanbans.local.get(w).map(|l| (*e, *l)))
            .collect();
        self.relabel(board, &ready);
        self.kanbans.versions_ready(board);
    }

    /// O quadro acabou de chegar no retrato (entrar ou voltar): as referências estão no id da sessão.
    pub(crate) fn kanbans_arrived(&mut self, board: &mut Board) {
        if !self.kanbans.on {
            return;
        }
        let all: Vec<(ElementId, KanbanId)> = board
            .doc
            .live_with_hidden()
            .filter_map(|e| kanban_of(e).map(|k| (e.id, k)))
            .collect();
        let fix: Vec<(ElementId, KanbanId)> = all
            .into_iter()
            .map(|(e, w)| (e, self.kanbans.to_local(e, w)))
            .collect();
        self.relabel(board, &fix);
        for v in &mut board.team.versions {
            self.kanbans.version_in(v);
        }
    }

    /// Liga a tradução sem pares (o convidado, antes do retrato do Quadro chegar).
    pub(crate) fn kanbans_on(&mut self) {
        self.kanbans.on = true;
    }

    fn relabel(&mut self, board: &mut Board, fix: &[(ElementId, KanbanId)]) {
        if fix.is_empty() {
            return;
        }
        // O que esta pessoa mudou sai antes (o `take_changes` abaixo não o pode engolir).
        let mine = self.local(board);
        self.outgoing.extend(mine);
        for (id, l) in fix {
            if let Some(mut el) = board.doc.get_any(*id).cloned()
                && kanban_of(&el) != Some(*l)
            {
                set_kanban(&mut el, *l);
                let _ = BoardOp::Put(el).apply(&mut board.doc);
            }
            if *l != KANBAN_WAITING {
                self.kanbans.waiting.remove(id);
            }
        }
        let _ = board.doc.take_changes();
        for (id, _) in fix {
            self.mark(board, *id);
        }
    }

    /// Os Kanbans traduzidos (para a sessão e os testes).
    #[must_use]
    pub fn kanban_wire(&self, local: KanbanId) -> Option<KanbanId> {
        self.kanbans.wire.get(&local).copied()
    }
}

#[cfg(test)]
#[path = "replica_kanban_tests.rs"]
mod tests;
