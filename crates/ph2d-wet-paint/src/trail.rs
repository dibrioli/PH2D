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
        self.accumulate_paint_impl(g, p, tex, dab, ext_bypass, None)
    }

    /// [`Self::accumulate_paint`] with the HOST's silhouette (the shaped
    /// product door): `sil(x, y)` replaces the engine's falloff + footprint,
    /// the bristle stays as the texture factor. ONE pixel body serves both
    /// faces (`accumulate_paint` delegates with `None` — the fingerprint
    /// pins that the port's own path did not move a bit).
    pub fn accumulate_paint_shaped(
        &mut self,
        g: &mut Grid,
        p: &Params,
        tex: &[f32],
        dab: &Dab,
        ext_bypass: bool,
        sil: &mut dyn FnMut(i32, i32) -> f64,
    ) -> bool {
        self.accumulate_paint_impl(g, p, tex, dab, ext_bypass, Some(sil))
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
            if lx < 0 || lx >= TRAIL_SIZE || ly < 0 || ly >= TRAIL_SIZE {
                return;
            }
            let l = (lx + ly * TRAIL_SIZE) as usize;
            pig[l] = (pig[l] as f64 + deposit * gain) as f32;
            water[l] = (water[l] as f64 + deposit * dab.water_amount) as f32;
            touch_ext(&mut ext, lx, ly);
        };
        match sil {
            Some(sil) => {
                for_each_stamp_pixel_shaped(*s, *w, *h, tex, dab.x, dab.y, dab.r, sil, body);
            }
            None => {
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
                if lx < 0 || lx >= TRAIL_SIZE || ly < 0 || ly >= TRAIL_SIZE {
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

    /// Land the paint window on the canvas (SPEC §10 "transfer"), then roll
    /// it. Returns the touched canvas rect or None.
    pub fn transfer_paint(&mut self, g: &mut Grid, p: &Params) -> Option<TouchedRect> {
        let mut out = [0.0f64; 3];
        let mix = p.mix;
        // 1. Tip self-cleaning: every texel eases back toward the stroke color.
        let clean = p.k(Knob::TipClean);
        if clean > 0.0 {
            for l in 0..N {
                self.tip_r[l] =
                    (self.tip_r[l] as f64 + (self.base_r - self.tip_r[l] as f64) * clean) as f32;
                self.tip_g[l] =
                    (self.tip_g[l] as f64 + (self.base_g - self.tip_g[l] as f64) * clean) as f32;
                self.tip_b[l] =
                    (self.tip_b[l] as f64 + (self.base_b - self.tip_b[l] as f64) * clean) as f32;
            }
        }
        if self.lx1 < self.lx0 {
            self.roll_window();
            return None;
        }
        let s = g.s;
        let w = g.w as i32;
        let h = g.h as i32;
        let tip_retain = 1.0 - p.k(Knob::Pickup);

        // 2. Tip pickup (the dirty brush) — reads the PRE-deposit canvas.
        for ly in self.ly0..=self.ly1 {
            let cy = self.anchor_y + (ly - TRAIL_HALF);
            if cy < 2 || cy > h - 1 {
                continue;
            }
            for lx in self.lx0..=self.lx1 {
                let cx = self.anchor_x + (lx - TRAIL_HALF);
                if cx < 2 || cx > w - 1 {
                    continue;
                }
                let i = cx as usize + cy as usize * s;
                let w_s = alpha_of_mass(g.sett[i] as f64) * 0.5;
                let w_f = alpha_of_mass(g.susp[i] as f64);
                let fe = w_s + w_f;
                if fe <= 0.0 {
                    continue; // blank paper keeps the tip color
                }
                let inv = 1.0 / fe;
                let sc = g.sett_rgb[i];
                let uc = g.susp_rgb[i];
                let cr = (sc[0] as f64 * w_s + uc[0] as f64 * w_f) * inv;
                let cg = (sc[1] as f64 * w_s + uc[1] as f64 * w_f) * inv;
                let cb = (sc[2] as f64 * w_s + uc[2] as f64 * w_f) * inv;
                let k = tip_retain + (1.0 - tip_retain) * (1.0 - fe.min(1.0));
                let l = (lx + ly * TRAIL_SIZE) as usize;
                self.tip_r[l] = (self.tip_r[l] as f64 * k + cr * (1.0 - k)) as f32;
                self.tip_g[l] = (self.tip_g[l] as f64 * k + cg * (1.0 - k)) as f32;
                self.tip_b[l] = (self.tip_b[l] as f64 * k + cb * (1.0 - k)) as f32;
            }
        }

        // 3. Soft-cap shedding means over cells holding trail pigment.
        let mut sum_pig = 0.0f64;
        let mut sum_water = 0.0f64;
        let mut count = 0u32;
        for ly in self.ly0..=self.ly1 {
            for lx in self.lx0..=self.lx1 {
                let l = (lx + ly * TRAIL_SIZE) as usize;
                if self.pig[l] > 0.0 {
                    sum_pig += self.pig[l] as f64;
                    sum_water += self.water[l] as f64;
                    count += 1;
                }
            }
        }
        let shed_pig = if count > 0 {
            (sum_pig / count as f64) * 1.05
        } else {
            0.0
        };
        let shed_water = if count > 0 {
            (sum_water / count as f64) * 1.05
        } else {
            0.0
        };

        // 4. Land the window + 5. paint drag.
        let drag = p.k(Knob::Drag);
        let drag_w1 = (drag * 255.0).trunc() as i32; // integer byte weights: they
        let drag_w2 = ((1.0 - drag) * 255.0).trunc() as i32; // sum < 256 on purpose, so
        let water_cap = p.k(Knob::WaterCap); // each drag leaks ~0.8% wetness
        let mut rx0 = w;
        let mut ry0 = h;
        let mut rx1 = 0;
        let mut ry1 = 0;
        for ly in self.ly0..=self.ly1 {
            let cy = self.anchor_y + (ly - TRAIL_HALF);
            if cy < 2 || cy > h - 1 {
                continue;
            }
            for lx in self.lx0..=self.lx1 {
                let l = (lx + ly * TRAIL_SIZE) as usize;
                let v = self.pig[l] as f64;
                if v <= 0.0 {
                    continue;
                }
                let cx = self.anchor_x + (lx - TRAIL_HALF);
                if cx < 2 || cx > w - 1 {
                    continue;
                }
                let i = cx as usize + cy as usize * s;
                let in_a = alpha_of_mass(v);
                let old = g.susp[i] as f64;
                if old > 0.0 {
                    // Opacity composite of the (possibly dirty) tip over the
                    // resident. (Sub-unit masses read alpha 0 from the table;
                    // guard the 0/0 case.)
                    let e9 = alpha_of_mass(old) * (1.0 - in_a);
                    if e9 + in_a > 0.0 {
                        let wgt = in_a / (e9 + in_a);
                        let uc = g.susp_rgb[i];
                        mix.mix(
                            uc[0] as f64,
                            uc[1] as f64,
                            uc[2] as f64,
                            self.tip_r[l] as f64,
                            self.tip_g[l] as f64,
                            self.tip_b[l] as f64,
                            wgt,
                            &mut out,
                        );
                        g.susp_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
                    }
                    // Soft cap: past 3000 the cell sheds the window mean
                    // instead of clamping, so heavy paint hovers at the cap.
                    let nm = if old < 3000.0 {
                        old + v
                    } else {
                        old + v - shed_pig
                    };
                    g.susp[i] = if nm < 0.0 { 0.0 } else { nm as f32 };
                } else {
                    // Virgin cell: takes the tip color outright, and drinks
                    // its first wetting twice (this add + the general below).
                    g.susp_rgb[i] = [self.tip_r[l], self.tip_g[l], self.tip_b[l]];
                    g.susp[i] = v as f32;
                    g.film[i] = (g.film[i] as f64 + self.water[l] as f64) as f32;
                }
                if g.sett[i] as f64 > 3000.0 {
                    let ns = g.sett[i] as f64 - shed_pig;
                    g.sett[i] = if ns < 0.0 { 0.0 } else { ns as f32 };
                }
                let film_i = g.film[i] as f64;
                let mut nf = if film_i < water_cap {
                    film_i + self.water[l] as f64
                } else {
                    film_i + self.water[l] as f64 - shed_water
                };
                if nf < 0.00001 {
                    nf = 0.00001; // trace floor: the wet map must know
                }
                g.film[i] = nf as f32;
                // 5. Paint drag: pull from the same window offset at the
                // PREVIOUS anchor — the film gets tugged along under the
                // stroke.
                let sx = self.prev_anchor_x + (lx - TRAIL_HALF);
                let sy = self.prev_anchor_y + (ly - TRAIL_HALF);
                if sx >= 2 && sx <= w - 1 && sy >= 2 && sy <= h - 1 {
                    let si = sx as usize + sy as usize * s;
                    let had_susp = g.susp[i] > 0.0;
                    let had_sett = g.sett[i] > 0.0;
                    g.susp[i] =
                        (g.susp[i] as f64 + (g.susp[si] as f64 - g.susp[i] as f64) * drag) as f32;
                    g.sett[i] =
                        (g.sett[i] as f64 + (g.sett[si] as f64 - g.sett[i] as f64) * drag) as f32;
                    g.film[i] =
                        (g.film[i] as f64 + (g.film[si] as f64 - g.film[i] as f64) * drag) as f32;
                    if !had_susp && g.susp[i] > 0.0 {
                        g.susp_rgb[i] = g.susp_rgb[si];
                    }
                    if !had_sett && g.sett[i] > 0.0 {
                        g.sett_rgb[i] = g.sett_rgb[si];
                    }
                    // Integer byte weights sum to < 256 on purpose: each
                    // transfer leaks ~0.8% of the wetness (a float lerp
                    // never decays it).
                    g.wet[i] =
                        ((g.wet[si] as i32 * drag_w1 + g.wet[i] as i32 * drag_w2) / 256) as u8;
                }
                if cx < rx0 {
                    rx0 = cx;
                }
                if cx > rx1 {
                    rx1 = cx;
                }
                if cy < ry0 {
                    ry0 = cy;
                }
                if cy > ry1 {
                    ry1 = cy;
                }
            }
        }
        self.roll_window();
        if rx1 < rx0 {
            return None;
        }
        g.expand_bbox(rx0 - 1, ry0 - 1, rx1 + 1, ry1 + 1);
        Some(TouchedRect {
            x0: rx0,
            y0: ry0,
            x1: rx1,
            y1: ry1,
        })
    }

    /// Blend transfer (SPEC §11 "blend"): window averages over mask-active
    /// cells, then every masked cell relaxes toward them — suspended AND
    /// settled (dry paint re-mixes), water, and the wetness byte.
    pub fn transfer_blend(&mut self, g: &mut Grid, p: &Params) -> Option<TouchedRect> {
        if self.lx1 < self.lx0 {
            self.roll_window();
            return None;
        }
        let s = g.s;
        let w = g.w as i32;
        let h = g.h as i32;
        let mut out = [0.0f64; 3];
        let mix = p.mix;
        // Pass 1: averages.
        let mut n = 0u32;
        let mut s_susp = 0.0f64;
        let mut s_sett = 0.0f64;
        let mut s_film = 0.0f64;
        let mut s_wet = 0.0f64;
        let (mut s_r, mut s_g, mut s_b) = (0.0f64, 0.0f64, 0.0f64);
        let (mut t_r, mut t_g, mut t_b) = (0.0f64, 0.0f64, 0.0f64);
        let mut w_susp = 0.0f64;
        let mut w_sett = 0.0f64;
        for ly in self.ly0..=self.ly1 {
            let cy = self.anchor_y + (ly - TRAIL_HALF);
            if cy < 2 || cy > h - 1 {
                continue;
            }
            for lx in self.lx0..=self.lx1 {
                if self.mask[(lx + ly * TRAIL_SIZE) as usize] <= 0.0 {
                    continue;
                }
                let cx = self.anchor_x + (lx - TRAIL_HALF);
                if cx < 2 || cx > w - 1 {
                    continue;
                }
                let i = cx as usize + cy as usize * s;
                n += 1;
                let susp = g.susp[i] as f64;
                let sett = g.sett[i] as f64;
                s_susp += susp;
                s_sett += sett;
                s_film += g.film[i] as f64;
                s_wet += g.wet[i] as f64;
                let uc = g.susp_rgb[i];
                let sc = g.sett_rgb[i];
                s_r += uc[0] as f64 * susp;
                s_g += uc[1] as f64 * susp;
                s_b += uc[2] as f64 * susp;
                w_susp += susp;
                t_r += sc[0] as f64 * sett;
                t_g += sc[1] as f64 * sett;
                t_b += sc[2] as f64 * sett;
                w_sett += sett;
            }
        }
        if n == 0 {
            self.roll_window();
            return None;
        }
        let nf = n as f64;
        let avg_susp = s_susp / nf;
        let avg_sett = s_sett / nf;
        let avg_film = s_film / nf;
        let avg_wet = s_wet / nf;
        let a_r = if w_susp > 0.0 { s_r / w_susp } else { 0.0 };
        let a_g = if w_susp > 0.0 { s_g / w_susp } else { 0.0 };
        let a_b = if w_susp > 0.0 { s_b / w_susp } else { 0.0 };
        let b_r = if w_sett > 0.0 { t_r / w_sett } else { 0.0 };
        let b_g = if w_sett > 0.0 { t_g / w_sett } else { 0.0 };
        let b_b = if w_sett > 0.0 { t_b / w_sett } else { 0.0 };
        // Pass 2: relax every masked cell toward the window mix.
        let mut rx0 = w;
        let mut ry0 = h;
        let mut rx1 = 0;
        let mut ry1 = 0;
        for ly in self.ly0..=self.ly1 {
            let cy = self.anchor_y + (ly - TRAIL_HALF);
            if cy < 2 || cy > h - 1 {
                continue;
            }
            for lx in self.lx0..=self.lx1 {
                let mut a = self.mask[(lx + ly * TRAIL_SIZE) as usize] as f64;
                if a <= 0.0 {
                    continue;
                }
                if a > 1.0 {
                    a = 1.0;
                }
                let cx = self.anchor_x + (lx - TRAIL_HALF);
                if cx < 2 || cx > w - 1 {
                    continue;
                }
                let i = cx as usize + cy as usize * s;
                if w_susp > 0.0 {
                    let in_c = alpha_of_mass(avg_susp) * a;
                    let res = alpha_of_mass(g.susp[i] as f64) * (1.0 - in_c);
                    if in_c + res > 0.0 {
                        let wgt = in_c / (in_c + res);
                        let uc = g.susp_rgb[i];
                        mix.mix(
                            uc[0] as f64,
                            uc[1] as f64,
                            uc[2] as f64,
                            a_r,
                            a_g,
                            a_b,
                            wgt,
                            &mut out,
                        );
                        g.susp_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
                    }
                    g.susp[i] = (g.susp[i] as f64 + (avg_susp - g.susp[i] as f64) * a) as f32;
                    g.film[i] = (g.film[i] as f64 + (avg_film - g.film[i] as f64) * a) as f32;
                }
                if w_sett > 0.0 {
                    let in_c = alpha_of_mass(avg_sett) * a;
                    let res = alpha_of_mass(g.sett[i] as f64) * (1.0 - in_c);
                    if in_c + res > 0.0 {
                        let wgt = in_c / (in_c + res);
                        let sc = g.sett_rgb[i];
                        mix.mix(
                            sc[0] as f64,
                            sc[1] as f64,
                            sc[2] as f64,
                            b_r,
                            b_g,
                            b_b,
                            wgt,
                            &mut out,
                        );
                        g.sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
                    }
                    g.sett[i] = (g.sett[i] as f64 + (avg_sett - g.sett[i] as f64) * a) as f32;
                    // The wetness byte relaxes in the settled pass. (JS
                    // Uint8Array store truncates; values stay in [0,255].)
                    g.wet[i] = (g.wet[i] as f64 + (avg_wet - g.wet[i] as f64) * a) as u8;
                }
                if cx < rx0 {
                    rx0 = cx;
                }
                if cx > rx1 {
                    rx1 = cx;
                }
                if cy < ry0 {
                    ry0 = cy;
                }
                if cy > ry1 {
                    ry1 = cy;
                }
            }
        }
        self.roll_window();
        if rx1 < rx0 {
            return None;
        }
        g.expand_bbox(rx0 - 1, ry0 - 1, rx1 + 1, ry1 + 1);
        Some(TouchedRect {
            x0: rx0,
            y0: ry0,
            x1: rx1,
            y1: ry1,
        })
    }

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
