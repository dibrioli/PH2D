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
    /// A largura da coluna da direita (a grade e a camada aberta), antes de saber as alturas.
    #[must_use]
    pub fn largura_direita(area: Rect, lista_w: f32) -> f32 {
        let pad = Spacing::Md.px();
        (area.w - (pad + pad + pad) - lista_w).max(0.0)
    }

    /// ⭐ Os dois cartões da direita têm o tamanho do conteúdo (maquete B): a grade com quadrados de uma
    /// linha e um vão, e a camada aberta com `camada_h`. Sem altura, os quadrados encolhem até caber.
    #[must_use]
    pub fn de(area: Rect, rotulo_w: f32, lista_w: f32, k: usize, camada_h: f32) -> Self {
        let pad = Spacing::Md.px();
        let row = ROW_H_PX;
        let dx = area.x + pad * 2.0 + lista_w;
        let dw = Self::largura_direita(area, lista_w);
        let n = k.max(1) as f32;
        // Título (uma linha), rótulos de cima, as células e o pé (uma linha), com os vãos.
        let fixo_h = pad + pad + pad + row * 2.0 + rotulo_w;
        let livre_h = area.h - (pad + pad + pad) - camada_h - fixo_h;
        let livre_w = dw - pad * 2.0 - rotulo_w;
        let passo = (livre_h / n)
            .min(livre_w / n)
            .min(row + Spacing::Sm.px())
            .max(Spacing::Md.px());
        let grade = Rect::new(dx, area.y + pad, dw, fixo_h + passo * n);
        let camada = Rect::new(dx, grade.y + grade.h + pad, dw, camada_h);
        let lista_h = Spacing::Md.px() + Spacing::Sm.px() + row * (k + 2) as f32;
        let lista = Rect::new(
            area.x + pad,
            area.y + pad,
            lista_w,
            (grade.h + pad + camada_h).max(lista_h),
        );
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

/// Uma fila que quebra: um retângulo de uma linha por largura, da esquerda para a direita a partir de
/// `(x, y)` dentro de `w`. Devolve os retângulos e quantas linhas ocupou.
#[must_use]
pub fn fila(x: f32, y: f32, w: f32, larguras: &[f32]) -> (Vec<Rect>, usize) {
    let (vao, vao_v) = (Spacing::Sm.px(), Spacing::Xs.px());
    let mut v = Vec::with_capacity(larguras.len());
    let (mut cx, mut linha) = (x, 0usize);
    for &lw in larguras {
        if cx > x && cx + lw > x + w {
            cx = x;
            linha += 1;
        }
        v.push(Rect::new(
            cx,
            y + linha as f32 * (ROW_H_PX + vao_v),
            lw,
            ROW_H_PX,
        ));
        cx += lw + vao;
    }
    (v, if larguras.is_empty() { 0 } else { linha + 1 })
}
