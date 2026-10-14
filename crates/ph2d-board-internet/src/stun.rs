//! STUN (RFC 5389), só o necessário: o pedido de ligação e o endereço mapeado da resposta — o mesmo
//! que a sonda `docs/MiroClone/ferramentas/internet_oracle/sonda_rede.py` faz.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub(crate) const MAGIC: u32 = 0x2112_A442;
const BINDING_REQUEST: u16 = 0x0001;
const BINDING_SUCCESS: u16 = 0x0101;
const MAPPED: u16 = 0x0001;
const XOR_MAPPED: u16 = 0x0020;
/// O código antigo do XOR-MAPPED-ADDRESS (servidores pré-RFC 5389).
const XOR_MAPPED_OLD: u16 = 0x8020;

pub(crate) type TxId = [u8; 12];

pub(crate) fn request(tx: &TxId) -> [u8; 20] {
    let mut out = [0u8; 20];
    out[..2].copy_from_slice(&BINDING_REQUEST.to_be_bytes());
    out[4..8].copy_from_slice(&MAGIC.to_be_bytes());
    out[8..].copy_from_slice(tx);
    out
}

fn u16_at(d: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*d.get(i)?, *d.get(i + 1)?]))
}

/// A resposta de sucesso: a transacção e o endereço que o servidor viu. O XOR ganha ao antigo.
pub(crate) fn response(d: &[u8]) -> Option<(TxId, SocketAddr)> {
    if d.len() < 20 || u16_at(d, 0)? != BINDING_SUCCESS || d[4..8] != MAGIC.to_be_bytes() {
        return None;
    }
    let tx: TxId = d[8..20].try_into().ok()?;
    let end = (20 + usize::from(u16_at(d, 2)?)).min(d.len());
    let (mut pos, mut plain, mut xored) = (20, None, None);
    while pos + 4 <= end {
        let (kind, len) = (u16_at(d, pos)?, usize::from(u16_at(d, pos + 2)?));
        let val = d.get(pos + 4..pos + 4 + len)?;
        match kind {
            MAPPED => plain = plain.or(addr(val, None)),
            XOR_MAPPED | XOR_MAPPED_OLD => xored = xored.or(addr(val, Some(&tx))),
            _ => {}
        }
        pos += 4 + len + (4 - len % 4) % 4;
    }
    Some((tx, xored.or(plain)?))
}

/// `famíla · porta · endereço`; com `tx` é o XOR (porta e IPv4 com o cookie, IPv6 com cookie + tx).
fn addr(val: &[u8], tx: Option<&TxId>) -> Option<SocketAddr> {
    let mut key = [0u8; 16];
    if let Some(tx) = tx {
        key[..4].copy_from_slice(&MAGIC.to_be_bytes());
        key[4..].copy_from_slice(tx);
    }
    let port = u16_at(val, 2)? ^ u16::from_be_bytes([key[0], key[1]]);
    let ip = match *val.get(1)? {
        1 => {
            let b: [u8; 4] = val.get(4..8)?.try_into().ok()?;
            IpAddr::V4(Ipv4Addr::from(std::array::from_fn::<u8, 4, _>(|i| {
                b[i] ^ key[i]
            })))
        }
        2 => {
            let b: [u8; 16] = val.get(4..20)?.try_into().ok()?;
            IpAddr::V6(Ipv6Addr::from(std::array::from_fn::<u8, 16, _>(|i| {
                b[i] ^ key[i]
            })))
        }
        _ => return None,
    };
    Some(SocketAddr::new(ip, port))
}

/// A resposta que um servidor STUN daria (os testes fazem de servidor com ela).
#[cfg(test)]
pub(crate) fn answer(tx: &TxId, seen: SocketAddr) -> Vec<u8> {
    let mut key = [0u8; 16];
    key[..4].copy_from_slice(&MAGIC.to_be_bytes());
    key[4..].copy_from_slice(tx);
    let mut val = vec![0u8];
    let raw: Vec<u8> = match seen.ip() {
        IpAddr::V4(v4) => {
            val.push(1);
            v4.octets().to_vec()
        }
        IpAddr::V6(v6) => {
            val.push(2);
            v6.octets().to_vec()
        }
    };
    val.extend_from_slice(&(seen.port() ^ u16::from_be_bytes([key[0], key[1]])).to_be_bytes());
    val.extend(raw.iter().zip(key).map(|(b, k)| b ^ k));
    let mut out = Vec::new();
    out.extend_from_slice(&BINDING_SUCCESS.to_be_bytes());
    out.extend_from_slice(&u16::try_from(4 + val.len()).unwrap_or(0).to_be_bytes());
    out.extend_from_slice(&MAGIC.to_be_bytes());
    out.extend_from_slice(tx);
    out.extend_from_slice(&XOR_MAPPED.to_be_bytes());
    out.extend_from_slice(&u16::try_from(val.len()).unwrap_or(0).to_be_bytes());
    out.extend_from_slice(&val);
    out
}

#[cfg(test)]
#[path = "stun_tests.rs"]
mod tests;
