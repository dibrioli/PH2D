//! The **Fill** (Bucket) tool's Procreate-style ColorDrop gesture: the drop, the live threshold re-fill
//! from the pre-fill snapshot, the selection clip and Done/Cancel. The pure flood ([`flood_fill`]) and the
//! selection component live in `fill_nucleo.rs` (★B52.R10).

#[path = "fill_nucleo.rs"]
mod nucleo;
#[cfg(test)]
pub(super) use nucleo::flood_fill;
#[path = "fill_rascunho.rs"]
mod rascunho;
pub(super) use rascunho::Rascunho;

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

impl PainterTool {
    /// Route the Fill "Fill adjust" modal's controls (the threshold slider + Done/Cancel). Returns
    /// `true` when handled — mirrors the other `route_*_event` early-dispatch helpers in
    /// [`crate::tool::PainterTool::handle_panel_event`]. The slider re-fills live from the pre-fill
    /// snapshot; Done keeps the fill (one undo entry), Cancel reverts it.
    pub(crate) fn route_fill_event(&mut self, event: &ph2d_editor_core::tool::PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_FILL_MODAL_SLIDER => {
                self.set_fill_threshold(*v as f32);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_FILL_MODAL_DONE => {
                self.fill_commit();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_FILL_MODAL_CANCEL => {
                self.fill_cancel();
                true
            }
            _ => false,
        }
    }

    /// Map the Fill threshold track (`0..1`) to a per-channel colour tolerance (`0..255`).
    fn fill_tolerance(&self) -> u8 {
        (self.paint.fill_threshold.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    /// Run the flood fill from the ORIGINAL (pre-fill) pixels: restore the snapshot into the canvas, then
    /// fill from `fill_seed` at the current threshold with the current brush colour. Shared by the drop
    /// and every live threshold adjustment. No-op without a seed / snapshot.
    pub(super) fn refill_from_snapshot(&mut self) {
        let (Some(seed), size) = (self.paint.fill_seed, self.source_size) else {
            return;
        };
        let (w, h) = (size.0 as usize, size.1 as usize);
        if w == 0 || h == 0 {
            return;
        }
        let sx = (seed[0].floor() as i32).clamp(0, w as i32 - 1) as usize;
        let sy = (seed[1].floor() as i32).clamp(0, h as i32 - 1) as usize;
        let tol = self.fill_tolerance();
        let color = [
            (self.paint.brush.color[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.paint.brush.color[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.paint.brush.color[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ];
        // The ColorDrop clips to the ONE selection region the colour was dropped on — the connected
        // component of the selection mask under the seed (Enio 2026-07-04). `None` without a selection.
        // B48.2: o `visited`, a componente e os planos do corpo circulam no rascunho entre tiques.
        let n = w * h;
        let clip = self.paint.selection_active && self.paint.selection_mask.len() == n;
        // O corpo do Impasto precisa do conjunto preenchido; o digital nem aloca o plano. A pergunta é do
        // PINCEL (`deposits_height`), não do MODO: um balde nunca está em `PaintMode::Paint`.
        let corpo = self.paint.brush.deposits_height();
        let prev = self.paint.fill_last_rect;
        // B17.6: o snapshot É a tela de antes do balde, então o flood corre SÓ DE LEITURA nele e o fork
        // captura só o que este tique escreve — o fill anterior (restaurado) ∪ a caixa do flood.
        let snap_ok = self.paint.fill_snapshot.len() == n * 4;
        let lido = snap_ok.then(|| {
            let visited = self.paint.fill_rascunho.visitado(n, w);
            nucleo::flood_visita_caixa(
                &self.paint.fill_snapshot,
                w,
                h,
                (sx, sy),
                color,
                tol,
                visited,
            )
        });
        let area = match lido {
            Some((_, Some(c))) => Some(prev.map_or(c, |p| super::union_region(p, c))),
            Some((_, None)) if prev.is_none() => return,
            Some((_, None)) => prev,
            None => None,
        };
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            area,
        );
        if buf.len() != n * 4 {
            return;
        }
        // B48.2: o buffer só difere do snapshot dentro do fill anterior — restaurar só essas linhas.
        if snap_ok && let Some(r) = prev {
            copia_caixa(buf, &self.paint.fill_snapshot, r, w);
        }
        let rascunho = &mut self.paint.fill_rascunho;
        let (filled, caixa) = match lido {
            Some((filled, caixa)) => {
                if let Some(c) = caixa {
                    nucleo::pinta_o_conjunto(buf, w, c, rascunho.visitados(), color);
                }
                (filled, caixa)
            }
            None => {
                let visited = rascunho.visitado(n, w);
                nucleo::flood_fill_caixa(buf, w, h, (sx, sy), color, tol, visited)
            }
        };
        rascunho.visitou(caixa);
        let Some(caixa) = caixa else {
            return;
        };
        if clip {
            let mask = std::sync::Arc::clone(&self.paint.selection_mask);
            self.paint.fill_rascunho.componente(&mask, sx, sy, w, h);
        }
        // A cobertura que a SELEÇÃO concede (a mesma porta do Color Fill), só na caixa do flood.
        let keep_mask = clip.then(|| self.fill_selection_keep_na(Some(caixa)));
        // Clip to the drop's selection region — the flood writes `canvas_rgba` directly, bypassing the
        // per-dab stamp gate (ADR-0103). Fora do que o flood MUDOU o pixel é o snapshot e a mistura é a
        // identidade, então só a caixa `filled` é percorrida.
        if let (Some(mask), Some(fr)) = (&keep_mask, filled) {
            let buf = crate::tool::paint::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                self.source_size.0,
                Some(fr),
            );
            let comp = self.paint.fill_rascunho.comp();
            let snap = &self.paint.fill_snapshot;
            let n = n.min(snap.len() / 4);
            for y in fr.y as usize..(fr.y + fr.h) as usize {
                for x in fr.x as usize..(fr.x + fr.w) as usize {
                    let i = y * w + x;
                    if i >= n {
                        continue;
                    }
                    let keep = if comp[i] >= 128 {
                        f32::from(mask.at(x, y, w)) / 255.0
                    } else {
                        0.0
                    };
                    if keep >= 1.0 {
                        continue;
                    }
                    let b = i * 4;
                    for c in 0..4 {
                        let painted = f32::from(buf[b + c]);
                        let orig = f32::from(snap[b + c]);
                        buf[b + c] = (painted * keep + orig * (1.0 - keep))
                            .round()
                            .clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
        // **O CORPO**: o mesmo conjunto que a cor pintou, pela mesma cobertura, re-armado a cada tique (o
        // `reset` antes); o commit é o `close_stroke` do Done. O que o `reset` apaga entra no sujo.
        let mut body = None;
        if corpo {
            body = self.paint.relief.stroke_relief_bbox;
            let planos = self.toma_planos_do_balde();
            self.reset_stroke_height();
            let (cx, cy, cw, ch) = (
                caixa.x as usize,
                caixa.y as usize,
                caixa.w as usize,
                caixa.h as usize,
            );
            let rascunho = &self.paint.fill_rascunho;
            let comp = clip.then(|| rascunho.comp());
            let mut cov = vec![0u8; cw * ch];
            for yy in 0..ch {
                for xx in 0..cw {
                    let (x, y) = (cx + xx, cy + yy);
                    let i = y * w + x;
                    let inside = comp.is_none_or(|c| c[i] >= 128);
                    if !inside || !rascunho.foi_visitado(i) {
                        continue;
                    }
                    // O conjunto do flood vale 255, e `255·k/255 = k`.
                    cov[yy * cw + xx] = keep_mask.as_ref().map_or(255, |keep| keep.at(x, y, w));
                }
            }
            if self.arm_fill_body_na(&cov, caixa, planos) {
                self.guarda_planos_do_balde();
            }
            body = match (body, self.paint.relief.stroke_relief_bbox) {
                (Some(gone), Some(armed)) => Some(super::union_region(gone, armed)),
                (a, b) => a.or(b),
            };
        }
        // The buffer now holds the snapshot everywhere EXCEPT the new fill. Dirty the UNION of the
        // previous fill and the new one so a SHRINKING fill re-uploads the pixels it vacated (restored to
        // the snapshot) — marking only the new, smaller rect would leave the old overflow on screen.
        let colour = match (self.paint.fill_last_rect, filled) {
            (Some(prev), Some(new)) => Some(super::union_region(prev, new)),
            (Some(prev), None) => Some(prev),
            (None, new) => new,
        };
        self.paint.fill_last_rect = filled;
        // ⚠️ **A região suja é o que mudou na TELA, e o corpo é uma mudança na tela.** Ela saía só do que
        // a COR escreveu, e o `flood_fill` devolve `None` quando nenhum pixel de cor mudou — então um
        // balde sobre uma área que JÁ está na cor do pincel armava o relevo e não marcava nada: a tinta
        // ganhava corpo que só aparecia no Done, porque o Done commita e marca. Medido na porta do
        // produto: `dirty=false` no drop e `true` no Done, com o corpo armado nos DOIS. E o caso é o
        // gesto mais natural do impasto — encher de novo o mesmo lugar para engrossar.
        //
        // Com corpo em mãos a bbox dele está DENTRO do que a cor escreveu sempre que a cor escreveu algo
        // (o `cov` é a máscara do flood vezes a cobertura da seleção), então a união é um no-op no caminho
        // comum; e sem corpo (`fill_mask` é `None` num balde digital) `body` fica `None` e isto é o código
        // de antes, ao byte.
        let dirty = match (colour, body) {
            (Some(c), Some(b)) => super::union_region(c, b),
            (Some(c), None) => c,
            (None, Some(b)) => b,
            (None, None) => return,
        };
        self.mark_dirty(dirty);
    }

    /// Route a Fill-mode canvas pointer — the ColorDrop drop gesture: drag the colour onto the canvas
    /// (Down begins + snapshots, Move tracks the cursor) and release (Up) to flood-fill at the drop
    /// point. The threshold is then tuned in the floating Fill modal (the shell drives
    /// [`Self::set_fill_threshold`] / [`Self::fill_commit`] / [`Self::fill_cancel`]).
    pub(super) fn fill_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => {
                // B48.3: um 2.º soltar com o 1.º pendente FECHA o 1.º com o seu passo (como a troca de
                // ferramenta) — senão o «antes» do 2.º seria o 1.º aplicado e o 1.º perdia o undo. A volta
                // do ColorDrop momentâneo fica para quando o último balde fechar.
                if self.has_active_fill() {
                    let volta = self.paint.fill_return_mode.take();
                    self.fill_commit();
                    self.paint.fill_return_mode = volta;
                }
                self.fill_begin_drop(ev.pos);
                true
            }
            PointerPhase::Move => {
                // The fill lands where the drag RELEASES — track the cursor here (fill runs on Up).
                if self.paint.fill_seed.is_some() {
                    self.paint.fill_seed = Some(ev.pos);
                }
                true
            }
            PointerPhase::Up => {
                if self.paint.fill_seed.is_some() {
                    self.paint.fill_seed = Some(ev.pos);
                    self.refill_from_snapshot();
                }
                true
            }
            PointerPhase::Hover => false,
        }
    }

    /// Start a ColorDrop: snapshot the layer (for undo + live re-fill) and record the seed. The fill
    /// itself runs on release ([`Self::fill_pointer`]).
    fn fill_begin_drop(&mut self, pos: [f32; 2]) {
        // One structural-undo entry for the whole drop + modal adjust (pre-fill → committed fill).
        self.begin_undo_step(); // doc 28 §5.26 — o journal passa a descrever ESTE passo
        self.paint.stroke_undo = Some(self.snapshot_model());
        self.paint.fill_snapshot = (*self.canvas_rgba).clone();
        self.paint.fill_seed = Some(pos);
        self.paint.fill_last_rect = None;
    }

    /// Whether a ColorDrop is pending its modal adjust (a drop happened, not yet committed/cancelled).
    #[must_use]
    pub fn has_active_fill(&self) -> bool {
        self.paint.fill_seed.is_some()
    }

    /// The current Fill threshold (`0..1`) — the Fill modal's slider is seeded from this.
    #[must_use]
    pub fn fill_threshold(&self) -> f32 {
        self.paint.fill_threshold
    }

    /// Set the Fill threshold from the modal slider (`0..1`) and re-fill live from the snapshot.
    pub fn set_fill_threshold(&mut self, t: f32) {
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::LimiarDoFill(t), {
            self.paint.fill_threshold = t.clamp(0.0, 1.0);
            self.refill_from_snapshot();
        })
    }

    /// Enter **Fill** for a momentary ColorDrop (the shell's C&F drag) and remember the mode to RESTORE when
    /// the fill finalizes, so the drag returns to the shape/brush the user was using (Enio 2026-07-03). A
    /// deliberate Fill-tool selection uses `set_paint_tool_mode("fill")` instead (no return).
    pub fn begin_colordrop_fill(&mut self, return_mode: &str) {
        let passo = |_: &Self| crate::macro_track::Passo::ColorDropComeca(return_mode.to_owned());
        crate::macro_track::porta!(self, passo, {
            self.set_paint_tool_mode("fill");
            self.paint.fill_return_mode = Some(return_mode.to_string());
        })
    }

    /// Finalize the current fill (modal **Done**, or the dwell live-adjust release): push the undo entry,
    /// drop the transient state. `pub` so the shell's dwell gesture can commit without the modal.
    pub fn fill_commit(&mut self) {
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::FillCommit, {
            if self.paint.fill_seed.is_none() {
                return;
            }
            self.close_stroke(); // pushes the pre-fill → filled structural-undo entry
            self.paint.fill_seed = None;
            self.paint.fill_snapshot = Vec::new();
            self.paint.fill_last_rect = None;
            self.paint.fill_rascunho = Rascunho::default();
            self.restore_after_colordrop();
        })
    }

    /// Restore the pre-ColorDrop mode after a momentary C&F fill finalizes (Done / Cancel / dwell release),
    /// or when a C&F drag never reached the canvas (released off-sprite → the picker path). No-op for a
    /// deliberate Fill / a plain click (no return recorded). `fill_seed` is already cleared, so the
    /// `set_paint_tool_mode` re-entrant `fill_commit` is a no-op (no recursion) and `fill_return_mode` is
    /// taken (drives it exactly once). `pub` so the shell's picker path can end a missed drag.
    pub fn restore_after_colordrop(&mut self) {
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::ColorDropRestaura, {
            if let Some(ret) = self.paint.fill_return_mode.take() {
                self.set_paint_tool_mode(&ret);
            }
        })
    }

    /// Discard the current fill (modal **Cancel**): restore the pre-fill pixels and drop the pending
    /// undo entry, so the layer is exactly as it was before the drop.
    pub fn fill_cancel(&mut self) {
        if self.paint.fill_seed.is_none() {
            return;
        }
        let n = self.paint.fill_snapshot.len();
        // B17.6: a tela só difere do snapshot no último fill (o invariante do B48.2) — restaurar e capturar
        // só essa caixa; sem fill nenhum, nada a restaurar.
        if n > 0
            && self.canvas_rgba.len() == n
            && let Some(r) = self.paint.fill_last_rect
        {
            let buf = crate::tool::paint::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                self.source_size.0,
                Some(r),
            );
            copia_caixa(
                buf,
                &self.paint.fill_snapshot,
                r,
                self.source_size.0 as usize,
            );
        }
        if let Some(rect) = self.paint.fill_last_rect {
            self.mark_dirty(rect); // repaint what the fill had painted
        }
        self.reset_stroke_height(); // o corpo armado morre com a cor que ele acompanhava
        self.paint.stroke_undo = None; // no committed change → no undo entry
        self.paint.fill_seed = None;
        self.paint.fill_snapshot = Vec::new();
        self.paint.fill_last_rect = None;
        self.paint.fill_rascunho = Rascunho::default();
        self.restore_after_colordrop();
    }

    /// Drop a pending Fill ColorDrop WITHOUT touching pixels or dirtying — for a document rebind, where
    /// the canvas is being replaced anyway (so `fill_cancel`'s snapshot-restore would just write into a
    /// buffer about to be discarded, and could stamp the OLD sprite's pixels onto the new one). See
    /// [`PainterTool::reset_transient_edit_state`].
    pub(super) fn abandon_pending_fill(&mut self) {
        self.paint.fill_seed = None;
        self.paint.fill_snapshot = Vec::new();
        self.paint.fill_last_rect = None;
        self.paint.fill_rascunho = Rascunho::default();
        self.paint.stroke_undo = None;
        self.paint.fill_return_mode = None; // drop the momentary-fill return (the doc is being replaced)
    }
}

/// Copia as linhas de `r` de `snap` para `buf` (RGBA8, largura `w`).
fn copia_caixa(buf: &mut [u8], snap: &[u8], r: crate::tool::paint::Region, w: usize) {
    let (x0, x1) = (r.x as usize * 4, (r.x + r.w) as usize * 4);
    for y in r.y as usize..(r.y + r.h) as usize {
        let (a, b) = (y * w * 4 + x0, y * w * 4 + x1);
        if b <= buf.len() && b <= snap.len() {
            buf[a..b].copy_from_slice(&snap[a..b]);
        }
    }
}

#[cfg(test)]
#[path = "fill_gesto_tests.rs"]
mod gesto_tests;

#[cfg(test)]
#[path = "fill_tique_tests.rs"]
mod tique_tests;
