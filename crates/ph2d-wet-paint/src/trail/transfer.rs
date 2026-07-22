//! §10's window-landing half (`transfer_paint` / `transfer_blend`) — child of
//! [`super`], split for the workspace file-LOC cap. Pure code motion; the
//! session fingerprint pins byte-identity.

use super::*;

impl Trail {
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
                if sx >= 2 && sx < w && sy >= 2 && sy < h {
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
    ///
    /// Doc 23 P3 — a WET blend re-suspends: where water stands, part of the
    /// settled layer lifts through the same door the Wet tool uses
    /// (`drying::lift_settled`), so a wet blend bleeds afterwards while a
    /// dry blend stays the in-place re-mix it always was.
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
        let lift_gain = crate::tools::active_lift_gain(p);
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
                // Doc 23 P3 — the wet-blend lift, after the relaxes so the
                // freshly suspended pigment is not immediately averaged
                // back down.
                if lift_gain > 0.0 && g.film[i] as f64 > 0.001 && g.sett[i] > 0.0 {
                    let b = crate::jsmath::clamp01(a * lift_gain);
                    if b > 0.0 {
                        let sett_c = g.sett_rgb[i];
                        crate::drying::lift_settled(
                            b,
                            &mut g.susp[i],
                            &mut g.sett[i],
                            &mut g.susp_rgb[i],
                            sett_c,
                            mix,
                            &mut out,
                        );
                    }
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
}
