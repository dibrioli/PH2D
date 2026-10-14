//! O CONVITE pela internet: um link com a chave de 128 bits no fragmento (`#`) e os endereços onde o
//! anfitrião se deixa encontrar. O código curto da rede local (~29,7 bits) só aguenta por causa do
//! bloqueio GLOBAL de 5 erros/30 s — na internet aberta esse bloqueio deixava qualquer um trancar a sala;
//! a chave longa dispensa-o (doc `11_ao_vivo_pela_internet.md` §2: o Excalidraw usa a mesma ordem).
//!
//! ```text
//! ph2d-board://v1/203.0.113.5:47321,[2001:db8::1]:47321#3q2-7wAAAAAAAAAAAAAAAA
//! ```

use std::fmt;
use std::net::SocketAddr;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand_core::{OsRng, RngCore};

/// 128 bits.
pub const KEY_LEN: usize = 16;
/// O começo de todo convite.
pub const SCHEME: &str = "ph2d-board://";
const V1: &str = "v1";
/// Os candidatos de um anfitrião são as suas interfaces e o endereço visto de fora (uns 4 numa casa);
/// o tecto é o de pacotes que um convidado manda por tentativa a endereços tirados de um texto colado.
pub const MAX_ADDRS: usize = 16;

/// A chave do convite: o segredo do aperto de mão na entrada pela internet.
#[derive(Clone, PartialEq, Eq)]
pub struct InviteKey([u8; KEY_LEN]);

impl InviteKey {
    #[must_use]
    pub fn random() -> Self {
        let mut b = [0u8; KEY_LEN];
        OsRng.fill_bytes(&mut b);
        Self(b)
    }

    #[must_use]
    pub fn from_bytes(b: [u8; KEY_LEN]) -> Self {
        Self(b)
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

/// Nunca imprime o segredo.
impl fmt::Debug for InviteKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InviteKey(..)")
    }
}

/// Porque o texto colado não é um convite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InviteError {
    /// Não começa por [`SCHEME`] ou falta o `#`.
    NotALink,
    /// Um convite de outra versão do PH2D.
    OtherVersion,
    /// A chave não tem 128 bits em base64url.
    BadKey,
    /// Um endereço ilegível, ou mais de [`MAX_ADDRS`].
    BadAddr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invite {
    pub key: InviteKey,
    /// Onde o anfitrião se deixa encontrar sem o ponto de encontro (pode ir vazio).
    pub addrs: Vec<SocketAddr>,
}

impl Invite {
    /// O link a copiar.
    #[must_use]
    pub fn link(&self) -> String {
        let addrs: Vec<String> = self
            .addrs
            .iter()
            .take(MAX_ADDRS)
            .map(ToString::to_string)
            .collect();
        format!(
            "{SCHEME}{V1}/{}#{}",
            addrs.join(","),
            URL_SAFE_NO_PAD.encode(self.key.0)
        )
    }

    /// O que a pessoa colou: espaços e quebras de linha (o chat parte links longos) são ignorados.
    ///
    /// # Errors
    /// Ver [`InviteError`].
    pub fn parse(text: &str) -> Result<Self, InviteError> {
        let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        // O esquema não distingue maiúsculas (um link escrito à mão); a chave sim.
        let rest = text
            .get(..SCHEME.len())
            .filter(|p| p.eq_ignore_ascii_case(SCHEME))
            .and_then(|_| text.get(SCHEME.len()..))
            .ok_or(InviteError::NotALink)?;
        let (path, key) = rest.split_once('#').ok_or(InviteError::NotALink)?;
        let (version, addrs) = path.split_once('/').unwrap_or((path, ""));
        if version != V1 {
            return Err(InviteError::OtherVersion);
        }
        let key: [u8; KEY_LEN] = URL_SAFE_NO_PAD
            .decode(key)
            .ok()
            .and_then(|b| b.try_into().ok())
            .ok_or(InviteError::BadKey)?;
        let addrs = addrs
            .split(',')
            .filter(|a| !a.is_empty())
            .map(|a| a.parse::<SocketAddr>().map_err(|_| InviteError::BadAddr))
            .collect::<Result<Vec<_>, _>>()?;
        if addrs.len() > MAX_ADDRS {
            return Err(InviteError::BadAddr);
        }
        Ok(Self {
            key: InviteKey(key),
            addrs,
        })
    }
}

#[cfg(test)]
#[path = "invite_tests.rs"]
mod tests;
