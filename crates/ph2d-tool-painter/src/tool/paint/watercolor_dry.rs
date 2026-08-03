//! Watercolor **SECAGEM**: o decaimento por-quadro do mapa de umidade (`canvas_wet`). Irmão do
//! `watercolor_backdrop` (que possui o mapa e o DESPEJO); separado dele em 2026-08-02 pelo teto de LOC
//! e por assunto — *o que molha o papel* e *o que o seca* são duas perguntas.
//!
//! ⚠️ O passe roda em **TODO quadro**, pinte-se ou não, e o log do produto o mediu como o maior item
//! isolado do quadro do artista. A história das quatro tentativas (três a ~1,00×, a quarta a 9,3×)
//! está no corpo do `dry_canvas_wet_inner` e na emenda de 2026-08-02 do ADR-0109.

use super::watercolor_backdrop::WET_PAR_MIN;
use super::*;

/// #12b (doc 14) — edges-to-centre drying: a wet pool recedes from its PERIMETER inward, not uniformly.
/// On each decay step a pixel loses the flat `step` PLUS this gain × the gap to its driest 4-neighbour
/// (`0` in a flat interior ⇒ uniform there; large at the wet front ⇒ the boundary recedes first). `2`
/// makes a sharp front dry ~3× the interior rate.
pub(super) const WET_ERODE_GAIN: u32 = 2;

impl PainterTool {
    /// EDGE-1: the paper DRIES on the heartbeat — [`CANVAS_WET_DRY_PER_S`] with a fractional carry (slow
    /// frames still dry at the same wall-clock rate). #12b: the drying is EDGES-TO-CENTRE — the boundary
    /// recedes faster than the interior ([`WET_ERODE_GAIN`]), a wet pool shrinking from its perimeter like
    /// real paper. Drops the buffer once fully dry, restoring the moisture-free fast path.
    pub(super) fn dry_canvas_wet(&mut self, dt_s: f32) {
        // Envelope de diagnóstico — o corpo é o `_inner` (ver `apply_watercolor`).
        let t0 = std::time::Instant::now();
        self.dry_canvas_wet_inner(dt_s);
        crate::wash_diag::note_dry(t0.elapsed().as_secs_f32() * 1e3);
    }

    fn dry_canvas_wet_inner(&mut self, dt_s: f32) {
        let Some((x0, y0, x1, y1)) = self.paint.canvas_wet_rect else {
            return;
        };
        let (fw, fh) = self.source_size;
        let fw = fw as usize;
        // Belt-and-braces (sweep 2026-07-12): the moisture map OUTLIVES the gesture, so a document rebind
        // can leave it shaped for the PREVIOUS sprite — and the loop below indexes it with the CURRENT
        // sprite's stride and a rect in the OLD sprite's coordinates (a bigger sprite ⇒ slice past the end
        // ⇒ SIGSEGV). `reset_transient_edit_state` now drops it at the rebind, so this is unreachable; it
        // stays because the buffer is canvas-sized and the guard asked "does it exist?" instead of "does
        // its SHAPE match?" — the exact hole that produced Bug #12. Physics untouched: when the shape
        // matches (always, in every real path) this is byte-identical to the old `is_empty()` check.
        if self.paint.canvas_wet.len() != fw * (fh as usize) {
            self.paint.canvas_wet = Vec::new();
            self.paint.canvas_wet_rect = None;
            return;
        }
        self.paint.canvas_wet_carry += dt_s.max(0.0) * self.paint.dry_rate_per_s.max(0.0);
        let step = self.paint.canvas_wet_carry.min(255.0) as u8;
        if step == 0 {
            return;
        }
        self.paint.canvas_wet_carry -= f32::from(step);
        // O gap de 4 vizinhos tem de ler a umidade PRÉ-passo (a lei é independente de ordem); fora do
        // rect conta como seco (0), então a frente molhada recua também na borda do rect.
        //
        // ⚠️ **O custo deste passe é o CAMINHAR, e isso foi medido, não suposto** (2026-08-02). O log do
        // produto o mostrou em 10-16 ms/quadro a 4096², o maior item isolado do quadro do artista. Três
        // curas byte-idênticas foram construídas e **MEDIDAS EM ~1,00×** antes desta: trocar o snapshot
        // por uma janela deslizante (1,02×), pular o gather onde a erosão não pode disparar, e encolher
        // o rect. A conta é 2,2 ns/texel, e nenhuma delas a move — o que a move é o número de núcleos.
        //
        // **Row-parallel sob o [ADR-0109]** (emenda de 2026-08-02): cada linha de saída lê o snapshot
        // IMUTÁVEL e escreve só a própria fatia; a redução é `max`/`min` sobre inteiros, associativa e
        // comutativa, então a ordem entre threads não pode mudar um byte — os três invariantes do ADR
        // valem verbatim. Medido: **28,87 -> 3,45 ms a 4096² (8,4x)**, 7,74 -> 1,30 a 2048² (6,0x).
        //
        // [ADR-0109]: ../../../../docs/architecture/decisions/0109-rayon-exception-watercolor-composite.md
        let (rw, rh) = (x1 - x0, y1 - y0);
        // A erosão `gap*step*2/255` só alcança 1 quando `gap >= ceil(255/(step*2))`, e `gap <= o`:
        // abaixo deste piso os quatro vizinhos não mudam um byte e não precisam ser lidos. No `step = 1`
        // que o produto anda quase sempre o piso é **128**. Byte-idêntico, não aproximação.
        let erode_floor = 255u32.div_ceil(u32::from(step) * WET_ERODE_GAIN);
        let snap = &mut self.paint.canvas_wet_snapshot;
        snap.resize(rw * rh, 0);
        for oy in 0..rh {
            let src = (y0 + oy) * fw + x0;
            snap[oy * rw..oy * rw + rw].copy_from_slice(&self.paint.canvas_wet[src..src + rw]);
        }
        let old = &self.paint.canvas_wet_snapshot[..rw * rh];
        let band = &mut self.paint.canvas_wet[y0 * fw..(y0 + rh) * fw];
        let fold = |oy: usize, row: &mut [u8]| {
            let mut acc = (0u8, usize::MAX, usize::MAX, 0usize, 0usize);
            for ox in 0..rw {
                let o = old[oy * rw + ox];
                if o == 0 {
                    continue;
                }
                let nv = if u32::from(o) < erode_floor {
                    o.saturating_sub(step)
                } else {
                    let up = if oy > 0 { old[(oy - 1) * rw + ox] } else { 0 };
                    let down = if oy + 1 < rh {
                        old[(oy + 1) * rw + ox]
                    } else {
                        0
                    };
                    let left = if ox > 0 { old[oy * rw + ox - 1] } else { 0 };
                    let right = if ox + 1 < rw {
                        old[oy * rw + ox + 1]
                    } else {
                        0
                    };
                    let gap = o.saturating_sub(up.min(down).min(left).min(right));
                    let erode = ((u32::from(gap) * u32::from(step) * WET_ERODE_GAIN) / 255) as u8;
                    o.saturating_sub(step.saturating_add(erode))
                };
                row[x0 + ox] = nv;
                if nv != 0 {
                    acc = (
                        acc.0.max(nv),
                        acc.1.min(ox),
                        acc.2.min(oy),
                        acc.3.max(ox),
                        acc.4.max(oy),
                    );
                }
            }
            acc
        };
        let join = |a: (u8, usize, usize, usize, usize), b: (u8, usize, usize, usize, usize)| {
            (
                a.0.max(b.0),
                a.1.min(b.1),
                a.2.min(b.2),
                a.3.max(b.3),
                a.4.max(b.4),
            )
        };
        let zero = (0u8, usize::MAX, usize::MAX, 0usize, 0usize);
        // Piso do pool MEDIDO (ver `the_cost_of_the_drying_pass_by_both_routes`): abaixo dele o fork
        // do rayon custa mais que o passe, exatamente como o limiar em BYTES do `plane_copy`.
        let (wettest, bx0, by0, bx1, by1) = if rw * rh >= WET_PAR_MIN {
            use rayon::prelude::*;
            band.par_chunks_mut(fw)
                .enumerate()
                .map(|(oy, row)| fold(oy, row))
                .reduce(|| zero, join)
        } else {
            band.chunks_mut(fw)
                .enumerate()
                .map(|(oy, row)| fold(oy, row))
                .fold(zero, join)
        };
        // O rect segue a poça para dentro (ver o doc do `canvas_wet_rect`); com `wettest == 0` ele é
        // assunto do teardown abaixo, que deixa o mapa zerado no lugar quando há traço aberto.
        if wettest > 0 {
            self.paint.canvas_wet_rect = Some((x0 + bx0, y0 + by0, x0 + bx1 + 1, y0 + by1 + 1));
        }
        // Fully dry = the wet SESSION is over — but the teardown is ATOMIC and deferred past any
        // OPEN stroke: the drying deadline can land mid-stroke (a stroke started near the end of
        // the window), and dropping the session base while the union buffers live on made the
        // pen-up bake fall back to the per-stroke base — which already CONTAINS the union baked —
        // re-rendering it over itself (double-count: the whole wash suddenly darkened hard, Enio
        // smoke 2026-07-09). With a stroke open we leave the zeroed map in place; its own bake
        // re-pours (session extends), or a later idle tick tears everything down together.
        if wettest == 0 && self.paint.stroke.is_none() {
            self.dry_session_now();
        }
    }
}
