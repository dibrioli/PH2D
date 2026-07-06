//! **Watercolor** section setters + panel router (the wet-media look: edge darkening + granulation +
//! pigment build-up; no fluid sim — `docs/Painter/08_plano_aquarela_edge_grain_pigment.md`). The single
//! clamp source for those UI edits, mirroring [`super::jitter_settings`]. A submodule of `paint` so it
//! shares `PainterTool`'s private `paint.brush` access. The stored values are consumed at stamp time
//! (granulation / pigment, `ph2d_painter_brush::dab`) and at stroke end (edge darkening).

use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::{BrushSpec, TextureKind, TextureMapping, TextureSettings};

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

    /// Install a tagged Hierarchy layer/group (its luminance `lum`, `width × height`) as the watercolor
    /// **paper**: the Grain slot becomes that image, canvas-anchored (Tiled), and the Watercolor
    /// render-path is turned on so the wash granulates against the layer's own tooth (Fase D — "Use as
    /// Paper"). Moderate granulation (a paper *surface*). A Group tag passes the composited group pixels.
    pub fn use_layers_as_watercolor_paper(&mut self, lum: Vec<u8>, width: u32, height: u32) {
        self.set_brush_texture_image(lum, width, height);
        self.paint.brush.texture.mapping = TextureMapping::Tiled;
        self.paint.brush.watercolor = true;
        if self.paint.brush.granulation <= 0.0 {
            self.paint.brush.granulation = 0.30; // LITERAL-OK: default paper-surface granulation
        }
    }

    /// As [`Self::use_layers_as_watercolor_paper`] but as the **granulation** map (Fase D — "Use as
    /// Granulation"): a stronger mineral-settling bite, so the pigment pools harder in the layer's valleys.
    pub fn use_layers_as_granulation(&mut self, lum: Vec<u8>, width: u32, height: u32) {
        self.set_brush_texture_image(lum, width, height);
        self.paint.brush.texture.mapping = TextureMapping::Tiled;
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
                // Grain slot = a canvas-anchored cold-press Paper, so the wash granulates against a real
                // paper tooth (the render-path reads the Tiled Grain — Fase C deep integration).
                texture: TextureSettings {
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
        // Grain slot wired to a canvas-anchored cold-press Paper (Fase C deep integration).
        assert_eq!(t.paint.brush.texture.kind, TextureKind::PaperCold, "Grain = cold-press paper");
        assert_eq!(t.paint.brush.texture.mapping, TextureMapping::Tiled, "paper is canvas-anchored");

        // Digital Basic (idx 0): back to the plain brush, colour + size still preserved.
        t.handle_panel_event(PanelEvent::SelectOption(core_ids::PAINTER_BRUSH_PRESET, "0".into()));
        let b = t.brush_settings();
        assert!(!b.watercolor, "Digital Basic turns the render-path off");
        assert_eq!(b.color, [0.2, 0.6, 0.9], "colour still preserved");
        assert_eq!(t.paint.brush.radius_px, 40.0, "radius still preserved");
    }

    /// Fase D — a tagged layer installs as the watercolor paper: the Grain slot becomes a canvas-anchored
    /// Image, the render-path turns on, and granulation is set (moderate for Paper, strong for Granulation).
    #[test]
    fn use_layers_as_paper_and_granulation_install_the_grain_image() {
        let lum = vec![128u8; 8 * 8];

        let mut t = PainterTool::default();
        t.use_layers_as_watercolor_paper(lum.clone(), 8, 8);
        let b = &t.paint.brush;
        assert_eq!(b.texture.kind, TextureKind::Image, "paper → Grain Image");
        assert_eq!(b.texture.mapping, TextureMapping::Tiled, "canvas-anchored");
        assert!(b.watercolor, "render-path on");
        assert!((b.granulation - 0.30).abs() < 1e-6, "moderate paper granulation");
        assert!(t.brush_texture_image().is_some(), "the layer pixels are stored");

        let mut t2 = PainterTool::default();
        t2.use_layers_as_granulation(lum, 8, 8);
        assert!(
            (t2.paint.brush.granulation - 0.65).abs() < 1e-6,
            "granulation is a stronger mineral-settling bite"
        );
        assert_eq!(t2.paint.brush.texture.mapping, TextureMapping::Tiled);
    }
}
