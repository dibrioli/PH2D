//! **The Smear as a warp** — the knife's transport, rewritten as an accumulated displacement.
//!
//! The route this replaces lifted pixels from one dab back and lerped them into place, per dab. Over a
//! stroke that is a **product** (`h·wⁿ`) and it decays to nothing everywhere except exactly on the drag
//! axis, so the knife delivered a one-texel needle and no body — Enio, twice: *"as fronteiras não são
//! vencidas. o relevo não é levado além. nada resolvido"*. The measurement, the law and the arithmetic
//! live in [`ph2d_painter_brush::smear_field`]; this module is the plumbing.
//!
//! Three things it is careful about, each of which was a named risk before it was written:
//!
//! 1. **It still rides the one dab list.** The accumulation hangs off `stamp_dabs_inner`'s list exactly
//!    where the colour blend used to, so Symmetry, Tiling, the shape editors, pressure, Jitter, **Shape**
//!    and **Grain** keep reaching the Smear for free. A warp session with geometry of its own inherits
//!    none of that, and *"Tiling doesn't work in Smear"* is how it would be discovered.
//! 2. **The body is not a second implementation.** Colour and relief are resolved from the SAME `disp` by
//!    the same door (`warp/relief.rs`), so the pigment and the thickness physically cannot disagree about
//!    where the paint went. The old `plow_dabs` was a parallel transport with its own chain; it is gone.
//! 3. **The session is per STROKE.** Deform's session spans strokes (Reconstruct needs the history); the
//!    knife has no Apply or Reset to close one, and a stroke's result must become the next stroke's
//!    baseline — the Smear edits the layer in place, as it always has.

use super::{PaintMode, Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushSpec, Dab};
use std::sync::Arc;

impl PainterTool {
    /// Accumulate one batch of Smear dabs into the session displacement, then re-render what moved.
    ///
    /// Returns `false` when there is no session to accumulate into (unsized canvas), so the caller can
    /// fall back rather than silently do nothing.
    pub(super) fn smear_dabs_field(&mut self, dabs: &[Dab], w: u32, h: u32) -> bool {
        if dabs.is_empty() {
            return true;
        }
        // The knife's session opens on the first dab of the stroke and is closed by `close_stroke`.
        // `ensure_warp_session` is idempotent, so later batches in the same stroke reuse it — which is
        // precisely what makes the transport a sum across the whole gesture rather than per batch.
        if !self.ensure_warp_session() {
            return false;
        }
        // The knife's Plow decides how much of the body comes along — through the ONE door, never a
        // second transport. Re-read every batch so the slider stays live within a stroke.
        self.paint.warp.relief_disp_scale =
            self.paint.brush.effective_impasto_plow().clamp(0.0, 1.0);
        let base = self.paint.brush;
        // The Smear's fold has never included Flow (`amount = strength · coverage`), and `walk_dab` folds
        // `coverage × flow × strength`. Neutralising Flow here keeps the knife's response to its sliders
        // exactly what it has always been — turning an inert slider live is not this fix's business.
        let spec_base = BrushSpec { flow: 1.0, ..base };

        // Resolve each dab's frames exactly as the colour route does — same Shape basis, same Grain
        // frame, same order, same RNG discipline (a COPY: this pass must not advance the stream).
        self.ensure_shape_ramp_lut();
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.clone());
        let shape_active = base.shape_silhouette_active(shape_image.is_some());
        let grain_active = base.texture.is_active();
        let groups = self.paint.dab_groups.clone();
        let mut dab_rng = super::tiling::DabRng::new(self.paint.tex_rng);
        // The Selection attenuates each dab AS IT LANDS (never the running total — see
        // `accumulate_dab_sculpt`: attenuating the total compounds once per pointer batch, and a Feather
        // makes that visible).
        let mask: Option<Arc<Vec<u8>>> = self
            .selection_restricts_paint()
            .then(|| Arc::clone(&self.paint.selection_mask));

        let tiling = self.paint.tiling;
        let tiled = tiling[0] || tiling[1];
        let source_size = self.source_size;

        // ⚠️ O transporte do Smear não é um dab do Reshape, então a lista do ADR-0157 não o contém — e a
        // sessão passa a dizer isso em vez de deixar quem re-cozinhar da lista achar que ela basta.
        self.paint.warp.derived = false;
        let mut disp = std::mem::take(Arc::make_mut(&mut self.paint.warp.disp));
        let mut scratch = std::mem::take(&mut self.paint.smear_scratch);
        let mut from = self.paint.last_smear_pos;
        let mut touched: Option<Region> = None;
        for (di, d) in dabs.iter().enumerate() {
            let tex_rng = dab_rng.enter(&groups, di);
            if let Some(prev) = from {
                let spec = BrushSpec {
                    radius_px: d.radius_px,
                    ..spec_base
                };
                let rotor = spec.dab_rotor(d);
                let fp = spec.dab_footprint(rotor);
                let shape_basis = shape_active.then(|| {
                    ph2d_painter_brush::texture::shape_basis(
                        &spec.shape,
                        &mut *tex_rng,
                        [w as f32, h as f32],
                        fp,
                        ph2d_painter_brush::texture::ShapeFrame::Stroke {
                            arc_len: d.arc_len,
                            unit_px: d.stroke_radius_px,
                        },
                    )
                });
                let grain_basis = grain_active.then(|| {
                    ph2d_painter_brush::texture::dab_basis(
                        &spec.texture,
                        &mut *tex_rng,
                        [w as f32, h as f32],
                        fp,
                    )
                });
                // This dab's motion, in canvas px and NOT rounded to whole texels: a displacement is
                // resampled bilinearly, so the integer quantisation the lift-and-blend kernel needed
                // (it indexed source pixels directly) is pure loss here.
                let step = [d.center[0] - prev[0], d.center[1] - prev[1]];
                // Tiling: the wrapped copies each accumulate at their own place, with the same step —
                // the same offsets the colour blend used to walk.
                let mut offs = [[0.0f32; 2]; 9];
                let n = if tiled {
                    super::tiling::tiled_offsets_into(
                        d.center,
                        d.radius_px,
                        source_size,
                        tiling,
                        &mut offs,
                    )
                } else {
                    1
                };
                for &off in &offs[..n] {
                    let hd = ph2d_painter_brush::height::HeightDab {
                        center: [d.center[0] + off[0], d.center[1] + off[1]],
                        radius: d.radius_px,
                        coverage: d.coverage,
                        footprint: fp,
                        // No sweep: like a sculpt dab, a smear dab marks where it IS. The field sums, so
                        // consecutive dabs blend by construction and there is no bead-seam to hide.
                        prev_center: None,
                        shape: shape_basis
                            .as_ref()
                            .map(|sb| ph2d_painter_brush::ShapeInput {
                                basis: sb,
                                image: shape_image.as_ref(),
                                ramp_lut: shape_ramp_lut.as_deref(),
                            }),
                        grain: grain_basis.as_ref(),
                        grain_image: grain_image.as_ref(),
                        // O smear TRANSPORTA o relevo que encontra; ele nao deposita uma forma capturada.
                        relief: None,
                    };
                    if let Some(r) = ph2d_painter_brush::accumulate_dab_smear(
                        ph2d_painter_brush::SmearOut {
                            disp: &mut disp,
                            scratch: &mut scratch,
                        },
                        step,
                        mask.as_ref().map(|m| m.as_slice()),
                        w,
                        h,
                        &spec,
                        &hd,
                    ) {
                        let rect = Region {
                            x: r.x,
                            y: r.y,
                            w: r.w,
                            h: r.h,
                        };
                        touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
                    }
                }
            }
            from = Some(d.center);
        }
        *Arc::make_mut(&mut self.paint.warp.disp) = disp;
        self.paint.smear_scratch = scratch;
        self.paint.last_smear_pos = from;
        self.paint.tex_rng = dab_rng.finish();
        if let Some(rect) = touched {
            // ⚠️ O render é sobre TUDO que a sessão já deslocou, não só o que este batch deslocou. A
            // fonte deixou de ser imutável — a camada Brush do Composite deposita dentro dela para a
            // pilha não perder tinta —, então um texel com `disp` ≠ 0 fora deste batch mostraria o
            // render de uma fonte que não existe mais, e a fronteira entre os dois seria a união dos
            // rects do batch: uma **escada axis-aligned** (Enio 2026-08-09).
            let all = self
                .paint
                .warp
                .touched_all
                .map_or(rect, |acc| union_region(acc, rect));
            self.paint.warp.touched_all = Some(all);
            // One resample of the frozen source over everything that moved — colour and body together.
            self.warp_render_from_session(all);
            // ⚠️ Sujo o que foi RENDERIZADO, não o que foi deslocado neste batch. Marcar só `rect`
            // deixava os texels re-resolvidos fora dele sem subir para a tela — o display ficava com os
            // pixels de uma fonte anterior, e a fronteira era a borda de `rect`. Isso explica a
            // assimetria que o Enio notou (2026-08-09: *"por que só na borda inferior?"*): `all` só
            // EXCEDE `rect` do lado para onde o traço já andou, então o degrau aparece nas pontas `max`
            // — a inferior num traço que desce, a direita num que vai para a direita.
            self.mark_dirty(all);
        }
        true
    }

    /// Fecha a sessão de warp POR-TRAÇO. Chamada de `close_stroke`.
    ///
    /// ⚠️ **A pergunta é sobre a SESSÃO, não sobre o MODO — e a versão antiga era uma ENUMERAÇÃO dos
    /// donos** (`paint_mode.smears()`, isto é *Smear ou Knife*). O doc do próprio `smears()` avisa que
    /// *"uma enumeração desses sítios é exatamente o que apodrece quando um segundo membro entra na
    /// família"*, e a **camada Smear do Composite Brush é o terceiro membro**: ela roda em
    /// `PaintMode::Paint`, então esta guarda dizia não e **a sessão nunca era encerrada**.
    ///
    /// **Medido (2026-08-09, `probe_composite_session_lifetime`), três traços em composite:**
    ///
    /// | | traço 1 | traço 2 | traço 3 |
    /// |---|---|---|---|
    /// | `warp.active` no pen-up | true | true | true |
    /// | texels com `disp` ≠ 0 | 9.904 | 19.808 | 29.712 |
    /// | região re-renderizada (h) | 41 | 81 | 121 |
    ///
    /// Ou seja: a fonte congelada era a do PRIMEIRO pen-down do documento, o campo de deslocamento
    /// somava para sempre, e todo batch re-resolvia **o desenho inteiro** através dele — *a arte não
    /// para de escorregar enquanto o artista pinta*, e o custo cresce com o desenho.
    ///
    /// **O único dono CROSS-traço é o Deform** (a sessão dele atravessa traços porque o Reconstruct
    /// precisa da história, e ele tem Apply/Reset explícitos para encerrá-la). Perguntando isso — em vez
    /// de listar quem abre —, o quarto membro da família nasce coberto.
    ///
    /// **Mutação que must bleed:** voltar a `paint_mode.smears()` ⇒ a sessão sobrevive ao pen-up em
    /// composite (`the_composite_stack_closes_its_smear_session_at_pen_up`).
    pub(super) fn end_smear_session(&mut self) {
        if self.paint.warp.active && self.paint.paint_mode != PaintMode::Deform {
            self.end_warp_session();
        }
    }
}
