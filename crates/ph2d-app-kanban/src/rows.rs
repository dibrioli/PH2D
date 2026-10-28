//! ⭐ **As LINHAS que a vista mostra** (W6) — por lista: os cartões e os ESPELHOS (o mesmo cartão
//! posto noutra lista, deste ou de outro Kanban), pela posição, menos os que o FILTRO da vista
//! esconde (o Trello tira-os da lista); e o nível de ENVELHECIMENTO de cada um.
//!
//! Um espelho entra na disposição com um id de cartão SEU: o número do `MirrorId` (o contador de um
//! Kanban é um só para todos os registos, por isso nunca colide com um `CardId` dele). A vista
//! guarda o mapa ([`Rows::mirrors`]): quem pinta, abre ou arrasta pergunta por ele.

use crate::front::{self, Counts, Front};
use ph2d_kanban_dates::Zone;
use ph2d_kanban_filter::{AGE_DAYS, Filter, last_activity_index, matches_at};
use ph2d_kanban_model::{
    CardId, CardRef, FracKey, Kanban, KanbanId, KanbanSet, ListId, Member, MirrorId, Moment,
};
use std::collections::BTreeMap;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// As linhas de cada lista de uma vista, num quadro de imagem.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Rows {
    /// O que se mostra, por lista e pela ordem.
    pub lists: BTreeMap<ListId, Vec<CardId>>,
    /// Quantas linhas a lista tem sem o filtro (o «1 of 3» da maquete).
    pub totals: BTreeMap<ListId, usize>,
    /// O filtro tem escolhas (as listas mostram «n of m» e a vazia diz porquê).
    pub filtered: bool,
    /// Os espelhos mostrados: o id na disposição → (o espelho, o original).
    pub mirrors: BTreeMap<CardId, (MirrorId, CardRef)>,
    /// O nível de envelhecimento `1..=3` de cada linha parada (só com o envelhecimento ligado).
    pub ages: BTreeMap<CardId, u8>,
}

/// O que a conta pede de fora: o filtro da vista, o relógio, o fuso, quem sou e se envelhece.
pub(crate) struct Ask<'a> {
    pub filter: &'a Filter,
    pub now: Moment,
    pub zone: &'a Zone,
    pub me: &'a Member,
    pub aging: bool,
}

/// O nível `0..=3` de um cartão parado desde `last` (os limiares do `ph2d-kanban-filter`).
fn level(last: Moment, now: Moment) -> u8 {
    let idle = now.0.saturating_sub(last.0);
    let n = AGE_DAYS.iter().filter(|d| idle >= **d * DAY_MS).count();
    u8::try_from(n).unwrap_or(u8::MAX)
}

impl Rows {
    /// O cartão que a linha `id` mostra (o original, se é um espelho).
    pub fn shows(&self, here: KanbanId, id: CardId) -> CardRef {
        self.mirrors
            .get(&id)
            .map_or(CardRef { kanban: here, card: id }, |(_, of)| *of)
    }

    /// A lista onde a linha `id` está, e o índice dela.
    pub fn find(&self, id: CardId) -> Option<(ListId, usize)> {
        self.lists
            .iter()
            .find_map(|(l, v)| v.iter().position(|c| *c == id).map(|i| (*l, i)))
    }
}

/// ⭐ As linhas de `k` (no conjunto `set`) para esta vista.
pub(crate) fn build(set: &KanbanSet, k: &Kanban, ask: &Ask<'_>) -> Rows {
    let filtered = !ask.filter.is_empty();
    let need_last = filtered || ask.aging;
    let mut lasts: BTreeMap<KanbanId, BTreeMap<CardId, Moment>> = BTreeMap::new();
    if need_last {
        lasts.insert(k.id, last_activity_index(k));
    }
    let mut by_list: BTreeMap<ListId, Vec<(&FracKey, u8, CardId)>> = BTreeMap::new();
    for c in k.cards.values().filter(|c| !c.meta.deleted && !c.archived) {
        by_list.entry(c.list).or_default().push((&c.pos, 0, c.id));
    }
    let mut mirrors = BTreeMap::new();
    for m in k.mirrors.values().filter(|m| !m.meta.deleted) {
        if set.card(m.of).is_none_or(|c| c.archived) {
            continue;
        }
        let id = CardId(m.id.0);
        by_list.entry(m.list).or_default().push((&m.pos, 1, id));
        mirrors.insert(id, (m.id, m.of));
        if need_last && !lasts.contains_key(&m.of.kanban)
            && let Some(src) = set.get(m.of.kanban)
        {
            lasts.insert(src.id, last_activity_index(src));
        }
    }
    let mut out = Rows {
        filtered,
        mirrors,
        ..Rows::default()
    };
    for (list, mut v) in by_list {
        v.sort();
        out.totals.insert(list, v.len());
        let mut shown = Vec::with_capacity(v.len());
        for (_, _, id) in v {
            let r = out.shows(k.id, id);
            let (Some(ck), Some(card)) = (set.get(r.kanban), set.card(r)) else {
                continue;
            };
            let last = lasts
                .get(&r.kanban)
                .and_then(|m| m.get(&r.card))
                .copied()
                .unwrap_or(card.created);
            if filtered && !matches_at(ask.filter, card, ck, k, last, ask.now, ask.zone, ask.me) {
                continue;
            }
            if ask.aging {
                let n = level(last, ask.now);
                if n > 0 {
                    out.ages.insert(id, n);
                }
            }
            shown.push(id);
        }
        out.lists.insert(list, shown);
    }
    out
}

/// As contagens da frente de cada Kanban de onde vêm espelhos (os comentários e os anexos do
/// original moram no Kanban dele).
pub(crate) fn sources(set: &KanbanSet, k: &Kanban, rows: &Rows, me: &Member) -> BTreeMap<KanbanId, Counts> {
    let mut out = BTreeMap::new();
    for (_, of) in rows.mirrors.values() {
        if of.kanban == k.id || out.contains_key(&of.kanban) {
            continue;
        }
        if let Some(src) = set.get(of.kanban) {
            let c = Counts::of(src).seen_by(Some(me.clone())).with_files(set, src);
            out.insert(src.id, c);
        }
    }
    out
}

/// ⭐ A frente da linha `id` de `k`: um cartão dele, ou o original de um espelho (com o selo).
pub(crate) fn front_of(
    set: &KanbanSet,
    k: &Kanban,
    counts: &Counts,
    others: &BTreeMap<KanbanId, Counts>,
    rows: &Rows,
    id: CardId,
) -> Option<Front> {
    let Some((_, of)) = rows.mirrors.get(&id) else {
        return front::front(k, counts, id);
    };
    let (src, c) = if of.kanban == k.id {
        (k, counts)
    } else {
        (set.get(of.kanban)?, others.get(&of.kanban)?)
    };
    let mut f = front::front(src, c, of.card)?;
    f.mirror = Some(if of.kanban == k.id {
        String::new()
    } else {
        src.header.name.clone()
    });
    Some(f)
}
