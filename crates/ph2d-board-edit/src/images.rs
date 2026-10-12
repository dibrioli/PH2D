//! W14 — **pôr IMAGENS no quadro** (soltar, colar, o botão) e levá-las numa cópia para outro quadro.
//!
//! Uma imagem nasce no tamanho NATURAL (1 píxel = 1 unidade do mundo), reduzida — nunca ampliada — até
//! caber em `fit` (quem chama dá metade da área à vista: uma fotografia grande não tapa o quadro);
//! várias ficam numa grelha quase quadrada, pela ordem em que vieram, centrada no ponto. UM passo de
//! desfazer, e ficam seleccionadas.

use ph2d_board_model::{
    BoardDoc, BoardOp, Element, ElementId, FileId, Files, History, Image, ImageFilter, Rgba, Style,
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

/// W16 — a POSE com que uma imagem nasce (a de uma sprite da cena no «Send to board…»): a escala do
/// natural em cada eixo, o ângulo (radianos, horário no ecrã, o de [`Element::angle`]) e o espelho.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub scale: [f64; 2],
    pub angle: f64,
    pub flip: [bool; 2],
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            scale: [1.0, 1.0],
            angle: 0.0,
            flip: [false; 2],
        }
    }
}

/// A caixa `[w, h]` com que uma imagem `natural` nasce em `pose`, e a caixa alinhada aos eixos que
/// ela ocupa rodada — reduzidas juntas (nunca ampliadas) até a rodada caber em `fit`.
fn posed_birth(natural: [u32; 2], pose: Pose, fit: [f64; 2]) -> ([f64; 2], [f64; 2]) {
    let [w, h] = [0, 1].map(|i| f64::from(natural[i].max(1)) * pose.scale[i].abs());
    let (c, s) = (pose.angle.cos().abs(), pose.angle.sin().abs());
    let aabb = [w * c + h * s, w * s + h * c];
    let k = (fit[0] / aabb[0]).min(fit[1] / aabb[1]).min(1.0);
    ([w * k, h * k], aabb.map(|v| v * k))
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
        let posed: Vec<_> = items
            .iter()
            .map(|&(f, n)| (f, n, Pose::default()))
            .collect();
        self.insert_posed_images(doc, history, &posed, at, fit)
    }

    /// ⭐ W16 — [`Self::insert_images`] com a POSE de cada uma: a caixa é o natural na escala dela, o
    /// ângulo e o espelho são os dela; a grelha arruma as caixas RODADAS.
    pub fn insert_posed_images(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        items: &[(FileId, [u32; 2], Pose)],
        at: [f64; 2],
        fit: [f64; 2],
    ) -> Vec<ElementId> {
        if items.is_empty() {
            return Vec::new();
        }
        self.commit_text(doc, history);
        let births: Vec<([f64; 2], [f64; 2])> = items
            .iter()
            .map(|(_, n, p)| posed_birth(*n, *p, fit))
            .collect();
        let sizes: Vec<[f64; 2]> = births.iter().map(|b| b.1).collect();
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
        let mut style = Style::new(None, None, Rgba(ph2d_board_model::DEFAULT_INK));
        // A moldura (W15) nasce no modo do quadro, como o contorno de uma forma.
        style.sketch = self.style.sketch;
        let mut made = Vec::with_capacity(items.len());
        for (i, ((file, _, pose), (s, _))) in items.iter().zip(&births).enumerate() {
            let (c, r) = (i % cols, i / cols);
            let cx = x0 + col_w[..c].iter().sum::<f64>() + gap * c as f64 + col_w[c] / 2.0;
            let cy = y0 + row_h[..r].iter().sum::<f64>() + gap * r as f64 + row_h[r] / 2.0;
            let bx = [cx - s[0] / 2.0, cy - s[1] / 2.0, s[0], s[1]];
            let mut img = Image::new(*file, style.clone());
            img.flip = pose.flip;
            let mut el = Element::new_image(doc.mint_id(), doc.z_on_top(), img, bx);
            el.angle = pose.angle;
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

    /// W15 — as imagens seleccionadas (editáveis) estão TODAS a preto e branco? `None` = nenhuma.
    #[must_use]
    pub fn images_gray(&self, doc: &BoardDoc) -> Option<bool> {
        let imgs: Vec<&Image> = self
            .editable(doc)
            .into_iter()
            .filter_map(Element::image)
            .collect();
        (!imgs.is_empty()).then(|| imgs.iter().all(|i| i.filter == ImageFilter::Gray))
    }

    /// ⭐ W15 — o botão preto e branco: as imagens seleccionadas ficam todas cinzentas, ou (se já o
    /// estavam todas) todas a cores. UM passo; só as que mudam. `false` = nenhuma imagem.
    pub fn toggle_gray(&mut self, doc: &mut BoardDoc, history: &mut History) -> bool {
        let Some(all) = self.images_gray(doc) else {
            return false;
        };
        let to = if all {
            ImageFilter::Original
        } else {
            ImageFilter::Gray
        };
        let ops = self
            .editable(doc)
            .into_iter()
            .filter(|el| el.image().is_some_and(|i| i.filter != to))
            .map(|el| {
                let mut el = el.clone();
                if let Some(i) = el.image_mut() {
                    i.filter = to;
                }
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
        true
    }

    /// ⭐ W16 — ESPELHA as imagens seleccionadas no eixo `axis` (`0` = na horizontal, Shift+H; `1` =
    /// na vertical, Shift+V), como o Excalidraw: à volta do centro da caixa de todas — cada uma vai
    /// para o lado espelhado, o ângulo troca de sinal (`M·R(θ) = R(−θ)·M`) e a imagem vira (o recorte,
    /// em píxeis naturais, fica: mostra a mesma parte, espelhada). Duas vezes = como estava. UM passo;
    /// `false` = nenhuma imagem.
    pub fn flip_images(&mut self, doc: &mut BoardDoc, history: &mut History, axis: usize) -> bool {
        let imgs: Vec<Element> = self
            .editable(doc)
            .into_iter()
            .filter(|el| el.image().is_some())
            .cloned()
            .collect();
        if imgs.is_empty() || axis > 1 {
            return false;
        }
        let (lo, hi) = imgs
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), el| {
                let b = el.aabb();
                (lo.min(b[axis]), hi.max(b[axis + 2]))
            });
        let mid = (lo + hi) / 2.0;
        let ops = imgs
            .into_iter()
            .map(|mut el| {
                let c = if axis == 0 {
                    el.x + el.w / 2.0
                } else {
                    el.y + el.h / 2.0
                };
                let moved = 2.0 * mid - c - c;
                if axis == 0 {
                    el.x += moved;
                } else {
                    el.y += moved;
                }
                el.angle = -el.angle;
                if let Some(i) = el.image_mut() {
                    i.flip[axis] = !i.flip[axis];
                }
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
        true
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
