//! O RASCUNHO do tique do balde (B48.2): os buffers `w·h` que cada tique do slider alocava (`visited`, a
//! componente da seleção) e os cinco planos do corpo passam a CIRCULAR entre tiques, limpos só na caixa
//! que o tique anterior sujou — o custo do tique é o da região, não o da tela.

use super::nucleo::componente_da_selecao_em;
use crate::tool::PainterTool;
use crate::tool::paint::Region;
use crate::tool::paint::relief_state::StrokePlanes;
use ph2d_painter_brush::height::{NO_GRAIN, derive_height};

/// Scratch do balde: não é estado (um `clone` devolve-o vazio) e morre no Done/Cancel.
#[derive(Default)]
pub(in crate::tool::paint) struct Rascunho {
    visitado: Vec<bool>,
    visitado_caixa: Option<Region>,
    comp: Vec<u8>,
    comp_caixa: Option<Region>,
    /// `(n, w)` dos dois planos acima — outra forma de tela ⇒ outro plano.
    forma: (usize, usize),
    /// A bbox que o tique anterior armou nos cinco planos e o endereço de cada um: só planos que este
    /// rascunho armou (e ninguém trocou) voltam a circular.
    planos: Option<(Region, [usize; 5])>,
}

impl Clone for Rascunho {
    fn clone(&self) -> Self {
        Self::default()
    }
}

/// Escreve `v` nas linhas de `r` (largura `w`).
fn limpa<T: Copy>(buf: &mut [T], r: Region, w: usize, v: T) {
    let (x0, x1) = (r.x as usize, (r.x + r.w) as usize);
    for y in r.y as usize..(r.y + r.h) as usize {
        let (a, b) = (y * w + x0, y * w + x1);
        if b <= buf.len() {
            buf[a..b].fill(v);
        }
    }
}

impl Rascunho {
    fn forma(&mut self, n: usize, w: usize) {
        if self.forma != (n, w) || self.visitado.len() != n || self.comp.len() != n {
            self.visitado = vec![false; n];
            self.comp = vec![0u8; n];
            (self.visitado_caixa, self.comp_caixa) = (None, None);
            self.forma = (n, w);
        }
    }

    /// O `visited` do flood, todo `false` (limpo só na caixa do flood anterior). Registe a caixa nova
    /// com [`Self::visitou`].
    pub(super) fn visitado(&mut self, n: usize, w: usize) -> &mut [bool] {
        self.forma(n, w);
        if let Some(r) = self.visitado_caixa.take() {
            limpa(&mut self.visitado, r, w, false);
        }
        &mut self.visitado
    }

    pub(super) fn visitou(&mut self, caixa: Option<Region>) {
        self.visitado_caixa = caixa;
    }

    /// O conjunto do último flood (válido até ao próximo [`Self::visitado`]).
    pub(super) fn foi_visitado(&self, i: usize) -> bool {
        self.visitado[i]
    }

    /// O conjunto do último flood inteiro (o `visited`).
    pub(super) fn visitados(&self) -> &[bool] {
        &self.visitado
    }

    /// A última [`Self::componente`].
    pub(super) fn comp(&self) -> &[u8] {
        &self.comp
    }

    /// A componente da seleção sob `(sx, sy)` neste buffer (limpo só na caixa da anterior).
    pub(super) fn componente(
        &mut self,
        mask: &[u8],
        sx: usize,
        sy: usize,
        w: usize,
        h: usize,
    ) -> &[u8] {
        self.forma(w * h, w);
        if let Some(r) = self.comp_caixa.take() {
            limpa(&mut self.comp, r, w, 0);
        }
        self.comp_caixa = componente_da_selecao_em(mask, sx, sy, w, h, &mut self.comp);
        &self.comp
    }
}

impl PainterTool {
    /// Os cinco planos que o tique anterior armou, de volta aos valores de fábrica do balde só na bbox
    /// dele — ou `None` (o chamador aloca), se não são os que este rascunho armou.
    pub(super) fn toma_planos_do_balde(&mut self) -> Option<StrokePlanes> {
        let (bb, ptrs) = self.paint.fill_rascunho.planos.take()?;
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let n = w * h;
        let r = &self.paint.relief;
        let agora = [
            r.stroke_height.as_ptr() as usize,
            r.stroke_paint.as_ptr() as usize,
            r.stroke_grain.as_ptr() as usize,
            r.stroke_film.as_ptr() as usize,
            r.stroke_radius.as_ptr() as usize,
        ];
        let lens = [
            r.stroke_height.len(),
            r.stroke_paint.len(),
            r.stroke_grain.len(),
            r.stroke_film.len(),
            r.stroke_radius.len(),
        ];
        if r.stroke_relief_bbox != Some(bb) || agora != ptrs || lens.iter().any(|&l| l != n) {
            return None;
        }
        let h0 = derive_height(&self.paint.brush, 0.0, f32::from(NO_GRAIN) / 255.0);
        let r = &mut self.paint.relief;
        let mut p = StrokePlanes {
            height: std::mem::take(&mut r.stroke_height),
            paint: std::mem::take(&mut r.stroke_paint),
            grain: std::mem::take(&mut r.stroke_grain),
            film: std::mem::take(&mut r.stroke_film),
            radius: std::mem::take(&mut r.stroke_radius),
        };
        limpa(&mut p.height, bb, w, h0);
        limpa(&mut p.paint, bb, w, 0.0);
        limpa(&mut p.grain, bb, w, NO_GRAIN);
        limpa(&mut p.film, bb, w, 0);
        limpa(&mut p.radius, bb, w, 0.0);
        Some(p)
    }

    /// Regista os cinco planos acabados de armar como deste rascunho.
    pub(super) fn guarda_planos_do_balde(&mut self) {
        let r = &self.paint.relief;
        self.paint.fill_rascunho.planos = r.stroke_relief_bbox.map(|bb| {
            (
                bb,
                [
                    r.stroke_height.as_ptr() as usize,
                    r.stroke_paint.as_ptr() as usize,
                    r.stroke_grain.as_ptr() as usize,
                    r.stroke_film.as_ptr() as usize,
                    r.stroke_radius.as_ptr() as usize,
                ],
            )
        });
    }
}
