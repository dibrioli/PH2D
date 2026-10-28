//! **Copiar e mover cartões** (v3) — dentro do Kanban ou para outro do projecto, e os modelos de
//! cartão. Cada função devolve o [`Plan`] do gesto (um passo de desfazer por Kanban, à mesma hora).
//!
//! Entre Kanbans o cartão renasce com ids do destino (os ids são de um espaço por Kanban): as
//! etiquetas e os campos casam-se pelo nome e cor/tipo e criam-se se faltarem; os membros são a
//! mesma Equipa; os anexos apontam para os mesmos ficheiros. Mover = copiar inteiro + lápide no
//! origem; os espelhos do cartão passam a apontar para o novo. A atividade fica no Kanban onde
//! aconteceu (o cartão chega ao destino com um «criado»).

use std::collections::BTreeMap;

use ph2d_frac::FracKey;

use crate::kanban::Kanban;
use crate::mint::Minter;
use crate::ops::{KanbanOp, Record, RecordId};
use crate::plan::{Place, Plan};
use crate::records::{
    Attachment, AttachmentId, Card, CardId, CardRef, CheckItem, CheckItemId, Checklist,
    ChecklistId, Comment, CommentId, FieldDef, FieldId, KanbanId, Label, LabelId, ListId, Meta,
    Mirror, MirrorId, Moment,
};
use crate::set::KanbanSet;

/// O que uma cópia leva (o «Manter…» do diálogo de copiar do Trello). A descrição, as datas, a
/// capa e a localização vão sempre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CopyKeep {
    pub checklists: bool,
    pub labels: bool,
    pub members: bool,
    pub attachments: bool,
    pub comments: bool,
    pub fields: bool,
}

impl Default for CopyKeep {
    /// Tudo marcado — o de abertura do diálogo do Trello.
    fn default() -> Self {
        Self {
            checklists: true,
            labels: true,
            members: true,
            attachments: true,
            comments: true,
            fields: true,
        }
    }
}

impl CopyKeep {
    /// O «Criar cartão a partir do modelo» do Trello: tudo menos os comentários.
    #[must_use]
    pub fn template() -> Self {
        Self {
            comments: false,
            ..Self::default()
        }
    }
}

/// Como o cartão renasce.
pub(crate) struct How<'a> {
    pub keep: CopyKeep,
    pub title: Option<&'a str>,
    pub created: Option<Moment>,
    pub template: Option<bool>,
    /// Mover: leva também votos, seguidores e o estado de arquivo.
    pub whole: bool,
}

impl How<'_> {
    pub(crate) fn whole() -> Self {
        How {
            keep: CopyKeep::default(),
            title: None,
            created: None,
            template: None,
            whole: true,
        }
    }
}

/// Escreve registos de `src` no Kanban `dst`, com ids novos do destino.
pub(crate) struct Copier<'a> {
    pub src: &'a Kanban,
    pub dst: &'a Kanban,
    ids: Minter<'a>,
    labels: BTreeMap<LabelId, Option<LabelId>>,
    fields: BTreeMap<FieldId, Option<FieldId>>,
    last_field: Option<FracKey>,
    pub ops: Vec<KanbanOp>,
}

impl<'a> Copier<'a> {
    pub(crate) fn new(src: &'a Kanban, dst: &'a Kanban) -> Self {
        let last_field = dst
            .fields
            .values()
            .filter(|f| !f.meta.deleted)
            .map(|f| f.pos.clone())
            .max();
        Self {
            src,
            dst,
            ids: dst.minter(),
            labels: BTreeMap::new(),
            fields: BTreeMap::new(),
            last_field,
            ops: Vec::new(),
        }
    }

    pub(crate) fn id(&mut self) -> u64 {
        self.ids.id()
    }

    fn same(&self) -> bool {
        self.src.id == self.dst.id
    }

    fn label(&mut self, id: LabelId) -> Option<LabelId> {
        if let Some(done) = self.labels.get(&id) {
            return *done;
        }
        let l = self.src.labels.get(&id).filter(|l| !l.meta.deleted);
        let out = match l {
            None => None,
            Some(_) if self.same() => Some(id),
            Some(l) => Some(
                self.dst
                    .labels
                    .values()
                    .find(|d| !d.meta.deleted && d.name == l.name && d.color == l.color)
                    .map_or_else(
                        || {
                            let new = LabelId(self.id());
                            self.ops.push(KanbanOp::put(Record::Label(Label {
                                id: new,
                                meta: Meta::default(),
                                ..l.clone()
                            })));
                            new
                        },
                        |d| d.id,
                    ),
            ),
        };
        self.labels.insert(id, out);
        out
    }

    fn field(&mut self, id: FieldId) -> Option<FieldId> {
        if let Some(done) = self.fields.get(&id) {
            return *done;
        }
        let f = self.src.fields.get(&id).filter(|f| !f.meta.deleted);
        let out = match f {
            None => None,
            Some(_) if self.same() => Some(id),
            Some(f) => Some(
                match self
                    .dst
                    .fields
                    .values()
                    .find(|d| !d.meta.deleted && d.name == f.name && d.kind == f.kind)
                {
                    Some(d) => d.id,
                    None => {
                        let new = FieldId(self.id());
                        let pos = FracKey::between(self.last_field.as_ref(), None);
                        self.last_field = Some(pos.clone());
                        self.ops.push(KanbanOp::put(Record::Field(FieldDef {
                            id: new,
                            pos,
                            meta: Meta::default(),
                            ..f.clone()
                        })));
                        new
                    }
                },
            ),
        };
        self.fields.insert(id, out);
        out
    }

    /// O cartão `c` (de `src`) renasce em `list`/`pos` do destino; devolve o id novo.
    pub(crate) fn card(&mut self, c: &Card, list: ListId, pos: FracKey, how: &How) -> CardId {
        let id = CardId(self.id());
        let created = how.created.unwrap_or(c.created);
        let mut n = Card::new(id, list, pos, how.title.unwrap_or(&c.title), created);
        n.description.clone_from(&c.description);
        (n.start, n.due, n.cover) = (c.start, c.due, c.cover);
        n.location.clone_from(&c.location);
        n.template = how.template.unwrap_or(c.template);
        if how.keep.labels {
            n.labels = c.labels.iter().filter_map(|l| self.label(*l)).collect();
        }
        if how.keep.members {
            n.members.clone_from(&c.members);
        }
        if how.keep.fields {
            for (f, v) in &c.fields {
                if let Some(nf) = self.field(*f) {
                    n.fields.insert(nf, v.clone());
                }
            }
        }
        if how.whole {
            n.votes.clone_from(&c.votes);
            n.watchers.clone_from(&c.watchers);
            n.archived = c.archived;
            n.origin = c.origin.filter(|_| self.same());
        }
        self.ops.push(KanbanOp::put(Record::card(n)));
        self.children(c.id, id, how.keep);
        id
    }

    fn children(&mut self, from: CardId, to: CardId, keep: CopyKeep) {
        let src = self.src;
        if keep.checklists {
            let mut cls: Vec<&Checklist> = src
                .checklists
                .values()
                .filter(|c| c.card == from && !c.meta.deleted)
                .collect();
            cls.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
            for cl in cls {
                let nid = ChecklistId(self.id());
                self.ops.push(KanbanOp::put(Record::Checklist(Checklist {
                    id: nid,
                    card: to,
                    meta: Meta::default(),
                    ..cl.clone()
                })));
                let items = src
                    .items
                    .values()
                    .filter(|i| i.checklist == cl.id && !i.meta.deleted);
                for it in items {
                    let id = CheckItemId(self.id());
                    self.ops.push(KanbanOp::put(Record::CheckItem(CheckItem {
                        id,
                        checklist: nid,
                        meta: Meta::default(),
                        ..it.clone()
                    })));
                }
            }
        }
        if keep.attachments {
            let atts = src
                .attachments
                .values()
                .filter(|a| a.card == from && !a.meta.deleted);
            for a in atts {
                let id = AttachmentId(self.id());
                self.ops.push(KanbanOp::put(Record::Attachment(Attachment {
                    id,
                    card: to,
                    meta: Meta::default(),
                    ..a.clone()
                })));
            }
            let links = src
                .project_links
                .values()
                .filter(|l| l.card == from && !l.meta.deleted);
            for l in links {
                let id = crate::records::ProjectLinkId(self.id());
                self.ops
                    .push(KanbanOp::put(Record::ProjectLink(crate::ProjectLink {
                        id,
                        card: to,
                        meta: Meta::default(),
                        ..l.clone()
                    })));
            }
        }
        if keep.comments {
            let cms = src
                .comments
                .values()
                .filter(|c| c.card == from && !c.meta.deleted);
            for c in cms {
                let id = CommentId(self.id());
                self.ops.push(KanbanOp::put(Record::Comment(Comment {
                    id,
                    card: to,
                    meta: Meta::default(),
                    ..c.clone()
                })));
            }
        }
    }
}

/// As lápides de um cartão e dos filhos dele (o lado de origem de um mover entre Kanbans).
pub(crate) fn tombstones(k: &Kanban, card: CardId) -> Vec<KanbanOp> {
    let mut v = vec![KanbanOp::Delete(RecordId::Card(card))];
    for cl in k
        .checklists
        .values()
        .filter(|c| c.card == card && !c.meta.deleted)
    {
        v.push(KanbanOp::Delete(RecordId::Checklist(cl.id)));
        v.extend(
            k.items
                .values()
                .filter(|i| i.checklist == cl.id && !i.meta.deleted)
                .map(|i| KanbanOp::Delete(RecordId::CheckItem(i.id))),
        );
    }
    v.extend(
        k.attachments
            .values()
            .filter(|a| a.card == card && !a.meta.deleted)
            .map(|a| KanbanOp::Delete(RecordId::Attachment(a.id))),
    );
    v.extend(
        k.project_links
            .values()
            .filter(|l| l.card == card && !l.meta.deleted)
            .map(|l| KanbanOp::Delete(RecordId::ProjectLink(l.id))),
    );
    v.extend(
        k.comments
            .values()
            .filter(|c| c.card == card && !c.meta.deleted)
            .map(|c| KanbanOp::Delete(RecordId::Comment(c.id))),
    );
    v
}

/// Move para o Kanban `dst` os cartões `cards` (`(cartão, lista, posição)` no destino) e os
/// espelhos `mirrors` mostrados em `src` — os espelhos de cartões que vão juntos seguem-nos. `head`
/// abre o lote do destino e usa os `reserved` primeiros ids dele (a lista nova de «mover lista»).
#[allow(clippy::too_many_arguments)]
pub(crate) fn cross_move(
    set: &KanbanSet,
    src: &Kanban,
    dst: &Kanban,
    cards: &[(CardId, ListId, FracKey)],
    mirrors: &[(MirrorId, ListId, FracKey)],
    plan: &mut Plan,
    head: Vec<KanbanOp>,
    reserved: u64,
) {
    let mut cp = Copier::new(src, dst);
    cp.ops = head;
    cp.ids.skip(reserved);
    let mut moved = BTreeMap::new();
    for (c, list, pos) in cards {
        let Some(card) = src.cards.get(c).filter(|c| !c.meta.deleted) else {
            continue;
        };
        let new = cp.card(card, *list, pos.clone(), &How::whole());
        moved.insert(*c, new);
        plan.extend(src.id, tombstones(src, *c));
    }
    let to = |of: CardRef| match moved.get(&of.card) {
        Some(n) if of.kanban == src.id => CardRef {
            kanban: dst.id,
            card: *n,
        },
        _ => of,
    };
    for (m, list, pos) in mirrors {
        let Some(mi) = src.mirrors.get(m).filter(|m| !m.meta.deleted) else {
            continue;
        };
        let id = MirrorId(cp.id());
        cp.ops.push(KanbanOp::put(Record::Mirror(Mirror {
            id,
            list: *list,
            pos: pos.clone(),
            of: to(mi.of),
            meta: Meta::default(),
        })));
        plan.push(src.id, KanbanOp::Delete(RecordId::Mirror(*m)));
    }
    plan.extend(dst.id, cp.ops);
    let leaving: Vec<MirrorId> = mirrors.iter().map(|m| m.0).collect();
    for k in set.iter() {
        for mi in k.mirrors.values().filter(|m| !m.meta.deleted) {
            let gone = k.id == src.id && leaving.contains(&mi.id);
            if !gone && to(mi.of) != mi.of {
                plan.push(
                    k.id,
                    KanbanOp::put(Record::Mirror(Mirror {
                        of: to(mi.of),
                        ..mi.clone()
                    })),
                );
            }
        }
    }
}

fn live_list(k: &Kanban, list: ListId) -> Option<()> {
    k.lists.get(&list).filter(|l| !l.meta.deleted).map(|_| ())
}

/// «Copiar cartão»: `src` renasce em `to` com o título `title`, levando o que `keep` marca.
#[must_use]
pub fn copy_card(
    set: &KanbanSet,
    src: CardRef,
    to: &Place,
    keep: CopyKeep,
    title: &str,
    created: Moment,
) -> Option<Plan> {
    copy_as(set, src, to, keep, title, created, None)
}

/// «Criar cartão a partir do modelo»: como copiar, e o cartão novo já não é modelo.
#[must_use]
pub fn card_from_template(
    set: &KanbanSet,
    template: CardRef,
    to: &Place,
    keep: CopyKeep,
    title: &str,
    created: Moment,
) -> Option<Plan> {
    set.card(template).filter(|c| c.template)?;
    copy_as(set, template, to, keep, title, created, Some(false))
}

fn copy_as(
    set: &KanbanSet,
    src: CardRef,
    to: &Place,
    keep: CopyKeep,
    title: &str,
    created: Moment,
    template: Option<bool>,
) -> Option<Plan> {
    let card = set.card(src)?;
    let (sk, dk) = (set.get(src.kanban)?, set.get(to.kanban)?);
    live_list(dk, to.list)?;
    let mut cp = Copier::new(sk, dk);
    let how = How {
        keep,
        title: Some(title),
        created: Some(created),
        template,
        whole: false,
    };
    cp.card(card, to.list, to.pos.clone(), &how);
    let mut plan = Plan::default();
    plan.extend(to.kanban, cp.ops);
    Some(plan)
}

/// «Mover cartão» para `to` (outra lista, outro Kanban, outra posição).
#[must_use]
pub fn move_card(set: &KanbanSet, src: CardRef, to: &Place) -> Option<Plan> {
    let card = set.card(src)?;
    let (sk, dk) = (set.get(src.kanban)?, set.get(to.kanban)?);
    live_list(dk, to.list)?;
    let mut plan = Plan::default();
    if src.kanban == to.kanban {
        plan.push(
            to.kanban,
            KanbanOp::put(Record::card(Card {
                list: to.list,
                pos: to.pos.clone(),
                ..card.clone()
            })),
        );
    } else {
        let cards = [(src.card, to.list, to.pos.clone())];
        cross_move(set, sk, dk, &cards, &[], &mut plan, Vec::new(), 0);
    }
    Some(plan)
}

/// Mover o espelho `id` (mostrado no Kanban `kanban`) para `to`.
#[must_use]
pub fn move_mirror(set: &KanbanSet, kanban: KanbanId, id: MirrorId, to: &Place) -> Option<Plan> {
    let (sk, dk) = (set.get(kanban)?, set.get(to.kanban)?);
    let m = sk.mirrors.get(&id).filter(|m| !m.meta.deleted)?;
    live_list(dk, to.list)?;
    let mut plan = Plan::default();
    if kanban == to.kanban {
        plan.push(
            kanban,
            KanbanOp::put(Record::Mirror(Mirror {
                list: to.list,
                pos: to.pos.clone(),
                ..m.clone()
            })),
        );
    } else {
        let ms = [(id, to.list, to.pos.clone())];
        cross_move(set, sk, dk, &[], &ms, &mut plan, Vec::new(), 0);
    }
    Some(plan)
}

#[cfg(test)]
#[path = "transfer_tests.rs"]
pub(crate) mod tests;
