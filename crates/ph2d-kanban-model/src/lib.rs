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
mod color;
mod convert;
mod files;
mod history;
mod kanban;
mod ops;
mod records;
mod set;
#[cfg(test)]
mod test_support;
mod v1;
mod view;

pub use activity::{Activity, Actor, Happened, Snapshot, Subject, activity_for, apply_logged};
pub use color::{Hue, LabelColor, Shade};
pub use convert::convert_item_to_card;
pub use files::{Blob, FileId, KanbanFile};
pub use history::{History, MAX_STEPS};
pub use kanban::{Background, Header, Kanban};
pub use ops::{KanbanOp, Record, RecordId, apply_batch};
pub use ph2d_frac::FracKey;
pub use records::{
    ActivityId, Attachment, AttachmentId, Card, CardId, CheckItem, CheckItemId, Checklist,
    ChecklistId, Comment, CommentId, Cover, CoverFill, Due, FieldDef, FieldId, FieldKind,
    FieldOption, FieldValue, KanbanId, Label, LabelId, Link, List, ListId, Member, Meta, Moment,
    OptionId, Reaction,
};
pub use set::{FORMAT_VERSION, KanbanSet};
pub use view::{KanbanView, Shown, ShownList};
