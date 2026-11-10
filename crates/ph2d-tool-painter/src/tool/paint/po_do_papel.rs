//! **O pó pousado** do traço ([`ph2d_painter_brush::texture::PoDoPapel`], dono 09/11): no 1.º lote de um traço cujo
//! grão enche o dente ([`ph2d_painter_brush::GrainLaw::enche_o_dente`]), o alfa da camada no pen-down — a tela que o
//! `before` do desfazer já segura — desfocado à escala de um vale ([`RAIO_DA_CAIXA`]). Imutável até o pen-up: a borracha, o desfazer e mover a camada
//! ficam coerentes sem plano novo, e o que o próprio traço pousa só conta no traço seguinte.

use crate::tool::PainterTool;
use ph2d_painter_brush::BrushSpec;
use ph2d_painter_brush::texture::{PoDoPapel, RAIO_DA_CAIXA, caixa};
use rayon::prelude::*;

impl PainterTool {
    /// Prepara o pó do traço para o pincel do lote, uma vez por traço; nada se o pincel não enche o dente.
    pub(super) fn prepara_o_po(&mut self, brush: &BrushSpec, w: u32, h: u32) {
        if self.paint.po_do_papel.is_some()
            || !brush.texture.is_active()
            || !brush.texture.lei.enche_o_dente()
        {
            return;
        }
        let (w, h) = (w as usize, h as usize);
        let base = self
            .tela_de_antes_do_traco(true)
            .filter(|b| w > 0 && b.len() == w * h * 4);
        let plano = base.map_or_else(Vec::new, |b| po_da_tela(&b, w, h));
        self.paint.po_do_papel = Some(plano);
    }
}

/// O pó do traço (o campo `po_do_papel`) numa tela de largura `w` (`None` = sem pó: o papel de sempre). Função do
/// CAMPO, não do tool: as rotas seguram-no ao lado dos empréstimos mutáveis da tela.
pub(super) fn po(plano: Option<&Vec<u8>>, w: u32) -> Option<PoDoPapel<'_>> {
    plano.filter(|p| !p.is_empty()).map(|plano| PoDoPapel {
        plano,
        largura: w as usize,
    })
}

/// O pó da tela `rgba` (`w`×`h`): o alfa desfocado por três caixas em cada eixo; vazio se a camada não tem tinta.
fn po_da_tela(rgba: &[u8], w: usize, h: usize) -> Vec<u8> {
    if !rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 0) {
        return Vec::new();
    }
    let tres = |v: &mut Vec<f32>, n: usize| {
        v.par_chunks_mut(n).for_each(|linha| {
            let mut tmp = vec![0.0f32; n];
            for _ in 0..3 {
                caixa(linha, &mut tmp, RAIO_DA_CAIXA);
                linha.copy_from_slice(&tmp);
            }
        });
    };
    let mut a: Vec<f32> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| f32::from(p[3]) / 255.0)
        .collect();
    tres(&mut a, w);
    let mut t = vec![0.0f32; w * h];
    t.par_chunks_mut(h).enumerate().for_each(|(x, col)| {
        for (y, c) in col.iter_mut().enumerate() {
            *c = a[y * w + x];
        }
    });
    tres(&mut t, h);
    let mut out = vec![0u8; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(y, linha)| {
        for (x, o) in linha.iter_mut().enumerate() {
            *o = (t[x * h + y].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    });
    out
}

#[cfg(test)]
mod tests {
    /// O custo do pó no pen-down a 4K (3840×2160, metade com tinta): `PH2D_RELEASE` não — `--profile smoke`.
    #[test]
    #[ignore = "medição: cargo test --profile smoke -- --ignored mede_o_po_a_4k --nocapture"]
    fn mede_o_po_a_4k() {
        let (w, h) = (3840usize, 2160usize);
        let rgba: Vec<u8> = (0..w * h)
            .flat_map(|i| [0, 0, 0, if (i / 7) % 2 == 0 { 255 } else { 0 }])
            .collect();
        let mut t = Vec::new();
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let p = super::po_da_tela(&rgba, w, h);
            t.push(t0.elapsed().as_secs_f64() * 1e3);
            assert_eq!(p.len(), w * h);
        }
        t.sort_by(f64::total_cmp);
        eprintln!("[po 4K] mínimo {:.1} ms · mediana {:.1} ms", t[0], t[3]);
    }
}
