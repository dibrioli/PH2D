//! A PORTA do transporte ([ADR-0181] §7): um FIO fiável, ordenado, nos dois sentidos, e a ENTRADA por
//! onde o anfitrião o aceita. O aperto de mão (SPAKE2) e a cifra (ChaCha20-Poly1305) vão POR CIMA de
//! qualquer fio: a rede local é TCP ([`crate::tcp`]); a internet implementa a mesma porta noutra crate.
//!
//! [ADR-0181]: ../../../docs/architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md

use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::time::Duration;

/// Um fio aberto. Os clones partilham a mesma ligação (um lê, outro escreve, um terceiro fecha).
pub trait Wire: Read + Write + Send + 'static {
    /// Outra mão sobre a MESMA ligação.
    ///
    /// # Errors
    /// A ligação já não aceita mãos novas (fechada, sem descritores).
    fn try_clone(&self) -> io::Result<Box<dyn Wire>>;

    /// `None` = espera para sempre. Vale para este clone e os outros (como no TCP).
    ///
    /// # Errors
    /// A ligação já fechou.
    fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()>;

    /// # Errors
    /// A ligação já fechou.
    fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()>;

    /// Fecha nos dois sentidos e ACORDA quem estiver bloqueado a ler ou a escrever em qualquer clone.
    fn shutdown(&self);
}

/// Uma entrada do anfitrião: devolve os fios de quem chega.
pub trait Door: Send + Sync + 'static {
    /// Bloqueia até alguém chegar (ou [`Door::wake`]).
    ///
    /// # Errors
    /// `BrokenPipe` = a entrada fechou de vez; outro erro = passageiro (tenta-se de novo).
    fn accept(&self) -> io::Result<(Box<dyn Wire>, SocketAddr)>;

    /// Destrava um [`Door::accept`] bloqueado (para quem fecha ver que é para parar).
    fn wake(&self);
}
