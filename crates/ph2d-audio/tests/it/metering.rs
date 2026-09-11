//! The renderer publishes the mixed block's peak level; the control side reads
//! it via [`AudioEngine::levels`] for metering UIs.

use ph2d_audio::{AudioEngine, AudioFormat, PlayParams, SampleData};

#[test]
fn levels_reflect_the_mixed_peak() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    // Constant-0.5 mono sample, default (center) pan.
    let data = SampleData::from_interleaved(vec![0.5; 256], AudioFormat::mono(48_000));
    engine.play(data, PlayParams::default()).unwrap();

    // Nothing rendered yet → silent.
    assert_eq!(engine.levels(), [0.0, 0.0]);

    let mut out = vec![0.0f32; 256];
    renderer.render(&mut out, 128);

    // Center pan is equal-power (−3 dB): 0.5 * 0.707 ≈ 0.354 on each channel.
    let [l, r] = engine.levels();
    let expected = 0.5 * std::f32::consts::FRAC_1_SQRT_2;
    assert!((l - expected).abs() < 1e-3, "L level {l} vs {expected}");
    assert!((r - expected).abs() < 1e-3, "R level {r} vs {expected}");
}

#[test]
fn silence_reads_zero() {
    let (engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    let mut out = vec![0.0f32; 256];
    renderer.render(&mut out, 128);
    assert_eq!(engine.levels(), [0.0, 0.0]);
}
