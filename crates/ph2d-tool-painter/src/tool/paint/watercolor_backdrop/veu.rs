//! **O VÉU de humidade** (prévia do cartão Wetness): o papel molhado escurece em `alfa ∝ humidade`,
//! com uma franja suave. Construído só quando a CHAVE muda — a revisão da humidade
//! ([`PainterTool::canvas_wet_revision`]), o recorte, o passo ou a força — e guardado como
//! [`StableImage`], cujo id fica estável no Vello (sem re-upload por quadro). B16.10:
//! `docs/Painter/missao_4k/BACKLOG.md`.
//!
//! [`PainterTool::canvas_wet_revision`]: crate::PainterTool::canvas_wet_revision

use ph2d_vector::StableImage;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Fonte ÚNICA das revisões da humidade: valores nunca repetidos no processo, então dois tools ou um
/// mapa recriado nunca reutilizam a chave de outro.
static REVISOES: AtomicU64 = AtomicU64::new(1);

/// Uma revisão nova (cada escrita do mapa de humidade chama isto).
pub(crate) fn nova_revisao() -> u64 {
    REVISOES.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Chave {
    rev: u64,
    canvas_w: usize,
    rect: [usize; 4],
    step: usize,
    alpha_bits: u32,
}

/// O véu guardado entre quadros.
#[derive(Default)]
pub(crate) struct VeuDeHumidade {
    guardado: Option<(Chave, StableImage, u32, u32)>,
    construcoes: u64,
}

/// O véu dentro do `PaintState` (lido por `&PainterTool`). Clonar dá um véu VAZIO: é derivado, a
/// cópia reconstrói-o no primeiro quadro.
#[derive(Default)]
pub(crate) struct VeuGuardado(std::sync::Mutex<VeuDeHumidade>);

impl Clone for VeuGuardado {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl crate::PainterTool {
    /// A revisão do mapa de humidade: muda a cada escrita dele (pour, secagem, Wet, Dry, rebind).
    #[must_use]
    pub fn canvas_wet_revision(&self) -> u64 {
        self.paint.canvas_wet_rev
    }

    /// O véu de humidade de `rect = [x0, y0, x1, y1]` (px de imagem) a `step`, com alfa máximo
    /// `max_alpha`, guardado entre quadros (o `StableImage` só muda quando a chave muda).
    #[must_use]
    pub fn wetness_veil(
        &self,
        rect: [u32; 4],
        step: usize,
        max_alpha: f32,
    ) -> Option<(StableImage, u32, u32)> {
        let (wet, cw, _, _) = self.canvas_wet_view()?;
        let mut v = self
            .paint
            .wet_veu
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        v.veu(
            wet,
            cw as usize,
            self.paint.canvas_wet_rev,
            rect.map(|c| c as usize),
            step,
            max_alpha,
        )
        .map(|(img, w, h)| (img.clone(), w, h))
    }
}

impl VeuDeHumidade {
    /// O véu de `rect = [x0, y0, x1, y1]` (px de imagem) amostrado a `step`. Refaz só se a chave
    /// mudou. `None` = região vazia.
    pub fn veu(
        &mut self,
        wet: &[u8],
        canvas_w: usize,
        rev: u64,
        rect: [usize; 4],
        step: usize,
        max_alpha: f32,
    ) -> Option<(&StableImage, u32, u32)> {
        let chave = Chave {
            rev,
            canvas_w,
            rect,
            step,
            alpha_bits: max_alpha.to_bits(),
        };
        if self.guardado.as_ref().is_none_or(|g| g.0 != chave) {
            self.guardado = None;
            let [x0, y0, x1, y1] = rect;
            let (veil, vw, vh) = build_veil(
                wet,
                canvas_w,
                (x0, y0),
                (x1.saturating_sub(x0), y1.saturating_sub(y0)),
                max_alpha,
                step,
            );
            self.construcoes += 1;
            #[allow(clippy::cast_possible_truncation)]
            let (w, h) = (vw as u32, vh as u32);
            let img = StableImage::from_rgba(Arc::new(veil), w, h)?;
            self.guardado = Some((chave, img, w, h));
        }
        self.guardado.as_ref().map(|(_, img, w, h)| (img, *w, *h))
    }

    /// Quantas vezes o véu foi construído (gate do quadro parado).
    #[must_use]
    #[cfg(test)]
    pub fn construcoes(&self) -> u64 {
        self.construcoes
    }
}

/// O BUILD do véu: média do bloco `step×step` (um `nearest` cintila na borda quando a câmera anda),
/// desfoque em caixa de raio `max(BLUR_R/step, 1)`, tinta quase neutra. ⛔ Percorrer o vertical por
/// LINHAS (acumulador por coluna, o mesmo bit) foi MEDIDO mais lento a 3840×2160 (B16.10).
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub(crate) fn build_veil(
    wet: &[u8],
    canvas_w: usize,
    (rx0, ry0): (usize, usize),
    (rw, rh): (usize, usize),
    max_alpha: f32,
    step: usize,
) -> (Vec<u8>, usize, usize) {
    const TINT: [u8; 3] = [34, 31, 28]; // LITERAL-COLOR-OK: damp-paper darkening (near-neutral, faint warm)
    const BLUR_R: usize = 4; // LITERAL-PX-OK: gentle veil softening — wet paper has no 1-px hard edges
    let step = step.max(1);
    let (vw, vh) = (rw.div_ceil(step), rh.div_ceil(step));
    if vw == 0 || vh == 0 {
        return (Vec::new(), 0, 0);
    }
    let mut alpha = vec![0.0f32; vw * vh];
    for vy in 0..vh {
        for vx in 0..vw {
            let (y0, x0) = (vy * step, vx * step);
            let (y1, x1) = ((y0 + step).min(rh), (x0 + step).min(rw));
            let mut acc = 0.0f32;
            for y in y0..y1 {
                let src = (ry0 + y) * canvas_w + rx0;
                for x in x0..x1 {
                    acc += f32::from(wet[src + x]);
                }
            }
            let n = ((y1 - y0) * (x1 - x0)).max(1) as f32;
            alpha[vy * vw + vx] = acc / n / 255.0 * max_alpha;
        }
    }
    let alpha = box_blur_f32(&alpha, vw, vh, (BLUR_R / step).max(1));
    let mut veil = vec![0u8; vw * vh * 4];
    for (i, &a) in alpha.iter().enumerate() {
        if a > 0.002 {
            let p = i * 4;
            veil[p] = TINT[0];
            veil[p + 1] = TINT[1];
            veil[p + 2] = TINT[2];
            veil[p + 3] = (a * 255.0).clamp(0.0, 255.0) as u8;
        }
    }
    (veil, vw, vh)
}

#[allow(clippy::cast_precision_loss)]
fn box_blur_f32(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    if r == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let win = (2 * r + 1) as f32;
    let mut tmp = vec![0.0f32; w * h];
    for y in 0..h {
        let base = y * w;
        let mut acc = 0.0f32;
        for x in 0..(r + 1).min(w) {
            acc += src[base + x];
        }
        for x in 0..w {
            tmp[base + x] = acc / win;
            if x + r + 1 < w {
                acc += src[base + x + r + 1];
            }
            if x >= r {
                acc -= src[base + x - r];
            }
        }
    }
    let mut out = vec![0.0f32; w * h];
    for x in 0..w {
        let mut acc = 0.0f32;
        for y in 0..(r + 1).min(h) {
            acc += tmp[y * w + x];
        }
        for y in 0..h {
            out[y * w + x] = acc / win;
            if y + r + 1 < h {
                acc += tmp[(y + r + 1) * w + x];
            }
            if y >= r {
                acc -= tmp[(y - r) * w + x];
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "veu_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "veu_prancha_tests.rs"]
mod prancha;
