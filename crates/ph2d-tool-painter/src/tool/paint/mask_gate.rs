//! **A porta da PROTEÇÃO** — a projeção `lerp(base, free, keep)` da máscara de proteção e da seleção sobre
//! a tela, por lote (`docs/Painter/25_avaliacao_gpu.md` §13.12–§13.13). Partida de `mask.rs` (B52.R13): ali
//! ficam o scratch, o filme e os eventos; aqui a época ([`crate::tool::GateSession`]) e a projeção.

use super::super::Region;
use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;
use std::sync::Arc;

impl PainterTool {
    /// Stamp a batch of dabs through the protection / selection gates, applying the keep factor
    /// **ONCE per texel** — the whole point of [`GateSession`], and the fix for Enio's crackled paint
    /// (doc 25 §13.12).
    ///
    /// The paint goes into the session's FREE plane (swapped into `canvas_rgba` for the stamp — the same
    /// trick the mask sub-brush uses for its scratch, [`Self::stamp_dabs_mask`]), and then the batch's
    /// region of the visible canvas is re-derived from scratch as `lerp(base, free, keep)`. So the gate
    /// never *reads back its own output*, which is exactly what made the old snapshot-and-restore door
    /// compound: it lerped against a per-BATCH snapshot, so the paint surviving `N` pointer events was
    /// `(1−keep)^N` — measured 0,886 at 4 events/stroke against 0,992 at 60, with the paint's contour
    /// moving 4 px on nothing but the mouse's report rate.
    ///
    /// **A single-batch, single-stroke gesture is byte-identical to that old door**, and the argument is one
    /// line: `base` *is* the canvas the old code snapshotted, `free` *is* what the old code stamped, and the
    /// blend is the same expression term for term. It is from the SECOND batch — and the second STROKE —
    /// that the two diverge, and both divergences are the fix (§13.12 and §13.13).
    pub(in crate::tool::paint) fn stamp_dabs_gated(
        &mut self,
        dabs: &[Dab],
        region: Region,
        mask_gate: bool,
        sel_gate: bool,
    ) {
        self.gate_scoped(region, mask_gate, sel_gate, |t| t.stamp_dabs_routed(dabs));
    }

    /// **A dança do gate, com o depósito como PARÂMETRO** — trocar o canvas pelo plano LIVRE, deixar
    /// o depósito pintar sem gate nenhum, trocar de volta e projetar `free·keep + base·(1−keep)`.
    ///
    /// ⚠️ Ela existe como porta porque a sequência é a lei, e a lei tem **dois** depositantes desde a
    /// forma sólida (plano 38 §1.1): os dabs e a região preenchida. Uma segunda cópia das seis linhas
    /// seria a segunda resposta a *"como um depósito atravessa a proteção?"*, e as duas divergiriam no
    /// dia em que uma ganhasse um caso especial — que é exactamente o defeito que o §13.13 pagou.
    pub(in crate::tool::paint) fn gate_scoped(
        &mut self,
        region: Region,
        mask_gate: bool,
        sel_gate: bool,
        deposit: impl FnOnce(&mut Self),
    ) {
        self.begin_or_keep_gate_epoch();
        let Some(mut sess) = self.gate.take() else {
            return; // unreachable (just seeded) — but never stamp ungated by accident
        };
        // Save the free plane under this batch, so a re-stamp preview can undo exactly this batch and no
        // more (the epoch outlives strokes, so resetting the plane to `base` would throw away every
        // earlier stroke's paint — see `Self::restore_region`).
        sess.preview_patch = Some((
            region,
            crate::tool::paint::region::region_pixels(&sess.free, region, self.source_size.0),
        ));
        crate::tool::paint::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut sess.free,
            &self.undo.write_state,
        );
        deposit(self);
        crate::tool::paint::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut sess.free,
            &self.undo.write_state,
        );
        capture_gate_base(
            &mut sess.base,
            &self.canvas_rgba,
            region,
            self.source_size.0,
        );
        self.project_gate_region(&sess, region, mask_gate, sel_gate);
        // The witness is taken AFTER our own writes, so the clock movement this batch caused is absorbed
        // and only somebody ELSE's write can make the next batch see a mismatch.
        sess.witness = self.pixel_clock;
        self.gate = Some(sess);
    }

    /// Open a protection epoch, or keep the live one — the **single** question that replaced §13.7's 22
    /// hand-maintained commit sites: *is the epoch still describing this canvas?*
    ///
    /// Four ways it is not, and each is a bug if projected over: the canvas was **resized**, the artist
    /// switched **layer**, the artist edited the **protection itself** (projecting then would reveal stroke
    /// history already buried under the old protection), or a **foreign writer** touched the pixels (Fill, an
    /// adjustment bake, a Deform, a paste — the class that killed the epoch, because a projection would
    /// silently undo their work).
    fn begin_or_keep_gate_epoch(&mut self) {
        let n4 = self.canvas_rgba.len();
        let layer = self.layers.active();
        let sgen = self.mask_scratch_gen;
        let clock = self.pixel_clock;
        let keep_it = self.gate.as_ref().is_some_and(|g| {
            g.base.pixels.len() == n4 && g.free.len() == n4 && g.layer == layer && g.scratch_gen == sgen
                // `witness == clock` normally; the epoch's FIRST batch has not written yet, and the
                // stroke's own undo snapshot does not move the clock, so nothing here is off by one.
                && g.witness == clock
        });
        if keep_it {
            return;
        }
        // ⭐ B40.A4: the ONE copy of the epoch is `free`; `base` is captured chunk by chunk where batches
        // project, so the canvas keeps one owner and the projection forks nothing.
        #[cfg(test)]
        let base = if caminho_antigo() {
            crate::tool::GateBase {
                pixels: Arc::clone(&self.canvas_rgba),
                have: None,
            }
        } else {
            gate_base_vazia(self.source_size)
        };
        #[cfg(not(test))]
        let base = gate_base_vazia(self.source_size);
        self.gate = Some(crate::tool::GateSession {
            base,
            free: Arc::new(self.canvas_rgba.as_ref().clone()),
            preview_patch: None,
            layer,
            scratch_gen: sgen,
            witness: clock,
        });
    }

    /// Re-derive `region` of the VISIBLE canvas from the session: `free·keep + base·(1−keep)`, where
    /// `keep = protection × selection` (each factor only when its gate is live — the two old restore
    /// doors, run in sequence against one snapshot, composed to exactly this product).
    ///
    /// Nothing is made invisible — only the paint is gated. ⚠️ The semantics is the **layer mask / alpha
    /// lock** one (Photoshop, Krita), not the per-stroke scaling of a Blender sculpt mask: the keep factor
    /// lands on the RESULT, once, so a protection is a ceiling the paint converges on rather than a
    /// discount each stroke gets separately (§13.13 — Enio's call, after the second one was measured
    /// letting everything through in eight passes).
    ///
    /// Writing the batch region alone is complete, not an approximation: `base` is frozen for the epoch,
    /// `keep` changing ENDS the epoch ([`Self::begin_or_keep_gate_epoch`]), and `free` changed only where
    /// this batch stamped.
    fn project_gate_region(
        &mut self,
        sess: &crate::tool::GateSession,
        region: Region,
        mask_gate: bool,
        sel_gate: bool,
    ) {
        let (w, _h) = self.source_size;
        let scratch = mask_gate.then(|| Arc::clone(&self.paint.mask_scratch_rgba));
        let sel = sel_gate.then(|| Arc::clone(&self.paint.selection_mask));
        let base = Arc::clone(&sess.base.pixels);
        let free = Arc::clone(&sess.free);
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            regiao_do_fork(region),
        );
        let n = buf.len() / 4;
        for ry in 0..region.h {
            for rx in 0..region.w {
                let gidx = ((region.y + ry) * w + (region.x + rx)) as usize;
                if gidx >= n {
                    continue;
                }
                // A texel past the end of a gate's buffer is a texel that gate says nothing about, so it
                // keeps the fresh paint — the same stance the old `n = min(len)` clamp took.
                let mut keep = 1.0_f32;
                if let Some(s) = &scratch
                    && gidx < s.len() / 4
                {
                    keep *= crate::compositor::mask_value(s, gidx); // 1 = keep paint, 0 = frozen
                }
                if let Some(m) = &sel
                    && gidx < m.len()
                {
                    keep *= f32::from(m[gidx]) / 255.0; // 1 = inside, 0 = outside
                }
                let b = gidx * 4;
                if keep >= 1.0 {
                    buf[b..b + 4].copy_from_slice(&free[b..b + 4]); // ungated → the free paint IS the display
                    continue;
                }
                for c in 0..4 {
                    let painted = f32::from(free[b + c]);
                    let orig = f32::from(base[b + c]);
                    buf[b + c] = (painted * keep + orig * (1.0 - keep))
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }
        }
        // The projection wrote `region` and nothing else: say so, or the commit sweeps the whole canvas.
        if regiao_do_fork(region).is_some() {
            self.declare_wrote(Some(region));
        }
    }

    /// End the protection session — the stroke is over (pen-up close, shape Apply, a new pen-down, an
    /// undo restore). The next gated batch re-seeds from whatever the canvas then holds, which is the
    /// only base a new stroke could honestly have.
    ///
    /// ⚠️ This is the ONLY thing standing between a per-stroke session and the reverted époque of §13.7:
    /// call it late and protection silently becomes a cross-stroke ceiling that caps the plain brush.
    pub(crate) fn end_gate_session(&mut self) {
        self.gate = None;
    }
}

/// Row-chunk width of the sparse `base` ([`crate::tool::GateBase`]): 64 px = 256 bytes, one page row-slice.
const CHUNK_PX: usize = 64;

fn gate_base_vazia((w, h): (u32, u32)) -> crate::tool::GateBase {
    let (w, h) = (w as usize, h as usize);
    crate::tool::GateBase {
        // `vec![0; n]` is a zeroed allocation: pages fault in only where a chunk is written.
        pixels: Arc::new(vec![0u8; w * h * 4]),
        have: Some(vec![0u8; h * w.div_ceil(CHUNK_PX)]),
    }
}

/// Copy into `base` every chunk of `region` it does not hold yet, from `canvas` — still pristine there,
/// because inside an epoch only the projection writes the canvas, and it writes a chunk only after this.
fn capture_gate_base(base: &mut crate::tool::GateBase, canvas: &[u8], region: Region, w: u32) {
    let Some(have) = base.have.as_mut() else {
        return; // the full-canvas oracle: nothing to capture
    };
    let w = w as usize;
    let chunks = w.div_ceil(CHUNK_PX);
    let pixels = Arc::make_mut(&mut base.pixels);
    let (c0, c1) = (
        region.x as usize / CHUNK_PX,
        ((region.x + region.w) as usize)
            .div_ceil(CHUNK_PX)
            .min(chunks),
    );
    for y in region.y as usize..(region.y + region.h) as usize {
        for c in c0..c1 {
            let k = y * chunks + c;
            if k >= have.len() || have[k] != 0 {
                continue;
            }
            let a = (y * w + c * CHUNK_PX) * 4;
            let b = (y * w + ((c + 1) * CHUNK_PX).min(w)) * 4;
            if b <= canvas.len() && b <= pixels.len() {
                pixels[a..b].copy_from_slice(&canvas[a..b]);
                have[k] = 1;
            }
        }
    }
}

/// The projection's fork window: the batch region (B40.A4) — the whole canvas on the frozen oracle path.
fn regiao_do_fork(region: Region) -> Option<Region> {
    #[cfg(test)]
    if caminho_antigo() {
        return None;
    }
    Some(region)
}

#[cfg(test)]
thread_local! {
    /// `true` = the pre-B40.A4 path verbatim (full `base` Arc, whole-canvas fork) — the parity oracle.
    pub(super) static CAMINHO_ANTIGO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn caminho_antigo() -> bool {
    CAMINHO_ANTIGO.with(std::cell::Cell::get)
}
