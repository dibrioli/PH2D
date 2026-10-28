//! ⭐ **O que o verso ESCREVE** — e só por aqui: cada gesto é um lote de `KanbanOp` aplicado pela
//! porta única dos gestos (`crate::hooks`), em nome de «eu»; as reações vêm em passos seus.

use crate::actions::now;
use crate::docs::KanbanDocs;
use ph2d_kanban_model::{
    Attachment, AttachmentId, Card, CardId, CheckItem, CheckItemId, Checklist, ChecklistId,
    Comment, CommentId, Cover, FieldDef, FieldId, FieldKind, FieldValue, FracKey, Kanban, KanbanOp,
    Label, LabelColor, LabelId, Link, ListId, Member, Meta, Moment, Reaction, Record, RecordId,
};

/// Aplica o lote que `f` monta sobre o Kanban da aba como um gesto da pessoa (a porta única: a
/// atividade em nome de «eu», o Butler e os scripts reagem). `true` = o documento mudou.
pub(crate) fn logged(
    docs: &mut KanbanDocs,
    id: u64,
    f: impl FnOnce(&Kanban) -> Option<Vec<KanbanOp>>,
) -> bool {
    crate::hooks::write_in(docs, id, f)
}

/// O cartão `c` mudado por `f` (nenhuma escrita se `f` o deixa igual).
pub(crate) fn card(docs: &mut KanbanDocs, id: u64, c: CardId, f: impl FnOnce(&mut Card)) -> bool {
    logged(docs, id, |k| {
        let old = k.cards.get(&c)?;
        let mut new = old.clone();
        f(&mut new);
        (new != *old).then(|| vec![KanbanOp::put(Record::Card(new.into()))])
    })
}

fn toggle<T: PartialEq + Clone>(v: &mut Vec<T>, x: &T) {
    if let Some(i) = v.iter().position(|y| y == x) {
        v.remove(i);
    } else {
        v.push(x.clone());
    }
}

pub(crate) fn toggle_member(docs: &mut KanbanDocs, id: u64, c: CardId, m: &Member) -> bool {
    card(docs, id, c, |c| toggle(&mut c.members, m))
}

pub(crate) fn toggle_label(docs: &mut KanbanDocs, id: u64, c: CardId, l: LabelId) -> bool {
    card(docs, id, c, |c| toggle(&mut c.labels, &l))
}

pub(crate) fn toggle_watch(docs: &mut KanbanDocs, id: u64, c: CardId) -> bool {
    let me = docs.me.clone();
    card(docs, id, c, |c| toggle(&mut c.watchers, &me))
}

pub(crate) fn toggle_vote(docs: &mut KanbanDocs, id: u64, c: CardId) -> bool {
    let me = docs.me.clone();
    card(docs, id, c, |c| toggle(&mut c.votes, &me))
}

/// A posição depois da maior de `keys` (o fim de uma colecção ordenada).
fn after<'a>(keys: impl Iterator<Item = &'a FracKey>) -> FracKey {
    let last = keys.max();
    FracKey::between(last, None)
}

/// Uma etiqueta nova (e posta no cartão, o idioma do Trello). Devolve-a.
pub(crate) fn create_label(
    docs: &mut KanbanDocs,
    id: u64,
    c: CardId,
    name: String,
    color: Option<LabelColor>,
) -> Option<LabelId> {
    let mut made = None;
    logged(docs, id, |k| {
        let mut card = k.cards.get(&c)?.clone();
        let lid = LabelId(k.mint());
        made = Some(lid);
        card.labels.push(lid);
        Some(vec![
            KanbanOp::put(Record::Label(Label {
                id: lid,
                name,
                color,
                meta: Meta::default(),
            })),
            KanbanOp::put(Record::Card(card.into())),
        ])
    })
    .then_some(made)
    .flatten()
}

pub(crate) fn edit_label(
    docs: &mut KanbanDocs,
    id: u64,
    l: LabelId,
    name: String,
    color: Option<LabelColor>,
) -> bool {
    logged(docs, id, |k| {
        let mut lb = k.labels.get(&l)?.clone();
        lb.name = name;
        lb.color = color;
        Some(vec![KanbanOp::put(Record::Label(lb))])
    })
}

/// Apaga a etiqueta do Kanban e tira-a de todos os cartões — um passo.
pub(crate) fn delete_label(docs: &mut KanbanDocs, id: u64, l: LabelId) -> bool {
    logged(docs, id, |k| {
        let mut ops: Vec<KanbanOp> = k
            .cards
            .values()
            .filter(|c| c.labels.contains(&l))
            .map(|c| {
                let mut c = c.clone();
                c.labels.retain(|x| *x != l);
                KanbanOp::put(Record::Card(c.into()))
            })
            .collect();
        ops.push(KanbanOp::Delete(RecordId::Label(l)));
        Some(ops)
    })
}

pub(crate) fn add_checklist(docs: &mut KanbanDocs, id: u64, c: CardId, title: String) -> bool {
    logged(docs, id, |k| {
        let pos = after(
            k.checklists
                .values()
                .filter(|x| x.card == c)
                .map(|x| &x.pos),
        );
        Some(vec![KanbanOp::put(Record::Checklist(Checklist {
            id: ChecklistId(k.mint()),
            card: c,
            pos,
            title,
            hide_checked: false,
            meta: Meta::default(),
        }))])
    })
}

pub(crate) fn checklist(
    docs: &mut KanbanDocs,
    id: u64,
    cl: ChecklistId,
    f: impl FnOnce(&mut Checklist),
) -> bool {
    logged(docs, id, |k| {
        let mut x = k.checklists.get(&cl)?.clone();
        f(&mut x);
        Some(vec![KanbanOp::put(Record::Checklist(x))])
    })
}

/// Apaga a checklist e os itens dela — um passo.
pub(crate) fn delete_checklist(docs: &mut KanbanDocs, id: u64, cl: ChecklistId) -> bool {
    logged(docs, id, |k| {
        let mut ops: Vec<KanbanOp> = k
            .items
            .values()
            .filter(|i| i.checklist == cl && !i.meta.deleted)
            .map(|i| KanbanOp::Delete(RecordId::CheckItem(i.id)))
            .collect();
        ops.push(KanbanOp::Delete(RecordId::Checklist(cl)));
        Some(ops)
    })
}

pub(crate) fn add_item(docs: &mut KanbanDocs, id: u64, cl: ChecklistId, text: String) -> bool {
    logged(docs, id, |k| {
        let pos = after(
            k.items
                .values()
                .filter(|i| i.checklist == cl && !i.meta.deleted)
                .map(|i| &i.pos),
        );
        Some(vec![KanbanOp::put(Record::CheckItem(CheckItem {
            id: CheckItemId(k.mint()),
            checklist: cl,
            pos,
            text,
            done: false,
            member: None,
            due: None,
            meta: Meta::default(),
        }))])
    })
}

pub(crate) fn item(
    docs: &mut KanbanDocs,
    id: u64,
    it: CheckItemId,
    f: impl FnOnce(&mut CheckItem),
) -> bool {
    logged(docs, id, |k| {
        let old = k.items.get(&it)?;
        let mut x = old.clone();
        f(&mut x);
        (x != *old).then(|| vec![KanbanOp::put(Record::CheckItem(x))])
    })
}

pub(crate) fn delete(docs: &mut KanbanDocs, id: u64, r: RecordId) -> bool {
    logged(docs, id, |_| Some(vec![KanbanOp::Delete(r)]))
}

pub(crate) fn convert(docs: &mut KanbanDocs, id: u64, it: CheckItemId) -> bool {
    logged(docs, id, |k| {
        ph2d_kanban_model::convert_item_to_card(k, it, now())
    })
}

pub(crate) fn add_link(
    docs: &mut KanbanDocs,
    id: u64,
    c: CardId,
    url: String,
    name: String,
) -> bool {
    logged(docs, id, |k| {
        let pos = after(
            k.attachments
                .values()
                .filter(|a| a.card == c && !a.meta.deleted)
                .map(|a| &a.pos),
        );
        Some(vec![KanbanOp::put(Record::Attachment(Attachment {
            id: AttachmentId(k.mint()),
            card: c,
            pos,
            name,
            link: Link::Url(url),
            at: now(),
            meta: Meta::default(),
        }))])
    })
}

/// Um anexo de ficheiro (já no conjunto); uma imagem num cartão sem capa — ou pedida como capa
/// (`cover`, o «Upload a cover image») — passa a capa, no MESMO passo de desfazer (o Trello).
pub(crate) fn add_file(
    docs: &mut KanbanDocs,
    id: u64,
    c: CardId,
    file: ph2d_kanban_model::FileId,
    name: String,
    image: bool,
    cover: bool,
) -> bool {
    logged(docs, id, |k| {
        let card = k.cards.get(&c)?;
        let pos = after(
            k.attachments
                .values()
                .filter(|a| a.card == c && !a.meta.deleted)
                .map(|a| &a.pos),
        );
        let mut ops = vec![KanbanOp::put(Record::Attachment(Attachment {
            id: AttachmentId(k.mint()),
            card: c,
            pos,
            name,
            link: Link::File(file),
            at: now(),
            meta: Meta::default(),
        }))];
        if image && (cover || card.cover.is_none()) {
            let mut new = card.clone();
            new.cover = Some(Cover {
                fill: ph2d_kanban_model::CoverFill::Image(file),
                full: card.cover.is_some_and(|c| c.full),
            });
            ops.push(KanbanOp::put(Record::Card(new.into())));
        }
        Some(ops)
    })
}

pub(crate) fn set_cover(docs: &mut KanbanDocs, id: u64, c: CardId, cover: Option<Cover>) -> bool {
    card(docs, id, c, |c| c.cover = cover)
}

/// Um comentário novo de «eu».
pub(crate) fn add_comment(docs: &mut KanbanDocs, id: u64, c: CardId, text: String) -> bool {
    let me = docs.me.clone();
    logged(docs, id, |k| {
        Some(vec![KanbanOp::put(Record::Comment(Comment {
            id: CommentId(k.mint()),
            card: c,
            author: me,
            at: now(),
            text,
            edited: None,
            reactions: Vec::new(),
            meta: Meta::default(),
        }))])
    })
}

pub(crate) fn comment(
    docs: &mut KanbanDocs,
    id: u64,
    cm: CommentId,
    f: impl FnOnce(&mut Comment),
) -> bool {
    logged(docs, id, |k| {
        let old = k.comments.get(&cm)?;
        let mut x = old.clone();
        f(&mut x);
        (x != *old).then(|| vec![KanbanOp::put(Record::Comment(x))])
    })
}

pub(crate) fn edit_comment(docs: &mut KanbanDocs, id: u64, cm: CommentId, text: String) -> bool {
    let at = now();
    comment(docs, id, cm, |c| {
        if c.text != text {
            c.text = text;
            c.edited = Some(at);
        }
    })
}

/// Alterna a reacção `emoji` de «eu» no comentário.
pub(crate) fn react(docs: &mut KanbanDocs, id: u64, cm: CommentId, emoji: &str) -> bool {
    let me = docs.me.clone();
    comment(docs, id, cm, |c| {
        match c.reactions.iter_mut().find(|r| r.emoji == emoji) {
            Some(r) => toggle(&mut r.by, &me),
            None => c.reactions.push(Reaction {
                emoji: emoji.to_owned(),
                by: vec![me],
            }),
        }
        c.reactions.retain(|r| !r.by.is_empty());
    })
}

/// Um campo personalizado novo (no fim).
pub(crate) fn create_field(docs: &mut KanbanDocs, id: u64, name: String, kind: FieldKind) -> bool {
    logged(docs, id, |k| {
        let pos = after(
            k.fields
                .values()
                .filter(|f| !f.meta.deleted)
                .map(|f| &f.pos),
        );
        Some(vec![KanbanOp::put(Record::Field(FieldDef {
            id: FieldId(k.mint()),
            name,
            kind,
            on_front: false,
            pos,
            meta: Meta::default(),
        }))])
    })
}

pub(crate) fn field_front(docs: &mut KanbanDocs, id: u64, f: FieldId) -> bool {
    logged(docs, id, |k| {
        let mut d = k.fields.get(&f)?.clone();
        d.on_front = !d.on_front;
        Some(vec![KanbanOp::put(Record::Field(d))])
    })
}

pub(crate) fn set_field(
    docs: &mut KanbanDocs,
    id: u64,
    c: CardId,
    f: FieldId,
    v: Option<FieldValue>,
) -> bool {
    card(docs, id, c, |c| match v {
        Some(v) => {
            c.fields.insert(f, v);
        }
        None => {
            c.fields.remove(&f);
        }
    })
}

/// Leva o cartão para o fim da lista `to` (o «Move» do Trello pelo verso).
pub(crate) fn move_to(docs: &mut KanbanDocs, id: u64, c: CardId, to: ListId) -> bool {
    logged(docs, id, |k| {
        let mut card = k.cards.get(&c)?.clone();
        if card.list == to {
            return None;
        }
        card.list = to;
        card.pos = k.pos_after_cards(to);
        Some(vec![KanbanOp::put(Record::Card(card.into()))])
    })
}

/// As datas do cartão: início, entrega (com o lembrete e o «concluída» que já tinha).
pub(crate) fn set_dates(
    docs: &mut KanbanDocs,
    id: u64,
    c: CardId,
    start: Option<Moment>,
    due: Option<(Moment, Option<u32>)>,
) -> bool {
    card(docs, id, c, |c| {
        c.start = start;
        let done = c.due.is_some_and(|d| d.done);
        c.due = due.map(|(at, remind)| ph2d_kanban_model::Due {
            at,
            done,
            remind_before_min: remind,
        });
    })
}

pub(crate) fn toggle_done(docs: &mut KanbanDocs, id: u64, c: CardId) -> bool {
    card(docs, id, c, |c| {
        if let Some(d) = c.due.as_mut() {
            d.done = !d.done;
        }
    })
}
