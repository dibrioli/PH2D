//! The master low-pass filter attenuates high frequencies end-to-end: a
//! near-Nyquist signal passes with the filter open (identity) and is knocked
//! down once a low cutoff is set.

use ph2d_audio::{AudioEngine, AudioFormat, PlayParams, SampleData};

#[test]
fn master_lowpass_attenuates_high_frequency() {
    let sr = 48_000;
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(sr));

    // Alternating +1/-1 = the Nyquist frequency; loop it forever.
    let alt: Vec<f32> = (0..256)
        .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
    let data = SampleData::from_interleaved(alt, AudioFormat::mono(sr));
    engine
        .play(
            data,
            PlayParams {
                looping: true,
                ..PlayParams::default()
            },
        )
        .unwrap();

    let mut out = vec![0.0f32; 512];

    // Filter open (default identity) → the Nyquist signal passes through.
    for _ in 0..4 {
        renderer.render(&mut out, 256);
    }
    let open_peak = out.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

    // Low cutoff → the Nyquist signal is heavily attenuated.
    engine.set_master_cutoff(500.0).unwrap();
    for _ in 0..8 {
        renderer.render(&mut out, 256);
    }
    let filtered_peak = out.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

    assert!(
        open_peak > 0.5,
        "open path should pass the signal (got {open_peak})"
    );
    assert!(
        filtered_peak < open_peak * 0.5,
        "low-pass must attenuate Nyquist: {filtered_peak} vs open {open_peak}"
    );
}
