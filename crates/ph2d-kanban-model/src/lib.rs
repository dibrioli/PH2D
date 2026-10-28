//! **O documento Kanban** (TrelloClone, [ADR-0195]) — os Kanbans de um projecto e o que há dentro
//! deles.
//!
//! Um Kanban é um documento do PROJECTO, nunca de um quadro: a aba e o elemento do canvas do Quadro
//! são duas VISTAS ([`KanbanView`]) que leem e escrevem o MESMO [`Kanban`], pelo [`KanbanId`]. O que é
//! de cada vista (listas recolhidas, filtro, rolagem) nunca entra no documento.
//!
//! Muda-se só por [`KanbanOp`] e grava-se só por [`KanbanSet::to_bytes`]. Cada registo nasce com o que
//! a colaboração ao vivo pede ([`Meta`]: versão, desempate, lápide) e a ordem é por [`FracKey`].
//!
//! [ADR-0195]: ../../../docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md

mod activity;
mod agenda;
mod background;
mod color;
mod convert;
mod files;
mod history;
mod kanban;
mod links;
mod lists;
mod mint;
mod mirror;
mod ops;
mod plan;
mod records;
mod series;
mod set;
#[cfg(test)]
mod test_support;
mod transfer;
mod v1;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;
mod view;

pub use activity::{Activity, Actor, Happened, Snapshot, Subject, activity_for, apply_logged};
pub use agenda::{Block, BlockOf, Event, Exception, InboxItem, Repeat, Seen};
pub use background::{Pattern, PatternKind, Rgb, Tint};
pub use color::{Hue, LabelColor, Shade};
pub use convert::convert_item_to_card;
pub use files::{Blob, FileId, KanbanFile};
pub use history::{History, MAX_STEPS};
pub use kanban::{Aging, Background, Header, Kanban};
pub use links::{ProjectLink, Target};
pub use lists::{archive_all_cards, copy_list, move_all_cards, move_list};
pub use mint::{SESSION_ID_BIT, splitmix64};
pub use mirror::{Entry, mirror_card, remove_card};
pub use ops::{KanbanOp, Record, RecordId, apply_batch};
pub use ph2d_frac::FracKey;
pub use plan::{ListRef, Place, Plan};
pub use records::{
    ActivityId, Attachment, AttachmentId, Automation, AutomationId, BlockId, Card, CardId, CardRef,
    CheckItem, CheckItemId, Checklist, ChecklistId, Comment, CommentId, Cover, CoverFill, Due,
    EventId, FieldDef, FieldId, FieldKind, FieldOption, FieldValue, InboxId, KanbanId, Label,
    LabelId, Link, List, ListId, Member, Meta, Mirror, MirrorId, Moment, OptionId, ProjectLinkId,
    Reaction, SeenId,
};
pub use series::{Scope, change_occurrence, delete_occurrence};
pub use set::{FORMAT_VERSION, KanbanSet};
pub use transfer::{CopyKeep, card_from_template, copy_card, move_card, move_mirror};
pub use view::{KanbanView, Shown, ShownList};
