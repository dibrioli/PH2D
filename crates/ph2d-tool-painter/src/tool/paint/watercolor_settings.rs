//! **Watercolor** section setters + panel router (the wet-media look: edge darkening + granulation +
//! pigment build-up; no fluid sim — `docs/Painter/08_plano_aquarela_edge_grain_pigment.md`). The single
//! clamp source for those UI edits, mirroring [`super::jitter_settings`]. A submodule of `paint` so it
//! shares `PainterTool`'s private `paint.brush` access. The stored values are consumed at stamp time
//! (granulation / pigment, `ph2d_painter_brush::dab`) and at stroke end (edge darkening).

use super::brush_settings::BrushTextureImage;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::{
    BrushSpec, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind, TextureMapping, TextureSettings,
};

impl PainterTool {
    /// Route the Watercolor section controls (master enable + Pigment toggle + section reset, and the
    /// Edge / Spread / Granulation / Mix sliders) from the layers panel's generic channel to the
    /// setters below. Returns `true` when it consumed the event. Mirrors
    /// [`Self::route_brush_jitter_event`]; called from `handle_panel_event` before the main match.
    pub(crate) fn route_brush_watercolor_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        match event {
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_ENABLE => {
                self.toggle_brush_watercolor();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_PIGMENT => {
                self.toggle_brush_pigment();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_RESET => {
                self.reset_brush_watercolor();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_GRAN_SAME => {
                self.toggle_granulation_use_paper();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_PAPER_RESET => {
                self.reset_brush_paper();
                true
            }
            PanelEvent::SetValue(id, v) => {
                let v = *v as f32;
                match *id {
                    x if x == core_ids::PAINTER_WATERCOLOR_EDGE => {
                        self.set_brush_edge_gain(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_SPREAD => {
                        self.set_brush_edge_spread(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_GRANULATION => {
                        self.set_brush_granulation(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_MIX => {
                        self.set_brush_pigment_mix(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_FILL => {
                        self.set_brush_fill(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_DEPTH => {
                        self.set_brush_depth(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_WARP => {
                        self.set_brush_warp(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_PAPER_SIZE_X => {
                        self.set_brush_paper_size(0, v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_PAPER_SIZE_Y => {
                        self.set_brush_paper_size(1, v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_PAPER_ANGLE => {
                        self.set_brush_paper_angle(v);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Toggle the **Wet edges** master enable — gates the whole section (edge / granulation / pigment).
    /// Off (default) makes a stroke byte-identical to a plain brush.
    pub fn toggle_brush_watercolor(&mut self) {
        self.paint.brush.watercolor = !self.paint.brush.watercolor;
    }

    /// Toggle **Pigment** (subtractive Kubelka–Munk wet-on-wet colour mixing).
    pub fn toggle_brush_pigment(&mut self) {
        self.paint.brush.pigment = !self.paint.brush.pigment;
    }

    /// Set the **Edge** darkening gain (the wet-edge "fringe"), clamped to `0..=8`.
    pub fn set_brush_edge_gain(&mut self, v: f32) {
        self.paint.brush.edge_gain = v.clamp(0.0, 8.0);
    }

    /// Set the **Spread** (edge-darkening blur radius in canvas px), clamped to `1..=24`.
    pub fn set_brush_edge_spread(&mut self, v: f32) {
        self.paint.brush.edge_spread = v.clamp(1.0, 24.0);
    }

    /// Set the **Granulation** amount (paper-tooth deposit gate), clamped to `0..=1`.
    pub fn set_brush_granulation(&mut self, v: f32) {
        self.paint.brush.granulation = v.clamp(0.0, 1.0);
    }

    /// Set the **Mix** (subtractive pigment amount), clamped to `0..=1`.
    pub fn set_brush_pigment_mix(&mut self, v: f32) {
        self.paint.brush.pigment_mix = v.clamp(0.0, 1.0);
    }

    /// Set the render-path **Fill** (wash interior density), clamped to `0..=1`.
    pub fn set_brush_fill(&mut self, v: f32) {
        self.paint.brush.fill = v.clamp(0.0, 1.0);
    }

    /// Set the render-path **Depth** (Beer–Lambert optical-depth scale), clamped to `0.1..=8` (must be
    /// `> 0` so `Tᵢ = pigmentᵢ^(D·depth)` is a real attenuation; the floor keeps a visible wash).
    pub fn set_brush_depth(&mut self, v: f32) {
        self.paint.brush.depth = v.clamp(0.1, 8.0);
    }

    /// Set the render-path **Warp** (organic-boundary displacement, canvas px), clamped to `0..=24`.
    pub fn set_brush_warp(&mut self, v: f32) {
        self.paint.brush.warp = v.clamp(0.0, 24.0);
    }

    /// Reset the **Paper** slot to empty (kind `None` → the render-path falls back to the built-in paper
    /// noise), dropping any tagged-layer image. Plain state edit (no undo / pixel touch).
    pub fn reset_brush_paper(&mut self) {
        self.paint.brush.paper = TextureSettings::default();
        self.paint.paper_image = None;
    }

    /// Set the **Paper** slot kind (`TextureKind` wire u8) + force canvas-anchored mapping.
    pub fn set_brush_paper_kind(&mut self, k: u8) {
        self.paint.brush.paper.kind = TextureKind::from_u8(k);
        self.paint.brush.paper.mapping = TextureMapping::Tiled;
    }

    /// Set the **Paper** slot Size on `axis` (0 = x, 1 = y), clamped to `[0.1, 100]`.
    pub fn set_brush_paper_size(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.paper.size[axis] = v.clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        }
    }

    /// Set the **Paper** slot Angle (whole degrees, wrapped to `0..360`).
    pub fn set_brush_paper_angle(&mut self, deg: f32) {
        self.paint.brush.paper.angle_deg = deg.rem_euclid(360.0) as u16;
    }

    /// Toggle **Granulation "Same as Paper"** — on = the granulation settles into the paper's own tooth
    /// (the Grain slot texture is ignored); off = the **Grain** slot IS the granulation map.
    pub fn toggle_granulation_use_paper(&mut self) {
        self.paint.brush.granulation_use_paper = !self.paint.brush.granulation_use_paper;
    }

    /// Reset the **Watercolor** section to defaults (section off; all params neutral). Plain paint
    /// state — no undo / pixel touch, like the other section resets.
    pub fn reset_brush_watercolor(&mut self) {
        let d = BrushSpec::default();
        let b = &mut self.paint.brush;
        b.watercolor = d.watercolor;
        b.edge_gain = d.edge_gain;
        b.edge_spread = d.edge_spread;
        b.granulation = d.granulation;
        b.pigment = d.pigment;
        b.pigment_mix = d.pigment_mix;
        b.fill = d.fill;
        b.depth = d.depth;
        b.warp = d.warp;
    }

    /// Install a tagged Hierarchy layer/group (its luminance `lum`, `width × height`) into the watercolor
    /// **Paper** slot: the substrate tooth the wash sits on (Fase D — "Use as Paper"). Canvas-anchored; the
    /// render-path is turned on. A Group tag passes the composited group pixels.
    pub fn use_layers_as_watercolor_paper(&mut self, lum: Vec<u8>, width: u32, height: u32) {
        self.paint.paper_image = Some(BrushTextureImage::new(lum, width, height));
        self.paint.brush.paper.kind = TextureKind::Image;
        self.paint.brush.paper.mapping = TextureMapping::Tiled;
        self.paint.brush.watercolor = true;
    }

    /// Install a tagged layer/group into the **Granulation** map — the **Grain** slot (`brush.texture`),
    /// DISTINCT from the paper (Fase D — "Use as Granulation"). Turns off "Same as Paper" so this map is
    /// used, and gives a pronounced granulation amount so the pigment pools in the layer's valleys.
    pub fn use_layers_as_granulation(&mut self, lum: Vec<u8>, width: u32, height: u32) {
        self.set_brush_texture_image(lum, width, height); // → the Grain slot (kind = Image)
        self.paint.brush.texture.mapping = TextureMapping::Tiled;
        self.paint.brush.granulation_use_paper = false;
        self.paint.brush.watercolor = true;
        self.paint.brush.granulation = 0.65; // LITERAL-OK: pronounced mineral-settling granulation
    }

    /// Apply a one-click **brush preset** (the top-of-panel dropdown): `0` = **Digital Basic** (the plain
    /// brush), `1` = **Watercolor Basic** (the optical wash configured to reproduce
    /// `docs/Painter/wet_edges_paint.html`). Both PRESERVE the current colour + radius (a preset is a
    /// look, not a reset of what/where you paint); everything else is set from scratch so switching is
    /// deterministic. Plain state edit (no undo / pixel touch), like the section resets.
    pub fn apply_brush_preset(&mut self, idx: u8) {
        let cur = self.paint.brush;
        self.paint.brush = match idx {
            // Watercolor Basic — wet_edges defaults: soft round dab (`Falloff::Smooth`, `hardness 0`),
            // Mix blend, dense spacing, the optical render-path on (Fill/Depth/Edge/Spread/Warp/Granulation
            // set to the wet_edges constants). Pigment stays off (wet_edges default `realistic = false`).
            1 => BrushSpec {
                radius_px: cur.radius_px,
                color: cur.color,
                spacing: 0.05, // LITERAL-OK: dense wash dabs (wet_edges segments ≈ r·0.22)
                watercolor: true,
                fill: 0.12,        // LITERAL-OK: wet_edges fillDensity
                depth: 1.2,        // LITERAL-OK: wet_edges DEPTH
                edge_gain: 3.0,    // LITERAL-OK: wet_edges edgeGain
                edge_spread: 7.0,  // LITERAL-OK: wet_edges spread
                warp: 6.0,         // LITERAL-OK: wet_edges warpAmp
                granulation: 0.30, // LITERAL-OK: wet_edges granAmt
                pigment: false,
                // Paper slot = a canvas-anchored cold-press paper (the substrate the wash sits on);
                // Granulation follows the paper's tooth (Same as Paper, the default).
                paper: TextureSettings {
                    kind: TextureKind::PaperCold,
                    mapping: TextureMapping::Tiled,
                    ..TextureSettings::default()
                },
                ..BrushSpec::default()
            },
            // Digital Basic (0 or any unknown) — the plain default brush, keeping the user's colour + size.
            _ => BrushSpec {
                radius_px: cur.radius_px,
                color: cur.color,
                ..BrushSpec::default()
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::tool::PainterTool;
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::{TextureKind, TextureMapping};

    /// The full panel→tool seam EFFECT (the other half of the panel's `tests/seam.rs` forward proof):
    /// the exact `PanelEvent`s the panel forwards, fed to `handle_panel_event`, mutate the observable
    /// brush state (read back through the published `BrushSettings` snapshot). Also pins the clamps.
    #[test]
    fn panel_events_drive_watercolor_state() {
        let mut t = PainterTool::default();
        assert!(!t.brush_settings().watercolor, "default off");

        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_ENABLE));
        assert!(t.brush_settings().watercolor, "Wet edges toggled on");

        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_EDGE, 3.0));
        assert_eq!(t.brush_settings().edge_gain, 3.0, "Edge slider set");

        t.handle_panel_event(PanelEvent::SetValue(
            core_ids::PAINTER_WATERCOLOR_GRANULATION,
            0.5,
        ));
        assert_eq!(t.brush_settings().granulation, 0.5, "Granulation set");

        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_PIGMENT));
        assert!(t.brush_settings().pigment, "Pigment toggled on");

        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_MIX, 0.75));
        assert_eq!(t.brush_settings().pigment_mix, 0.75, "Mix set");

        // Render-path optics: Fill / Depth / Warp drive the same seam.
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_FILL, 0.4));
        assert_eq!(t.brush_settings().fill, 0.4, "Fill set");
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_DEPTH, 2.0));
        assert_eq!(t.brush_settings().depth, 2.0, "Depth set");
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_WARP, 10.0));
        assert_eq!(t.brush_settings().warp, 10.0, "Warp set");

        // Paper + Granulation slots: kind picker, Size, Angle, and the "Same as Paper" toggle.
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_WATERCOLOR_PAPER_KIND,
            (TextureKind::PaperRough.to_u8()).to_string(),
        ));
        assert_eq!(
            t.paint.brush.paper.kind,
            TextureKind::PaperRough,
            "Paper kind picked"
        );
        assert_eq!(
            t.paint.brush.paper.mapping,
            TextureMapping::Tiled,
            "paper forced canvas-anchored"
        );
        t.handle_panel_event(PanelEvent::SetValue(
            core_ids::PAINTER_WATERCOLOR_PAPER_SIZE_X,
            50.0,
        ));
        assert_eq!(t.paint.brush.paper.size[0], 50.0, "Paper Size X set (0.1..100)");
        t.handle_panel_event(PanelEvent::SetValue(
            core_ids::PAINTER_WATERCOLOR_PAPER_ANGLE,
            45.0,
        ));
        assert_eq!(t.paint.brush.paper.angle_deg, 45, "Paper Angle set");
        assert!(t.brush_settings().granulation_use_paper, "Same as Paper default on");
        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_GRAN_SAME));
        assert!(!t.brush_settings().granulation_use_paper, "Same as Paper toggled off");

        // Clamp: Edge caps at 8, Spread at 24.
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_EDGE, 99.0));
        assert_eq!(t.brush_settings().edge_gain, 8.0, "Edge clamped to 8");
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_SPREAD, 99.0));
        assert_eq!(t.brush_settings().edge_spread, 24.0, "Spread clamped to 24");

        // Reset returns the whole section to defaults — the `watercolor`/`pigment` gates OFF (which is
        // what makes a brush neutral); the params go back to their sensible when-enabled defaults.
        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_RESET));
        let b = t.brush_settings();
        assert!(
            !b.watercolor && !b.pigment,
            "reset turned the Watercolor + Pigment gates off"
        );
        assert_eq!(b.edge_gain, 1.5, "reset restored the default Edge gain");
    }

    /// The Preset dropdown seam: `SelectOption(PAINTER_BRUSH_PRESET, idx)` reconfigures the whole brush.
    /// Watercolor Basic turns the render-path on with the wet_edges knobs; Digital Basic turns it back
    /// off — both PRESERVING the user's colour + radius (a preset is a look, not a what/where reset).
    #[test]
    fn preset_dropdown_reconfigures_the_brush() {
        let mut t = PainterTool::default();
        // Give the brush a distinctive colour + size the preset must preserve.
        t.paint.brush.color = [0.2, 0.6, 0.9];
        t.paint.brush.radius_px = 40.0;

        // Watercolor Basic (idx 1): render-path on + wet_edges optics.
        t.handle_panel_event(PanelEvent::SelectOption(core_ids::PAINTER_BRUSH_PRESET, "1".into()));
        let b = t.brush_settings();
        assert!(b.watercolor, "Watercolor Basic turns the render-path on");
        assert_eq!(b.edge_gain, 3.0, "wet_edges edge gain");
        assert_eq!(b.fill, 0.12, "wet_edges fill");
        assert_eq!(b.depth, 1.2, "wet_edges depth");
        assert_eq!(b.color, [0.2, 0.6, 0.9], "colour preserved across the preset");
        assert_eq!(t.paint.brush.radius_px, 40.0, "radius preserved across the preset");
        // Paper slot wired to a canvas-anchored cold-press paper (the substrate the wash sits on).
        assert_eq!(t.paint.brush.paper.kind, TextureKind::PaperCold, "Paper = cold-press");
        assert_eq!(t.paint.brush.paper.mapping, TextureMapping::Tiled, "paper is canvas-anchored");

        // Digital Basic (idx 0): back to the plain brush, colour + size still preserved.
        t.handle_panel_event(PanelEvent::SelectOption(core_ids::PAINTER_BRUSH_PRESET, "0".into()));
        let b = t.brush_settings();
        assert!(!b.watercolor, "Digital Basic turns the render-path off");
        assert_eq!(b.color, [0.2, 0.6, 0.9], "colour still preserved");
        assert_eq!(t.paint.brush.radius_px, 40.0, "radius still preserved");
    }

    /// A tagged layer installs into the RIGHT slot: "Use as Paper" → the Paper slot; "Use as Granulation"
    /// → the Granulation slot (Same-as-Paper off, its own map). The two are distinct destinations, not the
    /// same Grain slot (the bug Enio caught).
    #[test]
    fn use_layers_routes_paper_and_granulation_to_separate_slots() {
        let lum = vec![128u8; 8 * 8];

        let mut t = PainterTool::default();
        t.use_layers_as_watercolor_paper(lum.clone(), 8, 8);
        let b = &t.paint.brush;
        assert_eq!(b.paper.kind, TextureKind::Image, "paper → Paper slot Image");
        assert_eq!(b.paper.mapping, TextureMapping::Tiled, "canvas-anchored");
        assert!(b.watercolor, "render-path on");
        // The Grain slot is untouched (Paper is its own slot now).
        assert_eq!(b.texture.kind, TextureKind::None, "the per-dab Grain slot is not touched");

        let mut t2 = PainterTool::default();
        t2.use_layers_as_granulation(lum, 8, 8);
        let b2 = &t2.paint.brush;
        // Granulation = the GRAIN slot (its own section); "Same as Paper" turned off so the map is used.
        assert_eq!(b2.texture.kind, TextureKind::Image, "granulation → Grain slot Image");
        assert!(!b2.granulation_use_paper, "granulation uses the Grain map, not the paper");
        assert!((b2.granulation - 0.65).abs() < 1e-6, "pronounced mineral-settling amount");
        assert_eq!(b2.paper.kind, TextureKind::None, "the Paper slot is not touched by the granulation tag");
    }
}
