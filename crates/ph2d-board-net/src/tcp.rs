//! A porta na rede LOCAL: TCP na porta fixa, só origens da rede local ([`is_lan`]).

use std::io;
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use crate::lan::is_lan;
use crate::link::{Door, Wire};

impl Wire for TcpStream {
    fn try_clone(&self) -> io::Result<Box<dyn Wire>> {
        TcpStream::try_clone(self).map(|s| Box::new(s) as Box<dyn Wire>)
    }

    fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        TcpStream::set_read_timeout(self, t)
    }

    fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        TcpStream::set_write_timeout(self, t)
    }

    fn shutdown(&self) {
        let _ = TcpStream::shutdown(self, Shutdown::Both);
    }
}

/// A entrada TCP do anfitrião na rede local.
pub(crate) struct TcpDoor {
    listener: TcpListener,
    port: u16,
}

impl TcpDoor {
    /// A primeira porta livre de `port..port + tries` (`port == 0` = uma qualquer do sistema).
    pub(crate) fn bind(port: u16, tries: u16) -> io::Result<Self> {
        let listener = if port == 0 {
            TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0))?
        } else {
            let mut found = Err(io::Error::from(io::ErrorKind::AddrInUse));
            for p in (0..tries.max(1)).filter_map(|i| port.checked_add(i)) {
                found = TcpListener::bind((Ipv4Addr::UNSPECIFIED, p));
                if found.is_ok() {
                    break;
                }
            }
            found?
        };
        let port = listener.local_addr()?.port();
        Ok(Self { listener, port })
    }

    pub(crate) fn port(&self) -> u16 {
        self.port
    }
}

impl Door for TcpDoor {
    fn accept(&self) -> io::Result<(Box<dyn Wire>, SocketAddr)> {
        loop {
            let (stream, peer) = self.listener.accept()?;
            if !is_lan(peer.ip()) {
                continue;
            }
            let _ = stream.set_nodelay(true);
            return Ok((Box::new(stream), peer));
        }
    }

    /// Uma ligação a nós próprios destrava o `accept`.
    fn wake(&self) {
        let _ = TcpStream::connect_timeout(
            &SocketAddr::from((Ipv4Addr::LOCALHOST, self.port)),
            Duration::from_millis(200),
        );
    }
}
