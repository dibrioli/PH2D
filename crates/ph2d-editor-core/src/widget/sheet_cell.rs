//! **Pintar UMA célula de uma folha numa caixa** — a miniatura do cartão «Animation», a pré-visualização
//! e os quadros do editor de animações (plano GameEngineFuncional 07). A célula encaixa-se na caixa sem
//! deformar, centrada. Cada célula pinta-se de uma imagem SÓ dela: recortar a folha inteira deixava o filtro ler
//! a vizinha, e a arte que encosta na borda dela aparecia em risco (`docs/Sprite_projeto/22`).

use crate::zones::Rect;
use ph2d_vector::{Affine, ImageQuality, VectorScene};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// Os pixels da folha da sprite (RGBA reto), para miniaturas e para a grelha do editor.
///
/// ⚠️ Leva o [`ph2d_vector::StableImage`] já feito: o Vello guarda a imagem pelo id do blob, e um
/// `draw_image_rgba` por quadro voltaria a subi-la à placa em cada pintura.
#[derive(Clone)]
pub struct SheetImage {
    pub rgba: Arc<Vec<u8>>,
    pub w: u32,
    pub h: u32,
    pub image: ph2d_vector::StableImage,
    celulas: Arc<Mutex<CelulasRecortadas>>,
}

/// As células já recortadas, por `(hframes, vframes, célula)`. Somam no máximo o tamanho da folha por grelha.
type CelulasRecortadas = BTreeMap<(u32, u32, u32), ph2d_vector::StableImage>;

impl SheetImage {
    /// `None` se `rgba` não tem `w × h × 4` bytes.
    #[must_use]
    pub fn new(rgba: Arc<Vec<u8>>, w: u32, h: u32) -> Option<Self> {
        let image = ph2d_vector::StableImage::from_rgba(rgba.clone(), w, h)?;
        Some(Self {
            rgba,
            w,
            h,
            image,
            celulas: Arc::default(),
        })
    }

    /// Os limites `[x0, y0, x1, y1]` da célula em pixels inteiros (a grelha que não divide arredonda sem buraco).
    #[must_use]
    pub fn limites_da_celula(&self, hframes: u32, vframes: u32, cell: u32) -> [u32; 4] {
        let (hf, vf) = (hframes.max(1), vframes.max(1));
        let corte = |i: u32, n: u32, lado: u32| {
            ((f64::from(i) * f64::from(lado) / f64::from(n)).round() as u32).min(lado)
        };
        let (col, lin) = (cell % hf, cell / hf);
        [
            corte(col, hf, self.w),
            corte(lin, vf, self.h),
            corte(col + 1, hf, self.w),
            corte(lin + 1, vf, self.h),
        ]
    }

    /// Os pixels só da célula, linha a linha.
    #[must_use]
    pub fn pixels_da_celula(&self, hframes: u32, vframes: u32, cell: u32) -> Vec<u8> {
        let [x0, y0, x1, y1] = self.limites_da_celula(hframes, vframes, cell);
        let linha = self.w as usize * 4;
        let (a, b) = (x0 as usize * 4, x1 as usize * 4);
        let mut out = Vec::with_capacity((b - a) * (y1 - y0) as usize);
        for y in y0 as usize..y1 as usize {
            out.extend_from_slice(&self.rgba[y * linha + a..y * linha + b]);
        }
        out
    }

    fn imagem_da_celula(
        &self,
        hframes: u32,
        vframes: u32,
        cell: u32,
    ) -> Option<ph2d_vector::StableImage> {
        let chave = (hframes, vframes, cell);
        let mut celulas = self.celulas.lock().ok()?;
        if let Some(i) = celulas.get(&chave) {
            return Some(i.clone());
        }
        let [x0, y0, x1, y1] = self.limites_da_celula(hframes, vframes, cell);
        let px = Arc::new(self.pixels_da_celula(hframes, vframes, cell));
        let i = ph2d_vector::StableImage::from_rgba(px, x1 - x0, y1 - y0)?;
        celulas.insert(chave, i.clone());
        Some(i)
    }
}

impl PartialEq for SheetImage {
    /// A MESMA folha (o mesmo blob), não pixels iguais: comparar 4,7 MB por quadro seria o custo todo.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.rgba, &other.rgba) && self.w == other.w && self.h == other.h
    }
}

impl std::fmt::Debug for SheetImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SheetImage")
            .field("w", &self.w)
            .field("h", &self.h)
            .finish_non_exhaustive()
    }
}

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
    paint_sheet_cell_with(
        scene,
        sheet,
        (hframes, vframes, cell),
        dest,
        crate::ui_image_quality(),
    );
}

/// [`paint_sheet_cell`] com o filtro dado (`Low` = o píxel nítido de uma ampliação de pixel art).
pub fn paint_sheet_cell_with(
    scene: &mut VectorScene,
    sheet: &SheetImage,
    (hframes, vframes, cell): (u32, u32, u32),
    dest: Rect,
    quality: ImageQuality,
) {
    let (hf, vf) = (hframes.max(1), vframes.max(1));
    if cell >= hf * vf {
        return;
    }
    let (fit, s) = sheet_cell_fit(sheet, hf, vf, dest);
    if s <= 0.0 {
        return;
    }
    let Some(imagem) = sheet.imagem_da_celula(hf, vf, cell) else {
        return;
    };
    let [x0, y0, x1, y1] = sheet.limites_da_celula(hf, vf, cell);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let clip = ph2d_vector::Rect::new(
        f64::from(fit.x),
        f64::from(fit.y),
        f64::from(fit.x + fit.w),
        f64::from(fit.y + fit.h),
    );
    scene.push_clip(&clip);
    let t = Affine::translate((f64::from(fit.x), f64::from(fit.y)))
        * Affine::scale_non_uniform(
            f64::from(fit.w) / f64::from(x1 - x0),
            f64::from(fit.h) / f64::from(y1 - y0),
        );
    scene.draw_stable_image_transformed(&imagem, t, quality);
    scene.pop_layer();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A imagem de uma célula só tem os pixels dela: é o que impede o filtro de ler a vizinha.
    #[test]
    fn a_celula_recortada_so_tem_os_pixels_dela() {
        let (w, h, hf, vf) = (10_u32, 6_u32, 3_u32, 2_u32);
        // Cada pixel diz a célula dele (a grelha 10/3 não divide: 3, 4 e 3 px).
        let folha = SheetImage::new(Arc::new(vec![0; (w * h * 4) as usize]), w, h).unwrap();
        let mut todas = 0;
        let mut px = vec![0_u8; (w * h * 4) as usize];
        for cell in 0..hf * vf {
            let [x0, y0, x1, y1] = folha.limites_da_celula(hf, vf, cell);
            todas += (x1 - x0) * (y1 - y0);
            for y in y0..y1 {
                for x in x0..x1 {
                    px[((y * w + x) * 4) as usize] = cell as u8 + 1;
                }
            }
        }
        assert_eq!(
            todas,
            w * h,
            "as células têm de cobrir a folha, sem buraco nem sobra"
        );
        let folha = SheetImage::new(Arc::new(px), w, h).unwrap();
        for cell in 0..hf * vf {
            let [x0, y0, x1, y1] = folha.limites_da_celula(hf, vf, cell);
            let so = folha.pixels_da_celula(hf, vf, cell);
            assert_eq!(so.len(), ((x1 - x0) * (y1 - y0) * 4) as usize);
            assert!(
                so.chunks(4).all(|p| p[0] == cell as u8 + 1),
                "a célula {cell} leu outra"
            );
            assert!(folha.imagem_da_celula(hf, vf, cell).is_some());
        }
    }

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
