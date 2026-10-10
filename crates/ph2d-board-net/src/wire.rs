//! O fio: cada quadro é o comprimento (`u32` LE) e os bytes. Depois do aperto de mão os bytes vão
//! cifrados (ChaCha20-Poly1305): uma chave e um contador de nonce por DIRECÇÃO, o nonce nunca viaja
//! (o receptor conta como o emissor, por isso um quadro repetido, trocado ou perdido não abre).

use std::io::{self, Read};

use chacha20poly1305::aead::{AeadInPlace, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, Tag};

/// O maior quadro (decifrado) — tecto de RAM do RECEPTOR, que guarda o quadro inteiro antes de o
/// abrir. O maior quadro legítimo é o retrato de quem entra (`Msg::Welcome` da `ph2d-board-sync`),
/// MEDIDO em 10/10 com 100 000 rectângulos: 9,0 MiB fora de sessão, 54,3 MiB ao vivo sem texto e
/// 57,5 MiB ao vivo com 34 B de texto cada (603 B/elemento: os carimbos por registo são ~4/5 disso).
/// 256 MiB = 4,4× esse retrato ≈ 445 mil elementos ao vivo. A memória acompanha os bytes que CHEGAM,
/// nunca o comprimento anunciado: um cabeçalho mentiroso não aloca nada.
pub const MAX_FRAME: usize = 256 << 20;

/// O tecto ANTES de autenticar: a maior mensagem do aperto de mão tem 66 B.
pub(crate) const MAX_SHAKE: usize = 128;

pub(crate) const TAG: usize = 16;

/// A memória cresce aos saltos de no máximo o que já chegou (amortizado, nunca acima do anunciado).
const FIRST_CHUNK: usize = 64 << 10;

pub(crate) fn invalid(msg: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

/// Um quadro em claro (o aperto de mão).
pub(crate) fn frame(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + bytes.len());
    out.extend_from_slice(&len_le(bytes.len()));
    out.extend_from_slice(bytes);
    out
}

fn len_le(n: usize) -> [u8; 4] {
    // Quem chama já recusou acima do tecto, que cabe em `u32`.
    u32::try_from(n).unwrap_or(u32::MAX).to_le_bytes()
}

/// Lê um quadro de até `max` bytes. `UnexpectedEof` = a ligação fechou (a meio ou entre quadros).
pub(crate) fn read_frame(r: &mut impl Read, max: usize) -> io::Result<Vec<u8>> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let n = u32::from_le_bytes(len) as usize;
    if n > max {
        return Err(invalid("quadro acima do tecto"));
    }
    let mut buf = Vec::new();
    while buf.len() < n {
        let start = buf.len();
        let want = (n - start).min(start.max(FIRST_CHUNK));
        buf.reserve_exact(want);
        buf.resize(start + want, 0);
        r.read_exact(&mut buf[start..])?;
    }
    Ok(buf)
}

fn nonce(n: u64) -> Nonce {
    // 96 bits: o contador nos 64 de baixo, zeros em cima. Nunca se repete numa chave: o contador só
    // sobe e esgota-se (fecha a ligação) antes de dar a volta — 2⁶⁴ quadros nunca chegam.
    let mut b = [0u8; 12];
    b[..8].copy_from_slice(&n.to_le_bytes());
    *Nonce::from_slice(&b)
}

/// Cifra numa direcção.
pub(crate) struct Seal {
    aead: ChaCha20Poly1305,
    n: u64,
}

impl Seal {
    pub(crate) fn new(key: &[u8; 32]) -> Self {
        Self {
            aead: ChaCha20Poly1305::new(Key::from_slice(key)),
            n: 0,
        }
    }

    /// O quadro pronto para o fio: comprimento, texto cifrado, etiqueta.
    pub(crate) fn seal(&mut self, plain: &[u8]) -> io::Result<Vec<u8>> {
        if plain.len() > MAX_FRAME {
            return Err(invalid("quadro acima do tecto"));
        }
        let n = self.n;
        self.n = n
            .checked_add(1)
            .ok_or_else(|| invalid("nonces esgotados"))?;
        let mut out = Vec::with_capacity(4 + plain.len() + TAG);
        out.extend_from_slice(&len_le(plain.len() + TAG));
        out.extend_from_slice(plain);
        let tag = self
            .aead
            .encrypt_in_place_detached(&nonce(n), b"", &mut out[4..])
            .map_err(|_| invalid("cifra"))?;
        out.extend_from_slice(&tag);
        Ok(out)
    }

    #[cfg(test)]
    pub(crate) fn set_counter(&mut self, n: u64) {
        self.n = n;
    }
}

/// Decifra na outra direcção.
pub(crate) struct Open {
    aead: ChaCha20Poly1305,
    n: u64,
}

impl Open {
    pub(crate) fn new(key: &[u8; 32]) -> Self {
        Self {
            aead: ChaCha20Poly1305::new(Key::from_slice(key)),
            n: 0,
        }
    }

    /// Lê e abre o próximo quadro. Um byte trocado ⇒ erro (a ligação fecha; nunca entrega lixo).
    pub(crate) fn read(&mut self, r: &mut impl Read) -> io::Result<Vec<u8>> {
        let wire = read_frame(r, MAX_FRAME + TAG)?;
        self.open(wire)
    }

    pub(crate) fn open(&mut self, mut wire: Vec<u8>) -> io::Result<Vec<u8>> {
        let Some(body) = wire.len().checked_sub(TAG) else {
            return Err(invalid("quadro sem etiqueta"));
        };
        let n = self.n;
        self.n = n
            .checked_add(1)
            .ok_or_else(|| invalid("nonces esgotados"))?;
        let tag = Tag::clone_from_slice(&wire[body..]);
        wire.truncate(body);
        self.aead
            .decrypt_in_place_detached(&nonce(n), b"", &mut wire, &tag)
            .map_err(|_| invalid("quadro adulterado"))?;
        Ok(wire)
    }
}
