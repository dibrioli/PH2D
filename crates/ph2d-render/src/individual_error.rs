//! O **erro da loja de texturas individuais** — o [`IndividualTextureError`] e a mensagem de cada
//! variante —, irmão de `individual.rs` por tecto de LOC. O caminho público não muda:
//! `individual.rs` re-exporta-o.
//!
//! Corte mecânico: o tipo e os dois `impl` saíram inteiros, verbatim.

/// Errors returned by [`IndividualTextureStore::acquire`] and
/// [`IndividualTextureStore::readback`].
#[derive(Debug)]
pub enum IndividualTextureError {
    PixelLengthMismatch {
        got: usize,
        expected: usize,
    },
    /// `readback`'s requested texture id has no entry in the store.
    NotFound(u32),
    /// A [`IndividualTextureStore::replace_pixels_region`] sub-rect lies
    /// outside the entry's current texture dimensions (a partial write
    /// past the edge would corrupt neighbouring rows or panic in wgpu).
    RegionOutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        tex_width: u32,
        tex_height: u32,
    },
    /// **Um caminho de 8 bits foi apontado a uma textura de 16.**
    ///
    /// ⚠️ Existe porque o modo de falha alternativo é **corrupção silenciosa**: escrever bytes com
    /// passo de linha `w × 4` numa textura de `w × 8` não dá erro do wgpu (a validação só exige
    /// `bytes_per_row >= w × block`), preenche metade de cada linha e deixa a outra metade com o
    /// que lá estava. O sintoma seria a imagem esticada ao meio, sem uma palavra.
    ///
    /// Irmão do pânico de 2026-08-20 — o mesmo erro de *stride*, do lado da escrita.
    EightBitWriteToSixteenBitTexture {
        id: u32,
    },
    /// The GPU command queue accepted the copy but the buffer never
    /// finished mapping. Worth surfacing distinctly from a generic
    /// I/O error so the caller can decide whether to retry (device
    /// likely lost — see ADR-0020) or fail loudly.
    ReadbackFailed(String),
    /// A [`IndividualTextureStore::copy_from_texture`] source texture's
    /// dimensions did not match the destination entry's. A `copy_texture_to_
    /// texture` past the edge would be a wgpu validation error, so reject it
    /// at the boundary with a precise diagnostic.
    CopySizeMismatch {
        width: u32,
        height: u32,
        tex_width: u32,
        tex_height: u32,
    },
}

impl std::fmt::Display for IndividualTextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PixelLengthMismatch { got, expected } => write!(
                f,
                "rgba buffer length {got} doesn't match width*height*4 = {expected}"
            ),
            Self::NotFound(id) => write!(f, "no individual texture with id {id}"),
            Self::RegionOutOfBounds {
                x,
                y,
                width,
                height,
                tex_width,
                tex_height,
            } => write!(
                f,
                "region {width}×{height} at ({x},{y}) exceeds texture {tex_width}×{tex_height}"
            ),
            Self::EightBitWriteToSixteenBitTexture { id } => write!(
                f,
                "refused an 8-bit write to the 16-bit individual texture {id}: the row stride \
                 differs (w*4 vs w*8), so the write would fill half of every row and leave the \
                 rest stale, silently"
            ),
            Self::ReadbackFailed(detail) => write!(f, "GPU readback failed: {detail}"),
            Self::CopySizeMismatch {
                width,
                height,
                tex_width,
                tex_height,
            } => write!(
                f,
                "copy source {width}×{height} doesn't match texture {tex_width}×{tex_height}"
            ),
        }
    }
}

impl std::error::Error for IndividualTextureError {}
