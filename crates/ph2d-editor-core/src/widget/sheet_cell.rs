//! **Pintar UMA célula de uma folha numa caixa** — a miniatura do cartão «Animation», a pré-visualização
//! e os quadros do editor de animações (plano GameEngineFuncional 07). A célula encaixa-se na caixa sem
//! deformar, centrada; o resto da folha fica de fora pelo recorte.

use crate::SheetImage;
use crate::zones::Rect;
use ph2d_vector::{Affine, ImageQuality, VectorScene};

/// A caixa onde a célula `cell` de uma grelha `hframes × vframes` cai dentro de `dest`, e a escala.
#[must_use]
pub fn sheet_cell_fit(sheet: &SheetImage, hframes: u32, vframes: u32, dest: Rect) -> (Rect, f64) {
    let cw = f64::from(sheet.w) / f64::from(hframes.max(1));
    let ch = f64::from(sheet.h) / f64::from(vframes.max(1));
    if cw <= 0.0 || ch <= 0.0 {
        return (Rect::new(dest.x, dest.y, 0.0, 0.0), 0.0);
    }
    let s = (f64::from(dest.w) / cw).min(f64::from(dest.h) / ch);
    let (w, h) = ((cw * s) as f32, (ch * s) as f32);
    (
        Rect::new(
            dest.x + (dest.w - w) / 2.0,
            dest.y + (dest.h - h) / 2.0,
            w,
            h,
        ),
        s,
    )
}

/// Pinta a célula `cell` (índice por linhas) dentro de `dest`.
pub fn paint_sheet_cell(
    scene: &mut VectorScene,
    sheet: &SheetImage,
    hframes: u32,
    vframes: u32,
    cell: u32,
    dest: Rect,
) {
    let (hf, vf) = (hframes.max(1), vframes.max(1));
    if cell >= hf * vf {
        return;
    }
    let (fit, s) = sheet_cell_fit(sheet, hf, vf, dest);
    if s <= 0.0 {
        return;
    }
    let (col, row) = (f64::from(cell % hf), f64::from(cell / hf));
    let cw = f64::from(sheet.w) / f64::from(hf);
    let ch = f64::from(sheet.h) / f64::from(vf);
    let clip = ph2d_vector::Rect::new(
        f64::from(fit.x),
        f64::from(fit.y),
        f64::from(fit.x + fit.w),
        f64::from(fit.y + fit.h),
    );
    scene.push_clip(&clip);
    let t = Affine::translate((
        f64::from(fit.x) - col * cw * s,
        f64::from(fit.y) - row * ch * s,
    )) * Affine::scale(s);
    scene.draw_stable_image_transformed(&sheet.image, t, ImageQuality::Medium);
    scene.pop_layer();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn a_celula_encaixa_sem_deformar_e_centrada() {
        let sheet = SheetImage::new(Arc::new(vec![0; 1536 * 768 * 4]), 1536, 768).unwrap();
        let (r, s) = sheet_cell_fit(&sheet, 8, 4, Rect::new(10.0, 20.0, 96.0, 48.0));
        assert!(
            (s - 0.25).abs() < 1e-9,
            "192 px numa caixa de 48 de altura: {s}"
        );
        assert_eq!((r.x, r.y, r.w, r.h), (34.0, 20.0, 48.0, 48.0));
    }
}
