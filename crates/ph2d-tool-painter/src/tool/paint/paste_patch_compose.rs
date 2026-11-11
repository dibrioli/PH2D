//! O compositor da peça colada: o pristino (os quatro planos), o restauro e a composição da cor e do
//! corpo. Partido de [`super`] (B52.R16); a amostragem e a pose ficam lá, o gesto em `paste_patch_input`.

use super::super::{PainterTool, Region};
use super::PastePatch;
use crate::tool::paint::selection_actions::escrita_direta::GuardaDaEscrita;
use ph2d_painter_brush::material::MaterialBytes;

/// **O pristino da peça flutuante — os QUATRO planos, não só os pixels.**
///
/// A dança de um quadro (*restaura → guarda → compõe*) só devolve a tela ao que era se guardar tudo o que
/// o composite escreve. Guardar só o RGBA enquanto o composite passou a escrever relevo faria cada quadro
/// de arrasto **acumular corpo** sobre o corpo do quadro anterior — a peça engrossaria com o tempo que o
/// artista levasse para posicioná-la, que é a dependência-de-amostragem que esta linha já curou quatro
/// vezes no relevo.
///
/// ⚠️ Os três planos do impasto são `None` juntos: ou a peça tem corpo e o composite os toca, ou não tem e
/// eles nem são lidos.
#[derive(Clone)]
pub(crate) struct PastePristine {
    pub rect: Region,
    pub rgba: Vec<u8>,
    pub relief: Option<(Vec<f32>, Vec<u8>, Vec<MaterialBytes>)>,
}

/// Recorta `rect` de um plano canvas-shaped — o irmão genérico do `region::region_pixels` (que é do RGBA,
/// com o seu stride de 4).
fn region_of<T: Copy + Default>(buf: &[T], rect: Region, canvas_w: u32) -> Vec<T> {
    let row = rect.w as usize;
    let mut out = vec![T::default(); row * rect.h as usize];
    for ry in 0..rect.h {
        let src = (rect.y + ry) as usize * canvas_w as usize + rect.x as usize;
        if src + row <= buf.len() {
            out[ry as usize * row..(ry as usize + 1) * row].copy_from_slice(&buf[src..src + row]);
        }
    }
    out
}

impl PainterTool {
    /// Guarda o que a peça está prestes a cobrir: os pixels, e o corpo quando ela tem corpo a escrever.
    pub(super) fn snapshot_paste_pristine(
        &self,
        patch: &PastePatch,
        rect: Region,
        w: u32,
    ) -> PastePristine {
        // A peça vem por PARÂMETRO e não de `self.paint.paste_patch`: o chamador já a tem na mão, e
        // relê-la daqui seria uma segunda fonte para o mesmo fato. Camada sem plano guarda ZEROS do
        // tamanho: o composite cria o plano, e o restauro tem de o devolver liso (B06.5).
        let area = (rect.w as usize) * (rect.h as usize);
        let relief = patch
            .relief
            .is_some()
            .then(|| self.layers.active())
            .flatten()
            .map(|layer| {
                (
                    self.heights
                        .get(&layer)
                        .map_or_else(|| vec![0.0; area], |p| region_of(p, rect, w)),
                    self.covers
                        .get(&layer)
                        .map_or_else(|| vec![0; area], |p| region_of(p, rect, w)),
                    self.mats.get(&layer).map_or_else(
                        || vec![MaterialBytes::default(); area],
                        |p| region_of(p, rect, w),
                    ),
                )
            });
        PastePristine {
            rect,
            rgba: crate::tool::paint::region::region_pixels(&self.canvas_rgba, rect, w),
            relief,
        }
    }

    /// Devolve os três planos do impasto ao que estavam sob a peça.
    pub(super) fn restore_relief_region(&mut self, pr: &PastePristine) {
        let (Some((h, c, m)), Some(layer), (w, ch)) =
            (pr.relief.as_ref(), self.layers.active(), self.source_size)
        else {
            return;
        };
        let (wu, n) = (w as usize, (w as usize) * (ch as usize));
        let row = pr.rect.w as usize;
        if !h.is_empty()
            && let Some(e) = self.heights.get_mut(&layer)
            && e.len() == n
        {
            let dst = crate::tool::paint::plane_fork::fork_heights(
                e,
                &self.undo.write_state,
                layer,
                (w, ch),
                None,
            );
            for ry in 0..pr.rect.h as usize {
                let d = (pr.rect.y as usize + ry) * wu + pr.rect.x as usize;
                dst[d..d + row].copy_from_slice(&h[ry * row..(ry + 1) * row]);
            }
        }
        if !c.is_empty()
            && let Some(e) = self.covers.get_mut(&layer)
            && e.len() == n
        {
            let dst = crate::tool::paint::plane_fork::fork_covers(
                e,
                &self.undo.write_state,
                layer,
                (w, ch),
                None,
            );
            for ry in 0..pr.rect.h as usize {
                let d = (pr.rect.y as usize + ry) * wu + pr.rect.x as usize;
                dst[d..d + row].copy_from_slice(&c[ry * row..(ry + 1) * row]);
            }
        }
        if !m.is_empty()
            && let Some(e) = self.mats.get_mut(&layer)
            && e.len() == n
        {
            let dst = crate::tool::paint::plane_fork::fork_mats(
                e,
                &self.undo.write_state,
                layer,
                (w, ch),
                None,
            );
            for ry in 0..pr.rect.h as usize {
                let d = (pr.rect.y as usize + ry) * wu + pr.rect.x as usize;
                dst[d..d + row].copy_from_slice(&m[ry * row..(ry + 1) * row]);
            }
        }
        self.sync_relief_flags();
    }
}

impl PainterTool {
    /// Compõe a peça sobre a tela dentro de `rect` (source-over pelo alfa da peça) — **e o CORPO junto**.
    ///
    /// ## A lei de cada plano é a que o DEPÓSITO já usa, e não uma inventada aqui
    ///
    /// - **Cobertura por `max`.** Duas demãos sobre o mesmo pixel não fazem "200% de tinta": cobertura é
    ///   uma PRESENÇA. É literalmente o que o `commit_stroke_height` faz, e o que a chegada de matéria do
    ///   Inflate faz (`v > pre_cover` ⇒ assume).
    /// - **Material por `over`** na opacidade que de fato chegou: um material é uma IDENTIDADE, e tinta
    ///   por cima de tinta se parece com a de cima na proporção em que a cobre.
    /// - **Altura por `over` — e NÃO somada**, que é onde isto se afasta do depósito de propósito. Um
    ///   depósito acrescenta MATÉRIA nova (mais tinta É mais grossa, e o `commit_stroke_height` soma por
    ///   isso); um paste **coloca uma imagem** da tinta, exatamente como o RGBA dele compõe `over` em vez
    ///   de somar. A prova é uma propriedade que o artista lê: **colar uma cópia opaca no mesmo lugar tem
    ///   de ser a IDENTIDADE** — com `over` é (`a = 1` ⇒ o destino vira a fonte, que é ele mesmo), e
    ///   somando a peça ficaria com o dobro da espessura num gesto que ninguém lê como "engrossar".
    ///
    /// ⚠️ **Só escreve relevo quando a peça TEM relevo** — e a mutação corrigiu o que eu ia afirmar aqui.
    /// Os *pixels* do corpo já estão protegidos sem este early-out: sem relevo o `sample_relief` recusa
    /// todo texel, então nada seria escrito de qualquer forma. O que ele de fato impede é a **ALOCAÇÃO** —
    /// os `fork_*` criam os planos na camada, e colar tinta chapada daria 12 B/px de corpo vazio a uma
    /// camada que nunca teve nenhum. É essa metade que o gate afirma.
    pub(super) fn composite_patch(&mut self, patch: &PastePatch, rect: Region) {
        let w = self.source_size.0 as usize;
        // B40.A1: a proteção e o trinco de alfa valem para a peça como para o carimbo.
        let guarda = self.guarda_da_escrita_direta();
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            Some(rect),
        );
        for y in rect.y..rect.y + rect.h {
            for x in rect.x..rect.x + rect.w {
                let Some(s) = patch.sample([x as f32 + 0.5, y as f32 + 0.5]) else {
                    continue;
                };
                let a = f32::from(s[3]) / 255.0;
                let d = (y as usize * w + x as usize) * 4;
                let orig = [buf[d], buf[d + 1], buf[d + 2], buf[d + 3]];
                for c in 0..3 {
                    let src = f32::from(s[c]);
                    let dst = f32::from(buf[d + c]);
                    buf[d + c] = (src * a + dst * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
                }
                let da = f32::from(buf[d + 3]) / 255.0;
                buf[d + 3] = ((a + da * (1.0 - a)) * 255.0).round().clamp(0.0, 255.0) as u8;
                if !guarda.neutra() {
                    guarda.assenta(d / 4, orig, &mut buf[d..d + 4]);
                }
            }
        }
        self.composite_patch_relief(patch, rect, &guarda);
    }

    /// A metade do [`Self::composite_patch`] que escreve os três planos do impasto. No-op sem relevo na
    /// peça ou sem camada ativa.
    fn composite_patch_relief(
        &mut self,
        patch: &PastePatch,
        rect: Region,
        guarda: &GuardaDaEscrita,
    ) {
        if patch.relief.is_none() {
            return;
        }
        let (Some(layer), (w, h)) = (self.layers.active(), self.source_size) else {
            return;
        };
        let (wu, n) = (w as usize, (w as usize) * (h as usize));
        {
            let dst = crate::tool::paint::plane_fork::fork_heights(
                self.heights.entry(layer).or_default(),
                &self.undo.write_state,
                layer,
                (w, h),
                Some(rect),
            );
            if dst.len() != n {
                dst.resize(n, 0.0);
            }
            let cov = crate::tool::paint::plane_fork::fork_covers(
                self.covers.entry(layer).or_default(),
                &self.undo.write_state,
                layer,
                (w, h),
                Some(rect),
            );
            if cov.len() != n {
                cov.resize(n, 0);
            }
            let mat = crate::tool::paint::plane_fork::fork_mats(
                self.mats.entry(layer).or_default(),
                &self.undo.write_state,
                layer,
                (w, h),
                Some(rect),
            );
            if mat.len() != n {
                mat.resize(n, [0u8; 7]);
            }
            for y in rect.y..rect.y + rect.h {
                for x in rect.x..rect.x + rect.w {
                    let Some((sh, sc, sm)) = patch.sample_relief([x as f32 + 0.5, y as f32 + 0.5])
                    else {
                        continue;
                    };
                    let gi = y as usize * wu + x as usize;
                    // Com o trinco o alfa de depois É o de antes: a cor já assentou.
                    let (a, sc) = if guarda.neutra() {
                        (f32::from(sc) / 255.0, sc)
                    } else {
                        let alfa = self.canvas_rgba.get(gi * 4 + 3).copied().unwrap_or(0);
                        let k = guarda.fator_do_corpo(gi, alfa);
                        let sc = (f32::from(sc) * k).round().clamp(0.0, 255.0) as u8;
                        (f32::from(sc) / 255.0, sc)
                    };
                    dst[gi] += a * (sh - dst[gi]);
                    cov[gi] = cov[gi].max(sc);
                    for c in 0..7 {
                        let (old, new) = (f32::from(mat[gi][c]), f32::from(sm[c]));
                        mat[gi][c] = (old + a * (new - old)).round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
        // Sem isto a tinta tem corpo e a luz não sabe: o relevo só acende quando a camada se declara.
        self.sync_relief_flags();
    }
}
