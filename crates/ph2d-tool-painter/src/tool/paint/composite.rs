//! **Composite Brush** — run Brush + Smear + Blur together as a reorderable 3-layer stack.
//!
//! An upgrade to the Brush tool (a panel checkbox, not a rail tool): when on, one stroke applies all
//! three operations per dab, each with its own Strength. The three layers occupy FIXED positions
//! numbered 1 (top) · 2 · 3 (bottom); the tool at each position is reordered with the panel's up/down
//! buttons. The stroke runs the stack **bottom → top** (position 3 first), so each operation processes
//! the canvas as modified by the one below it — e.g. Brush(3) → Smear(2) → Blur(1) paints, then smears
//! that, then blurs the result; Blur(3) → Smear(2) → Brush(1) blurs the canvas, smears it, then paints
//! clean strokes on top (untouched by the blur/smear below). Split from `paint.rs` for the LOC cap.
//!
//! Only the Brush operation paints colour; the shared brush parameters (Size / Shape / Grain / Falloff
//! / Tiling / Jitter / Symmetry / Stroke) drive all three ops' dab geometry, while the colour-family
//! parameters (Color / Blend / ramps / Randomize / Accumulate) only affect the Brush layer — so the
//! panel keeps every control visible in composite mode (see `BrushSettings::paints_no_color`).

use super::PaintMode;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::Dab;

/// One of the three composite operations. Wire discriminant (`to_u8`) travels in the panel snapshot so
/// the panel can label each row; the panel maps it back to a name.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum CompositeOp {
    Brush,
    Smear,
    Blur,
}

impl CompositeOp {
    /// Wire discriminant for the panel snapshot (`0` Brush · `1` Smear · `2` Blur).
    pub(crate) fn to_u8(self) -> u8 {
        match self {
            Self::Brush => 0,
            Self::Smear => 1,
            Self::Blur => 2,
        }
    }
}

/// One composite stack layer: which operation sits here + its Strength (`0..1`; `0` = the layer is a
/// no-op and is skipped). Stored in a fixed `[_; 3]` in display order (index 0 = layer 1 = top).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct CompositeLayer {
    pub op: CompositeOp,
    pub strength: f32,
}

impl PainterTool {
    /// Whether the composite stack drives this stroke: the checkbox is on AND the active operation is
    /// the plain Brush (not a Smear/Blur/Eraser rail tool — composite is a Brush-tool upgrade).
    pub(crate) fn composite_active(&self) -> bool {
        self.paint.composite_enabled
            && matches!(self.paint.paint_mode, PaintMode::Paint)
            && !self.paint.eraser
    }

    /// Toggle the Composite Brush on/off (panel checkbox). Plain state — touches no pixels.
    pub fn toggle_composite(&mut self) {
        self.paint.composite_enabled = !self.paint.composite_enabled;
    }

    /// The Composite Brush enable flag (the panel snapshot mirrors it to show the card + hide Strength).
    #[must_use]
    pub fn composite_enabled(&self) -> bool {
        self.paint.composite_enabled
    }

    /// Set the Strength (`0..1`) of the composite layer at `pos` (`0` = layer 1 … `2` = layer 3).
    pub fn set_composite_layer_strength(&mut self, pos: usize, t: f32) {
        if pos < 3 {
            self.paint.composite[pos].strength = t.clamp(0.0, 1.0);
        }
    }

    /// Move the layer at `pos` one position UP (toward layer 1 / top) — swaps the tool with its upper
    /// neighbour. The position NUMBERS stay fixed; only which tool sits where changes. No-op at the top.
    pub fn move_composite_layer_up(&mut self, pos: usize) {
        if (1..3).contains(&pos) {
            self.paint.composite.swap(pos, pos - 1);
        }
    }

    /// Move the layer at `pos` one position DOWN (toward layer 3 / bottom). No-op at the bottom.
    pub fn move_composite_layer_down(&mut self, pos: usize) {
        if pos < 2 {
            self.paint.composite.swap(pos, pos + 1);
        }
    }

    /// The per-position operation discriminants `[layer1, layer2, layer3]` — for the panel snapshot.
    pub(crate) fn composite_ops_u8(&self) -> [u8; 3] {
        [
            self.paint.composite[0].op.to_u8(),
            self.paint.composite[1].op.to_u8(),
            self.paint.composite[2].op.to_u8(),
        ]
    }

    /// The per-position layer Strengths `[layer1, layer2, layer3]` — for the panel snapshot.
    pub(crate) fn composite_strengths(&self) -> [f32; 3] {
        [
            self.paint.composite[0].strength,
            self.paint.composite[1].strength,
            self.paint.composite[2].strength,
        ]
    }

    /// Route the Composite-card panel events (enable checkbox, per-position reorder buttons, per-position
    /// Strength sliders). Returns `true` iff consumed — chained ahead of the big `handle_panel_event` match.
    pub(crate) fn route_composite_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        match event {
            PanelEvent::Click(id) => {
                if *id == core_ids::PAINTER_BRUSH_COMPOSITE_ENABLE {
                    self.toggle_composite();
                    return true;
                }
                if let Some(p) = core_ids::PAINTER_BRUSH_COMPOSITE_UP
                    .iter()
                    .position(|x| x == id)
                {
                    self.move_composite_layer_up(p);
                    return true;
                }
                if let Some(p) = core_ids::PAINTER_BRUSH_COMPOSITE_DOWN
                    .iter()
                    .position(|x| x == id)
                {
                    self.move_composite_layer_down(p);
                    return true;
                }
                false
            }
            PanelEvent::SetValue(id, v) => {
                if let Some(p) = core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH
                    .iter()
                    .position(|x| x == id)
                {
                    self.set_composite_layer_strength(p, *v as f32);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// Stamp a dab batch through the composite stack (bottom → top). Each operation reuses its own route
    /// (`stamp_dabs_inner` / `stamp_dabs_smear` / `stamp_dabs_blur`) with the layer's Strength swapped
    /// into the brush spec; a zero-Strength layer is skipped. The per-op routes each read/write the
    /// canvas in place, so an upper op sees the lower op's result (the "affects the combination" order).
    pub(super) fn stamp_dabs_composite(&mut self, dabs: &[Dab]) {
        if dabs.is_empty() {
            return;
        }
        let (w, h) = self.source_size;
        let tiling = self.paint.tiling;
        let tiled = tiling[0] || tiling[1];
        let saved_strength = self.paint.brush.strength;
        // Bottom (position 2 / layer 3) → top (position 0 / layer 1).
        for pos in (0..3).rev() {
            let layer = self.paint.composite[pos];
            if layer.strength <= 0.0 {
                continue;
            }
            self.paint.brush.strength = layer.strength;
            match layer.op {
                CompositeOp::Brush => {
                    // The brush route expects already-tiled dabs (the Smear/Blur routes tile internally).
                    if tiled {
                        let wrapped = super::tiling::tiled_dabs(dabs, self.source_size, tiling);
                        self.stamp_dabs_inner(&wrapped);
                    } else {
                        self.stamp_dabs_inner(dabs);
                    }
                }
                CompositeOp::Smear => self.stamp_dabs_smear(dabs, w, h),
                CompositeOp::Blur => self.stamp_dabs_blur(dabs, w, h),
            }
        }
        self.paint.brush.strength = saved_strength;
    }
}
