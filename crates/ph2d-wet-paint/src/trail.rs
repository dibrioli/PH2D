//! The paint trail (port of `trail.js`, SPEC §10): a two-step deposit.
//!
//! Dabs never stamp straight into the canvas. They accumulate into a
//! stroke-local trail buffer, and the trail lands once per WINDOW — a
//! continuous footprint instead of a chain of beads. The window is also where
//! the wet-brush feel lives:
//! - a brush-tip color buffer picks up canvas color as it passes over wet
//!   paint (the dirty brush) and slowly self-cleans back to the base color;
//! - landing uses opacity-composite color and SOFT mass/water caps (shed the
//!   window mean instead of clamping, so pools hover at the cap);
//! - "paint drag" pulls a little state from the previous window position,
//!   which smears the film forward under the stroke.
//!
//! Blend mode reuses the window plumbing with a saturating mask and re-mixes
//! both pigment layers toward the window averages (dry paint included).

use crate::brush::{BrushShape, for_each_stamp_pixel, for_each_stamp_pixel_shaped};

mod transfer; // the window-landing half of §10 (LOC-cap split)
use crate::grid::{Grid, wet_byte_from_paper};
use crate::jsmath::js_round;
use crate::opacity::alpha_of_mass;
use crate::sim::Params;
use crate::tuning::Knob;

/// Buffer half-size: max radius 35 plus the max window drift (~4 x spacing).
pub const TRAIL_HALF: i32 = 61; // ceil(35 + 4*6) + 2
pub const TRAIL_SIZE: i32 = TRAIL_HALF * 2 + 1; // 123

const N: usize = (TRAIL_SIZE * TRAIL_SIZE) as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrailMode {
    Paint,
    Blend,
}

/// One dab's parameters (SPEC §9), computed once per dab by the facade.
#[derive(Clone, Copy)]
pub struct Dab {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub hardness: f64,
    pub intensity: f64,
    pub water_amount: f64,
    /// Dry-brush gate (only feeds the §17 dry-brush extension).
    pub dry_gate: f64,
    pub shape: BrushShape,
    pub dir_x: f64,
    pub dir_y: f64,
}

pub struct Trail {
    pig: Vec<f32>,
    water: Vec<f32>,
    /// Blend mode's saturating coverage.
    mask: Vec<f32>,
    tip_r: Vec<f32>,
    tip_g: Vec<f32>,
    tip_b: Vec<f32>,
    base_r: f64,
    base_g: f64,
    base_b: f64,
    anchor_x: i32,
    anchor_y: i32,
    prev_anchor_x: i32,
    prev_anchor_y: i32,
    dab_count: u32,
    /// C — dabs accumulated before a transfer.
    window_size: u32,
    // Touched extent, local coords (inclusive); lx0 > lx1 means empty.
    lx0: i32,
    ly0: i32,
    lx1: i32,
    ly1: i32,
    mode: TrailMode,
}

/// Canvas rect touched by a transfer (cell coords, inclusive).
pub struct TouchedRect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

/// Grow a local-extent tuple (lx0, ly0, lx1, ly1) to include (lx, ly).
#[inline]
fn touch_ext(ext: &mut (i32, i32, i32, i32), lx: i32, ly: i32) {
    if ext.2 < ext.0 {
        *ext = (lx, ly, lx, ly);
        return;
    }
    if lx < ext.0 {
        ext.0 = lx;
    }
    if lx > ext.2 {
        ext.2 = lx;
    }
    if ly < ext.1 {
        ext.1 = ly;
    }
    if ly > ext.3 {
        ext.3 = ly;
    }
}

impl Default for Trail {
    fn default() -> Self {
        Trail {
            pig: vec![0.0; N],
            water: vec![0.0; N],
            mask: vec![0.0; N],
            tip_r: vec![0.0; N],
            tip_g: vec![0.0; N],
            tip_b: vec![0.0; N],
            base_r: 0.0,
            base_g: 0.0,
            base_b: 0.0,
            anchor_x: 0,
            anchor_y: 0,
            prev_anchor_x: 0,
            prev_anchor_y: 0,
            dab_count: 0,
            window_size: 0,
            lx0: 0,
            ly0: 0,
            lx1: -1,
            ly1: -1,
            mode: TrailMode::Paint,
        }
    }
}

impl Trail {
    pub fn start_stroke(&mut self, x: f64, y: f64, color: [f64; 3], mode: TrailMode) {
        self.pig.fill(0.0);
        self.water.fill(0.0);
        self.mask.fill(0.0);
        self.tip_r.fill(color[0] as f32);
        self.tip_g.fill(color[1] as f32);
        self.tip_b.fill(color[2] as f32);
        self.base_r = color[0];
        self.base_g = color[1];
        self.base_b = color[2];
        self.anchor_x = js_round(x) as i32;
        self.anchor_y = js_round(y) as i32;
        self.prev_anchor_x = self.anchor_x;
        self.prev_anchor_y = self.anchor_y;
        self.dab_count = 0;
        self.window_size = 0;
        self.lx0 = 0;
        self.ly0 = 0;
        self.lx1 = -1;
        self.ly1 = -1;
        self.mode = mode;
    }

    /// Product door: RELOAD the stroke's ink mid-stroke (the host's per-dab
    /// Randomize Colour) — reservoir AND tip planes, exactly the colour half
    /// of `start_stroke`. A reservoir-only swap was measured INERT: the tip
    /// only eases toward the base at `Knob::TipClean`, whose boot default is
    /// 0.0, so the deposit never turned. A reload is the honest semantic
    /// anyway — each jittered dab is a fresh squeeze of paint; the pickup
    /// dirt re-accumulates from the canvas as the stroke continues.
    pub fn set_base_color(&mut self, color: [f64; 3]) {
        self.base_r = color[0];
        self.base_g = color[1];
        self.base_b = color[2];
        self.tip_r.fill(color[0] as f32);
        self.tip_g.fill(color[1] as f32);
        self.tip_b.fill(color[2] as f32);
    }

    /// Frame-segment callback: size the window from the chord length.
    pub fn on_segment(&mut self, chord_len: f64, spacing: f64) {
        let cap = if self.mode == TrailMode::Blend {
            4.0
        } else {
            2.0
        };
        self.window_size = (chord_len / spacing.max(0.0001)).floor().min(cap) as u32;
    }

    /// Accumulate one paint dab into the trail. Returns true when the window
    /// is full and the caller must transfer.
    pub fn accumulate_paint(
        &mut self,
        g: &mut Grid,
        p: &Params,
        tex: &[f32],
        dab: &Dab,
        ext_bypass: bool,
    ) -> bool {
        self.accumulate_paint_impl(g, p, tex, dab, ext_bypass, None, None)
    }

    /// [`Self::accumulate_paint`] with the HOST's silhouette (the shaped
    /// product door): `sil(x, y)` replaces the engine's falloff + footprint;
    /// `grain(x, y)`, when the host's Grain slot is armed, replaces the
    /// bristle as the texture factor (`None` = the bristle stays). ONE pixel
    /// body serves both faces (`accumulate_paint` delegates with `None` —
    /// the fingerprint pins that the port's own path did not move a bit).
    #[allow(clippy::too_many_arguments)]
    pub fn accumulate_paint_shaped(
        &mut self,
        g: &mut Grid,
        p: &Params,
        tex: &[f32],
        dab: &Dab,
        ext_bypass: bool,
        sil: &mut dyn FnMut(i32, i32) -> f64,
        grain: Option<&mut dyn FnMut(i32, i32) -> f64>,
    ) -> bool {
        self.accumulate_paint_impl(g, p, tex, dab, ext_bypass, Some(sil), grain)
    }

    #[allow(clippy::too_many_arguments)]
    fn accumulate_paint_impl(
        &mut self,
        g: &mut Grid,
        p: &Params,
        tex: &[f32],
        dab: &Dab,
        ext_bypass: bool,
        sil: Option<&mut dyn FnMut(i32, i32) -> f64>,
        grain: Option<&mut dyn FnMut(i32, i32) -> f64>,
    ) -> bool {
        if self.dab_count == 0 {
            // the window's first dab anchors it
            self.anchor_x = js_round(dab.x) as i32;
            self.anchor_y = js_round(dab.y) as i32;
        }
        let gain = p.k(Knob::PigmentPerDab);
        let gate = p.k(Knob::PaperGate);
        let pig_cap = p.k(Knob::GateSaturation);
        let ext_dry_brush = p.k(Knob::ExtDryBrush);
        let ext_wet_soften = p.k(Knob::ExtWetSoften);
        let anchor_x = self.anchor_x;
        let anchor_y = self.anchor_y;
        let pig = &mut self.pig;
        let water = &mut self.water;
        let mut ext = (self.lx0, self.ly0, self.lx1, self.ly1);
        let Grid {
            s,
            w,
            h,
            susp,
            sett,
            paper,
            film,
            wet,
            ..
        } = g;
        let body = |i: usize, x: i32, y: i32, fall: f64, texv: f64| {
            let mut stamp = fall * texv * dab.intensity;
            if stamp > 1.0 {
                stamp = 1.0;
            }
            if stamp <= 0.0 {
                return;
            }
            // Paper gate: tooth peaks always take pigment, valleys reject
            // it — that per-pixel pass/reject IS the granulation. Heavily
            // loaded cells read a flat 0.45 tooth (the grain is buried).
            let tooth = if (susp[i] as f64 + sett[i] as f64) < pig_cap {
                paper[i] as f64
            } else {
                0.45
            };
            let mut deposit = stamp - (1.0 - tooth) * gate;
            if !ext_bypass {
                // Dry-brush extension: raise the gate subtraction.
                deposit -= dab.dry_gate * ext_dry_brush * 0.6;
            }
            if deposit <= 0.0 {
                return;
            }
            if !ext_bypass {
                // Wet-edge softening extension: thin the rim on wet paper.
                let softness = (film[i] as f64 / 3.0).min(1.0) * ext_wet_soften;
                deposit *= 1.0 - softness * (1.0 - fall);
            }
            // Wetness seed: OVERWRITE (not max) — repainting can dry the
            // byte back down.
            wet[i] = wet_byte_from_paper(tooth);
            let lx = x - anchor_x + TRAIL_HALF;
            let ly = y - anchor_y + TRAIL_HALF;
            if !(0..TRAIL_SIZE).contains(&lx) || !(0..TRAIL_SIZE).contains(&ly) {
                return;
            }
            let l = (lx + ly * TRAIL_SIZE) as usize;
            pig[l] = (pig[l] as f64 + deposit * gain) as f32;
            water[l] = (water[l] as f64 + deposit * dab.water_amount) as f32;
            touch_ext(&mut ext, lx, ly);
        };
        match sil {
            Some(sil) => {
                for_each_stamp_pixel_shaped(*s, *w, *h, tex, dab.x, dab.y, dab.r, sil, grain, body);
            }
            None => {
                // A grain override without a silhouette cannot happen — the
                // only Some-grain caller is the shaped door, which requires
                // `sil`. Assert it so a future caller cannot pass a grain
                // that would be silently discarded.
                debug_assert!(grain.is_none(), "grain override requires the shaped path");
                for_each_stamp_pixel(
                    *s,
                    *w,
                    *h,
                    tex,
                    dab.x,
                    dab.y,
                    dab.r,
                    dab.hardness,
                    dab.shape,
                    dab.dir_x,
                    dab.dir_y,
                    body,
                );
            }
        }
        (self.lx0, self.ly0, self.lx1, self.ly1) = ext;
        self.dab_count += 1;
        self.dab_count > self.window_size
    }

    /// Accumulate one blend dab: a saturating mask, no water/tip/wetness
    /// writes.
    pub fn accumulate_blend(&mut self, g: &Grid, p: &Params, tex: &[f32], dab: &Dab) -> bool {
        if self.dab_count == 0 {
            self.anchor_x = js_round(dab.x) as i32;
            self.anchor_y = js_round(dab.y) as i32;
        }
        let gate = p.k(Knob::PaperGate);
        let pig_cap = p.k(Knob::GateSaturation);
        let force = p.k(Knob::BlendForce);
        let anchor_x = self.anchor_x;
        let anchor_y = self.anchor_y;
        let mask = &mut self.mask;
        let mut ext = (self.lx0, self.ly0, self.lx1, self.ly1);
        for_each_stamp_pixel(
            g.s,
            g.w,
            g.h,
            tex,
            dab.x,
            dab.y,
            dab.r,
            dab.hardness,
            dab.shape,
            dab.dir_x,
            dab.dir_y,
            |i, x, y, fall, texv| {
                let mut stamp = fall * texv * dab.intensity;
                if stamp > 1.0 {
                    stamp = 1.0;
                }
                let tooth = if (g.susp[i] as f64 + g.sett[i] as f64) < pig_cap {
                    g.paper[i] as f64
                } else {
                    0.45
                };
                let e = (stamp - (1.0 - tooth) * gate) * force;
                if e <= 0.0 {
                    return;
                }
                let lx = x - anchor_x + TRAIL_HALF;
                let ly = y - anchor_y + TRAIL_HALF;
                if !(0..TRAIL_SIZE).contains(&lx) || !(0..TRAIL_SIZE).contains(&ly) {
                    return;
                }
                let l = (lx + ly * TRAIL_SIZE) as usize;
                mask[l] = (mask[l] as f64 * (1.0 - e) + e) as f32; // scrubbing builds toward 1
                touch_ext(&mut ext, lx, ly);
            },
        );
        (self.lx0, self.ly0, self.lx1, self.ly1) = ext;
        self.dab_count += 1;
        self.dab_count > self.window_size
    }

    // `transfer_paint` / `transfer_blend` — the window-landing half of §10 —
    // live in the child `transfer` module (workspace file-LOC cap).

    /// Roll: previous anchor <- this anchor; clear the trail + its extent.
    fn roll_window(&mut self) {
        self.prev_anchor_x = self.anchor_x;
        self.prev_anchor_y = self.anchor_y;
        if self.lx1 >= self.lx0 {
            for ly in self.ly0..=self.ly1 {
                let b = (ly * TRAIL_SIZE) as usize;
                let a = b + self.lx0 as usize;
                let z = b + self.lx1 as usize + 1;
                self.pig[a..z].fill(0.0);
                self.water[a..z].fill(0.0);
                self.mask[a..z].fill(0.0);
            }
        }
        self.lx0 = 0;
        self.ly0 = 0;
        self.lx1 = -1;
        self.ly1 = -1;
        self.dab_count = 0;
    }

    /// Stroke end: the remainder window is DROPPED (the release tail already
    /// emitted the fade-out; landing it would double the ending).
    pub fn drop_remainder(&mut self) {
        self.roll_window();
    }
}
