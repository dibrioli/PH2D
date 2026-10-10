//! **O transporte do quadro ao vivo na rede local** ([ADR-0181] §7–§8). Move QUADROS DE BYTES opacos
//! entre um anfitrião e os convidados, cifrados; não conhece o quadro nem o protocolo (isso é a
//! `ph2d-board-sync`). A internet implementará a mesma porta depois.
//!
//! - **Quem entra** encontra o quadro pelo nome anunciado (mDNS, [`discover`]) e escreve o código curto
//!   que o anfitrião mostra ([`Host::code`]). O código autentica por SPAKE2 e a chave que sai cifra
//!   tudo (ChaCha20-Poly1305, uma chave e um contador por direcção): sem o código não se lê nem se
//!   escreve, e não se adivinha fora de linha.
//! - **Só a rede local**: porta fixa ([`PORT`]), origens não-privadas recusadas ([`is_lan`]); a
//!   firewall só é tocada pelo botão, com a senha do sistema, e só para a sub-rede local ([`firewall`]).
//! - **Threads std**, sem async: uma thread de leitura por ligação, escrita por um canal. Tudo pára no
//!   `Drop`. A UI acorda pelo [`Waker`], chamado da thread de rede quando chega algo.
//!
//! [ADR-0181]: ../../../docs/architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md

mod code;
mod discover;
pub mod firewall;
mod guest;
mod handshake;
mod host;
mod lan;
mod wire;

#[cfg(test)]
mod code_tests;
#[cfg(test)]
mod firewall_tests;
#[cfg(test)]
mod lan_tests;
#[cfg(test)]
mod wire_tests;

use std::net::SocketAddr;
use std::sync::Arc;

pub use code::{ALPHABET, CODE_LEN, LOCKOUT, MAX_FAILS};
pub use discover::{Discovery, Found, SERVICE, discover};
pub use firewall::Subnet;
pub use guest::Guest;
pub use host::{Host, HostConfig, PORT, PORT_TRIES};
pub use lan::{is_lan, lan_addrs, lan_subnets};
pub use wire::MAX_FRAME;

/// Uma ligação no anfitrião (os do convidado vêm todos com `0`: só há o anfitrião).
pub type ConnId = u64;

/// Chamado da thread de rede quando há eventos por ler: a UI acorda e chama `poll`.
pub type Waker = Arc<dyn Fn() + Send + Sync>;

/// O que a rede tem para contar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetEvent {
    /// Autenticou-se (o código estava certo).
    Joined(ConnId, SocketAddr),
    /// Um quadro, já decifrado e autenticado.
    Frame(ConnId, Vec<u8>),
    /// Saiu, caiu, foi expulso ou mandou bytes adulterados.
    Left(ConnId),
    /// Alguém tentou entrar com o código errado (no anfitrião; o convidado recebe `JoinError`).
    WrongCode(SocketAddr),
}

/// O que se anuncia na rede — NUNCA o código.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Announce {
    pub board: String,
    pub host: String,
}

/// Porque não se enviou.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendError {
    /// Acima de [`MAX_FRAME`].
    TooBig { len: usize, max: usize },
    /// A ligação já não existe.
    Gone,
}

/// Porque não se entrou.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinError {
    /// O texto escrito não é um código (6 símbolos do [`ALPHABET`]).
    InvalidCode,
    /// O endereço não é da rede local.
    NotLan,
    /// Ninguém respondeu nesse endereço.
    Unreachable(std::io::ErrorKind),
    /// O código não é o do anfitrião.
    WrongCode,
    /// Demasiados códigos errados: o anfitrião só aceita outra tentativa daqui a este tempo.
    Locked(std::time::Duration),
    /// O anfitrião fala outra versão do transporte.
    Incompatible,
    /// A ligação caiu ou chegou lixo a meio do aperto de mão.
    Broken,
}
