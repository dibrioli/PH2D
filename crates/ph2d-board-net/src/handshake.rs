//! O aperto de mão: SPAKE2 sobre o código curto, chaves por direcção (HKDF-SHA256) e a confirmação de
//! chave (HMAC do transcript) dos DOIS lados antes de qualquer quadro. Código errado ⇒ as chaves
//! diferem ⇒ a confirmação falha; nenhum quadro chega a ser cifrado.
//!
//! ```text
//! convidado → anfitrião   MAGIC · VERSION · msg_a (33 B)
//! anfitrião → convidado   GO · msg_b (33 B) · mac_h   |   LOCKED · segundos (u32 LE)   |   OTHER · versão
//! convidado → anfitrião   mac_g                        (código errado: fecha sem enviar)
//! ```

use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use spake2::{Ed25519Group, Identity, Password, Spake2};

use crate::JoinError;
use crate::code::{Code, Gate};
use crate::wire::{MAX_SHAKE, Open, Seal, frame, read_frame};

const MAGIC: &[u8; 8] = b"PH2D-BRD";
/// A versão do TRANSPORTE (a do protocolo do quadro é da `ph2d-board-sync`).
pub(crate) const VERSION: u8 = 1;
const ID_GUEST: &[u8] = b"ph2d-board guest";
const ID_HOST: &[u8] = b"ph2d-board host";
const SALT: &[u8] = b"ph2d-board-net v1";
const MSG: usize = 33;
const MAC: usize = 32;

const GO: u8 = 0;
const LOCKED: u8 = 1;
const OTHER: u8 = 2;

pub(crate) struct Keys {
    pub(crate) seal: Seal,
    pub(crate) open: Open,
}

struct Derived {
    g2h: [u8; 32],
    h2g: [u8; 32],
    conf_h: [u8; 32],
    conf_g: [u8; 32],
}

fn derive(k: &[u8]) -> Derived {
    let hk = Hkdf::<Sha256>::new(Some(SALT), k);
    let get = |info: &[u8]| {
        let mut out = [0u8; 32];
        // 32 B cabem sempre no HKDF-SHA256 (o tecto é 255 × 32).
        let _ = hk.expand(info, &mut out);
        out
    };
    Derived {
        g2h: get(b"guest->host"),
        h2g: get(b"host->guest"),
        conf_h: get(b"confirm host"),
        conf_g: get(b"confirm guest"),
    }
}

fn mac(key: &[u8; 32], transcript: &[u8]) -> Hmac<Sha256> {
    // O HMAC aceita chaves de qualquer tamanho: `new_from_slice` nunca falha aqui.
    let mut m = <Hmac<Sha256> as Mac>::new_from_slice(key).unwrap_or_else(|_| unreachable!());
    m.update(transcript);
    m
}

fn with<T>(gate: &Mutex<Gate>, f: impl FnOnce(&mut Gate) -> T) -> T {
    f(&mut gate
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner))
}

#[cfg(test)]
fn password(code: &Code) -> Password {
    Password::new(code.bytes())
}

/// Um travão opcional: a entrada pela internet (chave longa) não tem — trancá-la seria um DoS.
fn begin(gate: Option<&Mutex<Gate>>, now: Instant) -> Result<(), Duration> {
    gate.map_or(Ok(()), |g| with(g, |g| g.begin(now)))
}

/// Como acabou o aperto de mão no anfitrião.
pub(crate) enum HostShake {
    Joined(Keys),
    /// Testou um código e errou (ou desistiu depois de receber a resposta: também é um palpite).
    Wrong,
    /// Recusado sem testar (bloqueio, outra versão, lixo, queda antes do código).
    Refused,
}

pub(crate) fn host(
    s: &mut (impl Read + Write),
    code: &Code,
    gate: &Mutex<Gate>,
    now: &dyn Fn() -> Instant,
) -> HostShake {
    host_with(s, code.bytes(), Some(gate), now)
}

/// O aperto de mão do anfitrião com o segredo da ENTRADA (o código curto com o travão, ou a chave do
/// convite sem ele).
pub(crate) fn host_with(
    s: &mut (impl Read + Write),
    secret: &[u8],
    gate: Option<&Mutex<Gate>>,
    now: &dyn Fn() -> Instant,
) -> HostShake {
    let Ok(hello) = read_frame(s, MAX_SHAKE) else {
        return HostShake::Refused;
    };
    if hello.len() != MAGIC.len() + 1 + MSG || !hello.starts_with(MAGIC) {
        return HostShake::Refused;
    }
    if hello[MAGIC.len()] != VERSION {
        let _ = s.write_all(&frame(&[OTHER, VERSION]));
        return HostShake::Refused;
    }
    if let Err(wait) = begin(gate, now()) {
        let secs = u32::try_from(wait.as_secs() + 1).unwrap_or(u32::MAX);
        let mut reply = vec![LOCKED];
        reply.extend_from_slice(&secs.to_le_bytes());
        let _ = s.write_all(&frame(&reply));
        return HostShake::Refused;
    }

    let msg_a = &hello[MAGIC.len() + 1..];
    let (sp, msg_b) = Spake2::<Ed25519Group>::start_b(
        &Password::new(secret),
        &Identity::new(ID_GUEST),
        &Identity::new(ID_HOST),
    );
    let Ok(k) = sp.finish(msg_a) else {
        if let Some(g) = gate {
            with(g, Gate::abandon);
        }
        return HostShake::Refused;
    };
    let transcript = [msg_a, &msg_b].concat();
    let d = derive(&k);
    let mut reply = vec![GO];
    reply.extend_from_slice(&msg_b);
    reply.extend_from_slice(&mac(&d.conf_h, &transcript).finalize().into_bytes());

    let ok = s.write_all(&frame(&reply)).is_ok()
        && read_frame(s, MAX_SHAKE)
            .is_ok_and(|m| mac(&d.conf_g, &transcript).verify_slice(&m).is_ok());
    if let Some(g) = gate {
        with(g, |g| g.end(ok, now()));
    }
    if !ok {
        return HostShake::Wrong;
    }
    HostShake::Joined(Keys {
        seal: Seal::new(&d.h2g),
        open: Open::new(&d.g2h),
    })
}

#[cfg(test)]
pub(crate) fn guest(s: &mut (impl Read + Write), code: &Code) -> Result<Keys, JoinError> {
    guest_with(s, code.bytes())
}

pub(crate) fn guest_with(s: &mut (impl Read + Write), secret: &[u8]) -> Result<Keys, JoinError> {
    let (sp, msg_a) = Spake2::<Ed25519Group>::start_a(
        &Password::new(secret),
        &Identity::new(ID_GUEST),
        &Identity::new(ID_HOST),
    );
    let mut hello = MAGIC.to_vec();
    hello.push(VERSION);
    hello.extend_from_slice(&msg_a);
    s.write_all(&frame(&hello)).map_err(|_| JoinError::Broken)?;

    let reply = read_frame(s, MAX_SHAKE).map_err(|_| JoinError::Broken)?;
    let (msg_b, mac_h) = match reply.first() {
        Some(&GO) if reply.len() == 1 + MSG + MAC => (&reply[1..=MSG], &reply[1 + MSG..]),
        Some(&LOCKED) if reply.len() == 5 => {
            let secs = u32::from_le_bytes([reply[1], reply[2], reply[3], reply[4]]);
            return Err(JoinError::Locked(Duration::from_secs(u64::from(secs))));
        }
        Some(&OTHER) => return Err(JoinError::Incompatible),
        _ => return Err(JoinError::Broken),
    };
    let k = sp.finish(msg_b).map_err(|_| JoinError::Broken)?;
    let transcript = [&msg_a[..], msg_b].concat();
    let d = derive(&k);
    if mac(&d.conf_h, &transcript).verify_slice(mac_h).is_err() {
        return Err(JoinError::WrongCode);
    }
    let mac_g = mac(&d.conf_g, &transcript).finalize().into_bytes();
    s.write_all(&frame(&mac_g)).map_err(|_| JoinError::Broken)?;
    Ok(Keys {
        seal: Seal::new(&d.g2h),
        open: Open::new(&d.h2g),
    })
}

#[cfg(test)]
#[path = "handshake_tests.rs"]
mod tests;
