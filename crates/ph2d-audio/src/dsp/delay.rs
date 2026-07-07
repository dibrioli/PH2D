//! [`Delay`] — a stereo feedback delay (echo). A single delay line per channel
//! with a feedback path, so one input tap repeats and decays into echoes.
//!
//! Used as a parallel **return** fed by the per-bus delay aux-sends (mirroring
//! the reverb return). HR-3: the delay buffers are allocated once in
//! [`Delay::new`] (sized to one second at the rate) and never resized;
//! [`Delay::process`] is index math + mul/add only (HR-5, no transcendentals).

/// Feedback ceiling — kept below 1 so the echoes always decay (no runaway).
const MAX_FEEDBACK: f32 = 0.95;

/// A stereo feedback delay. Read a delayed tap, then write `input + feedback ×
/// tap` back — the classic single-line echo.
pub struct Delay {
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    write: usize,
    delay_samples: usize,
    feedback: f32,
    sample_rate: u32,
}

impl Delay {
    /// Build with a one-second maximum delay for `sample_rate`.
    pub fn new(sample_rate: u32) -> Self {
        let max = (sample_rate as usize).max(1); // one second of headroom
        Self {
            buf_l: vec![0.0; max],
            buf_r: vec![0.0; max],
            write: 0,
            delay_samples: max / 4, // 250 ms default tap
            feedback: 0.4,
            sample_rate,
        }
    }

    /// Set the echo `time` (seconds, clamped to the 1 s buffer) and `feedback`
    /// (0..1, clamped below unity so the echoes decay).
    pub fn set_params(&mut self, time_secs: f32, feedback: f32) {
        let max = self.buf_l.len();
        self.delay_samples = ((time_secs * self.sample_rate as f32) as usize).clamp(1, max - 1);
        self.feedback = feedback.clamp(0.0, MAX_FEEDBACK);
    }

    /// One stereo sample in → the delayed (wet) stereo pair out.
    #[inline]
    pub fn process(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let len = self.buf_l.len();
        let read = (self.write + len - self.delay_samples) % len;
        let (out_l, out_r) = (self.buf_l[read], self.buf_r[read]);
        // Write the input plus the fed-back tap, so the line re-echoes and decays.
        self.buf_l[self.write] = in_l + out_l * self.feedback;
        self.buf_r[self.write] = in_r + out_r * self.feedback;
        self.write = (self.write + 1) % len;
        (out_l, out_r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tap_reappears_after_the_delay_time() {
        let sr = 48_000;
        let mut d = Delay::new(sr);
        d.set_params(0.01, 0.0); // 10 ms, no feedback → a single echo
        let delay_n = (0.01 * sr as f32) as usize;
        // Impulse in, then silence.
        d.process(1.0, 1.0);
        for _ in 1..delay_n {
            let wet = d.process(0.0, 0.0).0;
            assert!(wet.abs() < 1e-6, "nothing before the delay time");
        }
        // At the delay time the impulse reappears.
        let echo = d.process(0.0, 0.0).0;
        assert!(
            (echo - 1.0).abs() < 1e-6,
            "the tap must reappear after the delay, got {echo}"
        );
    }

    #[test]
    fn feedback_makes_repeating_decaying_echoes() {
        let sr = 48_000;
        let mut d = Delay::new(sr);
        d.set_params(0.005, 0.5); // 5 ms, half feedback
        let delay_n = (0.005 * sr as f32) as usize;
        d.process(1.0, 1.0); // impulse
        let mut echoes = Vec::new();
        for i in 1..(delay_n * 3 + 3) {
            let w = d.process(0.0, 0.0).0;
            if i % delay_n == 0 {
                echoes.push(w);
            }
        }
        // Successive echoes decay by the feedback factor (~0.5 each), never grow.
        assert!(echoes.len() >= 2, "expected multiple echoes");
        assert!(
            echoes[0] > 0.4 && echoes[1] < echoes[0],
            "echoes must decay: {echoes:?}"
        );
    }

    #[test]
    fn feedback_is_clamped_below_unity() {
        let mut d = Delay::new(48_000);
        d.set_params(0.01, 5.0); // absurd feedback request
        // Feed an impulse then run long; a clamped (<1) feedback stays bounded.
        d.process(1.0, 1.0);
        let mut peak = 0.0f32;
        for _ in 0..48_000 {
            peak = peak.max(d.process(0.0, 0.0).0.abs());
        }
        assert!(
            peak.is_finite() && peak <= 1.0 + 1e-4,
            "feedback must not run away, peak {peak}"
        );
    }
}
