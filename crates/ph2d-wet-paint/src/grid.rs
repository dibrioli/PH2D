//! Per-cell simulation state (port of `grid.js`, SPEC §2) + the canvas-wide
//! actions that are pure field operations (SPEC §12: wet/dry canvas, clear).
//!
//! Layout: the paintable canvas is W x H cells, one cell per pixel, stored in
//! padded arrays with stride S = W + 2 and rows = H + 2 (one border cell on
//! every side). Interior cells are x in [1..W], y in [1..H]; index x + y*S.
//! Brushes may only touch [2..W-1] x [2..H-1] — the outermost interior ring is
//! a drain the boundary pass wipes every frame, so drips run off the sheet
//! instead of pooling at the edge.
//!
//! Two pigment layers is the core model: brushes deposit SUSPENDED pigment
//! (moves with the flow); drying transfers it to SETTLED (stuck to the paper);
//! re-wetting lifts a little back. Two velocity fields is the core trick:
//! gravity accumulates in the PERSISTENT field, while the TRANSIENT flow is
//! rebuilt from it every frame — the absorbency brake only applies during the
//! 1-in-4 rebuild, so a drip keeps its momentum on the other three frames.

use crate::colorops::ColorMix;
use crate::opacity::alpha_of_mass;
use crate::paper::PaperPreset;

pub const DEFAULT_WIDTH: usize = 900;
pub const DEFAULT_HEIGHT: usize = 450;

pub struct Grid {
    pub w: usize,
    pub h: usize,
    /// Stride = w + 2 (one pad column each side).
    pub s: usize,
    pub rows: usize,
    pub cells: usize,
    /// Free water depth, 0..waterCap.
    pub film: Vec<f32>,
    /// Suspended pigment mass (moves with the flow).
    pub susp: Vec<f32>,
    /// Suspended color, 0..255 floats, interleaved [r, g, b] per cell.
    /// (The JS keeps three planes; interleaving is a pure layout change —
    /// same f32 values — that halves the array count the hot passes touch.)
    pub susp_rgb: Vec<[f32; 3]>,
    /// Settled pigment mass (dried onto the paper, does not move).
    pub sett: Vec<f32>,
    /// Settled color, interleaved like `susp_rgb`.
    pub sett_rgb: Vec<[f32; 3]>,
    /// Persistent velocity (gravity lands here).
    pub vel_x: Vec<f32>,
    pub vel_y: Vec<f32>,
    /// Transient flow, rebuilt from the persistent field each frame.
    pub flow_x: Vec<f32>,
    pub flow_y: Vec<f32>,
    /// Wetness byte: "the sheet is damp here".
    pub wet: Vec<u8>,
    /// Paper tooth height 0..1, baked incl. the pad ring.
    pub paper: Vec<f32>,
    /// 0 = skip, 1/2 = solver processes this cell.
    pub active: Vec<u8>,
    /// Per-cell budget for the backrun extension.
    pub bloom: Vec<u8>,
    // Active bounding box (the solver iterates only inside it) + fluid flag.
    pub bx0: i32,
    pub by0: i32,
    pub bx1: i32,
    pub by1: i32,
    pub has_fluid: bool,
    // Paper identity (re-baked by paper.rs; part of history snapshots).
    pub paper_preset: PaperPreset,
    pub paper_sheet: u32,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let s = width + 2;
        let rows = height + 2;
        let n = s * rows;
        Grid {
            w: width,
            h: height,
            s,
            rows,
            cells: n,
            film: vec![0.0; n],
            susp: vec![0.0; n],
            susp_rgb: vec![[0.0; 3]; n],
            sett: vec![0.0; n],
            sett_rgb: vec![[0.0; 3]; n],
            vel_x: vec![0.0; n],
            vel_y: vec![0.0; n],
            flow_x: vec![0.0; n],
            flow_y: vec![0.0; n],
            wet: vec![0; n],
            paper: vec![0.0; n],
            active: vec![0; n],
            bloom: vec![0; n],
            bx0: 0,
            by0: 0,
            bx1: -1,
            by1: -1,
            has_fluid: false,
            paper_preset: PaperPreset::Cold,
            paper_sheet: 0,
        }
    }

    /// Empty the active bbox and drop the fluid flag.
    pub fn empty_bbox(&mut self) {
        self.bx0 = 0;
        self.by0 = 0;
        self.bx1 = -1;
        self.by1 = -1;
        self.has_fluid = false;
    }

    /// Grow the active bbox to include a rect (canvas cell coords), clamped
    /// to the interior.
    pub fn expand_bbox(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let x0 = x0.max(1);
        let y0 = y0.max(1);
        let x1 = x1.min(self.w as i32);
        let y1 = y1.min(self.h as i32);
        if x0 > x1 || y0 > y1 {
            return;
        }
        if !self.has_fluid || self.bx0 > self.bx1 {
            self.bx0 = x0;
            self.by0 = y0;
            self.bx1 = x1;
            self.by1 = y1;
        } else {
            if x0 < self.bx0 {
                self.bx0 = x0;
            }
            if y0 < self.by0 {
                self.by0 = y0;
            }
            if x1 > self.bx1 {
                self.bx1 = x1;
            }
            if y1 > self.by1 {
                self.by1 = y1;
            }
        }
        self.has_fluid = true;
    }
}

/// Wetness byte a given paper tooth stamps: valleys (low paper) read wetter.
#[inline]
pub fn wet_byte_from_paper(paper_value: f64) -> u8 {
    let mut v = 2.0 - 2.0 * paper_value;
    if v > 1.0 {
        v = 1.0;
    }
    if v < 0.0 {
        v = 0.0;
    }
    (v * 255.0) as u8
}

// ---------------------------------------------------------------------------
// Canvas-wide actions (SPEC §12)
// ---------------------------------------------------------------------------

/// Wet canvas: raise the wetness byte to the paper-derived value via max over
/// the whole interior. Injects NO water and touches no bbox — the sim stays
/// idle, but subsequent strokes bleed everywhere and show-wet reads damp.
pub fn wet_canvas(g: &mut Grid) {
    let s = g.s;
    for y in 1..=g.h {
        let mut i = 1 + y * s;
        for _x in 1..=g.w {
            let b = wet_byte_from_paper(g.paper[i] as f64);
            if b > g.wet[i] {
                g.wet[i] = b;
            }
            i += 1;
        }
    }
}

/// Dry canvas: one-shot O(area) — settle every cell's suspended mass into the
/// settled layer (opacity-composite color, same as the dry pass), zero water,
/// both velocity fields and wetness, and empty the bbox.
pub fn dry_canvas(g: &mut Grid, mix: ColorMix) {
    let s = g.s;
    let mut out = [0.0f64; 3];
    for y in 1..=g.h {
        let mut i = 1 + y * s;
        for _x in 1..=g.w {
            let dm = g.susp[i] as f64;
            if dm > 0.0 {
                settle_composite(g, i, dm, mix, &mut out);
                g.sett[i] = (g.sett[i] as f64 + dm) as f32;
                g.susp[i] = 0.0;
            }
            g.film[i] = 0.0;
            g.vel_x[i] = 0.0;
            g.vel_y[i] = 0.0;
            g.flow_x[i] = 0.0;
            g.flow_y[i] = 0.0;
            g.wet[i] = 0;
            i += 1;
        }
    }
    g.empty_bbox();
}

/// Opacity-composite `dm` of suspended pigment into the settled layer's color
/// at cell i (SPEC §6.2 step 3). NOT mass-weighted: coverage-weighted, so a
/// thin new glaze barely shifts an already-opaque settled color.
#[inline]
pub fn settle_composite(g: &mut Grid, i: usize, dm: f64, mix: ColorMix, out: &mut [f64; 3]) {
    let a_sett = alpha_of_mass(g.sett[i] as f64);
    let a_in = alpha_of_mass(dm);
    if a_sett > 0.0 {
        let u = a_sett * (1.0 - a_in);
        let w = a_in / (u + a_in);
        let sc = g.sett_rgb[i];
        let uc = g.susp_rgb[i];
        mix.mix(
            sc[0] as f64,
            sc[1] as f64,
            sc[2] as f64,
            uc[0] as f64,
            uc[1] as f64,
            uc[2] as f64,
            w,
            out,
        );
        g.sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
    } else {
        g.sett_rgb[i] = g.susp_rgb[i];
    }
}

/// Clear: zero all dynamic state; the paper is untouched.
pub fn clear_canvas(g: &mut Grid) {
    g.film.fill(0.0);
    g.susp.fill(0.0);
    g.susp_rgb.fill([0.0; 3]);
    g.sett.fill(0.0);
    g.sett_rgb.fill([0.0; 3]);
    g.vel_x.fill(0.0);
    g.vel_y.fill(0.0);
    g.flow_x.fill(0.0);
    g.flow_y.fill(0.0);
    g.wet.fill(0);
    g.active.fill(0);
    g.bloom.fill(0);
    g.empty_bbox();
}

// ---------------------------------------------------------------------------
// History snapshots (SPEC §15). The transient flow arrays are scratch rebuilt
// every frame from the persistent field, so they are not captured.
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct GridSnapshot {
    pub film: Vec<f32>,
    pub susp: Vec<f32>,
    pub susp_rgb: Vec<[f32; 3]>,
    pub sett: Vec<f32>,
    pub sett_rgb: Vec<[f32; 3]>,
    pub vel_x: Vec<f32>,
    pub vel_y: Vec<f32>,
    pub wet: Vec<u8>,
    pub active: Vec<u8>,
    pub bloom: Vec<u8>,
    pub bx0: i32,
    pub by0: i32,
    pub bx1: i32,
    pub by1: i32,
    pub has_fluid: bool,
    pub paper_preset: PaperPreset,
    pub paper_sheet: u32,
}

pub fn snapshot_grid(g: &Grid) -> GridSnapshot {
    GridSnapshot {
        film: g.film.clone(),
        susp: g.susp.clone(),
        susp_rgb: g.susp_rgb.clone(),
        sett: g.sett.clone(),
        sett_rgb: g.sett_rgb.clone(),
        vel_x: g.vel_x.clone(),
        vel_y: g.vel_y.clone(),
        wet: g.wet.clone(),
        active: g.active.clone(),
        bloom: g.bloom.clone(),
        bx0: g.bx0,
        by0: g.by0,
        bx1: g.bx1,
        by1: g.by1,
        has_fluid: g.has_fluid,
        paper_preset: g.paper_preset,
        paper_sheet: g.paper_sheet,
    }
}

/// Restore a snapshot. Returns true when the paper identity changed (the
/// caller re-bakes the sheet).
pub fn restore_grid(g: &mut Grid, s: &GridSnapshot) -> bool {
    g.film.copy_from_slice(&s.film);
    g.susp.copy_from_slice(&s.susp);
    g.susp_rgb.copy_from_slice(&s.susp_rgb);
    g.sett.copy_from_slice(&s.sett);
    g.sett_rgb.copy_from_slice(&s.sett_rgb);
    g.vel_x.copy_from_slice(&s.vel_x);
    g.vel_y.copy_from_slice(&s.vel_y);
    g.flow_x.fill(0.0);
    g.flow_y.fill(0.0);
    g.wet.copy_from_slice(&s.wet);
    g.active.copy_from_slice(&s.active);
    g.bloom.copy_from_slice(&s.bloom);
    g.bx0 = s.bx0;
    g.by0 = s.by0;
    g.bx1 = s.bx1;
    g.by1 = s.by1;
    g.has_fluid = s.has_fluid;
    let paper_changed = g.paper_preset != s.paper_preset || g.paper_sheet != s.paper_sheet;
    g.paper_preset = s.paper_preset;
    g.paper_sheet = s.paper_sheet;
    paper_changed
}
