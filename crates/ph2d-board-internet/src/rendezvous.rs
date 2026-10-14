//! O PONTO DE ENCONTRO sem servidor nosso: a DHT do BitTorrent (BEP 44, itens MUTÁVEIS assinados com
//! ed25519). Da chave do convite saem duas chaves de assinatura — a do anfitrião e a dos convidados —
//! e a cifra dos valores: quem não tem o convite não acha o item, e quem o guarda não o lê.
//!
//! O anfitrião publica os seus candidatos; cada convidado publica os seus no item dos convidados (o
//! mais recente ganha: dois a entrar no mesmo segundo só atrasam um ao outro, cada um volta a publicar
//! até entrar); o anfitrião vigia esse item e fura para cada convidado novo.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use futures_lite::StreamExt as _;
use futures_lite::future::block_on;
use hkdf::Hkdf;
use mainline::async_dht::AsyncDht;
use mainline::{Dht, MutableItem, SigningKey};
use ph2d_board_net::{InviteKey, MAX_ADDRS, random_u64};
use sha2::Sha256;

const SALT: &[u8] = b"ph2d-board dht v1";
const NONCE: usize = 12;

/// Como chegar à DHT.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DhtMode {
    /// A DHT pública (os nós de arranque do BitTorrent).
    Public,
    /// Uma DHT própria (testes: `mainline::Testnet`).
    Bootstrap(Vec<String>),
    /// Sem ponto de encontro: só os endereços do convite.
    Off,
}

/// De quem é o item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Host,
    Guest,
}

impl Role {
    fn tag(self) -> u8 {
        match self {
            Self::Host => 1,
            Self::Guest => 2,
        }
    }
}

struct Keys {
    host: SigningKey,
    guest: SigningKey,
    seal: ChaCha20Poly1305,
}

fn keys(invite: &InviteKey) -> Keys {
    let hk = Hkdf::<Sha256>::new(Some(SALT), invite.bytes());
    let get = |info: &[u8]| {
        let mut out = [0u8; 32];
        // 32 B cabem sempre no HKDF-SHA256.
        let _ = hk.expand(info, &mut out);
        out
    };
    Keys {
        host: SigningKey::from_bytes(&get(b"sign host")),
        guest: SigningKey::from_bytes(&get(b"sign guest")),
        seal: ChaCha20Poly1305::new(Key::from_slice(&get(b"seal"))),
    }
}

/// `n · (família · endereço · porta)*`.
fn encode(addrs: &[SocketAddr]) -> Vec<u8> {
    let addrs = &addrs[..addrs.len().min(MAX_ADDRS)];
    let mut out = vec![u8::try_from(addrs.len()).unwrap_or(0)];
    for a in addrs {
        match a.ip() {
            IpAddr::V4(v4) => {
                out.push(4);
                out.extend_from_slice(&v4.octets());
            }
            IpAddr::V6(v6) => {
                out.push(6);
                out.extend_from_slice(&v6.octets());
            }
        }
        out.extend_from_slice(&a.port().to_be_bytes());
    }
    out
}

fn decode(d: &[u8]) -> Option<Vec<SocketAddr>> {
    let (&n, mut rest) = d.split_first()?;
    let mut out = Vec::new();
    for _ in 0..usize::from(n).min(MAX_ADDRS) {
        let (&fam, r) = rest.split_first()?;
        let (ip, r): (IpAddr, &[u8]) = match fam {
            4 => (
                Ipv4Addr::from(<[u8; 4]>::try_from(r.get(..4)?).ok()?).into(),
                &r[4..],
            ),
            6 => (
                Ipv6Addr::from(<[u8; 16]>::try_from(r.get(..16)?).ok()?).into(),
                &r[16..],
            ),
            _ => return None,
        };
        let port = u16::from_be_bytes(<[u8; 2]>::try_from(r.get(..2)?).ok()?);
        rest = &r[2..];
        out.push(SocketAddr::new(ip, port));
    }
    Some(out)
}

/// Cifra os candidatos de `role` (o papel vai como dado associado: um item não passa pelo outro).
fn seal(k: &Keys, role: Role, addrs: &[SocketAddr]) -> Vec<u8> {
    let mut nonce = [0u8; NONCE];
    nonce[..8].copy_from_slice(&random_u64().to_le_bytes());
    nonce[8..].copy_from_slice(&random_u64().to_le_bytes()[..4]);
    let plain = encode(addrs);
    let body = k
        .seal
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &plain,
                aad: &[role.tag()],
            },
        )
        .unwrap_or_default();
    [&nonce[..], &body].concat()
}

fn open(k: &Keys, role: Role, value: &[u8]) -> Option<Vec<SocketAddr>> {
    let nonce = value.get(..NONCE)?;
    let plain = k
        .seal
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: &value[NONCE..],
                aad: &[role.tag()],
            },
        )
        .ok()?;
    decode(&plain)
}

/// O número de versão de um item: milissegundos do relógio (cada publicação é mais nova).
fn now_seq() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

/// A ligação à DHT para UM convite.
pub struct Rendezvous {
    dht: AsyncDht,
    keys: Keys,
}

impl Rendezvous {
    /// `Ok(None)` = [`DhtMode::Off`].
    ///
    /// # Errors
    /// O socket da DHT não abriu.
    pub fn new(mode: &DhtMode, invite: &InviteKey) -> std::io::Result<Option<Self>> {
        let dht = match mode {
            DhtMode::Off => return Ok(None),
            DhtMode::Public => Dht::client()?,
            DhtMode::Bootstrap(nodes) => Dht::builder().bootstrap(nodes).build()?,
        };
        Ok(Some(Self {
            dht: dht.as_async(),
            keys: keys(invite),
        }))
    }

    fn signer(&self, role: Role) -> &SigningKey {
        match role {
            Role::Host => &self.keys.host,
            Role::Guest => &self.keys.guest,
        }
    }

    /// Publica os candidatos de `role` (bloqueia o tempo de uma escrita na DHT, segundos).
    #[must_use]
    pub fn publish(&self, role: Role, addrs: &[SocketAddr]) -> bool {
        let value = seal(&self.keys, role, addrs);
        let item = MutableItem::new(self.signer(role).clone(), &value, now_seq(), None);
        block_on(self.dht.put_mutable(item, None)).is_ok()
    }

    /// O item mais recente de `role` mais novo que `newer_than` (bloqueia o tempo de uma leitura).
    #[must_use]
    pub fn fetch(&self, role: Role, newer_than: Option<i64>) -> Option<(i64, Vec<SocketAddr>)> {
        let key = self.signer(role).verifying_key().to_bytes();
        let items: Vec<MutableItem> =
            block_on(self.dht.get_mutable(&key, None, newer_than).collect());
        items
            .iter()
            .filter_map(|it| Some((it.seq(), open(&self.keys, role, it.value())?)))
            .max_by_key(|(seq, _)| *seq)
    }
}

#[cfg(test)]
#[path = "rendezvous_tests.rs"]
mod tests;
