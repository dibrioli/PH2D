//! **Com que PIXELS uma forma entra no composite booleano** — o irmão de `stroke_boolean.rs` por
//! ASSUNTO. Lá mora *que formas entram e o que sai* (a conversão, a janela, o traçado, o merge); aqui,
//! *como uma forma vira cobertura*: a sub-janela dela, os dois rasterizadores e a régua do diagnóstico.
//!
//! O corte é o que a wave da sub-janela por forma (2026-08-06) tornou natural — foi ela que fez esta
//! metade crescer, e é ela que tem uma pergunta própria.

use super::selection_shapes::{SelectionShape, rasterize_ellipse, sel_polygon_vertices};
use super::stroke_boolean::SS;
use super::*;

impl PainterTool {
    /// Rasterise one fill shape scaled by `SS` into the supersampled `crisp` via `op` (transcendental-free:
    /// exact ellipse inside-test / baked-step polygon / flattened freehand → scanline fill).
    /// Rasteriza UMA forma na janela local, com `origin` (em texels supersampleados) subtraído.
    ///
    /// ⚠️ **A janela é uma TELA VIRTUAL**, o mesmo truque da banda do `stamp_banded`: o rasterizador
    /// recebe as coordenadas deslocadas e a largura/altura da janela, e o recorte cai do `clamp` que
    /// ele já faz contra o tamanho da tela. Nenhuma segunda resposta a *"que pixels esta forma cobre?"*.
    pub(super) fn rasterize_fill_ss(
        &self,
        sel: &SelectionShape,
        sw: usize,
        sh: usize,
        origin: [f32; 2],
        region: &mut [u8],
    ) {
        let s = SS as f32;
        let at = |p: [f32; 2]| [p[0] * s - origin[0], p[1] * s - origin[1]];
        match sel {
            SelectionShape::Ellipse { center, u, rx, ry } => {
                rasterize_ellipse(at(*center), *u, rx * s, ry * s, sw, sh, region);
            }
            SelectionShape::Polygon {
                center,
                u,
                rx,
                ry,
                sides,
            } => {
                let verts: Vec<[f32; 2]> = sel_polygon_vertices(*center, *u, *rx, *ry, *sides)
                    .iter()
                    .map(|p| at(*p))
                    .collect();
                scanline_fill(&verts, sw, sh, region);
            }
            SelectionShape::Freehand { model, .. } => {
                let spine = self.freehand_spine(&model.points, &model.handles);
                let verts: Vec<[f32; 2]> = spine.iter().map(|p| at(*p)).collect();
                scanline_fill(&verts, sw, sh, region);
            }
            SelectionShape::Raster { .. } => {}
        }
    }
}

/// A janela de UMA forma dentro da janela do composite — origem local + tamanho, em texels
/// supersampleados — ou `None` quando a caixa dela não intersecta a janela (uma forma de Remove longe
/// dos Add) e a sub-janela não teria o que compor.
///
/// ⚠️ **O `PAD` é FOLGA DECLARADA, e a mutação que o zera SOBREVIVE de propósito** — eu o justifiquei
/// primeiro por dois alcances que a leitura desmentiu, e o registro fica porque a aritmética real é o
/// que torna a sub-janela correta:
///
/// - o `rasterize_ellipse` clampa cada semi-eixo em meio TEXEL, mas o
///   [`Self::stroke_state_to_fill_shape`] já entrega `rx`/`ry` clampados em meio **px de imagem** —
///   `SS` vezes maior —, então aquele clamp é **inalcançável por esta porta** e não alarga nada;
/// - o `scanline_fill` arredonda os extremos de cada span, e `round(v)` fica em `[floor(v), ceil(v)]`
///   **por definição** ⇒ um span nunca sai de `[floor(lo), ceil(hi))`, que é exatamente esta caixa;
/// - e o teste de dentro da elipse escreve o texel `xx` só se `xx + 0.5` está entre `lo` e `hi`, o que
///   dá `xx ∈ [floor(lo), ceil(hi))` pelo mesmo argumento.
///
/// Com `PAD = 0` a caixa já seria exata. O texel de folga fica porque o primeiro item acima é uma
/// premissa que mora em **outra função**: se alguém afrouxar o clamp da porta, com folga o preço é um
/// texel de trabalho a mais, e sem folga é **forma truncada em silêncio**.
pub(super) fn window_sub_rect(
    bb: [f32; 4],
    s: f32,
    win: [usize; 2],
    sw: usize,
    sh: usize,
) -> Option<(usize, usize, usize, usize)> {
    const PAD: f32 = 1.0;
    #[allow(clippy::cast_precision_loss)]
    let (ox, oy) = (win[0] as f32, win[1] as f32);
    // NaN cai em 0 nos dois extremos (cast saturante), então uma caixa não-finita devolve `None` e a
    // forma segue pela rota de janela cheia — exatamente o que ela fazia antes desta sub-janela existir.
    let cut = |v: f32, hi: usize| -> usize {
        #[allow(clippy::cast_precision_loss)]
        let hi_f = hi as f32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let out = v.clamp(0.0, hi_f) as usize;
        out.min(hi)
    };
    let rx = cut((bb[0] * s - PAD).floor() - ox, sw);
    let ry = cut((bb[1] * s - PAD).floor() - oy, sh);
    let rx1 = cut((bb[2] * s + PAD).ceil() - ox, sw);
    let ry1 = cut((bb[3] * s + PAD).ceil() - oy, sh);
    (rx1 > rx && ry1 > ry).then_some((rx, ry, rx1 - rx, ry1 - ry))
}

/// Even-odd scanline polygon fill of the closed polyline `pts` into `cov` (0/255) at `w`×`h`. Mirrors the
/// selection `raster_lasso` but writes into a caller buffer at an arbitrary (supersampled) resolution.
fn scanline_fill(pts: &[[f32; 2]], w: usize, h: usize, cov: &mut [u8]) {
    if pts.len() < 3 {
        return;
    }
    for yy in 0..h {
        let yc = yy as f32 + 0.5;
        let mut xs: Vec<f32> = Vec::new();
        for i in 0..pts.len() {
            let p = pts[i];
            let q = pts[(i + 1) % pts.len()];
            if (p[1] <= yc && yc < q[1]) || (q[1] <= yc && yc < p[1]) {
                xs.push(p[0] + (yc - p[1]) / (q[1] - p[1]) * (q[0] - p[0]));
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut i = 0;
        while i + 1 < xs.len() {
            let xa = xs[i].max(0.0).round() as usize;
            let xb = (xs[i + 1].min(w as f32).round() as usize).min(w);
            for xx in xa..xb {
                cov[yy * w + xx] = 255;
            }
            i += 2;
        }
    }
}
