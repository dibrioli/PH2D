//! Pixel-region helpers for the interactive drag-preview: the touchable dab bbox + save/restore of a
//! `canvas_rgba` rectangle. Split from `paint.rs` for the workspace file-LOC cap (a child module, so it
//! keeps access to `PaintState`'s private fields). Used by `paint::stamp_drag_preview`.

use super::Region;
use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;
use std::sync::Arc;

impl PainterTool {
    /// Pixel bbox a dab of `radius` at `center` can touch — the drag-preview **save/restore + dirty-upload**
    /// region; MUST superset the blit/accumulate write bounds (`floor(c−r) .. ceil(c+r)+1`). A
    /// `round(c)±(ceil(r)+1)` box misses the high edge 1px for `frac(c) < 0.5` → stale un-restored edge row
    /// (thin horizontal trails). Enio 2026-06-27.
    pub(super) fn dab_bbox(&self, center: [f32; 2], radius: f32) -> Option<Region> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        let x0 = (center[0] - radius).floor().max(0.0) as i64;
        let y0 = (center[1] - radius).floor().max(0.0) as i64;
        let x1 = ((center[0] + radius).ceil() as i64 + 1).min(w as i64);
        let y1 = ((center[1] + radius).ceil() as i64 + 1).min(h as i64);
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        Some(Region {
            x: x0 as u32,
            y: y0 as u32,
            w: (x1 - x0) as u32,
            h: (y1 - y0) as u32,
        })
    }

    /// Copy the RGBA8 pixels of `rect` out of `canvas_rgba` (row-major over the region).
    pub(super) fn save_region(&self, rect: &Region) -> Vec<u8> {
        let stride = self.source_size.0 as usize * 4;
        let rw = rect.w as usize * 4;
        let mut out = Vec::with_capacity(rw * rect.h as usize);
        for row in 0..rect.h {
            let start = (rect.y + row) as usize * stride + rect.x as usize * 4;
            out.extend_from_slice(&self.canvas_rgba[start..start + rw]);
        }
        out
    }

    /// Write `pixels` (from [`Self::save_region`]) back into `rect` and flag it dirty.
    ///
    /// **This is also where a live protection session's FREE plane is put back**, and it belongs here
    /// rather than at the five call sites for the reason the repo keeps re-learning: a rule spelled out
    /// once per caller is a rule the sixth caller is born without. Every caller of this function is
    /// undoing a preview — putting the canvas back to what it was before the last re-stamp — and the free
    /// plane's "before" is the session base. Leave it un-restored and each preview frame's paint stays in
    /// `free` forever, so the projection keeps showing a shape the artist already dragged away from: the
    /// fan of ghosts that `reset_stroke_height` (one line above the first caller) exists to prevent in the
    /// relief channel, one plane over.
    ///
    /// The reset spans the session's OWN recorded rect, not `rect`: the two are the same in practice but
    /// they are computed by different functions (`dab_bbox` fold vs `dab_batch_region`), and "in practice"
    /// is how a 1-texel sliver of stale paint survives to be reported as a faint trail.
    pub(super) fn restore_region(&mut self, rect: &Region, pixels: &[u8]) {
        let stride = self.source_size.0 as usize * 4;
        let rw = rect.w as usize * 4;
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            None,
        );
        for row in 0..rect.h {
            let dst = (rect.y + row) as usize * stride + rect.x as usize * 4;
            let src = row as usize * rw;
            buf[dst..dst + rw].copy_from_slice(&pixels[src..src + rw]);
        }
        self.restore_gate_free();
        self.mark_dirty(*rect);
        // ⚠️ And re-witness: `mark_dirty` moved the pixel clock, and this write is the EPOCH'S OWN — the
        // undo of its last batch. Without this the foreign-write witness fires on our own hand, so every
        // preview frame of a re-stamp method re-seeded the epoch: the ceiling silently reverted to
        // per-gesture for every shape editor, and each frame paid a full canvas clone. Found by a
        // surviving mutation (the free plane could be restored from `base` with no observable effect,
        // because the plane was being thrown away and rebuilt anyway).
        if let Some(sess) = self.gate.as_mut() {
            sess.witness = self.pixel_clock;
        }
    }

    /// Undo the last gated batch's write to the protection epoch's free plane, from the patch that batch
    /// saved. No-op without a live epoch (the overwhelmingly common case: one `is_none` test).
    ///
    /// ⚠️ It restores a **patch**, not `base`. The epoch outlives strokes ([`crate::tool::GateSession`]), so
    /// every earlier stroke's paint is still owed by the free plane — resetting it to `base` would delete
    /// the whole session's work the first time a shape editor re-stamped.
    fn restore_gate_free(&mut self) {
        let stride = self.source_size.0 as usize * 4;
        let Some(sess) = self.gate.as_mut() else {
            return;
        };
        let Some((rect, pixels)) = sess.preview_patch.take() else {
            return;
        };
        let free = Arc::make_mut(&mut sess.free);
        let rw = rect.w as usize * 4;
        for row in 0..rect.h {
            let at = (rect.y + row) as usize * stride + rect.x as usize * 4;
            let src = row as usize * rw;
            if at + rw <= free.len() && src + rw <= pixels.len() {
                free[at..at + rw].copy_from_slice(&pixels[src..src + rw]);
            }
        }
    }
}

/// Copy `region`'s RGBA8 out of a canvas-sized `buf` (row-major over the region), tolerating a short
/// buffer by leaving those rows zero. The free-plane sibling of [`PainterTool::save_region`], which reads
/// `canvas_rgba` and cannot be pointed at another plane.
pub(super) fn region_pixels(buf: &[u8], region: Region, canvas_w: u32) -> Vec<u8> {
    let row = region.w as usize * 4;
    let mut out = vec![0u8; row * region.h as usize];
    for ry in 0..region.h {
        let src = ((region.y + ry) as usize * canvas_w as usize + region.x as usize) * 4;
        let dst = ry as usize * row;
        if src + row <= buf.len() {
            out[dst..dst + row].copy_from_slice(&buf[src..src + row]);
        }
    }
    out
}

/// Smallest region covering both `a` and `b` — the routes' dirty-rect fold (every stamp route
/// imports it as `super::union_region`; moved here from `paint.rs` for the workspace file-LOC cap).
pub(super) fn union_region(a: Region, b: Region) -> Region {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.w).max(b.x + b.w);
    let y1 = (a.y + a.h).max(b.y + b.h);
    Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

/// Grow `r` by `pad` texels on every side, clipped to a `w × h` canvas.
///
/// The Impasto commit uses it to turn a stroke's dab footprint into the window the settle needs: the
/// blur reaches `SETTLE_MAX_PX` beyond the paint, and outside that window the relief is exactly zero —
/// which is what makes cropping the commit **byte-identical** to running it over the whole canvas
/// (`docs/Painter/16_impasto_plano_implementacao.md` §10.11).
pub(super) fn grow_region(r: Region, pad: u32, w: u32, h: u32) -> Option<Region> {
    if w == 0 || h == 0 {
        return None;
    }
    let x0 = r.x.saturating_sub(pad);
    let y0 = r.y.saturating_sub(pad);
    let x1 = (r.x + r.w + pad).min(w);
    let y1 = (r.y + r.h + pad).min(h);
    (x1 > x0 && y1 > y0).then_some(Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
}

// **A região que uma lista de dabs vai escrever, respondida ANTES do laço** — o que faltava para a
// porta do canvas capturar uma região em vez do plano inteiro (doc 28 §7).
//
// # O problema que ela resolve
//
// O journal de undo captura os bytes velhos **antes** da escrita. As rotas de depósito conhecem a
// região que tocaram só **depois** (elas acumulam o `touched` *enquanto* carimbam, a partir do
// `DirtyRect` que cada blit devolve), então a porta do canvas era chamada com `None` e o journal
// guardava o plano inteiro: **67,11 MB a 4096²**, exatamente `n × 4`. Com esse número a troca do S3
// sai *lateral* — um fork de 67 MB por uma captura de 67 MB.
//
// A saída não é adivinhar: a footprint de um dab é **função pura** do centro e do raio, e
// [`ph2d_painter_brush::dab_write_bounds`] a responde como o **superconjunto** que as duas rotas de
// blit honram (gate `the_write_bounds_door_contains_what_both_blit_routes_touch`). Somar as
// footprints antes do laço custa uma passada sobre a lista de dabs — dezenas de itens — e devolve uma
// região que o journal pode reter.
//
// # ⚠️ A premissa que torna isto correto, e onde ela pode quebrar
//
// **A lista de dabs que chega às rotas de depósito é a FINAL.** O Tiling replica cada dab que cruza a
// borda numa cópia deslocada (`super::tiling::tiled_dabs`) e a Symmetry espelha — as duas **expandem a
// lista**, então as cópias estão nela e a união das footprints as cobre. Se algum dia uma rota passar
// a fazer o wrap *dentro* do blit (em vez de na lista), esta função passaria a devolver um
// **subconjunto**, que é a direção que perde texels em silêncio.
//
// Por isso o gate irmão no tool carimba um traço com Tiling ligado e afirma que o `touched` real cabe
// no que esta função previu: ele falha no dia em que a premissa mudar, em vez de o undo passar a
// esquecer a borda oposta.
/// A união das footprints de `dabs`, clampada ao canvas — ou `None` se nenhum deles alcança a tela.
///
/// **Superconjunto por construção**: usa a footprint máxima de cada dab (a porta do crate do pincel,
/// que inclui o pad de AA da rota per-pixel) e ignora cobertura/blend, então um dab que acaba não
/// escrevendo nada só amplia a região. Amplia é seguro; encolher não é.
#[must_use]
pub(super) fn dabs_bounds(dabs: &[Dab], w: u32, h: u32) -> Option<Region> {
    let mut acc: Option<Region> = None;
    for d in dabs {
        let Some(r) = ph2d_painter_brush::dab_write_bounds(d.center, d.radius_px, w, h) else {
            continue;
        };
        let r = Region {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
        };
        acc = Some(acc.map_or(r, |a| union_region(a, r)));
    }
    acc
}

#[cfg(test)]
mod dab_bounds_tests {
    use super::dabs_bounds;

    #[test]
    fn an_empty_dab_list_writes_nowhere() {
        assert_eq!(dabs_bounds(&[], 256, 256), None);
    }
}
