//! **Um Kanban**: o cabeçalho e as tabelas de registos. Lê-se à vontade; muda-se só por
//! [`crate::KanbanOp`].

use std::collections::BTreeMap;

use ph2d_frac::FracKey;
use serde::{Deserialize, Serialize};

use crate::activity::Activity;
use crate::color::Hue;
use crate::files::FileId;
use crate::records::{
    ActivityId, Attachment, AttachmentId, Automation, AutomationId, Card, CardId, CheckItem,
    CheckItemId, Checklist, ChecklistId, Comment, CommentId, FieldDef, FieldId, KanbanId, Label,
    LabelId, List, ListId, Member, Mirror, MirrorId,
};

/// O envelhecimento dos cartões parados (v3, o Power-Up «Card Aging»): como a frente o pinta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Aging {
    /// O cartão desbota.
    Regular,
    /// O cartão amarelece, racha e rasga (o mapa de pirata).
    Pirate,
}

/// O fundo do Kanban.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Background {
    Color(Hue),
    /// Um dos gradientes dos tokens do Kanban, pelo índice.
    Gradient(u16),
    /// Uma imagem do projecto pela chave do conteúdo (ADR-0184).
    Image(FileId),
}

/// O que é do Kanban inteiro (um registo só, com versão própria).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub starred: bool,
    pub background: Background,
    pub version: u64,
    /// Quem segue o Kanban inteiro (v2).
    pub watchers: Vec<Member>,
    /// O envelhecimento ligado (v3; `None` = desligado, o de nascença).
    pub aging: Option<Aging>,
}

/// Um Kanban do projecto.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Kanban {
    pub id: KanbanId,
    pub header: Header,
    pub lists: BTreeMap<ListId, List>,
    pub cards: BTreeMap<CardId, Card>,
    pub labels: BTreeMap<LabelId, Label>,
    pub checklists: BTreeMap<ChecklistId, Checklist>,
    pub items: BTreeMap<CheckItemId, CheckItem>,
    pub comments: BTreeMap<CommentId, Comment>,
    pub attachments: BTreeMap<AttachmentId, Attachment>,
    pub fields: BTreeMap<FieldId, FieldDef>,
    /// O maior id local já dado a um registo (os ids são de UM espaço por Kanban).
    pub(crate) next_id: u64,
    /// A história do Kanban (v2): nasce das ops aplicadas por [`crate::apply_logged`] e o desfazer
    /// não a apaga (como no Trello).
    pub activities: BTreeMap<ActivityId, Activity>,
    /// Os espelhos MOSTRADOS neste Kanban (v3; o original pode ser de outro).
    pub mirrors: BTreeMap<MirrorId, Mirror>,
    /// As automações do Kanban (v4): regras, botões e comandos, pela ordem `pos`.
    pub automations: BTreeMap<AutomationId, Automation>,
    /// Em sessão ao vivo: o sorteio dos ids ([`Kanban::set_session`]). Não vai para o ficheiro.
    #[serde(skip)]
    pub(crate) session: crate::mint::Lottery,
}

impl Kanban {
    /// Um Kanban vazio, de fundo azul (o de nascença do Trello).
    #[must_use]
    pub fn new(id: KanbanId, name: &str) -> Self {
        Self {
            id,
            header: Header {
                name: name.to_owned(),
                starred: false,
                background: Background::Color(Hue::Blue),
                version: 0,
                watchers: Vec::new(),
                aging: None,
            },
            lists: BTreeMap::new(),
            cards: BTreeMap::new(),
            labels: BTreeMap::new(),
            checklists: BTreeMap::new(),
            items: BTreeMap::new(),
            comments: BTreeMap::new(),
            attachments: BTreeMap::new(),
            fields: BTreeMap::new(),
            next_id: 0,
            activities: BTreeMap::new(),
            mirrors: BTreeMap::new(),
            automations: BTreeMap::new(),
            session: crate::mint::Lottery::default(),
        }
    }

    /// As listas vivas e não arquivadas, pela ordem (`pos`, e o id desempata).
    #[must_use]
    pub fn lists_in_order(&self) -> Vec<&List> {
        let mut v: Vec<&List> = self
            .lists
            .values()
            .filter(|l| !l.meta.deleted && !l.archived)
            .collect();
        v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
        v
    }

    /// Os cartões vivos e não arquivados de `list`, pela ordem.
    #[must_use]
    pub fn cards_in(&self, list: ListId) -> Vec<&Card> {
        let mut v: Vec<&Card> = self
            .cards
            .values()
            .filter(|c| c.list == list && !c.meta.deleted && !c.archived)
            .collect();
        v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
        v
    }

    /// A posição depois da última lista viva (arquivadas contam: desarquivar não colide).
    #[must_use]
    pub fn pos_after_lists(&self) -> FracKey {
        let last = self
            .lists
            .values()
            .filter(|l| !l.meta.deleted)
            .map(|l| &l.pos)
            .max();
        FracKey::between(last, None)
    }

    /// A posição depois do último cartão (ou espelho, v3) de `list`.
    #[must_use]
    pub fn pos_after_cards(&self, list: ListId) -> FracKey {
        let cards = self
            .cards
            .values()
            .filter(|c| c.list == list && !c.meta.deleted)
            .map(|c| &c.pos);
        let mirrors = self
            .mirrors
            .values()
            .filter(|m| m.list == list && !m.meta.deleted)
            .map(|m| &m.pos);
        FracKey::between(cards.chain(mirrors).max(), None)
    }
}
