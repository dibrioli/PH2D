//! Input conditioning — the "Input Samples" box-average at the front of the stroke pipeline.
//!
//! Behavioural reference (clean-room, no code copied): Blender
//! `editors/sculpt_paint/paint_stroke.cc::PaintStroke::add_sample` + `calc_average_sample` — a ring
//! buffer of up to `input_samples` recent (position, pressure) samples, averaged *unweighted*
//! before the sample feeds spacing/stabilize. Blender clamps the window to
//! `PAINT_MAX_INPUT_SAMPLES` (64).

use crate::stroke::StrokePoint;

/// Largest input-sample averaging window (Blender `PAINT_MAX_INPUT_SAMPLES`).
pub const MAX_INPUT_SAMPLES: usize = 64;

/// Ring buffer that returns the unweighted mean of the most recent `window` raw samples.
#[derive(Clone, Debug)]
pub(crate) struct InputSampler {
    buf: [StrokePoint; MAX_INPUT_SAMPLES],
    /// Next write slot.
    cur: usize,
    /// Count of valid samples (`<= window`).
    len: usize,
    /// Active window, `clamp(input_samples, 1, MAX_INPUT_SAMPLES)`.
    window: usize,
}

impl InputSampler {
    pub(crate) fn new(input_samples: u32) -> Self {
        Self {
            buf: [StrokePoint {
                pos: [0.0, 0.0],
                pressure: 1.0,
            }; MAX_INPUT_SAMPLES],
            cur: 0,
            len: 0,
            window: clamp_window(input_samples),
        }
    }

    /// Re-clamp the window mid-stroke (the artist dragged the Input Samples slider). Keeps the
    /// existing contents valid by clamping `len`/`cur` into the new window.
    pub(crate) fn set_window(&mut self, input_samples: u32) {
        self.window = clamp_window(input_samples);
        self.len = self.len.min(self.window);
        self.cur %= self.window;
    }

    /// Reset to hold just `first` (call on stroke begin).
    pub(crate) fn reset(&mut self, first: StrokePoint) {
        self.buf[0] = first;
        self.len = 1;
        self.cur = 1 % self.window;
    }

    /// Push `raw` into the ring and return the unweighted mean of the active window.
    pub(crate) fn push_average(&mut self, raw: StrokePoint) -> StrokePoint {
        self.buf[self.cur] = raw;
        self.cur = (self.cur + 1) % self.window;
        if self.len < self.window {
            self.len += 1;
        }
        let (mut sx, mut sy, mut sp) = (0.0_f32, 0.0_f32, 0.0_f32);
        for s in &self.buf[..self.len] {
            sx += s.pos[0];
            sy += s.pos[1];
            sp += s.pressure;
        }
        let n = self.len as f32;
        StrokePoint {
            pos: [sx / n, sy / n],
            pressure: sp / n,
        }
    }
}

fn clamp_window(s: u32) -> usize {
    (s.max(1) as usize).min(MAX_INPUT_SAMPLES)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f32, y: f32, pr: f32) -> StrokePoint {
        StrokePoint {
            pos: [x, y],
            pressure: pr,
        }
    }

    #[test]
    fn window_of_one_is_passthrough() {
        let mut s = InputSampler::new(1);
        s.reset(p(0.0, 0.0, 1.0));
        let a = s.push_average(p(10.0, 4.0, 0.5));
        assert_eq!(a.pos, [10.0, 4.0]);
        assert_eq!(a.pressure, 0.5);
    }

    #[test]
    fn window_box_averages_then_slides() {
        let mut s = InputSampler::new(3);
        s.reset(p(0.0, 0.0, 1.0)); // window holds [ (0,0) ]
        let a = s.push_average(p(3.0, 0.0, 1.0)); // [(0,0),(3,0)] → mean (1.5,0)
        assert!((a.pos[0] - 1.5).abs() < 1e-6);
        let b = s.push_average(p(3.0, 3.0, 1.0)); // [(0,0),(3,0),(3,3)] → (2,1)
        assert!((b.pos[0] - 2.0).abs() < 1e-6 && (b.pos[1] - 1.0).abs() < 1e-6);
        let c = s.push_average(p(6.0, 3.0, 1.0)); // slides → [(3,0),(3,3),(6,3)] → (4,2)
        assert!((c.pos[0] - 4.0).abs() < 1e-6 && (c.pos[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn window_clamped_to_max() {
        let s = InputSampler::new(9999);
        assert_eq!(s.window, MAX_INPUT_SAMPLES);
        let s = InputSampler::new(0);
        assert_eq!(s.window, 1);
    }
}
