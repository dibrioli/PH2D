//! The PRODUCT's lane doors (child of [`super`] — LOC-cap split): strokes fed
//! by REAL dabs from the host's stroke engine, one trail per symmetry copy /
//! tile offset. See the block comment on the first door for the whole story.

use super::*;

impl Engine {
    /// §9's pure half: pressure/radius → the [`Dab`]. Extracted so the LANE
    /// door and the engine's own dispatch share ONE mapping (same f64 order —
    /// the fingerprint pins it). Returns `(dab, water_unit)`; `water_unit`
    /// feeds the Wet tool's arm only.
    pub(crate) fn pressure_dab(
        &self,
        p: &crate::sim::Params,
        x: f64,
        y: f64,
        b: f64,
        dir_x: f64,
        dir_y: f64,
        r: f64,
    ) -> (Dab, f64) {
        let s = self.sliders;
        let pressure_base = 0.2 + 0.7 * s.pressure; // even minimum pressure deposits
        let mut intensity = b * 0.5 * pressure_base;
        intensity = intensity.clamp(0.0, 3.0);
        intensity *= p.k(Knob::Intensity); // knob applied post-clamp
        let water_unit = s.water * 0.35;
        // Pressure-INVERSE water: a light touch is wetter, a heavy stroke
        // denser.
        let water_amount =
            (2.0 * water_unit * water_unit) / (pressure_base + 0.01) * p.k(Knob::WaterPerDab);
        let mut h = 0.5 * b;
        if h < 1.0 {
            h = 1.0;
        } else if h > 3.0 {
            h = 3.0;
        }
        if h > r / 2.0 {
            h = r / 2.0;
        }
        let hardness = h * h; // slow => 9 crisp flat-top, fast => ~2 soft puff
        let dry_gate = clamp01(1.0 - water_amount / 1.5) * clamp01((11.0 - b - 4.0) / 4.0);
        let dab = Dab {
            x,
            y,
            r,
            hardness,
            intensity,
            water_amount,
            dry_gate,
            shape: self.shape,
            dir_x,
            dir_y,
        };
        (dab, water_unit)
    }

    // -----------------------------------------------------------------------
    // Product doors — LANES. A stroke fed by REAL dabs from the host's stroke
    // engine (the synthetic §8 recurrence and the engine's own spacing are
    // bypassed; §9's mapping runs per dab in `pressure_dab`). One lane = one
    // symmetry copy / tile offset: the trail is a 123² window anchored at its
    // first dab, so ONE trail fed an interleaved multi-copy list silently
    // drops every copy outside the window — the "Simetria Circular não está
    // correta" bug. No history capture (the host owns undo).
    // -----------------------------------------------------------------------

    /// Begin (or re-begin) a lane's stroke at `(x, y)` — grows the lane pool
    /// on demand; a lane can be born mid-stroke (a Tiling wrap starts when
    /// the stroke reaches the edge).
    pub fn begin_direct_stroke(&mut self, lane: usize, x: f64, y: f64) {
        self.stroke_down = true;
        while self.lane_trails.len() <= lane {
            self.lane_trails.push(Trail::default());
        }
        let mode = if self.tool == Tool::Blend {
            TrailMode::Blend
        } else {
            TrailMode::Paint
        };
        self.lane_trails[lane].start_stroke(x, y, self.color, mode);
    }

    /// The lane's fresh-ink colour (sRGB 0..255), settable PER DAB — the
    /// host's Randomize Colour. `self.color` follows for the next stroke.
    pub fn set_stroke_color(&mut self, lane: usize, color: [f64; 3]) {
        self.color = color;
        if let Some(t) = self.lane_trails.get_mut(lane) {
            t.set_base_color(color);
        }
    }

    /// The lane's path advanced by `chord` px — rolls that lane's deposit
    /// window exactly as the engine's own Segment events do.
    pub fn direct_segment(&mut self, lane: usize, chord: f64) {
        if (self.tool == Tool::Paint || self.tool == Tool::Blend)
            && let Some(t) = self.lane_trails.get_mut(lane)
        {
            let spacing = self.tuning.get(Knob::Spacing);
            t.on_segment(chord, spacing);
        }
    }

    /// §9 with real pressure + real radius, deposited through the LANE's
    /// trail (Paint mode's arm; the engine's own tools keep `self.trail`).
    /// `sil`: the host's per-dab silhouette (falloff × Shape × footprint) —
    /// `Some` routes the deposit through the shaped stamp (the engine's own
    /// falloff/footprint step aside; the bristle stays); `None` = the
    /// engine's round stamp.
    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_pressure_dab_lane(
        &mut self,
        lane: usize,
        x: f64,
        y: f64,
        b: f64,
        dir_x: f64,
        dir_y: f64,
        r: f64,
        sil: Option<&mut dyn FnMut(i32, i32) -> f64>,
    ) {
        let p = self.sim.gather_params(&self.tuning);
        let (dab, _water_unit) = self.pressure_dab(&p, x, y, b, dir_x, dir_y, r);
        if let Some(hook) = self.on_dab.as_mut() {
            hook(b, &dab);
        }
        let ext_bypass = self.sim.ext_bypass;
        self.texture();
        let tex = self.brush_tex.take().unwrap();
        let layer = &mut self.layers[self.active_layer];
        let Some(trail) = self.lane_trails.get_mut(lane) else {
            self.brush_tex = Some(tex);
            return;
        };
        let full = match sil {
            Some(sil) => {
                trail.accumulate_paint_shaped(&mut layer.grid, &p, &tex, &dab, ext_bypass, sil)
            }
            None => trail.accumulate_paint(&mut layer.grid, &p, &tex, &dab, ext_bypass),
        };
        let rect = if full {
            trail.transfer_paint(&mut layer.grid, &p)
        } else {
            None
        };
        self.brush_tex = Some(tex);
        if let Some(rc) = rect {
            self.merge_dirty(rc.x0, rc.y0, rc.x1, rc.y1);
        }
    }

    /// End a real-dab stroke: every lane's tip remainder is dropped by
    /// design, mirroring the tail of the engine's `end_stroke`. The lane pool
    /// stays allocated for the next stroke of the session.
    pub fn end_direct_stroke(&mut self) {
        self.stroke_down = false;
        for t in &mut self.lane_trails {
            t.drop_remainder();
        }
    }
}
