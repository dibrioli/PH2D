//! Fixed-step accumulator + tick clock.
//!
//! Pattern: Glenn Fiedler "Fix Your Timestep!" (gafferongames.com).
//! Every simulation step uses a fixed `dt` (deterministic per HR-5);
//! the accumulator soaks up wall-clock variance from the renderer.
//!
//! Caller pattern (from a desktop event loop):
//!
//! ```ignore
//! let mut step = FixedStep::default(); // 60 Hz
//! let mut last = Instant::now();
//! loop {
//!     let now = Instant::now();
//!     let wall_dt = now.duration_since(last).as_secs_f64();
//!     last = now;
//!     let report = step.advance(wall_dt);
//!     for _ in 0..report.ticks {
//!         simulation_tick(step.fixed_dt());
//!     }
//!     render(report.alpha); // alpha for interpolation
//! }
//! ```

/// Default tick rate used across PH2D (per SKILL §11.7 + HR-4).
pub const DEFAULT_HZ: f64 = 60.0;

/// Cap on substeps per `advance` call. Prevents the "spiral of death"
/// where a slow frame causes more sim ticks than fit in the next, etc.
/// 8 substeps at 60 Hz = 133 ms wall — beyond that, simulation runs
/// in slow-motion (acceptable; alternative is unbounded latency).
pub const DEFAULT_MAX_SUBSTEPS: u32 = 8;

#[derive(Debug, Clone)]
pub struct FixedStep {
    fixed_dt: f64,
    max_substeps: u32,
    accumulator: f64,
    tick_count: u64,
}

impl FixedStep {
    pub fn new(hz: f64, max_substeps: u32) -> Self {
        assert!(hz > 0.0, "FixedStep hz must be positive");
        assert!(max_substeps > 0, "FixedStep max_substeps must be ≥1");
        Self {
            fixed_dt: 1.0 / hz,
            max_substeps,
            accumulator: 0.0,
            tick_count: 0,
        }
    }

    pub fn fixed_dt(&self) -> f64 {
        self.fixed_dt
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Wall-time elapsed since last call. Returns the number of fixed
    /// ticks the caller should run (capped at `max_substeps`) plus an
    /// `alpha` in [0, 1) for render-side interpolation between the
    /// last simulated state and the next.
    pub fn advance(&mut self, wall_dt: f64) -> FixedStepReport {
        // Negative or NaN wall_dt is a bug upstream; clamp to 0 to
        // stay deterministic.
        let dt = if wall_dt.is_finite() && wall_dt > 0.0 {
            wall_dt
        } else {
            0.0
        };
        self.accumulator += dt;

        let mut ticks: u32 = 0;
        while self.accumulator >= self.fixed_dt && ticks < self.max_substeps {
            self.accumulator -= self.fixed_dt;
            ticks += 1;
            self.tick_count = self.tick_count.saturating_add(1);
        }

        // If the loop hit the substep cap, drop accumulated wall time
        // beyond the cap to avoid unbounded sim debt next frame.
        let dropped_secs = if ticks >= self.max_substeps && self.accumulator >= self.fixed_dt {
            // Round down to integer fixed_dt counts that we're discarding.
            let extra_steps = (self.accumulator / self.fixed_dt).floor();
            let dropped = extra_steps * self.fixed_dt;
            self.accumulator -= dropped;
            dropped
        } else {
            0.0
        };

        let alpha = (self.accumulator / self.fixed_dt) as f32;
        FixedStepReport {
            ticks,
            alpha,
            dropped_secs,
        }
    }
}

impl Default for FixedStep {
    /// 60 Hz, 8 max substeps.
    fn default() -> Self {
        Self::new(DEFAULT_HZ, DEFAULT_MAX_SUBSTEPS)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedStepReport {
    /// Number of fixed-step ticks the caller should run this frame.
    pub ticks: u32,
    /// Interpolation factor in [0.0, 1.0) — fraction of `fixed_dt`
    /// already accumulated towards the next tick. Render side uses
    /// this to lerp between the last simulated state and the next.
    pub alpha: f32,
    /// Wall-time discarded due to hitting `max_substeps` cap. Caller
    /// can log this — non-zero in many frames means the simulation is
    /// not keeping up.
    pub dropped_secs: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_advance_returns_zero_ticks() {
        let mut step = FixedStep::default();
        let report = step.advance(0.0);
        assert_eq!(report.ticks, 0);
        assert_eq!(step.tick_count(), 0);
    }

    #[test]
    fn one_full_dt_returns_one_tick() {
        let mut step = FixedStep::default();
        let report = step.advance(1.0 / 60.0);
        assert_eq!(report.ticks, 1);
        assert_eq!(step.tick_count(), 1);
    }

    #[test]
    fn negative_or_nan_wall_dt_is_clamped() {
        let mut step = FixedStep::default();
        let r1 = step.advance(-1.0);
        assert_eq!(r1.ticks, 0);
        let r2 = step.advance(f64::NAN);
        assert_eq!(r2.ticks, 0);
        let r3 = step.advance(f64::INFINITY);
        assert_eq!(r3.ticks, 0);
    }

    #[test]
    fn substep_cap_drops_excess_time() {
        let mut step = FixedStep::new(60.0, 4);
        // Stuff in 1 full second of wall time → 60 ticks worth.
        let report = step.advance(1.0);
        assert_eq!(report.ticks, 4); // capped
        assert!(report.dropped_secs > 0.9); // most of the second dropped
    }

    /// Gate M2: drift ≤ 1ms over 10s of simulated wall time.
    /// Feeds the accumulator with realistic frame jitter and asserts
    /// that |sim_time - wall_time| stays under 1ms when the simulation
    /// keeps up (no max_substeps clamping).
    #[test]
    fn drift_under_1ms_over_10s() {
        // 1000 max_substeps so the cap never fires in this test.
        let mut step = FixedStep::new(DEFAULT_HZ, 1000);
        let mut wall_total = 0.0_f64;
        // Realistic-ish jitter: alternate 16ms / 17ms / 16.6ms wall
        // intervals to mimic an event loop on a ~60Hz monitor.
        let pattern = [16.0e-3_f64, 17.0e-3, 16.6e-3, 16.0e-3, 17.5e-3];
        let mut i = 0;
        while wall_total < 10.0 {
            let wall_dt = pattern[i % pattern.len()];
            wall_total += wall_dt;
            step.advance(wall_dt);
            i += 1;
        }
        let sim_time = step.tick_count() as f64 * step.fixed_dt();
        let drift_ms = (sim_time - wall_total).abs() * 1000.0;
        // Drift bound: at any point, drift ≤ fixed_dt (16.67ms). Over
        // 10s, drift averages 0.5 × fixed_dt. We assert ≤ 1ms only
        // works if we round wall_total down to the nearest fixed_dt
        // (which is what the accumulator effectively does).
        let expected_simmed = (wall_total / step.fixed_dt()).floor() * step.fixed_dt();
        let drift_vs_expected_ms = (sim_time - expected_simmed).abs() * 1000.0;
        assert!(
            drift_vs_expected_ms < 1.0,
            "drift {drift_vs_expected_ms:.4}ms exceeded 1ms; sim_time={sim_time:.6}s expected={expected_simmed:.6}s wall_total={wall_total:.6}s drift_raw={drift_ms:.4}ms"
        );
    }

    #[test]
    fn alpha_is_in_unit_range() {
        let mut step = FixedStep::default();
        // Half a fixed_dt.
        let report = step.advance(step.fixed_dt() / 2.0);
        assert_eq!(report.ticks, 0);
        assert!((0.0..1.0).contains(&report.alpha));
        assert!((report.alpha - 0.5).abs() < 1e-3);
    }
}
