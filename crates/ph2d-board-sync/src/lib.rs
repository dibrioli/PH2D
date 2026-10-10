//! **O quadro ao vivo** ([ADR-0181]) — várias pessoas no MESMO quadro. Esta crate é PURA: o protocolo,
//! os carimbos de Lamport por registo, a fusão, e as sessões de anfitrião e convidado como máquinas de
//! estado (entradas → saídas). O transporte (a rede local hoje, a internet depois) é outra crate, atrás
//! de uma porta; nenhum dos dois sabe do outro.
//!
//! As leis vêm dos oráculos CORRIDOS (`docs/MiroClone/08_ao_vivo.md`): funde por registo (nunca perde
//! uma edição inteira), apagar é um registo, a ordem e a duplicação não importam, o derivado (as rotas
//! das setas) não viaja.
//!
//! [ADR-0181]: ../../../docs/architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md

mod presence;
mod replica;
mod session;
mod tokens;

pub use presence::{Presence, RENDER_DELAY_MS, SEND_EVERY_MS, Seen, Track};
pub use replica::{BoardField, Lamport, PeerId, Replica, Snapshot, Write};
pub use session::{ConnId, Event, Guest, Host, Msg, PROTOCOL, PeerInfo, To};
