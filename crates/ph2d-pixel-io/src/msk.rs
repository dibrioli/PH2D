//! **Select ▸ Load from MSK file / Save to MSK file** — o `.msk` do Aseprite/LibreSprite.
//!
//! Formato INFERIDO das saídas do LibreSprite 1.3 corrido como oráculo (nunca lido):
//! [`docs/PixelArt/ferramentas/oraculo_msk.md`](../../../docs/PixelArt/ferramentas/oraculo_msk.md),
//! fixtura `tests/fixtures/oracle_msk.json`. Tudo little-endian:
//!
//! ```text
//! 0  u32  tamanho do ficheiro inteiro (tem de ser igual)
//! 4  u16  0x9500
//! 6  u16  largura   8 u16 altura   10 i16 x   12 i16 y   (a caixa da selecção, pode sair da tela)
//! 14 ..   zeros, excepto u16 = 1 no byte 18 (outro valor: erro); até ao byte 64
//! 64 u32  tamanho do bloco (6 + dados)   68 u16 2   70 linhas de ⌈largura/8⌉ bytes, bit 7 = x mais à esquerda
//! ```
//!
//! A escrita usa a caixa JUSTA e o enchimento da linha a `1` (o oráculo); a leitura ignora o
//! enchimento. Lê-se também o legado de exatamente 8000 bytes: bitmap 320×200 sem cabeçalho, na
//! origem — o oráculo lê-o 6 bytes adiante (defeito medido, não copiado).

use ph2d_pixel_core::Rect;
use ph2d_pixel_core::selection::Mask;

/// O `.msk` legado: 320×200 a 1 bit, sem cabeçalho.
pub const LEGACY_LEN: usize = 8000;
pub const LEGACY_W: u32 = 320;
pub const LEGACY_H: u32 = 200;
const MAGIC: u16 = 0x9500;
const HEADER_LEN: usize = 64;
const CHUNK_HEAD: usize = 6;
const CHUNK_MASK: u16 = 2;

/// Uma selecção como o `.msk` a guarda: a caixa (`x`, `y` e o tamanho de `mask`) e os bits dentro dela.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskFile {
    pub x: i32,
    pub y: i32,
    /// `width × height` da caixa; `mask.get(0,0)` é a célula `(x, y)` do sprite.
    pub mask: Mask,
}

/// Porque um `.msk` não se lê (o oráculo responde a todos com «Error loading .msk file»).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MskError {
    /// Menor que o cabeçalho, ou os dados acabam antes das linhas.
    Truncated,
    /// Nem o `0x9500` no byte 4 (com `1` no byte 18), nem os 8000 bytes do legado.
    NotMsk,
    /// O tamanho do byte 0 não é o do ficheiro (maior OU menor: o oráculo recusa os dois).
    SizeMismatch,
    /// Um bloco que não avança ou um resto partido — o oráculo fica parado; aqui é erro.
    BadChunk,
    /// Gravar: a caixa passa de `u16` de lado ou a origem sai de `i16` (o formato não a guarda).
    OutOfRange,
}

impl MaskFile {
    /// A caixa no sprite.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        Rect::new(self.x, self.y, self.mask.width(), self.mask.height())
    }

    /// A selecção do sprite cortada à caixa justa — o que o Save grava. `None` sem selecção (o
    /// oráculo recusa gravar: «Error saving .msk file»).
    #[must_use]
    pub fn from_selection(sel: &Mask) -> Option<Self> {
        let b = sel.bounds()?;
        let mut mask = Mask::empty(b.w, b.h);
        for (x, y) in sel.cells() {
            mask.set(x - b.x, y - b.y, true);
        }
        Some(Self {
            x: b.x,
            y: b.y,
            mask,
        })
    }

    /// A selecção numa tela `w × h`: a caixa no sítio, o que sai da tela cortado — o que o Load põe.
    #[must_use]
    pub fn to_selection(&self, w: u32, h: u32) -> Mask {
        let mut out = Mask::empty(w, h);
        for (x, y) in self.mask.cells() {
            out.set(x + self.x, y + self.y, true);
        }
        out
    }
}

fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}

fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}

fn unpack(rows: &[u8], w: u32, h: u32) -> Mask {
    let stride = w.div_ceil(8) as usize;
    let mut m = Mask::empty(w, h);
    for y in 0..h as usize {
        let row = &rows[y * stride..(y + 1) * stride];
        for x in 0..w as usize {
            if row[x / 8] & (0x80 >> (x % 8)) != 0 {
                m.set(x as i32, y as i32, true);
            }
        }
    }
    m
}

/// Lê um `.msk` (o actual ou o legado de 8000 bytes).
///
/// # Errors
/// [`MskError`] quando o oráculo também não o carrega.
pub fn read_msk(bytes: &[u8]) -> Result<MaskFile, MskError> {
    if bytes.len() == LEGACY_LEN {
        return Ok(MaskFile {
            x: 0,
            y: 0,
            mask: unpack(bytes, LEGACY_W, LEGACY_H),
        });
    }
    if bytes.len() < HEADER_LEN + CHUNK_HEAD {
        return Err(MskError::Truncated);
    }
    if u16_at(bytes, 4) != MAGIC {
        return Err(MskError::NotMsk);
    }
    if u32_at(bytes, 0) as usize != bytes.len() {
        return Err(MskError::SizeMismatch);
    }
    if u16_at(bytes, 18) != 1 {
        return Err(MskError::NotMsk);
    }
    let w = u32::from(u16_at(bytes, 6));
    let h = u32::from(u16_at(bytes, 8));
    let x = i32::from(u16_at(bytes, 10) as i16);
    let y = i32::from(u16_at(bytes, 12) as i16);
    let need = w.div_ceil(8) as usize * h as usize;
    let mut mask = None;
    let mut pos = HEADER_LEN;
    while pos < bytes.len() {
        // Um bloco cujo tamanho não avança, ou um resto menor que um cabeçalho de bloco, deixa o
        // oráculo PARADO para sempre: aqui é erro.
        if pos + CHUNK_HEAD > bytes.len() {
            return Err(MskError::BadChunk);
        }
        let size = u32_at(bytes, pos) as usize;
        if size < CHUNK_HEAD {
            return Err(MskError::BadChunk);
        }
        if mask.is_none() && u16_at(bytes, pos + 4) == CHUNK_MASK {
            let data = &bytes[pos + CHUNK_HEAD..];
            if data.len() < need {
                return Err(MskError::Truncated);
            }
            mask = Some(unpack(data, w, h));
        }
        pos = pos.saturating_add(size);
    }
    // Sem bloco de máscara o oráculo carrega sem erro e fica SEM selecção.
    Ok(MaskFile {
        x,
        y,
        // Vazia sem caixa: a caixa do cabeçalho não aloca nada que ninguém vá ler.
        mask: mask.unwrap_or_else(|| Mask::empty(0, 0)),
    })
}

/// Escreve o `.msk` byte a byte como o oráculo (enchimento a `1`).
///
/// # Errors
/// [`MskError::OutOfRange`] com a caixa maior que `u16` de lado ou a origem fora de `i16`.
pub fn write_msk(file: &MaskFile) -> Result<Vec<u8>, MskError> {
    let (w, h) = (file.mask.width(), file.mask.height());
    let side = |v: u32| u16::try_from(v).map_err(|_| MskError::OutOfRange);
    let pos = |v: i32| i16::try_from(v).map_err(|_| MskError::OutOfRange);
    let (w16, h16, x16, y16) = (side(w)?, side(h)?, pos(file.x)?, pos(file.y)?);
    let stride = w.div_ceil(8) as usize;
    let data_len = stride * h as usize;
    let total = HEADER_LEN + CHUNK_HEAD + data_len;
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&u32::try_from(total).expect("cabe em u32").to_le_bytes());
    out.extend_from_slice(&MAGIC.to_le_bytes());
    out.extend_from_slice(&w16.to_le_bytes());
    out.extend_from_slice(&h16.to_le_bytes());
    out.extend_from_slice(&x16.to_le_bytes());
    out.extend_from_slice(&y16.to_le_bytes());
    out.resize(HEADER_LEN, 0);
    out[18] = 1;
    out.extend_from_slice(
        &u32::try_from(CHUNK_HEAD + data_len)
            .expect("cabe em u32")
            .to_le_bytes(),
    );
    out.extend_from_slice(&CHUNK_MASK.to_le_bytes());
    for y in 0..h as i32 {
        let mut row = vec![0xFF_u8; stride];
        for x in 0..w as i32 {
            if !file.mask.get(x, y) {
                row[x as usize / 8] &= !(0x80 >> (x as usize % 8));
            }
        }
        out.extend_from_slice(&row);
    }
    Ok(out)
}
