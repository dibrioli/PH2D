//! W14 — **pôr IMAGENS no quadro** (soltar, colar, o botão) e levá-las numa cópia para outro quadro.
//!
//! Uma imagem nasce no tamanho NATURAL (1 píxel = 1 unidade do mundo), reduzida — nunca ampliada — até
//! caber em `fit` (quem chama dá metade da área à vista: uma fotografia grande não tapa o quadro);
//! várias ficam numa grelha quase quadrada, pela ordem em que vieram, centrada no ponto. UM passo de
//! desfazer, e ficam seleccionadas.

use ph2d_board_model::{
    BoardDoc, BoardOp, Element, ElementId, FileId, Files, History, Image, Rgba, Style,
};

use crate::Editor;

/// O intervalo entre imagens da mesma leva, em fracção do maior lado delas — o da grelha de imagens
/// largadas na cena (`IMPORT_GRID_GAP_FRAC`, `ph2d-image-import`).
const GAP_FRAC: f64 = 0.08;

/// O tamanho com que uma imagem `natural` nasce: o natural, reduzido até caber em `fit`.
#[must_use]
pub fn birth_size(natural: [u32; 2], fit: [f64; 2]) -> [f64; 2] {
    let [w, h] = natural.map(|v| f64::from(v.max(1)));
    let k = (fit[0] / w).min(fit[1] / h).min(1.0);
    [w * k, h * k]
}

impl Editor {
    /// ⭐ As imagens de `items` (a chave e o tamanho natural de cada ficheiro, já no quadro) numa
    /// grelha centrada em `at`, cada uma a caber em `fit` — UM passo, seleccionadas.
    pub fn insert_images(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        items: &[(FileId, [u32; 2])],
        at: [f64; 2],
        fit: [f64; 2],
    ) -> Vec<ElementId> {
        if items.is_empty() {
            return Vec::new();
        }
        self.commit_text(doc, history);
        let sizes: Vec<[f64; 2]> = items.iter().map(|(_, n)| birth_size(*n, fit)).collect();
        let cols = (items.len() as f64).sqrt().ceil() as usize;
        let rows = items.len().div_ceil(cols);
        let big = sizes.iter().map(|s| s[0].max(s[1])).fold(0.0, f64::max);
        let gap = big * GAP_FRAC;
        // A largura de cada coluna e a altura de cada linha da grelha.
        let mut col_w = vec![0.0f64; cols];
        let mut row_h = vec![0.0f64; rows];
        for (i, s) in sizes.iter().enumerate() {
            col_w[i % cols] = col_w[i % cols].max(s[0]);
            row_h[i / cols] = row_h[i / cols].max(s[1]);
        }
        let total = |v: &[f64]| v.iter().sum::<f64>() + gap * (v.len() as f64 - 1.0);
        let (x0, y0) = (at[0] - total(&col_w) / 2.0, at[1] - total(&row_h) / 2.0);
        let style = Style::new(None, None, Rgba(ph2d_board_model::DEFAULT_INK));
        let mut made = Vec::with_capacity(items.len());
        for (i, ((file, _), s)) in items.iter().zip(&sizes).enumerate() {
            let (c, r) = (i % cols, i / cols);
            let cx = x0 + col_w[..c].iter().sum::<f64>() + gap * c as f64 + col_w[c] / 2.0;
            let cy = y0 + row_h[..r].iter().sum::<f64>() + gap * r as f64 + row_h[r] / 2.0;
            let bx = [cx - s[0] / 2.0, cy - s[1] / 2.0, s[0], s[1]];
            let el = Element::new_image(
                doc.mint_id(),
                doc.z_on_top(),
                Image::new(*file, style.clone()),
                bx,
            );
            made.push(el.id);
            let _ = BoardOp::Put(el).apply(doc);
        }
        history.record(
            made.iter().rev().map(|id| BoardOp::Delete(*id)).collect(),
            doc,
        );
        self.selection = made.iter().copied().collect();
        made
    }

    /// Depois de copiar: guarda os bytes das imagens da cópia (de `files`, os do quadro de onde se
    /// copia) — colar noutro quadro leva-os.
    pub fn keep_clip_files(&mut self, files: &Files) {
        self.clip_files = self
            .clipboard
            .iter()
            .filter_map(Element::image)
            .filter_map(|i| Some((i.file, files.get(&i.file)?.clone())))
            .collect();
    }

    /// Antes de colar: os bytes das imagens da cópia entram no quadro de destino (`files`).
    pub fn give_clip_files(&self, files: &mut Files) {
        for (k, f) in &self.clip_files {
            files.entry(*k).or_insert_with(|| f.clone());
        }
    }
}

#[cfg(test)]
#[path = "images_tests.rs"]
mod tests;
