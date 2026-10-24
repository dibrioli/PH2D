//! ⭐ **O DUAL nas FERRAMENTAS** (L6, `docs/Painter/Pinceis/03_MOTORES.md` §4): a borracha, o Blur, o Smear e o
//! Clone do Digital respeitam a ponta INTEIRA (o Dual Brush do Photoshop e o Masked Brush do Krita valem em toda
//! ferramenta de pincel).
//!
//! A lei é a do plano do traço ([`super::traco_plano`]) com a ferramenta no lugar da tinta: a ferramenta corre como
//! sempre numa tela LIVRE do traço, a 2.ª ponta pinta a máscara `m`, e cada texel da tela é
//! `base + (livre − base) · modo(a, m) / a` — o efeito da ferramenta com a cobertura `a` da 1.ª recortada. A base é
//! a tela no 1.º lote do traço (nunca a do lote anterior: o efeito não se compõe consigo mesmo, e o Smear, que
//! re-resolve tudo o que deslocou a cada lote, não acumula o recorte). Em Multiply `modo(a, m)/a = m`; nos outros
//! modos `a` é a cobertura dos MESMOS dabs, com o mesmo sorteio, carimbada em `Mix` num plano transparente.
//!
//! ⚠️ A pergunta é UMA ([`PainterTool::ferramenta_com_dual`]): o motor arma por ela e a linha do painel deixa de
//! esmaecer por ela (`dual_recusado`, `inercia`). Sem o dual nada disto corre (o carimbo de sempre, ao bit).

use std::mem::swap;
use std::sync::Arc;

use ph2d_painter_brush::{BrushBlend, Dab, DualMode};
use rayon::prelude::*;

use super::traco_plano::{PlanoDoTraco, une};
use super::{PaintMode, Region};
use crate::tool::PainterTool;

/// A janela (texels) a partir da qual a projecção vai ao pool, em blocos de [`LINHAS_POR_TAREFA`] (a régua do
/// plano do traço, `traco_plano::COMPOE_EM_PARALELO_DESDE`).
const PROJECTA_EM_PARALELO_DESDE: usize = 16_384;
const LINHAS_POR_TAREFA: usize = 16;

/// O que o traço de uma ferramenta com o dual guarda além da base, da tela livre e da máscara.
#[derive(Clone, Debug, Default)]
pub(crate) struct Ferramenta {
    /// A cobertura `a` da 1.ª ponta no traço (o alfa) — só fora do Multiply.
    cobertura: Option<Arc<Vec<u8>>>,
    /// O tecto do Accumulate e o trilho de píxel DA cobertura: os da ferramenta não andam duas vezes por lote.
    stroke_mask: Vec<u8>,
    trilhos: super::pixel_route::Trilhos,
}

impl PainterTool {
    /// ⭐ **A porta**: a ferramenta em mão lê a 2.ª ponta? — a borracha, o Blur, o Smear e o Clone do Digital, num
    /// traço livre (o método incremental, sem o Solid, os fios nem o Composite).
    pub(crate) fn ferramenta_com_dual(&self) -> bool {
        if self.paint_media() != crate::PaintMedia::Digital {
            return false;
        }
        let b = &self.paint.brush;
        let ferramenta = match self.paint.paint_mode {
            PaintMode::Paint => self.stroke_erases(),
            PaintMode::Blur | PaintMode::Smear | PaintMode::Clone => true,
            _ => false,
        };
        ferramenta
            && b.stroke_method.is_incremental()
            && !b.style_solid
            && !b.sews_threads()
            && !self.composite_active()
    }

    /// O lote de agora é de uma ferramenta com a 2.ª ponta armada.
    pub(super) fn dual_na_ferramenta(&self) -> bool {
        self.paint.brush.dual.is_some() && self.ferramenta_com_dual()
    }

    /// Um lote da ferramenta com o dual, dentro do gate de proteção/seleção alargado pelas marcas da 2.ª ponta.
    pub(super) fn lote_da_ferramenta(&mut self, dabs: &[Dab], mask_gate: bool, sel_gate: bool) {
        // Marcas da 2.ª ponta sem lote da 1.ª esperam pelo lote seguinte (ou pelo fim do traço), como no plano.
        if dabs.is_empty() && !self.dual_no_fim() {
            return;
        }
        if (mask_gate || sel_gate)
            && let Some(r) = self.regiao_do_lote_no_plano(dabs)
        {
            self.gate_scoped(r, mask_gate, sel_gate, |t| t.ferramenta_com_a_segunda(dabs));
        } else {
            self.ferramenta_com_a_segunda(dabs);
        }
    }

    /// ⭐ A ferramenta na tela livre, a cobertura da 1.ª (fora do Multiply), a 2.ª ponta na máscara, e a projecção
    /// do que mudou.
    fn ferramenta_com_a_segunda(&mut self, dabs: &[Dab]) {
        let Some(d) = self.paint.brush.dual else {
            return;
        };
        let n4 = self.canvas_rgba.len();
        let mut s = match self.paint.plano_do_traco.take() {
            Some(s) if s.ferramenta.is_some() && s.base.len() == n4 && s.plano.len() == n4 => s,
            _ => PlanoDoTraco::da_ferramenta(&self.canvas_rgba),
        };
        let r_lote = self.dab_batch_region(dabs);
        let rng = self.paint.tex_rng;
        // O que a ferramenta escreve (o Smear re-resolve tudo o que já deslocou, além da pegada do lote).
        let sujo = self.dirty_rect.take();
        let ws = &self.undo.write_state;
        super::plane_fork::swap_canvas_plane(&mut self.canvas_rgba, &mut s.plano, ws);
        self.stamp_dabs_routed(dabs);
        let ws = &self.undo.write_state;
        super::plane_fork::swap_canvas_plane(&mut self.canvas_rgba, &mut s.plano, ws);
        let escrito = self.dirty_rect;
        self.dirty_rect = une(sujo, escrito);
        if d.modo != DualMode::Multiply && !dabs.is_empty() {
            let depois = self.paint.tex_rng;
            self.paint.tex_rng = rng;
            self.carimba_a_cobertura(&mut s, dabs);
            self.paint.tex_rng = depois;
        }
        let r_dual = self.carimba_a_segunda(&mut s.mascara);
        if let Some(r) = une(une(escrito, r_lote), r_dual) {
            self.projecta_a_ferramenta(&s, d.modo, r);
        }
        self.paint.plano_do_traco = Some(s);
    }

    /// A cobertura `a` dos dabs: carimbados em `Mix` (o pincel da ferramenta, sem a borracha, sem rampas) num plano
    /// transparente, com o tecto e o trilho de píxel DELE.
    fn carimba_a_cobertura(&mut self, s: &mut PlanoDoTraco, dabs: &[Dab]) {
        let Some(f) = s.ferramenta.as_mut() else {
            return;
        };
        let n4 = self.canvas_rgba.len();
        let mut plano = f
            .cobertura
            .take()
            .filter(|c| c.len() == n4)
            .unwrap_or_else(|| Arc::new(vec![0u8; n4]));
        let p = &mut self.paint;
        swap(&mut p.stroke_mask, &mut f.stroke_mask);
        swap(&mut p.pixel_trilhos, &mut f.trilhos);
        let guardado = (
            p.paint_mode,
            p.eraser,
            p.brush.blend,
            p.acumulando_no_plano,
            p.shape_color_ramp_enabled,
            p.texture_ramp_enabled,
        );
        p.paint_mode = PaintMode::Paint;
        p.eraser = false;
        p.brush.blend = BrushBlend::Mix;
        p.acumulando_no_plano = true;
        p.shape_color_ramp_enabled = false;
        p.texture_ramp_enabled = false;
        let ws = &self.undo.write_state;
        super::plane_fork::swap_canvas_plane(&mut self.canvas_rgba, &mut plano, ws);
        self.stamp_dabs_routed(dabs);
        let ws = &self.undo.write_state;
        super::plane_fork::swap_canvas_plane(&mut self.canvas_rgba, &mut plano, ws);
        let p = &mut self.paint;
        (
            p.paint_mode,
            p.eraser,
            p.brush.blend,
            p.acumulando_no_plano,
            p.shape_color_ramp_enabled,
            p.texture_ramp_enabled,
        ) = guardado;
        swap(&mut p.stroke_mask, &mut f.stroke_mask);
        swap(&mut p.pixel_trilhos, &mut f.trilhos);
        f.cobertura = Some(plano);
    }

    /// ⭐ **A projecção** de `r`: `base + (livre − base) · modo(a, m) / a` — função pura da sessão (re-projectar
    /// uma janela já projectada dá os mesmos bytes).
    fn projecta_a_ferramenta(&mut self, s: &PlanoDoTraco, modo: DualMode, r: Region) {
        if r.w == 0 || r.h == 0 {
            return;
        }
        let w = self.source_size.0 as usize;
        let (base, livre) = (&s.base[..], &s.plano[..]);
        let mascara = s.mascara.as_deref().map(|m| &m[..]);
        let cobertura = s
            .ferramenta
            .as_ref()
            .and_then(|f| f.cobertura.as_deref())
            .map(|c| &c[..]);
        let buf = super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            Some(r),
        );
        let dec = |v: u8| f32::from(v) / 255.0;
        let linha_projectada = |(dy, linha): (usize, &mut [u8])| {
            let y = r.y as usize + dy;
            for x in r.x as usize..(r.x + r.w) as usize {
                let i = (y * w + x) * 4;
                let (b, l) = (&base[i..i + 4], &livre[i..i + 4]);
                let m = mascara.map_or(0.0, |m| dec(m[i + 3]));
                let f = if b == l {
                    0.0
                } else if modo == DualMode::Multiply {
                    m
                } else {
                    let a = cobertura.map_or(0.0, |c| dec(c[i + 3]));
                    if a > 0.0 { modo.recorta(a, m) / a } else { 0.0 }
                };
                let o = &mut linha[x * 4..x * 4 + 4];
                if f <= 0.0 {
                    o.copy_from_slice(b);
                } else if f == 1.0 {
                    o.copy_from_slice(l);
                } else {
                    for k in 0..4 {
                        let v = f32::from(b[k]) + (f32::from(l[k]) - f32::from(b[k])) * f;
                        o[k] = v.round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
        };
        let linhas = &mut buf[r.y as usize * w * 4..(r.y + r.h) as usize * w * 4];
        if (r.w as usize) * (r.h as usize) < PROJECTA_EM_PARALELO_DESDE {
            linhas
                .chunks_mut(w * 4)
                .enumerate()
                .for_each(linha_projectada);
        } else {
            linhas
                .par_chunks_mut(w * 4 * LINHAS_POR_TAREFA)
                .enumerate()
                .for_each(|(k, bloco)| {
                    for (j, l) in bloco.chunks_mut(w * 4).enumerate() {
                        linha_projectada((k * LINHAS_POR_TAREFA + j, l));
                    }
                });
        }
        self.declare_wrote(Some(r));
        self.mark_dirty(r);
    }
}
