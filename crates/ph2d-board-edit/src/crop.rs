//! W16 — **RECORTAR uma imagem pelo gesto**: a lei do Excalidraw 0.18.1, MEDIDA
//! (`docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w16/w16_sondas.json`, README §W16).
//!
//! - Entra-se com duplo clique na imagem, `Enter` com ela seleccionada, ou o botão da barra.
//! - Quatro pegas nos cantos e quatro nos meios dos lados (estas sem desenho): a pega arrasta o lado
//!   do RECORTE e a imagem fica parada no ecrã; a proporção é livre, `Shift` dá a da imagem natural.
//! - Arrastar DENTRO desliza a imagem sob a janela — ⚠️ aqui segue o dedo 1:1 (no Excalidraw anda o
//!   DOBRO do ponteiro, a qualquer tamanho: o oráculo leu-o como defeito deles e não se copia).
//! - Pára nas bordas da imagem natural; nunca menos de 10 × 10 unidades do mundo; cobrir a imagem
//!   inteira volta a «sem recorte».
//! - Sai-se com `Enter`, `Escape` ou um clique fora (que também larga a selecção); o recorte fica.
//! - Cada arrasto é UM passo de desfazer. O recorte conta-se em píxeis da imagem natural NÃO
//!   espelhada; numa imagem rodada as pegas seguem a rotação.

use ph2d_board_geom::{image_to_box, to_world};
use ph2d_board_model::{BoardDoc, BoardOp, Crop, Element, ElementId, FileId, History};
use ph2d_vector::{Affine, Point};

use crate::{Editor, Pointer};

/// O lado mínimo do recorte, em unidades do MUNDO (o 10 × 10 medido: a meio tamanho dá 20 px naturais).
const MIN_SIDE: f64 = 10.0;

/// A zona das pegas dos MEIOS dos lados, em fracção da das pegas dos cantos (medido: ±2 contra ±6 px
/// de ecrã).
const EDGE_OF_CORNER: f64 = 2.0 / 6.0;

/// O modo de recorte aberto numa imagem.
#[derive(Clone, Debug)]
pub(crate) struct Cropping {
    pub(crate) id: ElementId,
    drag: Option<Drag>,
}

#[derive(Clone, Debug)]
struct Drag {
    grip: Grip,
    original: Element,
    start: [f64; 2],
}

/// O que se agarrou: um lado ou canto (a direcção na caixa LOCAL, cada eixo `−1 · 0 · 1`) ou o meio.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Grip {
    Side([i8; 2]),
    Pan,
}

/// Píxel natural → mundo de `el` (a imagem de tamanho `natural`).
fn natural_to_world(el: &Element, natural: [u32; 2]) -> Option<Affine> {
    let img = el.image()?;
    Some(to_world(el) * image_to_box(el, img.source(natural), img.flip))
}

impl Editor {
    /// W16 — os tamanhos naturais dos ficheiros do quadro (a vista dá-os a cada imagem): o recorte
    /// conta-se em píxeis deles.
    pub fn know_files(&mut self, files: &ph2d_board_model::Files) {
        if self.natural.len() != files.len() {
            self.natural = files.iter().map(|(k, f)| (*k, f.size)).collect();
        }
    }

    fn natural_of(&self, file: FileId) -> Option<[u32; 2]> {
        self.natural.get(&file).copied()
    }

    /// ⭐ Abre o recorte na imagem seleccionada (uma só, editável, com o ficheiro conhecido).
    /// `false` = não há o que recortar.
    pub fn begin_crop(&mut self, doc: &mut BoardDoc, history: &mut History) -> bool {
        let one = match self.editable(doc).as_slice() {
            [el] => el.image().map(|i| (el.id, i.file)),
            _ => None,
        };
        let Some((id, _)) = one.filter(|(_, f)| self.natural_of(*f).is_some()) else {
            return false;
        };
        self.commit_text(doc, history);
        self.cropping = Some(Cropping { id, drag: None });
        true
    }

    /// A imagem em recorte, se o modo está aberto.
    #[must_use]
    pub fn cropping(&self) -> Option<ElementId> {
        self.cropping.as_ref().map(|c| c.id)
    }

    /// Fecha o recorte (o que se recortou fica).
    pub fn end_crop(&mut self) {
        self.cropping = None;
    }

    /// O botão do rato no modo de recorte. `false` = o modo não está aberto (o clique segue).
    pub(crate) fn crop_down(&mut self, doc: &BoardDoc, p: Pointer) -> bool {
        let Some(id) = self.cropping() else {
            return false;
        };
        let Some(el) = doc.get(id).filter(|e| e.image().is_some()) else {
            self.cropping = None;
            return false;
        };
        let local = to_world(el).inverse() * Point::new(p.world[0], p.world[1]);
        let corner = self.metrics.handle * p.px;
        let edge = corner * EDGE_OF_CORNER;
        let side = |v: f64, len: f64, tol: f64| -> Option<i8> {
            if (v - 0.0).abs() <= tol {
                Some(-1)
            } else if (v - len).abs() <= tol {
                Some(1)
            } else {
                None
            }
        };
        let inside = |v: f64, len: f64| (0.0..=len).contains(&v);
        let grip = match (side(local.x, el.w, corner), side(local.y, el.h, corner)) {
            (Some(x), Some(y)) => Some(Grip::Side([x, y])),
            _ => match (side(local.x, el.w, edge), side(local.y, el.h, edge)) {
                (Some(x), None) if inside(local.y, el.h) => Some(Grip::Side([x, 0])),
                (None, Some(y)) if inside(local.x, el.w) => Some(Grip::Side([0, y])),
                _ if inside(local.x, el.w) && inside(local.y, el.h) => Some(Grip::Pan),
                _ => None,
            },
        };
        let Some(grip) = grip else {
            // Fora: sai, e o clique larga a selecção (o do Excalidraw).
            self.cropping = None;
            self.selection.clear();
            return true;
        };
        let original = el.clone();
        if let Some(c) = &mut self.cropping {
            c.drag = Some(Drag {
                grip,
                original,
                start: p.world,
            });
        }
        true
    }

    /// O arrasto no modo de recorte. `false` = não há arrasto de recorte.
    pub(crate) fn crop_move(&mut self, doc: &mut BoardDoc, p: Pointer) -> bool {
        let Some(d) = self.cropping.as_ref().and_then(|c| c.drag.clone()) else {
            return false;
        };
        let Some(img) = d.original.image() else {
            return true;
        };
        let Some(natural) = self.natural_of(img.file) else {
            return true;
        };
        if let Some(el) = cropped(&d, natural, p) {
            let _ = BoardOp::Put(el).apply(doc);
        }
        true
    }

    /// Larga o arrasto: UM passo de desfazer. `false` = não havia arrasto de recorte.
    pub(crate) fn crop_up(&mut self, doc: &BoardDoc, history: &mut History) -> bool {
        let Some(d) = self.cropping.as_mut().and_then(|c| c.drag.take()) else {
            return false;
        };
        if doc.get(d.original.id) != Some(&d.original) {
            history.record(crate::gesture_end::undo_ops(doc, &[d.original]), doc);
        }
        true
    }

    /// Os cantos do recorte no MUNDO (as pegas), se o modo está aberto.
    pub(crate) fn crop_corners(&self, doc: &BoardDoc) -> Option<(ElementId, [[f64; 2]; 4])> {
        let el = doc.get(self.cropping()?)?;
        let t = to_world(el);
        let at = |x: f64, y: f64| {
            let q = t * Point::new(x, y);
            [q.x, q.y]
        };
        Some((
            el.id,
            [at(0.0, 0.0), at(el.w, 0.0), at(el.w, el.h), at(0.0, el.h)],
        ))
    }
}

/// O elemento depois do arrasto `d` com o ponteiro em `p` — a imagem PARADA no ecrã (o mapa natural →
/// mundo do início do arrasto), o recorte a mudar.
fn cropped(d: &Drag, natural: [u32; 2], p: Pointer) -> Option<Element> {
    let img = d.original.image()?;
    let m = natural_to_world(&d.original, natural)?;
    let inv = m.inverse();
    let n = inv * Point::new(p.world[0], p.world[1]);
    let [sx, sy, sw, sh] = img.source(natural);
    let (nw, nh) = (f64::from(natural[0]), f64::from(natural[1]));
    // Unidades do mundo por píxel natural, e o mínimo em píxeis naturais.
    let (kx, ky) = (d.original.w / sw.max(1e-9), d.original.h / sh.max(1e-9));
    let (mx, my) = (MIN_SIDE / kx, MIN_SIDE / ky);
    let [mut x0, mut y0, mut x1, mut y1] = [sx, sy, sx + sw, sy + sh];
    let mut el = d.original.clone();
    match d.grip {
        Grip::Pan => {
            let s = inv * Point::new(d.start[0], d.start[1]);
            let (dx, dy) = (
                (s.x - n.x).clamp(-x0, nw - x1),
                (s.y - n.y).clamp(-y0, nh - y1),
            );
            (x0, x1, y0, y1) = (x0 + dx, x1 + dx, y0 + dy, y1 + dy);
        }
        Grip::Side([gx, gy]) => {
            // A pega LOCAL → o lado NATURAL (o espelho troca-os).
            let ax = if img.flip[0] { -gx } else { gx };
            let ay = if img.flip[1] { -gy } else { gy };
            match ax {
                1 => x1 = n.x.clamp(x0 + mx, nw),
                -1 => x0 = n.x.clamp(0.0, x1 - mx),
                _ => {}
            }
            match ay {
                1 => y1 = n.y.clamp(y0 + my, nh),
                -1 => y0 = n.y.clamp(0.0, y1 - my),
                _ => {}
            }
            if p.mods.shift && ax != 0 && ay != 0 {
                // A proporção da imagem natural, na maior das duas caixas que cabe.
                let (w, h) = (x1 - x0, y1 - y0);
                let r = nw / nh;
                let (mut w2, mut h2) = if w / h < r { (h * r, h) } else { (w, w / r) };
                let room_x = if ax > 0 { nw - x0 } else { x1 };
                let room_y = if ay > 0 { nh - y0 } else { y1 };
                let fit = (room_x / w2).min(room_y / h2).min(1.0);
                (w2, h2) = (w2 * fit, h2 * fit);
                if ax > 0 {
                    x1 = x0 + w2
                } else {
                    x0 = x1 - w2
                }
                if ay > 0 { y1 = y0 + h2 } else { y0 = y1 - h2 }
            }
            // A caixa nova é a janela no MESMO sítio do ecrã: o centro dela pelo mapa do início.
            let c = m * Point::new((x0 + x1) / 2.0, (y0 + y1) / 2.0);
            el.w = (x1 - x0) * kx;
            el.h = (y1 - y0) * ky;
            el.x = c.x - el.w / 2.0;
            el.y = c.y - el.h / 2.0;
        }
    }
    let eps = 1e-6;
    let whole = x0 <= eps && y0 <= eps && x1 >= nw - eps && y1 >= nh - eps;
    el.image_mut()?.crop = (!whole).then_some(Crop {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    });
    Some(el)
}

#[cfg(test)]
#[path = "crop_tests.rs"]
mod tests;
