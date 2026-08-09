//! The single **inverse-warp kernel** — `out[dst] = bilinear(stroke_src, dst − D(dst))`, backward-gather
//! so there are never holes. Shared by every Reshape sub-mode (only the [`DabField`] differs).
//!
//! **Anti-blur (per-stroke single resample).** Each dab does NOT re-gather the already-warped canvas
//! (which compounds bilinear softening into a smear). It advances the session's backward MAP and
//! re-renders the dab bbox from the frozen session pixels using the TOTAL displacement — so the whole
//! stroke costs exactly one resample per texel and stays sharp.
//!
//! ## Como a lista de dabs é dobrada — e por que a SOMA morreu ([ADR-0157])
//!
//! Este arquivo somava: `disp[i] += v_i`. ⚠️ **Somar é composição EXATA para translação e para mais
//! nada** — e é por isso que só o **Push** parecia bom. Somar as cordas `R(θ)v − v` de um Twist `N` vezes
//! dá `N·corda`, uma reta tangente sem teto; compor dá `R(Nθ)`, limitado. Medido na fixture dos gates
//! (Twist parado, pincel r=100, sonda a r=30, onde uma rotação não pode passar de **2r = 60 px**): a soma
//! deslocava **69,34 px com 20 dabs** e **693,36 px com 200**, enquanto a composição nunca passa de 60 e
//! *oscila* dentro do intervalo — a assinatura de uma rotação. A linha preta de 3 px sobrevivia a **28,1%**
//! sob a soma e a **103,9%** sob a composição (passa de 100 porque girar uma horizontal a deixa diagonal,
//! que cobre mais texels: tinta espalhada, não criada).
//!
//! A lei é a mesma que `ph2d_painter_brush::smear_field` já carrega, e o doc-comment dele diz em tantas
//! palavras que a acumulação óbvia *"é ERRADA, e errada de um jeito que vale registrar porque ela PARECE
//! certa"*. Um traço é um **REVEZAMENTO**: o dab `k` entrega ao `k+1`, então
//!
//! ```text
//! D_k(p) = v_k(p) + D_{k−1}(p − v_k(p))
//! ```
//!
//! ⚠️ **Ela é INCREMENTAL, e isso não é sorte — é a razão de o cook caber num dab:** fora do disco do dab
//! `k` vale `v_k(p) = 0`, logo `D_k(p) = D_{k−1}(p)`. Um dab novo só toca a PRÓPRIA pegada, exatamente
//! como a soma tocava, então a travessia não custa uma ordem de grandeza — ela custa uma janela.
//!
//! ⚠️ E com `v = 0` em toda parte a leitura cai em coordenada INTEIRA, onde a bilinear devolve o texel
//! exato ⇒ `disp` não se move: o campo identidade continua **byte-idêntico**.
//!
//! [ADR-0157]: ../../../../../../docs/architecture/decisions/0157-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md

use super::field::DabField;
use crate::tool::PainterTool;
use ph2d_painter_brush::MapWindow;
use std::sync::Arc;

impl PainterTool {
    /// Apply one Reshape dab of the given field at `center` (image px) with pixel `radius`. **COMPÕE** o
    /// campo no mapa de deslocamento da sessão (`D_k = v_k + D_{k−1}(p − v_k)`, a lei do módulo) e
    /// re-renderiza a bbox do dab a partir do `pre` congelado. No-op fora de um canvas dimensionado ou
    /// antes de os buffers da sessão existirem. Com `D = 0` em toda parte o render resolve cada `dst` para
    /// si mesmo → **byte-idêntico**.
    pub(super) fn warp_apply_dab(&mut self, field: &DabField, center: [f32; 2], radius: f32) {
        let Some(bbox) = self.dab_bbox(center, radius) else {
            return;
        };
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.warp.pre.len() != n * 4 || self.paint.warp.disp.len() != n {
            return; // session not set up (unsized canvas)
        }
        // Deform is confined to the SELECTED area (whole sprite when nothing is selected) — the brush only
        // moves texels the selection covers, so it can't smear content outside the region the artist chose.
        let restrict = self.deform_restricts_to_selection();

        // Pass 1 (immutable self): compute each texel's displacement contribution, scaled by the selection
        // coverage (0 outside → no movement). Collected into a bbox-local buffer so the mutable
        // accumulate/render passes don't fight the `selection_coverage_at` borrow.
        let mut adds: Vec<[f32; 2]> = Vec::with_capacity((bbox.w * bbox.h) as usize);
        // O quanto ESTE dab consegue retro-traçar — a margem que a janela do mapa antigo precisa. Sai da
        // MEDIÇÃO do campo em vez de um teto por modo: a passada 1 já avalia todo texel, então um limite
        // analítico seria uma segunda resposta, e é o tipo de número que envelhece quando um modo novo
        // entra.
        //
        // ⚠️ **Defesa em camada, e ela NÃO é observável hoje** (medido: a mutação `reach → 0` sobrevive aos
        // gates, inclusive ao de um Push de 25 px). O motivo é geométrico: todo modo multiplica por `f`,
        // que é 1 no centro e **0 na borda**, então `|v|` é grande só onde o dab está fundo dentro do
        // próprio bbox — o retro-traço aponta para DENTRO e o `+2` fixo da bilinear basta. Fica porque a
        // alternativa é depender de uma invariante não dita: um modo cujo `v` não desapareça na borda a
        // torna necessária no mesmo dia. Custa um `max` num laço que já avalia o campo.
        let mut reach = 0.0_f32;
        for ry in 0..bbox.h {
            let dy = bbox.y + ry;
            for rx in 0..bbox.w {
                let dx = bbox.x + rx;
                let mut d = field.at([dx as f32, dy as f32]);
                if restrict {
                    let allow = f32::from(self.selection_coverage_at(dx, dy)) / 255.0;
                    d[0] *= allow;
                    d[1] *= allow;
                }
                reach = reach.max(d[0].abs()).max(d[1].abs());
                adds.push(d);
            }
        }

        // Pass 2 (mutable): COMPOR a contribuição deste dab no mapa da sessão, e re-renderizar a bbox a
        // partir do `pre` pristino com o deslocamento TOTAL (uma reamostragem por texel → nítido, sem blur
        // composto, e entre traços ainda amostra a fonte pristina).
        let src = Arc::clone(&self.paint.warp.pre);
        let disp = Arc::make_mut(&mut self.paint.warp.disp);
        // ⚠️ A janela é o mapa ANTIGO congelado: a atualização lê `disp_old(p − v)` de vizinhos de `p`, e
        // ler-e-escrever o mesmo buffer no lugar deixaria um texel já atualizado poluir um que ainda não
        // foi — a dependência sequencial que esta lei existe para remover.
        let mut win_buf: Vec<[f32; 2]> = Vec::new();
        let win = MapWindow::snapshot(
            &mut win_buf,
            disp,
            w,
            h,
            (bbox.x, bbox.y, bbox.w, bbox.h),
            reach,
        );
        let buf = super::super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            None,
        );
        for ry in 0..bbox.h {
            let dy = bbox.y + ry;
            for rx in 0..bbox.w {
                let dx = bbox.x + rx;
                let gi = (dy * w + dx) as usize;
                let v = adds[(ry * bbox.w + rx) as usize];
                let back = win.sample(dx as f32 - v[0], dy as f32 - v[1]);
                let d = [v[0] + back[0], v[1] + back[1]];
                disp[gi] = d;
                let px = bilinear_clamped(&src, w, h, dx as f32 - d[0], dy as f32 - d[1]);
                let b = gi * 4;
                buf[b..b + 4].copy_from_slice(&px);
            }
        }
        // ⚠️ **A LISTA é o estado; o mapa acima é o cache dela** (ADR-0157). O dab entra DEPOIS de o mapa
        // ter sido avançado com ele, para que as duas metades descrevam sempre o mesmo instante — um push
        // antes do laço deixaria a lista à frente do mapa por uma linha, e um early-return no meio a
        // deixaria à frente para sempre.
        Arc::make_mut(&mut self.paint.warp.dabs).push(field.clone());
        // W4: the body rides the same displacement — one door for every warp render (`warp/relief.rs`).
        self.warp_render_relief(bbox);
        self.mark_dirty(bbox);
    }
}

/// Bilinear-sample straight-alpha RGBA8 `canvas` (`w×h`) at fractional `(x, y)`, clamping to the edge
/// (extend, never wrap → no holes). Interpolates in **premultiplied** space then un-premultiplies, so a
/// transparent texel's (arbitrary) RGB never bleeds a dark fringe near an alpha edge — the speckle a
/// straight per-channel lerp produced. At integer coords the fractions are `0`, so it returns the exact
/// texel (identity-safe). Canvas is straight RGBA (the shell premultiplies on GPU upload).
#[inline]
pub(super) fn bilinear_clamped(canvas: &[u8], w: u32, h: u32, x: f32, y: f32) -> [u8; 4] {
    if w == 0 || h == 0 {
        return [0, 0, 0, 0];
    }
    let x0f = x.floor();
    let y0f = y.floor();
    let fx = x - x0f;
    let fy = y - y0f;
    let (wi, hi) = (w as i64, h as i64);
    let x0 = (x0f as i64).clamp(0, wi - 1);
    let y0 = (y0f as i64).clamp(0, hi - 1);
    let x1 = (x0 + 1).clamp(0, wi - 1);
    let y1 = (y0 + 1).clamp(0, hi - 1);
    let stride = w as usize * 4;
    // A corner as PREMULTIPLIED `[r·a, g·a, b·a, a]` (rgb on the `0..=255` scale, `a` raw).
    let corner = |xi: i64, yi: i64| -> [f32; 4] {
        let b = yi as usize * stride + xi as usize * 4;
        let a = f32::from(canvas[b + 3]);
        let s = a / 255.0;
        [
            f32::from(canvas[b]) * s,
            f32::from(canvas[b + 1]) * s,
            f32::from(canvas[b + 2]) * s,
            a,
        ]
    };
    let lerp = |p: [f32; 4], q: [f32; 4], t: f32| {
        [
            p[0] + (q[0] - p[0]) * t,
            p[1] + (q[1] - p[1]) * t,
            p[2] + (q[2] - p[2]) * t,
            p[3] + (q[3] - p[3]) * t,
        ]
    };
    let top = lerp(corner(x0, y0), corner(x1, y0), fx);
    let bot = lerp(corner(x0, y1), corner(x1, y1), fx);
    let m = lerp(top, bot, fy); // [premR, premG, premB, A]
    let a = m[3];
    if a <= 0.0 {
        return [0, 0, 0, 0];
    }
    let inv = 255.0 / a; // un-premultiply
    [
        (m[0] * inv).round().clamp(0.0, 255.0) as u8,
        (m[1] * inv).round().clamp(0.0, 255.0) as u8,
        (m[2] * inv).round().clamp(0.0, 255.0) as u8,
        a.round().clamp(0.0, 255.0) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bilinear_at_integer_coords_is_exact() {
        // 2×2 image, distinct pixels; sampling exactly on a texel returns it byte-for-byte (identity).
        let w = 2;
        let h = 2;
        let px = [
            10, 20, 30, 40, // (0,0)
            50, 60, 70, 80, // (1,0)
            90, 100, 110, 120, // (0,1)
            130, 140, 150, 160, // (1,1)
        ];
        assert_eq!(bilinear_clamped(&px, w, h, 0.0, 0.0), [10, 20, 30, 40]);
        assert_eq!(bilinear_clamped(&px, w, h, 1.0, 0.0), [50, 60, 70, 80]);
        assert_eq!(bilinear_clamped(&px, w, h, 1.0, 1.0), [130, 140, 150, 160]);
        // Out of bounds clamps to the edge texel.
        assert_eq!(bilinear_clamped(&px, w, h, -5.0, -5.0), [10, 20, 30, 40]);
        assert_eq!(bilinear_clamped(&px, w, h, 9.0, 9.0), [130, 140, 150, 160]);
    }

    #[test]
    fn bilinear_midpoint_of_opaque_is_the_average() {
        // Two OPAQUE texels: premultiplied == straight, so the midpoint is the plain average.
        let w = 2;
        let h = 1;
        let px = [0, 0, 0, 255, 100, 100, 100, 255];
        assert_eq!(bilinear_clamped(&px, w, h, 0.5, 0.0), [50, 50, 50, 255]);
    }

    #[test]
    fn bilinear_across_an_alpha_edge_has_no_dark_fringe() {
        // A fully-transparent texel with GARBAGE rgb (255,0,0) next to an opaque black one. A straight
        // per-channel lerp would bleed the red in; premultiplied resampling keeps the colour clean — the
        // midpoint alpha is 50% but the RGB is the opaque texel's, NOT muddied toward the transparent rgb.
        let w = 2;
        let h = 1;
        let px = [
            255, 0, 0, 0, /* transparent, junk rgb */
            0, 0, 0, 255, /* opaque black */
        ];
        let m = bilinear_clamped(&px, w, h, 0.5, 0.0);
        assert_eq!(m[3], 128, "alpha is the average (50%)");
        assert!(
            m[0] < 8 && m[1] < 8 && m[2] < 8,
            "no red fringe from the transparent texel: got {m:?}"
        );
    }
}
