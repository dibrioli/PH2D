//! **O retrato de quem entra**, compacto e em blocos (W5, lote S2 — tabela em
//! `docs/TrelloClone/loop/MEDICOES.md`).
//!
//! Sem perda: os carimbos agrupam-se por alvo (o Kanban e o registo escrevem-se UMA vez, não um por
//! campo — um id sorteado são 10 bytes) e cada carimbo igual ao mais comum do alvo (o de quando ele
//! nasceu, quase sempre) é um byte. O retrato grande viaja em [`KMsg::SnapshotPart`]s de
//! [`SNAPSHOT_BLOCK`] bytes.

use std::collections::BTreeMap;

use ph2d_board_sync::Lamport;
use ph2d_kanban_model::KanbanId;
use serde::{Deserialize, Serialize};

use crate::msg::{Elem, ElemKey, FieldKey, KMsg, Target};

/// O tamanho de um bloco do retrato. Recurso: a moldura do transporte (`ph2d_board_net::MAX_FRAME`,
/// 256 MiB, cifrada e copiada inteira na memória de cada lado) e o progresso visível de quem entra —
/// medido: 20 000 cartões nascidos na sessão = ~3 MB compactos (MEDICOES.md, W5 S2).
pub const SNAPSHOT_BLOCK: usize = 1 << 20;

/// Um carimbo: o mais comum do alvo, ou outro.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum At {
    Base,
    Is(Lamport),
}

/// Os carimbos de UM alvo (o cabeçalho ou um registo).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Packed {
    pub target: Target,
    pub base: Lamport,
    /// O maior carimbo que tocou o alvo (`None` = nenhum).
    pub top: Option<At>,
    /// `(campo, carimbo)`.
    pub stamps: Vec<(u8, At)>,
    /// `(campo, chave, valor, posição de entrada, carimbo)`.
    pub elems: Vec<PackedElem>,
}

/// `(campo, chave, valor, posição de entrada, carimbo)` de um elemento.
pub(crate) type PackedElem = (u8, Vec<u8>, Option<Vec<u8>>, u32, At);

/// Os carimbos de todos os Kanbans, agrupados por Kanban e por alvo.
pub(crate) type Stamps = Vec<(KanbanId, Vec<Packed>)>;

fn at(base: Lamport, l: Lamport) -> At {
    if l == base { At::Base } else { At::Is(l) }
}

fn of(base: Lamport, a: At) -> Lamport {
    match a {
        At::Base => base,
        At::Is(l) => l,
    }
}

/// Agrupa os três mapas da réplica por `(Kanban, alvo)`.
pub(crate) fn pack(
    stamps: &BTreeMap<FieldKey, Lamport>,
    elems: &BTreeMap<ElemKey, Elem>,
    tops: &BTreeMap<(KanbanId, Target), Lamport>,
) -> Stamps {
    type Group<'a> = (
        Vec<(u8, Lamport)>,
        Vec<(u8, &'a Vec<u8>, &'a Elem)>,
        Option<Lamport>,
    );
    let mut groups: BTreeMap<(KanbanId, Target), Group<'_>> = BTreeMap::new();
    for ((k, t, ix), l) in stamps {
        groups.entry((*k, *t)).or_default().0.push((*ix, *l));
    }
    for ((k, t, ix, key), e) in elems {
        groups.entry((*k, *t)).or_default().1.push((*ix, key, e));
    }
    for ((k, t), l) in tops {
        groups.entry((*k, *t)).or_default().2 = Some(*l);
    }
    let mut out: Stamps = Vec::new();
    for ((k, t), (st, el, top)) in groups {
        let mut count: BTreeMap<Lamport, usize> = BTreeMap::new();
        let all = st.iter().map(|s| s.1).chain(el.iter().map(|e| e.2.at));
        for l in all.chain(top) {
            *count.entry(l).or_default() += 1;
        }
        let base = count
            .iter()
            .max_by_key(|(l, n)| (**n, std::cmp::Reverse(**l)))
            .map(|(l, _)| *l)
            .unwrap_or_default();
        let p = Packed {
            target: t,
            base,
            top: top.map(|l| at(base, l)),
            stamps: st.iter().map(|(ix, l)| (*ix, at(base, *l))).collect(),
            elems: el
                .iter()
                .map(|(ix, key, e)| {
                    (
                        (*ix),
                        (*key).clone(),
                        e.value.clone(),
                        e.idx,
                        at(base, e.at),
                    )
                })
                .collect(),
        };
        match out.last_mut() {
            Some((last, v)) if *last == k => v.push(p),
            _ => out.push((k, vec![p])),
        }
    }
    out
}

/// O inverso de [`pack`]: põe os carimbos nos três mapas (por cima do que lá houver).
pub(crate) fn unpack(
    packed: &Stamps,
    stamps: &mut BTreeMap<FieldKey, Lamport>,
    elems: &mut BTreeMap<ElemKey, Elem>,
    tops: &mut BTreeMap<(KanbanId, Target), Lamport>,
) {
    for (k, ps) in packed {
        for p in ps {
            for (ix, a) in &p.stamps {
                stamps.insert((*k, p.target, *ix), of(p.base, *a));
            }
            for (ix, key, value, idx, a) in &p.elems {
                let e = Elem {
                    value: value.clone(),
                    at: of(p.base, *a),
                    idx: *idx,
                };
                elems.insert((*k, p.target, *ix, key.clone()), e);
            }
            if let Some(a) = p.top {
                tops.insert((*k, p.target), of(p.base, a));
            }
        }
    }
}

/// O retrato numa mensagem só, ou em blocos de `block` bytes ([`SNAPSHOT_BLOCK`]).
pub(crate) fn messages(snap: crate::KSnapshot, block: usize) -> Vec<KMsg> {
    let bytes = postcard::to_allocvec(&snap).expect("postcard escreve o retrato");
    if bytes.len() <= block {
        return vec![KMsg::Snapshot(Box::new(snap))];
    }
    let of = u32::try_from(bytes.len().div_ceil(block)).unwrap_or(u32::MAX);
    bytes
        .chunks(block)
        .enumerate()
        .map(|(i, b)| KMsg::SnapshotPart {
            part: u32::try_from(i).unwrap_or(u32::MAX),
            of,
            bytes: b.to_vec(),
        })
        .collect()
}

/// Os blocos de um retrato a chegar (pela ordem, mas cada um diz o seu lugar).
#[derive(Clone, Debug, Default)]
pub(crate) struct Parts {
    of: u32,
    got: BTreeMap<u32, Vec<u8>>,
}

impl Parts {
    /// Guarda um bloco; devolve o retrato quando o último chegou. Um bloco de outro retrato (outro
    /// `of`) recomeça.
    pub(crate) fn offer(&mut self, part: u32, of: u32, bytes: Vec<u8>) -> Option<crate::KSnapshot> {
        if of != self.of {
            *self = Self {
                of,
                got: BTreeMap::new(),
            };
        }
        if part >= of {
            return None;
        }
        self.got.insert(part, bytes);
        if self.got.len() < of as usize {
            return None;
        }
        let all: Vec<u8> = std::mem::take(&mut self.got)
            .into_values()
            .flatten()
            .collect();
        self.of = 0;
        postcard::from_bytes(&all).ok()
    }

    /// Está a meio de um retrato?
    pub(crate) fn busy(&self) -> bool {
        !self.got.is_empty()
    }
}

#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod tests;
