//! The probe readout (Motion Nodes F2) — a small card beside the probed node
//! showing its current output and a **sparkline** of the last second of readings.
//!
//! The question a node graph cannot answer by looking at it is *"what is actually
//! flowing through this wire right now?"*. A number alone answers it for one tick;
//! the sparkline answers it for the last sixty, which is what tells an oscillator
//! from a stuck value, or a spring that is settling from one that has died.
//!
//! The panel keeps **no history**: the shell samples the pump's own cook each tick
//! and publishes the whole ring on the snapshot (`ProbeView`). A second history
//! here would be a second source of truth, and the two would drift the moment a
//! frame is dropped.

use crate::geom::{self, View};
use crate::snapshot::{GraphNodeView, ProbeView};
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_title, resolve, stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

// Screen-space chrome for the readout — it is an OVERLAY (like the add-menu), so
// it does not scale with zoom: a readout you must zoom in to read is not a readout.
const W: f32 = 150.0; // LITERAL-PX-OK: probe card width
const H: f32 = 58.0; // LITERAL-PX-OK: probe card height
const GAP: f32 = 10.0; // LITERAL-PX-OK: gap between the card and the node
const RADIUS: f32 = 6.0; // LITERAL-PX-OK: probe card corner radius
const PAD: f32 = 7.0; // LITERAL-PX-OK: probe card inner padding
const TEXT_SIZE: f32 = 12.0; // LITERAL-PX-OK: readout font size
const TEXT_H: f32 = 16.0; // LITERAL-PX-OK: readout line height
const SPARK_W: f32 = 1.4; // LITERAL-PX-OK: sparkline stroke width
const RING_W: f32 = 2.0; // LITERAL-PX-OK: probed-node ring stroke width

/// Draw the probe: a ring on the node it reads, and its readout card above it.
pub(crate) fn draw(
    ctx: &mut PaintCtx,
    probe: &ProbeView,
    node: &GraphNodeView,
    view: &View,
    theme: Theme,
) {
    // The ring says WHICH card the number belongs to — without it a floating panel
    // of numbers is an orphan.
    let card = geom::card_rect(node, view);
    stroke_rounded_rect(
        ctx.scene,
        card,
        RADIUS,
        RING_W,
        resolve(ColorToken::Accent, theme),
    );

    let panel = Rect::new(card.x, card.y - H - GAP, W, H);
    fill_rounded_rect(ctx.scene, panel, RADIUS, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(
        ctx.scene,
        panel,
        RADIUS,
        1.0,
        resolve(ColorToken::Accent, theme),
    );

    // The reading, and what it MEANS (a bare number is a riddle: 144 what?).
    paint_text_title(
        ctx.text_system,
        ctx.scene,
        &format!("{}: {:.3}", probe.label, probe.value),
        panel.x + PAD,
        panel.y + PAD * 0.5,
        TEXT_SIZE,
        panel.w - 2.0 * PAD,
        resolve(ColorToken::Text1, theme),
    );

    let plot = Rect::new(
        panel.x + PAD,
        panel.y + TEXT_H + PAD * 0.5,
        panel.w - 2.0 * PAD,
        panel.h - TEXT_H - PAD * 1.5,
    );
    stroke_polyline(
        ctx.scene,
        &sparkline(&probe.samples, plot),
        SPARK_W,
        resolve(ColorToken::Accent, theme),
    );
}

/// The sparkline's screen points — the ring mapped onto `plot`, auto-scaled to its
/// own min/max. Auto-scale, not a fixed range: the useful thing about a probe is
/// the SHAPE of the signal, and a fixed range would flatten anything small into a
/// straight line (a wiggle of ±0.01 reads as dead against a 0..1 axis).
///
/// A flat signal (min == max) draws down the middle rather than dividing by zero.
fn sparkline(samples: &[f32], plot: Rect) -> Vec<(f32, f32)> {
    if samples.is_empty() {
        return Vec::new();
    }
    let finite = |v: &&f32| v.is_finite();
    let lo = samples
        .iter()
        .filter(finite)
        .copied()
        .fold(f32::MAX, f32::min);
    let hi = samples
        .iter()
        .filter(finite)
        .copied()
        .fold(f32::MIN, f32::max);
    let span = hi - lo;
    let n = samples.len();
    (0..n)
        .map(|i| {
            let t = if n > 1 {
                i as f32 / (n - 1) as f32
            } else {
                0.0
            };
            let v = samples[i];
            let u = if span > 0.0 && v.is_finite() {
                (v - lo) / span
            } else {
                0.5
            };
            // y grows downward on screen, so the HIGH sample sits at the top.
            (plot.x + t * plot.w, plot.y + (1.0 - u) * plot.h)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLOT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 100.0,
        h: 40.0,
    };

    /// The sparkline auto-scales to its own extremes and puts the HIGHEST sample at
    /// the TOP (screen y grows downward — the classic sign flip that draws every
    /// graph upside-down).
    #[test]
    fn the_sparkline_scales_to_its_own_range_and_puts_the_peak_on_top() {
        let pts = sparkline(&[0.0, 10.0, 5.0], PLOT);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], (0.0, 40.0), "the lowest sample sits on the floor");
        assert_eq!(pts[1], (50.0, 0.0), "the peak sits at the top");
        assert_eq!(pts[2], (100.0, 20.0), "the middle value lands mid-plot");
    }

    /// A tiny wiggle still fills the plot — the shape is the point. FALSIFIED by a
    /// fixed 0..1 axis, which would flatten this into a dead straight line.
    #[test]
    fn a_tiny_wiggle_still_reads_as_a_wiggle() {
        let pts = sparkline(&[0.500, 0.501, 0.500], PLOT);
        assert_eq!(
            pts[1].1, 0.0,
            "the peak of a ±0.001 wiggle still reaches the top"
        );
    }

    /// A flat signal draws down the middle instead of dividing by zero, and a
    /// non-finite sample (a diverged sim) does not poison the scale.
    #[test]
    fn a_flat_or_non_finite_signal_does_not_break_the_plot() {
        for p in sparkline(&[2.0, 2.0, 2.0], PLOT) {
            assert_eq!(p.1, 20.0, "flat: down the middle");
        }
        for p in sparkline(&[0.0, f32::NAN, 1.0], PLOT) {
            assert!(p.1.is_finite(), "a NaN sample never yields a NaN point");
        }
        assert!(sparkline(&[], PLOT).is_empty());
    }
}
