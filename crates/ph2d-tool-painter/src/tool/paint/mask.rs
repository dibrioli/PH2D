//! The **Mask** brush — a live, layerless PROTECTION mask on the current layer (Blender Sculpt-mask
//! style; it does NOT hide anything and is NOT the layer-system visibility mask).
//!
//! The mask brush keeps its own tool-side **scratch** buffer ([`super::PaintState`]'s `mask_scratch_*`),
//! white = unprotected, black = protected, tied to the active raster layer. It never creates a stack
//! layer and nothing is made invisible — the scratch FREEZES the painted pixels against every paint tool.
//! The scratch:
//! - is edited by the sub-brush (Paint = protect / Erase = unprotect / Blur / Smear) — painting swaps the
//!   scratch into `canvas_rgba` so the whole stamp pipeline edits it, then swaps back;
//! - is edited by the whole-canvas **Modifiers** (Expand / Contract / Blur / Sharpen / Invert / Clear);
//! - PROTECTS the painted region LIVE and **never erodes**: every paint op ([`Self::stamp_dabs`]) paints
//!   into the protection's FREE plane and what shows is `lerp(base, free, keep)`
//!   ([`Self::stamp_dabs_gated`]) — applied once per texel, so no number of passes can wear the protection
//!   away (§13.13; it used to let `1 − (1−keep)^N` through, and eight passes erased it). The layer stays
//!   fully visible; [`Self::apply_mask_overlay`] only TINTS the protected area so you can see it;
//! - PERSISTS across tool switches (it stays live while its target layer is active, so you protect a
//!   region in Mask mode, switch to the Brush, and paint freely around the frozen area); it goes dormant
//!   when you switch layers. **Apply** ([`Self::apply_mask_scratch`]) is the (WIP) bridge to the
//!   layer-system mask.

use super::{PainterTool, Region};
use ph2d_det_math::DetMath as _;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::{Dab, MaskCanvasOp};
use std::sync::Arc;

/// Overlay film opacity — how strongly the PROTECTED region reads in the mask colour. High so the frozen
/// area is clearly marked (a solid-ish tint), not a faint wash. Purely a visualisation, not visibility.
const OVERLAY_STRENGTH: f32 = 0.8;

impl PainterTool {
    /// Set the Mask sub-brush: `0` Paint (protect) · `1` Erase (unprotect) · `2` Blur · `3` Smear.
    /// Entering Blur/Smear densifies Spacing to 5% (a sparse chain leaves gaps); leaving them for
    /// Paint/Erase gives the artist's Spacing back (B40.4).
    pub fn set_mask_brush(&mut self, v: u8) {
        let v = v.min(3);
        let denso = |b: u8| b == 2 || b == 3;
        if denso(v) && !denso(self.paint.mask_brush) {
            self.paint.mask_spacing_saved = Some(self.paint.brush.spacing);
            self.paint.brush.spacing = 0.05;
        } else if !denso(v)
            && denso(self.paint.mask_brush)
            && let Some(s) = self.paint.mask_spacing_saved.take()
        {
            self.paint.brush.spacing = s;
        }
        self.paint.mask_brush = v;
    }

    /// The active Mask sub-brush discriminant (`0..=3`) — mirrored into the panel snapshot.
    #[must_use]
    pub fn mask_brush(&self) -> u8 {
        self.paint.mask_brush
    }

    /// Set the on-canvas mask overlay tint colour (`0` dark gray + `1..=4` fluorescent). Invalidates the
    /// composite (when a scratch is active) so the new tint shows at once.
    pub fn set_mask_overlay_color(&mut self, v: u8) {
        self.paint.mask_overlay_color = v.min(4);
        if self.mask_scratch_active() {
            self.invalidate_composite();
        }
    }

    /// The active mask overlay colour index (`0..=4`) — mirrored into the panel snapshot.
    #[must_use]
    pub fn mask_overlay_color(&self) -> u8 {
        self.paint.mask_overlay_color
    }

    /// `true` when a transient scratch mask is live on the CURRENT layer (target still active, non-empty).
    ///
    /// ⚠️ **É também a pergunta *"o overlay da proteção está sendo desenhado?"***, e por isso ela é
    /// `pub`: o tinte por-pixel ([`Self::apply_mask_overlay`]) roda exatamente sob esta condição, e a
    /// SHELL precisa dela para decidir quem produz o preview — o produtor de GPU não desenha o overlay,
    /// então enquanto ele existe a pista de CPU é a única que pode mostrá-lo
    /// (`painter_gpu_preview::gpu_eligible`). Um segundo nome para o mesmo fato é como as duas metades
    /// passam a discordar sobre quando a proteção está na tela.
    #[must_use]
    pub fn mask_scratch_active(&self) -> bool {
        self.paint.mask_scratch_target.is_some()
            && self.paint.mask_scratch_target == self.layers.active()
            && !self.paint.mask_scratch_rgba.is_empty()
    }

    /// Ensure a white (fully-revealed) scratch mask exists for the active RASTER layer, re-targeting (and
    /// discarding any stale scratch) if the active layer changed. No-op if the active layer isn't a raster
    /// or the canvas is empty. Called before a Mask stroke / a Modifier op.
    pub(super) fn ensure_mask_scratch(&mut self) {
        let Some(active) = self.layers.active() else {
            return;
        };
        if !matches!(
            self.layers.get(active).map(|l| &l.kind),
            Some(crate::layers::LayerKind::Raster(_))
        ) {
            return;
        }
        if self.mask_scratch_active() {
            return;
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        self.paint.mask_scratch_rgba = Arc::new(vec![255u8; (w as usize) * (h as usize) * 4]);
        self.bump_mask_scratch_gen();
        self.paint.mask_scratch_target = Some(active);
        self.invalidate_composite();
    }

    /// Snapshot the Mask scratch buffer + target for the undo model — the scratch lives in `PaintState`
    /// (private to this module), so the general `snapshot_model` (in `tool::layers::undo`) reaches it
    /// through here. `Arc`-shared, so the clone is cheap.
    pub(crate) fn mask_scratch_for_snapshot(
        &self,
    ) -> (Arc<Vec<u8>>, Option<crate::layers::LayerId>) {
        (
            Arc::clone(&self.paint.mask_scratch_rgba),
            self.paint.mask_scratch_target,
        )
    }

    /// Reinstate the Mask scratch buffer + target from an undo model (structural undo/redo), keeping the
    /// live mask-in-progress in sync with the restored layers/pixels.
    pub(crate) fn restore_mask_scratch(
        &mut self,
        rgba: Arc<Vec<u8>>,
        target: Option<crate::layers::LayerId>,
    ) {
        self.paint.mask_scratch_rgba = rgba;
        self.bump_mask_scratch_gen();
        self.paint.mask_scratch_target = target;
    }

    /// **Apply** — promote the transient scratch to a real **layer-system mask** on the current layer
    /// (it appears in the Layers panel with eye / Invert and is editable by any tool), then clear the
    /// scratch. The parent raster stays the active edit layer (you were painting the image, not the
    /// mask) and its pixels are untouched. If the layer already has a mask, the scratch is multiplied
    /// INTO it (coverage refine, `α_new = α_old × scratch`). One structural undo step. No-op without a
    /// live scratch, or at the layer hard-cap (the scratch is then left intact). The Apply button.
    pub fn apply_mask_scratch(&mut self) {
        if !self.mask_scratch_active() {
            return;
        }
        let Some(target) = self.layers.active() else {
            return;
        };
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.mask_scratch_rgba.len() < n * 4 {
            return;
        }
        let before = self.abre_edicao_estrutural();
        let scratch = self.paint.mask_scratch_rgba.as_ref().clone();
        // Store the scratch as the mask's pixels. The mask is owner-attached (NOT the active layer), so
        // its pixels live in `images[mask_id]` — exactly where the compositor reads a mask from
        // (`ToolPixelSource::layer_rgba`). The target raster stays active and its pixels are untouched.
        let touched = match self.layers.add_mask_for(target, w, h) {
            Some(mask_id) => {
                self.images.insert(
                    mask_id,
                    Arc::new(crate::compositor::LayerImage {
                        width: w,
                        height: h,
                        rgba8: Arc::new(scratch),
                    }),
                );
                Some(mask_id)
            }
            // `add_mask_for` rejected the add: either the layer already HAS a mask (merge the scratch
            // into it by multiplying coverage) or the hard-cap is hit (leave the scratch alone and bail).
            None => {
                let existing = self.layers.get(target).and_then(|l| l.mask);
                if let Some(mask_id) = existing
                    && let Some(img) = self.images.get_mut(&mask_id).map(std::sync::Arc::make_mut)
                    && img.rgba8.len() >= n * 4
                {
                    for i in 0..n {
                        let m = crate::compositor::mask_value(&scratch, i);
                        for c in 0..3 {
                            let base = f32::from(img.rgba8[i * 4 + c]);
                            img.rgba8_mut()[i * 4 + c] = (base * m).round().clamp(0.0, 255.0) as u8;
                        }
                    }
                }
                existing
            }
        };
        let Some(touched) = touched else {
            return; // hard-cap: no mask created/found — keep the live scratch, record no undo step.
        };
        self.paint.mask_scratch_target = None;
        self.paint.mask_scratch_rgba = Arc::new(Vec::new());
        self.bump_mask_scratch_gen();
        self.bump_layer_pixels(Some(touched));
        self.commit_structural_edit(before);
        self.invalidate_composite();
    }

    /// Whether a live protection scratch must FREEZE paint on the active layer for the current op: a
    /// scratch is live AND we are not in Mask mode (Mask mode edits the scratch itself, never gated).
    #[must_use]
    pub(super) fn mask_protection_active(&self) -> bool {
        self.mask_scratch_active() && !matches!(self.paint.paint_mode, super::PaintMode::Mask)
    }

    /// The canvas region a dab batch can touch — the union of each dab's reach ([`caixa_do_dab`]: the disc, and
    /// the capsule back to the previous centre) plus the previous Smear/Clone position, clamped to the canvas;
    /// `None` if empty / off-canvas. With
    /// **Tiling** on an axis a border dab wraps to the far edge, so that axis spans the full extent. Used
    /// to snapshot + restore ONLY the footprint for the protection gate (never the whole canvas).
    pub(super) fn dab_batch_region(&self, dabs: &[Dab]) -> Option<Region> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || dabs.is_empty() {
            return None;
        }
        let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        let mut max_r = 0.0_f32;
        // O alcance da ponta de imagem do Shape (Bug #52) — `1` sem ela, ao bit.
        let reach = self.paint.brush.tip_reach();
        for d in dabs {
            let (lo, hi, r) = caixa_do_dab(d, reach);
            max_r = max_r.max(r);
            minx = minx.min(lo[0]);
            maxx = maxx.max(hi[0]);
            miny = miny.min(lo[1]);
            maxy = maxy.max(hi[1]);
        }
        if let Some((p, _)) = self.paint.last_smear_pos {
            minx = minx.min(p[0] - max_r);
            maxx = maxx.max(p[0] + max_r);
            miny = miny.min(p[1] - max_r);
            maxy = maxy.max(p[1] + max_r);
        }
        if self.paint.tiling[0] {
            (minx, maxx) = (0.0, w as f32);
        }
        if self.paint.tiling[1] {
            (miny, maxy) = (0.0, h as f32);
        }
        let x0 = minx.floor().clamp(0.0, w as f32) as u32;
        let y0 = miny.floor().clamp(0.0, h as f32) as u32;
        let x1 = maxx.ceil().clamp(0.0, w as f32) as u32;
        let y1 = maxy.ceil().clamp(0.0, h as f32) as u32;
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        Some(Region {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        })
    }

    /// Bump the mask scratch's content generation — called by **every** write to it. Editing the protection
    /// ends the epoch, and this is how the epoch finds out.
    pub(super) fn bump_mask_scratch_gen(&mut self) {
        self.mask_scratch_gen = self.mask_scratch_gen.wrapping_add(1);
    }

    /// Apply a whole-canvas Modifier (Expand / Contract / Blur / Sharpen / Invert / Clear) to the scratch
    /// mask (creating it first). No-op if the active layer can't hold a scratch or the canvas is empty.
    pub fn mask_canvas_op(&mut self, op: u8) {
        let Some(mop) = mask_op_from_u8(op) else {
            return;
        };
        self.ensure_mask_scratch();
        if !self.mask_scratch_active() {
            return;
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        // One structural undo step (pre-op scratch → post-op), so a canvas Modifier rolls back like a
        // mask stroke does.
        let before = self.abre_edicao_estrutural();
        let radius = self.raio_do_pincel();
        {
            let buf = Arc::make_mut(&mut self.paint.mask_scratch_rgba);
            ph2d_painter_brush::apply_mask_op(buf, w, h, mop, radius);
        }
        {
            self.bump_mask_scratch_gen();
        }
        self.invalidate_composite();
        self.commit_structural_edit(before);
    }

    /// Blend the mask overlay film over the composited RGBA `buf`: where the scratch PROTECTS, tint by the
    /// selected colour at [`OVERLAY_STRENGTH`] (straight-alpha src-over, raising alpha). Purely a marker of
    /// the frozen area — it never hides the layer. No-op without a live scratch. Called AFTER the compose.
    pub(crate) fn apply_mask_overlay(&self, buf: &mut [u8]) {
        if !self.mask_scratch_active() {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if buf.len() < n * 4 || self.paint.mask_scratch_rgba.len() < n * 4 {
            return;
        }
        let film = mask_overlay_rgb(self.paint.mask_overlay_color);
        let scratch = &self.paint.mask_scratch_rgba;
        for i in 0..n {
            let cov = crate::compositor::mask_value(scratch, i);
            tint_pixel(&mut buf[i * 4..i * 4 + 4], film, cov);
        }
    }

    /// **A versão do filme da máscara** (W8 item 6b, `docs/Macros/05_plano_W8.md`): muda a cada escrita
    /// na máscara e a cada cor do filme; `None` = sem máscara viva (nada a tingir).
    #[must_use]
    pub fn versao_da_mascara(&self) -> Option<u64> {
        self.mask_scratch_active()
            .then(|| self.mask_scratch_gen << 8 | u64::from(self.paint.mask_overlay_color))
    }

    /// **O filme da máscara sobre a região `caixa` = (x, y, w, h) de uma imagem já composta** (`rgba`
    /// straight, do tamanho da caixa) — a mesma tinta que a pré-visualização do editor põe por cima do
    /// composto. Para quem mostra a imagem por outra porta (o palco de uma macro). Sem máscara viva, nada.
    pub fn tinge_com_a_mascara(&self, rgba: &mut [u8], (x, y, w, h): (u32, u32, u32, u32)) {
        self.apply_mask_overlay_region(rgba, Region { x, y, w, h });
    }

    /// The overlay applied to ONE re-composited sub-region — the partial fast lane's twin of
    /// [`Self::apply_mask_overlay`]. `buf` is a `bbox`-sized region buffer (row-major within `bbox`);
    /// the scratch is full-canvas, so each region pixel reads its GLOBAL coverage. The tint is
    /// **per-pixel** (a texel's tint depends only on its own coverage and colour — no global term), so
    /// re-tinting only the dab's `bbox` is byte-identical to a full re-tint there. The genuinely global
    /// overlay changes (colour swatch, canvas-op, the first scratch) all `invalidate_composite()`, which
    /// forces the full arm; the only non-invalidating change is a per-dab coverage edit, and that lives
    /// inside `bbox`. Shares [`tint_pixel`] with the full path so the two cannot disagree on a texel.
    pub(crate) fn apply_mask_overlay_region(&self, buf: &mut [u8], bbox: Region) {
        if !self.mask_scratch_active() {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.mask_scratch_rgba.len() < n * 4 {
            return;
        }
        let film = mask_overlay_rgb(self.paint.mask_overlay_color);
        let scratch = &self.paint.mask_scratch_rgba;
        for ry in 0..bbox.h {
            for rx in 0..bbox.w {
                let gidx = ((bbox.y + ry) * w + (bbox.x + rx)) as usize;
                let b = ((ry * bbox.w + rx) * 4) as usize;
                if gidx >= n || b + 4 > buf.len() {
                    continue;
                }
                let cov = crate::compositor::mask_value(scratch, gidx);
                tint_pixel(&mut buf[b..b + 4], film, cov);
            }
        }
    }

    /// Route the Mask-section panel Clicks: sub-brush segments, canvas-op buttons, overlay-colour
    /// swatches, and Apply. Returns `true` iff consumed. Called from `route_brush_dab_event`.
    pub(crate) fn route_mask_event(&mut self, event: &PanelEvent) -> bool {
        let PanelEvent::Click(id) = event else {
            return false;
        };
        if *id == crate::ids::PAINTER_MASK_APPLY {
            self.apply_mask_scratch();
            return true;
        }
        if let Some(i) = crate::ids::PAINTER_MASK_BRUSH.iter().position(|x| x == id) {
            self.set_mask_brush(i as u8);
            return true;
        }
        if let Some(i) = crate::ids::PAINTER_MASK_OP.iter().position(|x| x == id) {
            self.mask_canvas_op(i as u8);
            return true;
        }
        if let Some(i) = crate::ids::PAINTER_MASK_COLOR.iter().position(|x| x == id) {
            self.set_mask_overlay_color(i as u8);
            return true;
        }
        false
    }
}

/// Tint one composited RGBA pixel (`px` = a 4-byte slice) by the protection coverage `cov`: straight-
/// alpha src-over of the overlay `film` at `(1 - cov) * OVERLAY_STRENGTH`, raising alpha. The single
/// per-pixel kernel BOTH [`PainterTool::apply_mask_overlay`] (whole canvas) and
/// [`PainterTool::apply_mask_overlay_region`] (partial lane) call — two copies of this math would let
/// the full and partial recomposite paths disagree on a texel, which is exactly the class of bug the
/// partial lane must never introduce.
fn tint_pixel(px: &mut [u8], film: [u8; 3], cov: f32) {
    let sa = (1.0 - cov) * OVERLAY_STRENGTH;
    if sa <= 0.0 {
        return;
    }
    let da = f32::from(px[3]) / 255.0;
    let oa = sa + da * (1.0 - sa);
    if oa <= 0.0 {
        return;
    }
    for c in 0..3 {
        let s = f32::from(film[c]);
        let d = f32::from(px[c]);
        px[c] = ((s * sa + d * da * (1.0 - sa)) / oa)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    px[3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
}

/// Map a wire discriminant to the engine op (`None` for an out-of-range index).
fn mask_op_from_u8(op: u8) -> Option<MaskCanvasOp> {
    Some(match op {
        0 => MaskCanvasOp::Expand,
        1 => MaskCanvasOp::Contract,
        2 => MaskCanvasOp::Blur,
        3 => MaskCanvasOp::Sharpen,
        4 => MaskCanvasOp::Invert,
        5 => MaskCanvasOp::Clear,
        _ => return None,
    })
}

// The mask's own measurement probes + gates live in sibling files rather than at the end of the
// 23k-line `paint::tests` (a wave's worth of gates appended there is a wave nobody can find again).
// They hang off THIS module because `paint.rs` sits exactly at the 700-LOC cap.
#[path = "mask_gate.rs"]
mod mask_gate;
#[cfg(test)]
#[path = "mask_gate_copia_tests.rs"]
mod mask_gate_copia_tests;
#[cfg(test)]
#[path = "mask_gate_tests.rs"]
mod mask_gate_tests;
#[cfg(test)]
#[path = "mask_probe.rs"]
pub(super) mod mask_probe; // a fixture de mascara serve tambem os gates do journal (doc 28 §5.23)
#[cfg(test)]
#[path = "mask_probe_gate.rs"]
mod mask_probe_gate;
#[path = "mask_shape.rs"]
pub(super) mod mask_shape;
#[cfg(test)]
#[path = "mask_shape_tests.rs"]
mod mask_shape_tests;
#[cfg(test)]
#[path = "mask_tests.rs"]
mod mask_tests;
#[cfg(test)]
#[path = "prancha_b40b_tests.rs"]
mod prancha_b40b_tests;

/// The overlay tint palette (straight sRGB `0..=255`): index `0` a DARK gray (default), `1..=4`
/// fluorescent highlighter hues (yellow / pink / green / orange). Out-of-range falls back to dark gray.
fn mask_overlay_rgb(idx: u8) -> [u8; 3] {
    match idx {
        1 => [220, 255, 0],  // fluorescent yellow
        2 => [255, 42, 160], // fluorescent pink
        3 => [80, 255, 60],  // fluorescent green
        4 => [255, 120, 0],  // fluorescent orange
        _ => [51, 51, 51],   // dark gray (default)
    }
}

// O gate de DEPÓSITO — a máscara alcançando o RELEVO (`height::DepositGate`). Irmão dos dois acima, e
// arquivo próprio pelo mesmo motivo: o `mask_gate_tests` já está em 649 das 700 linhas.
#[cfg(test)]
#[path = "mask_relief_tests.rs"]
mod mask_relief_tests;

/// ⭐ **O que um dab pode tocar**: o disco (`center ± radius·reach`, +1 px) e, com o passo da cápsula (Continuous,
/// [`Dab::capsula`]), o caminho de volta ao centro anterior ao raio do dab de trás, com o Jitter que cada sub-dab
/// re-sorteia. Sem a cápsula a região de um lote parava no disco da frente: o plano do traço compunha só ela e a
/// faixa de trás de cada passo nunca chegava à tela (dono 25/10: o traço com a cauda «em degraus», o início em
/// falta). Devolve `(mín, máx, maior raio)`.
pub(super) fn caixa_do_dab(d: &Dab, reach: f32) -> ([f32; 2], [f32; 2], f32) {
    let r = d.radius_px * reach + 1.0;
    let (mut lo, mut hi) = (
        [d.center[0] - r, d.center[1] - r],
        [d.center[0] + r, d.center[1] + r],
    );
    if d.step == [0.0, 0.0] {
        return (lo, hi, r);
    }
    let atras = d.ante.map_or(d.radius_px, |a| a.radius_px).max(d.radius_px);
    let k = if d.radius_px > 1e-6 {
        atras / d.radius_px
    } else {
        1.0
    };
    let salto = d.ante.and_then(|a| a.salto).map_or(0.0, |j| {
        j.desvio[0].det_hypot(j.desvio[1])
            + j.alcance * if j.por_raio { k } else { 1.0 }
            + j.atraves * k
    });
    let rb = atras * reach + 1.0 + salto;
    let tras = [d.center[0] - d.step[0], d.center[1] - d.step[1]];
    for c in [d.center, tras] {
        lo = [lo[0].min(c[0] - rb), lo[1].min(c[1] - rb)];
        hi = [hi[0].max(c[0] + rb), hi[1].max(c[1] + rb)];
    }
    (lo, hi, r.max(rb))
}
