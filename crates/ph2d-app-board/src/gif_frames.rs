//! W18 — **os quadros de um GIF, um de cada vez, num iterador que muda de fio** (`Send`): o desenho
//! descodifica cada quadro como um passo no grupo de fios das imagens (`ph2d_board_render::anim`), e o
//! iterador do `image` (`Frames`, uma `Box<dyn Iterator>` sem `Send`) não pode viajar entre fios.
//!
//! A composição é a do `image` 0.25 (`codecs/gif.rs` `GifFrameIterator`, a lei dos visualizadores): um
//! píxel de alfa 0 deixa ver o anterior; depois do quadro, `Keep`/`Any` guarda-o, `Background` limpa a
//! transparente, `Previous` deixa o que havia. O gate compara as duas leituras píxel a píxel
//! (`gif_frames_tests`). O tecto é o da porta das imagens (`ph2d_asset::loader::MAX_DIMENSION` por lado);
//! o `image` lia o GIF SEM tecto (`Limits::no_limits`).

use gif::{ColorOutput, DecodeOptions, Decoder, DisposalMethod};
use ph2d_board_render::Pixels;

/// Os quadros compostos sobre o ecrã lógico do GIF, cada um com a duração do ficheiro em ms.
pub(crate) struct GifFrames {
    dec: Decoder<std::io::Cursor<Vec<u8>>>,
    size: [u32; 2],
    /// O ecrã lógico depois do descarte do quadro anterior.
    kept: Vec<u8>,
}

impl GifFrames {
    /// `None` = não é um GIF que se lê, ou o ecrã lógico passa do tecto.
    pub(crate) fn new(bytes: &[u8]) -> Option<Self> {
        let mut opts = DecodeOptions::new();
        opts.set_color_output(ColorOutput::RGBA);
        let dec = opts.read_info(std::io::Cursor::new(bytes.to_vec())).ok()?;
        let size = [u32::from(dec.width()), u32::from(dec.height())];
        let cap = ph2d_asset::loader::MAX_DIMENSION;
        if size[0] == 0 || size[1] == 0 || size[0] > cap || size[1] > cap {
            return None;
        }
        let kept = vec![0; size[0] as usize * size[1] as usize * 4];
        Some(Self { dec, size, kept })
    }
}

impl Iterator for GifFrames {
    type Item = (Pixels, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let f = self.dec.read_next_frame().ok()??;
        let (left, top) = (u32::from(f.left), u32::from(f.top));
        let (fw, fh) = (u32::from(f.width), u32::from(f.height));
        let [w, h] = self.size;
        let mut out = self.kept.clone();
        for fy in 0..fh {
            let y = top + fy;
            if y >= h {
                break;
            }
            for fx in 0..fw {
                let x = left + fx;
                if x >= w {
                    break;
                }
                let s = (fy * fw + fx) as usize * 4;
                let d = (y * w + x) as usize * 4;
                let px = f.buffer.get(s..s + 4)?;
                // Alfa 0 deixa ver o que havia.
                let cur: [u8; 4] = if px[3] == 0 {
                    out[d..d + 4].try_into().ok()?
                } else {
                    px.try_into().ok()?
                };
                out[d..d + 4].copy_from_slice(&cur);
                match f.dispose {
                    DisposalMethod::Any | DisposalMethod::Keep => {
                        self.kept[d..d + 4].copy_from_slice(&cur);
                    }
                    DisposalMethod::Background => self.kept[d..d + 4].fill(0),
                    DisposalMethod::Previous => {}
                }
            }
        }
        let ms = u32::from(f.delay) * 10;
        Some((
            Pixels {
                size: self.size,
                rgba: out,
            },
            ms,
        ))
    }
}

#[cfg(test)]
#[path = "gif_frames_tests.rs"]
mod tests;
