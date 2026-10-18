//! **O quadro ao vivo PELA INTERNET** ([ADR-0187]): a mesma porta da rede local ([`ph2d_board_net::Wire`]
//! e [`ph2d_board_net::Door`]) sobre UDP, para se poder furar o NAT de casa.
//!
//! - **Um nó** ([`Node`]) = um socket UDP por família e UMA thread que conduz o QUIC (`quinn-proto`,
//!   sem I/O nem `tokio`) e responde a STUN e furos no MESMO socket: a porta mapeada pelo NAT é uma só.
//! - **O fio** é um fluxo QUIC fiável e ordenado; por cima dele vão o aperto de mão SPAKE2 (com a chave
//!   do convite) e a cifra ChaCha20 da `ph2d-board-net`, iguais aos da rede local. O TLS do QUIC é só
//!   envelope.
//! - **Encontrar-se**: o convite leva os candidatos do anfitrião ([`candidates`]); cada lado fura para
//!   os do outro ([`Node::punch`]). O ponto de encontro sem servidor nosso é a DHT do BitTorrent.
//!
//! [ADR-0187]: ../../../docs/architecture/decisions/0187-o-quadro-pela-internet-e-quic-sobre-udp-com-encontro-na-dht.md

mod candidates;
mod driver;
mod node;
mod pipe;
mod rendezvous;
mod session;
mod stun;
mod tls;

#[cfg(test)]
mod node_tests;
#[cfg(test)]
mod test_net;

pub use candidates::{Mapping, STUN_SERVERS, candidates, mapping, resolve};
pub use driver::CLOSE_GRACE;
pub use node::{DialAdder, Dialer, Node, NodeConfig, Reflexive, decode_via, encode_via};
pub use rendezvous::{DhtMode, Rendezvous, Role};
pub use session::{InternetConfig, InternetHost, LinkGuest, PUNCH_FOR, join, join_noting};
