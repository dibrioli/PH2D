//! ⭐ **O que um gesto ESCREVE no documento** — e só por aqui: cada ação é um lote de `KanbanOp`
//! que entra no `History` do Kanban com a hora do `undo_tick` único (ADR-0195 §5). Um passo de
//! desfazer por gesto: arrastar um cartão, criar uma lista, mudar uma cor.

use ph2d_editor_core::documents::undo_tick;
use ph2d_kanban_model::{
    Card, CardId, FracKey, History, Hue, Kanban, KanbanId, KanbanOp, KanbanSet, List, ListId, Meta,
    Moment, Record,
};

/// Um Kanban aberto para escrever: o documento e o desfazer dele.
pub(crate) struct Doc<'a> {
    pub set: &'a mut KanbanSet,
    pub history: &'a mut History,
    pub id: KanbanId,
}

/// O instante de agora no relógio do documento (ms UTC).
pub(crate) fn now() -> Moment {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
    Moment(ms)
}

fn card(id: CardId, list: ListId, pos: FracKey, title: String, created: Moment) -> Card {
    Card {
        id,
        list,
        pos,
        title,
        description: String::new(),
        labels: Vec::new(),
        members: Vec::new(),
        start: None,
        due: None,
        cover: None,
        fields: std::collections::BTreeMap::new(),
        location: String::new(),
        votes: Vec::new(),
        created,
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
        origin: None,
    }
}

/// A chave que põe um registo entre `lo` e `hi` (as vizinhas, já sem quem se move).
fn between(lo: Option<&FracKey>, hi: Option<&FracKey>) -> FracKey {
    FracKey::between(lo, hi)
}

impl Doc<'_> {
    pub fn kanban(&self) -> Option<&Kanban> {
        self.set.get(self.id)
    }

    /// Aplica um lote como UM passo de desfazer. `true` = o documento mudou.
    pub fn apply(&mut self, ops: Vec<KanbanOp>) -> bool {
        let Some(k) = self.set.doc_mut(self.id) else {
            return false;
        };
        let before = self.history.undo_at();
        self.history.apply(k, ops, undo_tick());
        self.history.undo_at() != before
    }

    pub fn undo(&mut self, redo: bool) -> bool {
        let Some(k) = self.set.doc_mut(self.id) else {
            return false;
        };
        if redo {
            self.history.redo(k, undo_tick())
        } else {
            self.history.undo(k, undo_tick())
        }
    }

    fn put_list(&mut self, f: impl FnOnce(&mut List), id: ListId) -> bool {
        let Some(mut l) = self.kanban().and_then(|k| k.lists.get(&id)).cloned() else {
            return false;
        };
        f(&mut l);
        self.apply(vec![KanbanOp::put(Record::List(l))])
    }

    fn put_card(&mut self, f: impl FnOnce(&mut Card), id: CardId) -> bool {
        let Some(mut c) = self.kanban().and_then(|k| k.cards.get(&id)).cloned() else {
            return false;
        };
        f(&mut c);
        self.apply(vec![KanbanOp::put(Record::Card(c))])
    }

    pub fn add_list(&mut self, title: String) -> Option<ListId> {
        let k = self.kanban()?;
        let id = ListId(k.mint());
        let list = List {
            id,
            title,
            pos: k.pos_after_lists(),
            color: None,
            card_limit: None,
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
        };
        self.apply(vec![KanbanOp::put(Record::List(list))])
            .then_some(id)
    }

    /// Um cartão novo na lista, logo depois de `after` (`None` = no fim).
    pub fn add_card(
        &mut self,
        list: ListId,
        after: Option<CardId>,
        title: String,
    ) -> Option<CardId> {
        let k = self.kanban()?;
        let id = CardId(k.mint());
        let pos = match after {
            None => k.pos_after_cards(list),
            Some(a) => {
                let shown = k.cards_in(list);
                let i = shown.iter().position(|c| c.id == a)?;
                between(Some(&shown[i].pos), shown.get(i + 1).map(|c| &c.pos))
            }
        };
        let c = card(id, list, pos, title, now());
        self.apply(vec![KanbanOp::put(Record::Card(c))])
            .then_some(id)
    }

    pub fn rename_kanban(&mut self, name: String) -> bool {
        let Some(mut h) = self.kanban().map(|k| k.header.clone()) else {
            return false;
        };
        h.name = name;
        self.apply(vec![KanbanOp::Header(h)])
    }

    pub fn rename_list(&mut self, id: ListId, title: String) -> bool {
        self.put_list(|l| l.title = title, id)
    }

    pub fn rename_card(&mut self, id: CardId, title: String) -> bool {
        self.put_card(|c| c.title = title, id)
    }

    pub fn set_list_color(&mut self, id: ListId, color: Option<Hue>) -> bool {
        self.put_list(|l| l.color = color, id)
    }

    /// Leva o cartão à lista `list`, na posição `index` entre os cartões VISÍVEIS dela (sem ele).
    pub fn move_card(&mut self, id: CardId, list: ListId, index: usize) -> bool {
        let Some(k) = self.kanban() else {
            return false;
        };
        let others: Vec<&Card> = k
            .cards_in(list)
            .into_iter()
            .filter(|c| c.id != id)
            .collect();
        let i = index.min(others.len());
        let lo = i.checked_sub(1).map(|j| &others[j].pos);
        let hi = others.get(i).map(|c| &c.pos);
        let Some(cur) = k.cards.get(&id) else {
            return false;
        };
        // Já está ali: nenhuma escrita (o passo de desfazer seria vazio).
        let stays = cur.list == list
            && lo.is_none_or(|lo| *lo < cur.pos)
            && hi.is_none_or(|hi| cur.pos < *hi);
        if stays {
            return false;
        }
        let pos = between(lo, hi);
        self.put_card(
            |c| {
                c.list = list;
                c.pos = pos;
            },
            id,
        )
    }

    /// Leva a lista à posição `index` entre as listas visíveis (sem ela).
    pub fn move_list(&mut self, id: ListId, index: usize) -> bool {
        let Some(k) = self.kanban() else {
            return false;
        };
        let others: Vec<&List> = k
            .lists_in_order()
            .into_iter()
            .filter(|l| l.id != id)
            .collect();
        let i = index.min(others.len());
        let lo = i.checked_sub(1).map(|j| &others[j].pos);
        let hi = others.get(i).map(|l| &l.pos);
        let Some(cur) = k.lists.get(&id) else {
            return false;
        };
        if lo.is_none_or(|lo| *lo < cur.pos) && hi.is_none_or(|hi| cur.pos < *hi) {
            return false;
        }
        let pos = between(lo, hi);
        self.put_list(|l| l.pos = pos, id)
    }

    pub fn archive_card(&mut self, id: CardId, archived: bool) -> bool {
        self.put_card(|c| c.archived = archived, id)
    }

    pub fn archive_list(&mut self, id: ListId, archived: bool) -> bool {
        self.put_list(|l| l.archived = archived, id)
    }

    /// Arquiva todos os cartões visíveis da lista — um passo.
    pub fn archive_all_cards(&mut self, list: ListId) -> bool {
        let Some(k) = self.kanban() else {
            return false;
        };
        let ops = k
            .cards_in(list)
            .into_iter()
            .map(|c| {
                let mut c = c.clone();
                c.archived = true;
                KanbanOp::put(Record::Card(c))
            })
            .collect();
        self.apply(ops)
    }

    /// Apaga de vez (do arquivo). O desfazer devolve-o.
    pub fn delete_card(&mut self, id: CardId) -> bool {
        self.apply(vec![KanbanOp::Delete(ph2d_kanban_model::RecordId::Card(
            id,
        ))])
    }

    pub fn delete_list(&mut self, id: ListId) -> bool {
        self.apply(vec![KanbanOp::Delete(ph2d_kanban_model::RecordId::List(
            id,
        ))])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> (KanbanSet, History, KanbanId) {
        let mut set = KanbanSet::default();
        let id = set.create("Short film");
        (set, History::default(), id)
    }

    #[test]
    fn a_card_dragged_to_another_list_is_one_write_and_one_undo() {
        let (mut set, mut h, id) = fresh();
        let mut d = Doc {
            set: &mut set,
            history: &mut h,
            id,
        };
        let a = d.add_list("To Do".into()).expect("lista");
        let b = d.add_list("Doing".into()).expect("lista");
        let c1 = d.add_card(a, None, "Storyboard".into()).expect("cartão");
        let c2 = d.add_card(b, None, "Animate".into()).expect("cartão");
        assert!(d.move_card(c1, b, 0), "mover escreve");
        let k = d.kanban().expect("kanban");
        let ids: Vec<CardId> = k.cards_in(b).iter().map(|c| c.id).collect();
        assert_eq!(ids, vec![c1, c2], "foi para o topo da outra lista");
        assert!(d.undo(false));
        let k = d.kanban().expect("kanban");
        assert_eq!(
            k.cards_in(a).len(),
            1,
            "o desfazer devolve-o à lista de origem"
        );
    }

    #[test]
    fn dropping_a_card_where_it_already_is_writes_nothing() {
        let (mut set, mut h, id) = fresh();
        let mut d = Doc {
            set: &mut set,
            history: &mut h,
            id,
        };
        let a = d.add_list("To Do".into()).expect("lista");
        let c1 = d.add_card(a, None, "1".into()).expect("cartão");
        d.add_card(a, None, "2".into());
        let at = d.history.undo_at();
        assert!(!d.move_card(c1, a, 0));
        assert_eq!(d.history.undo_at(), at, "nenhum passo de desfazer vazio");
    }

    #[test]
    fn a_new_card_after_another_lands_between_it_and_the_next() {
        let (mut set, mut h, id) = fresh();
        let mut d = Doc {
            set: &mut set,
            history: &mut h,
            id,
        };
        let a = d.add_list("L".into()).expect("lista");
        let c1 = d.add_card(a, None, "1".into()).expect("cartão");
        let c3 = d.add_card(a, None, "3".into()).expect("cartão");
        let c2 = d.add_card(a, Some(c1), "2".into()).expect("cartão");
        let k = d.kanban().expect("kanban");
        let ids: Vec<CardId> = k.cards_in(a).iter().map(|c| c.id).collect();
        assert_eq!(ids, vec![c1, c2, c3]);
    }

    #[test]
    fn moving_a_list_reorders_and_archiving_hides_it() {
        let (mut set, mut h, id) = fresh();
        let mut d = Doc {
            set: &mut set,
            history: &mut h,
            id,
        };
        let a = d.add_list("A".into()).expect("lista");
        let b = d.add_list("B".into()).expect("lista");
        assert!(d.move_list(b, 0));
        let order: Vec<ListId> = d
            .kanban()
            .expect("k")
            .lists_in_order()
            .iter()
            .map(|l| l.id)
            .collect();
        assert_eq!(order, vec![b, a]);
        assert!(d.archive_list(b, true));
        let order: Vec<ListId> = d
            .kanban()
            .expect("k")
            .lists_in_order()
            .iter()
            .map(|l| l.id)
            .collect();
        assert_eq!(order, vec![a]);
    }
}
