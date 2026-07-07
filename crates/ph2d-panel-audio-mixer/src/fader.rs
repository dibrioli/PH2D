//! Fader taper — slider position (0..1) ↔ gain, **dB-tapered** like a real
//! console fader (linear-in-dB), with unity (0 dB) sitting *below* the top so
//! there's a little boost headroom and a clear unity mark. A linear
//! position→gain map crams the whole useful range into the top few percent;
//! this makes the travel perceptually even.

/// Bottom of the fader's dB range; position 0 is treated as silence (`-inf`).
const FADER_MIN_DB: f32 = -60.0; // LITERAL-PX-OK: fader floor in dB (audio domain, not a UI metric)
/// Top of the fader's dB range — a little boost above unity, like a console.
const FADER_MAX_DB: f32 = 6.0; // LITERAL-PX-OK: fader ceiling in dB (audio domain, not a UI metric)

/// The slider position where the fader sits at unity (0 dB, gain 1.0) — the
/// tick each strip paints, and every fader's start value.
pub const FADER_UNITY_POS: f32 = -FADER_MIN_DB / (FADER_MAX_DB - FADER_MIN_DB);

/// Fader position (0..1) → dB (linear-in-dB across `[FADER_MIN_DB, FADER_MAX_DB]`).
pub fn fader_db(pos: f32) -> f32 {
    FADER_MIN_DB + pos.clamp(0.0, 1.0) * (FADER_MAX_DB - FADER_MIN_DB)
}

/// Fader position (0..1) → linear gain. Position 0 → exactly 0 (true silence,
/// not just the −60 dB floor).
pub fn fader_gain(pos: f32) -> f32 {
    let p = pos.clamp(0.0, 1.0);
    if p <= 0.0 {
        0.0
    } else {
        // dB → gain = 10^(dB/20). Runs on the control/UI thread (HR-5 exempt).
        10.0_f32.powf(fader_db(p) / 20.0) // LITERAL-PX-OK: dB→gain constants (audio domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unity_position_is_zero_db_unity_gain() {
        assert!(
            (fader_db(FADER_UNITY_POS)).abs() < 1e-4,
            "unity pos must be 0 dB"
        );
        assert!(
            (fader_gain(FADER_UNITY_POS) - 1.0).abs() < 1e-4,
            "unity pos must be gain 1.0"
        );
    }

    #[test]
    fn bottom_is_silence_top_is_boost() {
        assert_eq!(fader_gain(0.0), 0.0, "bottom = true silence");
        assert!(fader_gain(1.0) > 1.0, "top boosts above unity");
    }

    #[test]
    fn taper_is_monotonic_and_below_unity_attenuates() {
        assert!(
            fader_gain(0.5) < fader_gain(FADER_UNITY_POS),
            "mid-fader is below unity"
        );
        assert!(fader_gain(0.3) < fader_gain(0.6), "monotonic increasing");
    }
}
