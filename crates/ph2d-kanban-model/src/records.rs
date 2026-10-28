//! **Os registos** de um Kanban. Cada um é uma linha própria do documento, com [`Meta`]: dois colegas
//! a comentar o MESMO cartão escrevem dois registos `Comment`, nunca o mesmo `Card` (ADR-0195 §3.4).
//! Os filhos apontam para o pai (`card`, `checklist`) e ordenam-se por [`FracKey`].

use std::collections::BTreeMap;

use ph2d_frac::FracKey;
use serde::{Deserialize, Serialize};

use crate::color::{Hue, LabelColor};
use crate::files::FileId;

macro_rules! ids {
    ($($(#[$m:meta])* $name:ident),* $(,)?) => {$(
        $(#[$m])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(pub u64);
    )*};
}

ids! {
    /// Um Kanban do projecto — a chave que a aba e o elemento do Quadro guardam.
    KanbanId,
    ListId,
    CardId,
    LabelId,
    ChecklistId,
    CheckItemId,
    CommentId,
    AttachmentId,
    FieldId,
    /// Uma opção de um campo «lista suspensa».
    OptionId,
    /// Uma entrada da atividade (v2).
    ActivityId,
    /// Um cartão espelho (v3).
    MirrorId,
    /// Uma automação (v4).
    AutomationId,
    /// Um evento próprio do Calendário do projecto (v5).
    EventId,
    /// Um item da Inbox de uma pessoa (v5).
    InboxId,
    /// Um bloco de tempo do Planner de uma pessoa (v5).
    BlockId,
    /// As notificações lidas de uma pessoa (v5).
    SeenId,
}

/// Um instante: milissegundos desde 1970-01-01 UTC. Valor NEUTRO — fusos, «hoje», «atrasado» e
/// recorrência interpretam-se fora do documento (ADR-0195 §4, `jiff`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Moment(pub i64);

/// Uma pessoa — a MESMA identidade que o Quadro dá a `Who::Person` (o nome na Equipa), para membros,
/// menções e votos atravessarem os dois documentos sem tradução.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Member(pub String);

/// O nome reservado do autor das automações (o «Butler» do Trello), com o prefixo `ph2d:` que
/// nenhum nome da Equipa leva; a UI mostra-o pelo nome traduzido.
const BUTLER: &str = "ph2d:butler";

impl Member {
    /// O autor das automações: as acções delas aparecem na atividade em nome dele.
    #[must_use]
    pub fn butler() -> Self {
        Self(BUTLER.to_owned())
    }

    #[must_use]
    pub fn is_butler(&self) -> bool {
        self.0 == BUTLER
    }
}

/// O que a colaboração ao vivo pede a cada registo (o molde do `Element` do Quadro, ADR-0181).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    /// Sobe a cada mudança deste registo.
    pub version: u64,
    /// Desempate entre duas mudanças com a mesma `version`.
    pub nonce: u32,
    /// Lápide: apagado continua no mapa, para não ressuscitar numa fusão.
    pub deleted: bool,
}

/// Uma lista (coluna).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct List {
    pub id: ListId,
    pub title: String,
    pub pos: FracKey,
    pub color: Option<Hue>,
    /// Limite de cartões (o «WIP limit»); passar dele avisa, não proíbe.
    pub card_limit: Option<u32>,
    pub archived: bool,
    pub meta: Meta,
    /// Quem segue a lista (v2): avisado de cada cartão que entra.
    pub watchers: Vec<Member>,
}

/// O prazo de um cartão.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Due {
    pub at: Moment,
    pub done: bool,
    /// Lembrar N minutos antes (`None` = sem lembrete).
    pub remind_before_min: Option<u32>,
}

/// O que enche a capa.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverFill {
    Color(Hue),
    /// Uma imagem do projecto pela chave do conteúdo (sha-256 dos bytes, a mesma do Quadro, ADR-0184).
    Image(FileId),
}

/// A capa: metade (acima do título) ou inteira (o cartão é a capa).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cover {
    pub fill: CoverFill,
    pub full: bool,
}

/// O valor de um campo personalizado num cartão.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FieldValue {
    Text(String),
    Number(f64),
    Date(Moment),
    Checkbox(bool),
    Option(OptionId),
}

/// Um cartão.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub id: CardId,
    pub list: ListId,
    pub pos: FracKey,
    pub title: String,
    /// A descrição em markdown (CommonMark; o editor rico mostra-a e grava-a assim).
    pub description: String,
    pub labels: Vec<LabelId>,
    pub members: Vec<Member>,
    pub start: Option<Moment>,
    pub due: Option<Due>,
    pub cover: Option<Cover>,
    pub fields: BTreeMap<FieldId, FieldValue>,
    /// Localização em texto (sem vista Mapa — decisão do dono, plano §8).
    pub location: String,
    pub votes: Vec<Member>,
    pub created: Moment,
    pub archived: bool,
    pub meta: Meta,
    /// Quem segue o cartão (v2, o «Seguir» do Trello).
    pub watchers: Vec<Member>,
    /// O cartão de cuja checklist este nasceu ([`crate::convert_item_to_card`], v2).
    pub origin: Option<CardId>,
    /// Um modelo de cartão (v3): a frente mostra-o como modelo e «Criar cartão» copia-o.
    pub template: bool,
}

impl Card {
    /// Um cartão vazio — a porta que não parte quando o registo ganha um campo.
    #[must_use]
    pub fn new(id: CardId, list: ListId, pos: FracKey, title: &str, created: Moment) -> Self {
        Self {
            id,
            list,
            pos,
            title: title.to_owned(),
            description: String::new(),
            labels: Vec::new(),
            members: Vec::new(),
            start: None,
            due: None,
            cover: None,
            fields: BTreeMap::new(),
            location: String::new(),
            votes: Vec::new(),
            created,
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
            origin: None,
            template: false,
        }
    }
}

/// Um cartão de um Kanban do projecto, de qualquer Kanban.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CardRef {
    pub kanban: KanbanId,
    pub card: CardId,
}

/// Um cartão ESPELHO (v3, o «Mirror» do Trello de 2025): o MESMO cartão `of`, mostrado nesta lista.
/// Vive no Kanban onde aparece (a ordem é dessa lista); o conteúdo é sempre o do original — editar
/// de qualquer lado escreve o original.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mirror {
    pub id: MirrorId,
    pub list: ListId,
    pub pos: FracKey,
    pub of: CardRef,
    pub meta: Meta,
}

/// Uma etiqueta do Kanban (sem cor = só texto; sem nome = só cor).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub id: LabelId,
    pub name: String,
    pub color: Option<LabelColor>,
    pub meta: Meta,
}

/// Uma checklist de um cartão.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Checklist {
    pub id: ChecklistId,
    pub card: CardId,
    pub pos: FracKey,
    pub title: String,
    pub hide_checked: bool,
    pub meta: Meta,
}

/// Um item de checklist (com responsável e data próprios).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckItem {
    pub id: CheckItemId,
    pub checklist: ChecklistId,
    pub pos: FracKey,
    pub text: String,
    pub done: bool,
    pub member: Option<Member>,
    pub due: Option<Moment>,
    pub meta: Meta,
}

/// Uma reacção a um comentário: o emoji e quem a deu.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub by: Vec<Member>,
}

/// Um comentário (o texto em markdown; as menções são `@nome` da Equipa).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub id: CommentId,
    pub card: CardId,
    pub author: Member,
    pub at: Moment,
    pub text: String,
    pub edited: Option<Moment>,
    pub reactions: Vec<Reaction>,
    pub meta: Meta,
}

/// Para onde um anexo aponta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Link {
    /// Um ficheiro do projecto, pela chave do conteúdo (ADR-0184).
    File(FileId),
    Url(String),
    Card(KanbanId, CardId),
    Kanban(KanbanId),
}

/// Um anexo de um cartão.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    pub id: AttachmentId,
    pub card: CardId,
    pub pos: FracKey,
    pub name: String,
    pub link: Link,
    pub at: Moment,
    pub meta: Meta,
}

/// Uma opção de um campo «lista suspensa».
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldOption {
    pub id: OptionId,
    pub text: String,
    pub color: Option<Hue>,
}

/// O tipo de um campo personalizado.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldKind {
    Text,
    Number,
    Date,
    Checkbox,
    Dropdown(Vec<FieldOption>),
}

/// A definição de um campo personalizado do Kanban.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    pub id: FieldId,
    pub name: String,
    pub kind: FieldKind,
    /// Mostrar na frente do cartão.
    pub on_front: bool,
    pub pos: FracKey,
    pub meta: Meta,
}

/// Uma automação do Kanban (v4): regra, botão ou comando. O CORPO é opaco aqui — a linguagem das
/// automações vive em `ph2d-kanban-auto`, com versão própria, porque as condições dela são as do
/// filtro (que depende deste modelo). O documento só garante que viaja, se ordena e se desfaz.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Automation {
    pub id: AutomationId,
    pub pos: FracKey,
    /// Os bytes da automação (`ph2d_kanban_auto::Spec`).
    pub body: Vec<u8>,
    /// O último instante agendado já corrido (comandos de calendário e de prazo): reabrir o
    /// projecto não volta a correr o que já correu.
    pub last_run: Option<Moment>,
    pub meta: Meta,
}
