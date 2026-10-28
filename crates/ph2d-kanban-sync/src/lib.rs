//! **O Kanban ao vivo** (TrelloClone W5, [ADR-0195] §3 sobre o [ADR-0181]) — vários colegas no MESMO
//! Kanban. PURA, como a `ph2d-board-sync`, e com a MESMA lei de fusão ([`ph2d_board_sync::wins`]):
//!
//! - cada registo do documento (listas, cartões, checklists, itens, comentários, anexos, campos,
//!   espelhos, automações, atividade) e o cabeçalho dividem-se em CAMPOS
//!   ([`ph2d_kanban_model::Record::fields`] — a mesma divisão do desfazer por campo); cada campo é um
//!   registo com carimbo de Lamport (o par desempata); a ordem é por `FracKey`; apagar é a lápide;
//! - as listas de pessoas e etiquetas (membros, votos, quem segue, etiquetas) e os campos
//!   personalizados do cartão são CONJUNTOS: um registo por elemento — dois votos ao mesmo tempo são
//!   dois votos;
//! - os ficheiros viajam uma vez por conteúdo.
//!
//! [`KanbanReplica`] funde; [`HostRoom`]/[`GuestRoom`] falam [`KMsg`] — dentro da sessão de um Quadro
//! (`ph2d_board_sync::Msg::Kanban`) ou numa sessão SÓ do Kanban ([`KanbanHost`]/[`KanbanGuest`]),
//! pelo mesmo transporte. [`Viewers`] diz quem vê que Kanban e que cartão.
//!
//! [ADR-0195]: ../../../docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md
//! [ADR-0181]: ../../../docs/architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md

mod msg;
pub mod port;
mod portrait;
mod presence;
mod replica;
mod room;
mod session;
mod shape;
mod snapshot;
mod table;
#[cfg(test)]
mod test_support;

pub use msg::{KMsg, KSnapshot, KWrite, Target};
pub use portrait::Portrait;
pub use presence::{KanbanPresence, Viewers};
pub use replica::KanbanReplica;
pub use room::{GuestRoom, HostRoom, KEvent};
pub use session::{KanbanGuest, KanbanHost, SoloMsg};
pub use snapshot::SNAPSHOT_BLOCK;
