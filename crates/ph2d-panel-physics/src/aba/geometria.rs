//! A geometria da aba das camadas.

use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

/// A geometria da aba numa `area`: a lista à esquerda, a grade em cima à direita, a camada aberta
/// embaixo. `k` = quantas camadas visíveis.
#[derive(Clone, Copy, Debug)]
pub struct Geometria {
    pub lista: Rect,
    pub grade: Rect,
    pub camada: Rect,
    /// O canto de cima à esquerda dos rótulos da grade.
    pub x: f32,
    pub y: f32,
    pub rotulo_w: f32,
    pub passo: f32,
    pub k: usize,
}

impl Geometria {
    #[must_use]
    pub fn de(area: Rect, rotulo_w: f32, lista_w: f32, k: usize) -> Self {
        let pad = Spacing::Md.px();
        let row = ROW_H_PX;
        let lista = Rect::new(
            area.x + pad,
            area.y + pad,
            lista_w,
            (area.h - pad * 2.0).max(0.0),
        );
        let dx = lista.x + lista.w + pad;
        let dw = (area.x + area.w - pad - dx).max(0.0);
        let camada_h = row * 4.0; // LITERAL-PX-OK: título + duas filas da 9b + folga, em linhas
        let grade_h = (lista.h - camada_h - pad).max(0.0);
        let grade = Rect::new(dx, lista.y, dw, grade_h);
        let camada = Rect::new(dx, grade.y + grade.h + pad, dw, camada_h);
        let n = k.max(1) as f32;
        // Título (uma linha), rótulos de cima, as células e o pé (uma linha).
        let livre_h = grade.h - (pad * 2.0 + Spacing::Lg.px()) - row * 2.0 - rotulo_w;
        let livre_w = grade.w - pad * 2.0 - rotulo_w;
        let passo = (livre_h / n)
            .min(livre_w / n)
            .min(row + Spacing::Sm.px())
            .max(Spacing::Md.px());
        Self {
            lista,
            grade,
            camada,
            x: grade.x + pad,
            y: grade.y + pad + row,
            rotulo_w,
            passo,
            k,
        }
    }

    /// A célula da linha `l` × coluna `c` (posições na lista das visíveis).
    #[must_use]
    pub fn celula(&self, l: usize, c: usize) -> Rect {
        let vao = Spacing::Xs.px();
        Rect::new(
            self.x + self.rotulo_w + c as f32 * self.passo,
            self.y + self.rotulo_w + l as f32 * self.passo,
            self.passo - vao,
            self.passo - vao,
        )
    }

    #[must_use]
    pub fn rotulo_da_linha(&self, l: usize) -> Rect {
        Rect::new(
            self.x,
            self.y + self.rotulo_w + l as f32 * self.passo,
            self.rotulo_w,
            self.passo,
        )
    }

    #[must_use]
    pub fn rotulo_da_coluna(&self, c: usize) -> Rect {
        Rect::new(
            self.x + self.rotulo_w + c as f32 * self.passo,
            self.y,
            self.passo,
            self.rotulo_w,
        )
    }

    /// A frase do pé, logo abaixo da última linha da grade.
    #[must_use]
    pub fn pe(&self) -> Rect {
        let pad = Spacing::Md.px();
        Rect::new(
            self.grade.x + pad,
            self.y + self.rotulo_w + self.k as f32 * self.passo + pad,
            self.grade.w - pad * 2.0,
            ROW_H_PX,
        )
    }

    /// A linha `p` da lista (debaixo do título «Layers»).
    #[must_use]
    pub fn linha_da_lista(&self, p: usize) -> Rect {
        let pad = Spacing::Sm.px();
        Rect::new(
            self.lista.x + pad,
            self.lista.y + pad + ROW_H_PX * (p + 1) as f32,
            self.lista.w - pad * 2.0,
            ROW_H_PX,
        )
    }

    /// O «+ Add», debaixo da última linha da lista.
    #[must_use]
    pub fn add(&self, w: f32) -> Rect {
        let r = self.linha_da_lista(self.k);
        Rect::new(r.x, r.y + Spacing::Sm.px(), w, ROW_H_PX)
    }
}
