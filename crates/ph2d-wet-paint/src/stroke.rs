//! Stroke engine (port of `stroke.js`, SPEC §8): synthetic pressure +
//! Catmull-Rom geometry.
//!
//! There is no pen pressure; SLOW = HEAVY. The recurrence
//!   `t' = 11 - min(8, (t + speedPrevFrame) / 2)`, slew-limited to ±0.5/frame,
//! starts at 0 on press (strokes fade in over ~0.25 s instead of landing as a
//! blob), builds toward 7.33 while holding still, and drops toward 3 on a
//! fast drag. Speed is read ONE FRAME LATE by design. The engine must be fed
//! exactly one sample per 40 Hz sim step — feeding per pointer event would
//! change the pressure model and over-deposit.
//!
//! Geometry: a uniform Catmull-Rom spline through the frame samples (first /
//! last control points duplicated), walked in ~1 px micro-steps, emitting a
//! dab every `spacing` px of arc length regardless of speed. Each dab's
//! pressure lerps between the segment endpoints' t values.

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
    t: f64,
}

/// Callbacks: `on_segment(chord)` fires BEFORE that segment's dabs (the trail
/// sizes its window with it); `on_dab(x, y, pressure, dir_x, dir_y)` per dab.
pub struct Stroke {
    pub active: bool,
    pub releasing: bool,
    t: f64,
    prev_speed: f64,
    last_x: f64,
    last_y: f64,
    pts: Vec<Point>,
    arc_accum: f64,
    dir_x: f64,
    dir_y: f64,
    first_dab_done: bool,
    spacing: f64,
}

impl Default for Stroke {
    fn default() -> Self {
        Stroke {
            active: false,
            releasing: false,
            t: 0.0,
            prev_speed: 0.0,
            last_x: 0.0,
            last_y: 0.0,
            pts: Vec::new(),
            arc_accum: 0.0,
            dir_x: 0.0,
            dir_y: 0.0,
            first_dab_done: false,
            spacing: 2.0,
        }
    }
}

/// The per-frame stroke outputs, drained by the caller after each tick — the
/// JS uses closures; a drained event list keeps the borrow checker out of the
/// facade's way without changing WHEN anything fires.
pub enum StrokeEvent {
    /// Chord length of a spline segment, reported before its dabs.
    Segment { chord: f64 },
    Dab { x: f64, y: f64, pressure: f64, dir_x: f64, dir_y: f64 },
}

impl Stroke {
    pub fn begin(&mut self, x: f64, y: f64, spacing: f64) {
        self.active = true;
        self.releasing = false;
        self.t = 0.0;
        self.prev_speed = 0.0;
        self.last_x = x;
        self.last_y = y;
        self.pts.clear();
        self.pts.push(Point { x, y, t: 0.0 });
        self.arc_accum = 0.0;
        self.dir_x = 0.0;
        self.dir_y = 0.0;
        self.first_dab_done = false;
        // JS `Math.max(0.5, spacing || 2)`: a falsy spacing (0 or NaN) falls
        // back to the default 2 BEFORE the floor (port-verify finding).
        let spacing = if spacing == 0.0 || spacing.is_nan() { 2.0 } else { spacing };
        self.spacing = spacing.max(0.5);
    }

    /// One pointer sample per 40 Hz step while the pointer is down. Pressure
    /// ticks every frame; geometry only advances when the hop is >= 0.5 px.
    pub fn tick(&mut self, x: f64, y: f64, out: &mut Vec<StrokeEvent>) {
        if !self.active || self.releasing {
            return;
        }
        let dx = x - self.last_x;
        let dy = y - self.last_y;
        let hop = (dx * dx + dy * dy).sqrt();
        self.tick_pressure(hop);
        self.last_x = x;
        self.last_y = y;
        if hop >= 0.5 {
            self.push_point(x, y, false, out);
        }
    }

    fn tick_pressure(&mut self, speed_this_frame: f64) {
        // The recurrence reads the PREVIOUS frame's speed.
        let target = 11.0 - ((self.t + self.prev_speed) / 2.0).min(8.0);
        let mut d = target - self.t;
        if d > 0.5 {
            d = 0.5;
        } else if d < -0.5 {
            d = -0.5;
        }
        self.t += d;
        if self.t < 0.0 {
            self.t = 0.0;
        }
        self.prev_speed = speed_this_frame;
    }

    /// Pointer released: enter the fade-out tail.
    pub fn release(&mut self) {
        if self.active {
            self.releasing = true;
        }
    }

    /// Tick the release tail at 40 Hz with the current (hover) position:
    /// pressure decays as t = (t - 0.5) * 0.6, and geometry keeps flowing
    /// while the cursor still moves (hop >= 1 px) and t > 0.01 — the stroke
    /// lifts off with a fading tail. Returns false once fully finished
    /// (spline flushed; the caller then drops the trail remainder).
    pub fn release_tick(&mut self, x: f64, y: f64, out: &mut Vec<StrokeEvent>) -> bool {
        if !self.active {
            return false;
        }
        self.t = (self.t - 0.5) * 0.6;
        if self.t < 0.0 {
            self.t = 0.0;
        }
        let dx = x - self.last_x;
        let dy = y - self.last_y;
        let hop = (dx * dx + dy * dy).sqrt();
        self.last_x = x;
        self.last_y = y;
        if hop >= 1.0 && self.t > 0.01 {
            self.push_point(x, y, false, out);
            return true;
        }
        self.finish(out);
        false
    }

    /// Flush the spline tail (duplicate the last point once) and deactivate.
    pub fn finish(&mut self, out: &mut Vec<StrokeEvent>) {
        if !self.active {
            return;
        }
        if self.pts.len() >= 2 {
            let last = *self.pts.last().unwrap();
            self.push_point(last.x, last.y, true, out);
        }
        self.active = false;
        self.releasing = false;
    }

    fn push_point(&mut self, x: f64, y: f64, force_duplicate: bool, out: &mut Vec<StrokeEvent>) {
        self.pts.push(Point { x, y, t: self.t });
        let m = self.pts.len();
        // Segment Pk -> Pk+1 becomes emittable once Pk+2 exists (p3). The
        // first segment duplicates P0 as its p0; the duplicate flush passes
        // p3 = p2 (same call shape — the fresh point IS the duplicate).
        if m >= 3 {
            let p3 = self.pts[m - 1];
            let p2 = self.pts[m - 2];
            let p1 = self.pts[m - 3];
            let p0 = if m >= 4 { self.pts[m - 4] } else { p1 };
            self.emit_segment(p0, p1, p2, if force_duplicate { p2 } else { p3 }, out);
        }
        let n = self.pts.len();
        if n > 64 {
            self.pts.drain(0..n - 4); // keep the tail only
        }
    }

    fn emit_segment(&mut self, p0: Point, p1: Point, p2: Point, p3: Point, out: &mut Vec<StrokeEvent>) {
        let cdx = p2.x - p1.x;
        let cdy = p2.y - p1.y;
        let chord = (cdx * cdx + cdy * cdy).sqrt();
        out.push(StrokeEvent::Segment { chord }); // trail sizes its window BEFORE the dabs
        // The very first dab of a stroke lands exactly at the press point
        // with its recorded pressure (0) — the fade-in starts from nothing.
        if !self.first_dab_done {
            self.first_dab_done = true;
            out.push(StrokeEvent::Dab {
                x: p1.x,
                y: p1.y,
                pressure: p1.t,
                dir_x: self.dir_x,
                dir_y: self.dir_y,
            });
        }
        let steps = (chord.ceil() as i64).max(2);
        let mut px = p1.x;
        let mut py = p1.y;
        for j in 1..=steps {
            let u = j as f64 / steps as f64;
            let (sx, sy) = catmull_rom(p0, p1, p2, p3, u);
            let dx = sx - px;
            let dy = sy - py;
            let d = (dx * dx + dy * dy).sqrt();
            if d > 0.0 {
                self.dir_x = dx / d;
                self.dir_y = dy / d;
            }
            self.arc_accum += d;
            while self.arc_accum >= self.spacing {
                self.arc_accum -= self.spacing;
                let b = p1.t + (p2.t - p1.t) * u; // pressure lerped along the segment
                out.push(StrokeEvent::Dab {
                    x: sx,
                    y: sy,
                    pressure: b,
                    dir_x: self.dir_x,
                    dir_y: self.dir_y,
                });
            }
            px = sx;
            py = sy;
        }
    }
}

/// Uniform Catmull-Rom position at fraction u in [0,1] of segment p1 -> p2.
fn catmull_rom(p0: Point, p1: Point, p2: Point, p3: Point, u: f64) -> (f64, f64) {
    let u2 = u * u;
    let u3 = u2 * u;
    let x = 0.5
        * (2.0 * p1.x
            + (-p0.x + p2.x) * u
            + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * u2
            + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * u3);
    let y = 0.5
        * (2.0 * p1.y
            + (-p0.y + p2.y) * u
            + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * u2
            + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * u3);
    (x, y)
}
